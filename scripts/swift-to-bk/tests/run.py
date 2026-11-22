import re
import subprocess
import sys
import unittest
from pathlib import Path

TESTS = Path(__file__).resolve().parent
TOOL = TESTS.parent
ROOT = TOOL.parent.parent
sys.path.insert(0, str(TOOL))
import swiftast  # noqa: E402
import translate  # noqa: E402

RECIPES = ROOT / 'agents/recipes'
BLESS = '--bless' in sys.argv


COMPLETE = ('counter', 'form', 'lists', 'components', 'styles')


def expected_path(case):
    return RECIPES / f'swift_{case.name}.rs' if case.name in COMPLETE else case / 'expected.rs'


def cases():
    return sorted(p for p in TESTS.iterdir() if p.is_dir() and any(p.glob('*.swift')))


class ParserTests(unittest.TestCase):
    def test_attributes_keep_quoted_values_with_parentheses(self):
        node = swiftast.parse('(call_expr type="(Font?) -> Text" decl="A.(file).B.init(_:)" implicit)')
        self.assertEqual(node.kind, 'call_expr')
        self.assertEqual(node.get('decl'), 'A.(file).B.init(_:)')
        self.assertTrue(node.get('implicit'))

    def test_label_prefixes_the_kind_of_a_child(self):
        node = swiftast.parse('(a (original_init=call_expr type="Text"))')
        self.assertEqual(node.children[0].label, 'original_init')
        self.assertEqual(node.children[0].kind, 'call_expr')


class DeclTests(unittest.TestCase):
    def test_extension_member_with_labels(self):
        decl = translate.Decl('SwiftUI.(file).TextField extension.init(_:text:)')
        self.assertEqual((decl.module, decl.owner, decl.member, decl.labels), ('SwiftUI', 'TextField', 'init', ['_', 'text']))

    def test_generic_suffix_and_location_are_dropped(self):
        decl = translate.Decl('Demo.(file).Demo.count@/tmp/Demo.swift:4:24')
        self.assertEqual((decl.owner, decl.member), ('Demo', 'count'))

    def test_range_operator(self):
        decl = translate.Decl('Swift.(file).Comparable extension...<')
        self.assertEqual((decl.owner, decl.member), ('Comparable', '..<'))


class TableTests(unittest.TestCase):
    def test_table_json_is_current(self):
        result = subprocess.run([sys.executable, '-I', str(TOOL / 'make-table.py'), '--check'], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_every_row_has_a_status(self):
        for row in translate.TABLE.values():
            self.assertIn(row['status'], ('mapped', 'partial', 'missing', 'n/a', 'out-of-scope'))

    def test_a_missing_row_names_its_gap_row(self):
        self.assertRegex(translate.gap_ref('View.tag'), r'^swiftui-api-gap\.md §\d\.\d ')


class GoldenTests(unittest.TestCase):
    def test_translations_match_their_expected_output(self):
        for case in cases():
            with self.subTest(case=case.name):
                swift = next(case.glob('*.swift'))
                source, report = translate.translate(str(swift))
                want = expected_path(case)
                if BLESS:
                    want.write_text(source)
                self.assertEqual(source, want.read_text())
                self.assertEqual(bool(report), case.name not in COMPLETE, 'a complete case has an empty report')

    def test_a_gap_is_reported_in_the_output_and_in_the_summary(self):
        _, report = translate.translate(str(TESTS / 'gaps/Gaps.swift'))
        self.assertIn('TODO(swift: ZStack.init) -> swiftui-api-gap.md §3.2 types, partial', report)
        text = (TESTS / 'gaps/expected.rs').read_text()
        self.assertIn('// TODO(swift: View.tag)', text)
        self.assertIn('// Translation report:', text)

    def test_counter_has_the_shape_of_the_counter_recipe(self):
        drop = lambda text: re.sub(r'\n\s*\.options\(.*\)', '', text)
        source, _ = translate.translate(str(TESTS / 'counter/Counter.swift'))
        self.assertEqual(drop(source), drop((RECIPES / 'counter.rs').read_text()))


if __name__ == '__main__':
    argv = [a for a in sys.argv if a != '--bless']
    unittest.main(argv=argv, verbosity=2)
