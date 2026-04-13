//! JSON scanner layer: single-pass duplicate-key detection via serde visitor.

use crate::error::AadError;
use serde::de::{self, Deserialize, DeserializeSeed, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::Value;
use std::cell::Cell;
use std::collections::HashSet;
use std::fmt;

thread_local! {
    /// Carries the duplicate key out of the serde visitor and into `into_aad_error`,
    /// avoiding any dependency on `serde_json`'s error message format.
    static LAST_DUPLICATE_KEY: Cell<Option<String>> = const { Cell::new(None) };
}

/// Parses JSON, rejecting any object with duplicate keys.
///
/// # Errors
///
/// Returns `DuplicateKey` if a duplicate key is found.
/// Returns `InvalidJson` if the JSON syntax is invalid.
pub(super) fn parse_json_with_duplicate_check(json: &str) -> Result<Value, AadError> {
    let mut de = serde_json::Deserializer::from_str(json);
    DupCheckValue::deserialize(&mut de).map(|v| v.0).map_err(|e| into_aad_error(&e))
}

/// Maps a `serde_json::Error` to `AadError`, using the thread-local to recover
/// the duplicate key without parsing `serde_json`'s error message text.
fn into_aad_error(e: &serde_json::Error) -> AadError {
    let msg = e.to_string();
    if msg.starts_with("__dup__") {
        let key = LAST_DUPLICATE_KEY.with(Cell::take).unwrap_or_default();
        return AadError::DuplicateKey { key };
    }
    AadError::InvalidJson { message: msg }
}

struct DupCheckValue(Value);

impl<'de> Deserialize<'de> for DupCheckValue {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_any(DupCheckVisitor).map(DupCheckValue)
    }
}

struct DupCheckVisitor;

impl<'de> Visitor<'de> for DupCheckVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "any valid JSON value")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Value, E> {
        Ok(Value::Number(v.into()))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Value, E> {
        Ok(Value::Number(v.into()))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Value, E> {
        serde_json::Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| de::Error::custom("non-finite float"))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.to_owned()))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Value, E> {
        Ok(Value::String(v))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut arr = Vec::new();
        while let Some(elem) = seq.next_element_seed(DupCheckSeed)? {
            arr.push(elem);
        }
        Ok(Value::Array(arr))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut obj = serde_json::Map::new();

        while let Some(key) = map.next_key::<String>()? {
            if !seen.insert(key.clone()) {
                LAST_DUPLICATE_KEY.with(|slot| slot.set(Some(key)));
                return Err(de::Error::custom("__dup__"));
            }
            let value: DupCheckValue = map.next_value()?;
            obj.insert(key, value.0);
        }

        Ok(Value::Object(obj))
    }
}

struct DupCheckSeed;

impl<'de> DeserializeSeed<'de> for DupCheckSeed {
    type Value = Value;

    fn deserialize<D: Deserializer<'de>>(self, de: D) -> Result<Self::Value, D::Error> {
        DupCheckValue::deserialize(de).map(|v| v.0)
    }
}
