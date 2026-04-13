# Security Policy

## Reporting

Do not open a public GitHub issue for security vulnerabilities.

**Preferred:** Use the "Report a vulnerability" button on the [Security tab](../../security/advisories/new).

**Alternative:** Email `bugs@gnu.foo`. Reports are acknowledged within 72 hours; fixes within 30 days depending on severity.

## Scope

In scope:
- Canonicalization correctness bugs that could cause AAD mismatch at encryption/decryption boundaries
- Input validation bypasses
- Dependency vulnerabilities with a realistic attack path

Out of scope: issues with no security impact, theoretical bugs without a proof of concept.

## GPG Key

`EA92 184C E5A3 4B0B C9EE  3A91 8E28 40A2 97D4 7681`

[Fetch from keys.openpgp.org](https://keys.openpgp.org/search?q=EA92184CE5A34B0BC9EE3A918E2840A297D47681) · [keys/EA92184CE5A34B0BC9EE3A918E2840A297D47681.asc](keys/EA92184CE5A34B0BC9EE3A918E2840A297D47681.asc)
