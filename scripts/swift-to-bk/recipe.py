import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
sys.path.insert(0, str(HERE))
import translate  # noqa: E402

CARGO = ROOT / 'crates/bk/Cargo.toml'


def register(example):
    text = CARGO.read_text()
    if f'name = "{example}"' in text:
        return
    block = f'[[example]]\nname = "{example}"\npath = "../../agents/recipes/{example}.rs"\n\n'
    for m in re.finditer(r'\[\[example\]\]\nname = "(\w+)"', text):
        if m.group(1) > example:
            CARGO.write_text(text[:m.start()] + block + text[m.start():])
            return
    CARGO.write_text(text.rstrip('\n') + '\n\n' + block.rstrip('\n') + '\n')


def main(argv):
    if len(argv) != 3 or not re.fullmatch(r'[a-z0-9_]+', argv[1]):
        sys.exit('usage: recipe.py <snake_case_name> File.swift')
    name, swift = argv[1], argv[2]
    source, report = translate.translate(swift, name=name.replace('_', '-'))
    if report:
        sys.exit('open TODOs, no recipe written:\n' + ''.join(f'  {line}\n' for line in report))
    example = f'swift_{name}'
    (ROOT / f'agents/recipes/{example}.rs').write_text(source)
    register(example)
    return subprocess.call(['cargo', 'build', '-p', 'bk', '--example', example], cwd=ROOT)


if __name__ == '__main__':
    sys.exit(main(sys.argv))
