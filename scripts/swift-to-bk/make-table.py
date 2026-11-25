import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
GLOSSARY = ROOT / 'docs/barrierkit/20-glossary.md'
CONSTRUCTS = ROOT / 'docs/barrierkit/16-swiftui-to-the-kit.md'
GAP = ROOT / 'docs/research/swiftui-api-gap.md'
GAP_STATUS = {'implemented': 'mapped', 'partial': 'partial', 'absent': 'missing',
              'n/a': 'n/a', 'out of scope': 'out-of-scope'}
PIPE = re.compile(r'(?<!\\)\|')
TICKS = re.compile(r'`([^`]+)`')


def cells(line):
    return [c.strip().replace('\\|', '|') for c in PIPE.split(line.strip())[1:-1]]


def table_rows(path):
    for line in path.read_text().splitlines():
        if line.startswith('|') and not re.match(r'^\|[-| ]+\|$', line):
            yield cells(line)


def swift_key(token):
    token = token.lstrip('≈ ').strip()
    m = re.match(r'^\.(\w+)(?:\((.*)\))?$', token)
    if m:
        return 'View.' + m.group(1), m.group(2)
    m = re.match(r'^([A-Z]\w*)\((.*)\)$', token)
    if m:
        return m.group(1) + '.init', m.group(2)
    m = re.match(r'^(@\w+)', token)
    if m:
        return m.group(1), None
    m = re.match(r'^([A-Z]\w*(?:\.\w+)*)$', token)
    if m:
        return m.group(1), None
    return None, None


def glossary_rows():
    for c in table_rows(GLOSSARY):
        if len(c) != 7 or c[0] in ('BarrierKit', 'Before', 'Name'):
            continue
        kit = TICKS.search(c[0])
        for token in TICKS.findall(c[2]):
            key, labels = swift_key(token)
            if key and kit:
                row = {'swift': key, 'kit': kit.group(1), 'status': 'partial' if '≈' in c[2] else 'mapped',
                       'origin': 'glossary', 'chapter': c[6]}
                if labels is not None:
                    row['labels'] = labels
                yield row


def construct_rows():
    for c in table_rows(CONSTRUCTS):
        if len(c) != 3 or c[0] in ('Swift', 'SwiftUI', 'SwiftUI or AppKit'):
            continue
        for token in TICKS.findall(c[0]):
            key, labels = swift_key(token.split(' ')[0] if token.startswith('@') else token)
            if key:
                yield {'swift': key, 'kit': c[1].replace('`', ''), 'status': 'mapped',
                       'origin': 'chapter 16', 'recipe': (re.search(r'\((\.\./\.\./crates/bk/examples/\w+\.rs)\)', c[2]) or [None, ''])[1].split('/')[-1]}


def gap_rows():
    text = GAP.read_text().splitlines()
    part = None
    category = None
    family = {}
    for i, line in enumerate(text):
        m = re.match(r'^### (\d)\.(\d) ', line)
        if m:
            part = (m.group(1), m.group(2))
        elif line.startswith('## '):
            part = None
        if part and part[1] == '1' and line.startswith('|') and not line.startswith('|---'):
            c = cells(line)
            if len(c) >= 8:
                for token in TICKS.findall(c[0]):
                    family.setdefault(token, {'family': c[0], 'size': c[-2], 'blocker': c[-1]})
        if part and part[1] == '2':
            anchor = f'{part[0]}.2'
            m = re.match(r'^(?:- )?(types|view modifiers|scene modifiers|static style members|environment values)(?: \([^)]*\))? - (.*)$', line)
            heading = re.match(r'^\*\*(types|view modifiers|scene modifiers|static style members|environment values)(?: \(\d+\))?\*\*', line)
            if heading:
                category = heading.group(1)
                continue
            if m:
                category, rest = m.group(1), m.group(2)
            else:
                bullet = re.match(r'^- (implemented|partial|absent|n/a|out of scope) \(\d+\): (.*)$', line)
                if not bullet or category is None:
                    continue
                rest = f'{bullet.group(1)} (0): {bullet.group(2)}'
            for sm in re.finditer(r'(implemented|partial|absent|n/a|out of scope) \((\d+)\): ([^;]*)', rest + ';'):
                status = GAP_STATUS[sm.group(1)]
                for name in segment_names(sm.group(3)):
                    key = {'view modifiers': 'View.', 'scene modifiers': 'Scene.', 'environment values': 'Environment.'}.get(category, '') + name
                    row = {'swift': key, 'status': status, 'origin': 'gap', 'gap': f'swiftui-api-gap.md §{anchor} {category}, {sm.group(1)}'}
                    if name in family:
                        row.update(size=family[name]['size'], blocker=family[name]['blocker'])
                    yield row


def segment_names(text):
    if '`' in text:
        return TICKS.findall(text)
    names = [re.split(r' [(\[]', n.strip())[0] for n in text.split(', ')]
    return [n for n in names if re.fullmatch(r'[A-Za-z_][\w.]*', n) and n != 'none']


def symbol_rows():
    inside = False
    for line in GLOSSARY.read_text().splitlines():
        if line.startswith('### Symbol variants'):
            inside = True
        elif inside and line.startswith('## '):
            break
        elif inside and line.startswith('| `'):
            c = cells(line)
            if len(c) >= 2 and TICKS.search(c[1]):
                yield {'swift': 'SF:' + TICKS.search(c[1]).group(1), 'kit': 'Symbol::' + TICKS.search(c[0]).group(1),
                       'status': 'mapped', 'origin': 'glossary', 'chapter': '4'}


def build():
    rules = json.loads((HERE / 'rules.json').read_text())
    rows, gaps = {}, {}
    for row in [*rules, *construct_rows(), *glossary_rows(), *symbol_rows(), *gap_rows()]:
        rows.setdefault(row['swift'], row)
        if row['origin'] == 'gap':
            gaps[row['swift']] = row
    for key, row in rows.items():
        if row['origin'] != 'gap' and key in gaps:
            for field in ('gap', 'size', 'blocker'):
                if field in gaps[key]:
                    row.setdefault(field, gaps[key][field])
    return [rows[k] for k in sorted(rows)]


def render(rows):
    return '[\n' + ',\n'.join(json.dumps(r, ensure_ascii=False, sort_keys=True) for r in rows) + '\n]\n'


if __name__ == '__main__':
    out = render(build())
    target = HERE / 'table.json'
    if '--check' in sys.argv:
        if target.read_text() != out:
            sys.exit('table.json is stale: run python3 -I scripts/swift-to-bk/make-table.py')
    else:
        target.write_text(out)
        print(f'{len(json.loads(out))} rows')
