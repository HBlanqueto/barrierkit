import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
sys.path.insert(0, str(HERE))
import translate  # noqa: E402

def main(argv):
    if len(argv) != 3 or not re.fullmatch(r'[a-z0-9_]+', argv[1]):
        sys.exit('usage: recipe.py <snake_case_name> File.swift')
    name, swift = argv[1], argv[2]
    source, report = translate.translate(swift, name=name.replace('_', '-'))
    if report:
        sys.exit('open TODOs, no recipe written:\n' + ''.join(f'  {line}\n' for line in report))
    example = f'swift_{name}'
    (ROOT / f'crates/bk/examples/{example}.rs').write_text(source)
    return subprocess.call(['cargo', 'build', '-p', 'bk', '--example', example], cwd=ROOT)


if __name__ == '__main__':
    sys.exit(main(sys.argv))
