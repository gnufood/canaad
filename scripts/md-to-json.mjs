import { remark } from 'remark';

const version = process.argv[2];

let md = '';
process.stdin.setEncoding('utf8');
for await (const chunk of process.stdin) md += chunk;

const tree = remark().parse(md);

function text(node) {
  if ('value' in node) return node.value;
  if (node.children) return node.children.map(text).join('');
  return '';
}

function itemSource(node) {
  const para = node.children[0];
  if (para?.position) {
    return md.slice(para.position.start.offset, para.position.end.offset);
  }
  return text(node);
}

const sections = [];
let current = null;

for (const node of tree.children) {
  if (node.type === 'heading' && node.depth === 3) {
    current = { title: text(node), items: [] };
    sections.push(current);
  } else if (node.type === 'list' && current) {
    for (const item of node.children) {
      current.items.push(itemSource(item).trim());
    }
  }
}

process.stdout.write(JSON.stringify({ version, sections }));
