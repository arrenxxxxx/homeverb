"""Tests for repository tooling only; no appliance or network access."""
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from validate_commits import validate_message
from validate_repository import check_file, non_latin_letters


class CommitTests(unittest.TestCase):
    def test_approved_types(self):
        for kind in ('feat', 'fix', 'docs', 'test', 'refactor', 'perf', 'ci', 'build', 'chore', 'style', 'revert'):
            with self.subTest(kind=kind):
                self.assertEqual(validate_message(f'{kind}(core): update command validation'), [])

    def test_breaking_change(self):
        self.assertEqual(validate_message('feat(core)!: change execution outcomes\n\nBREAKING CHANGE: remove ambiguous success.'), [])

    def test_invalid_types(self):
        for text in ('fateture: add controls', 'feature: add controls', 'Update README', 'feat:no space', 'fix: '):
            with self.subTest(text=text):
                self.assertTrue(validate_message(text))

    def test_non_english_body(self):
        self.assertTrue(validate_message('docs: update instructions\n\n\u4e2d\u6587'))

    def test_header_limit(self):
        self.assertTrue(validate_message('docs: ' + 'a' * 101))

    def test_body_requires_blank_line(self):
        self.assertTrue(validate_message('docs: update instructions\nA body without separation.'))

    def test_empty_message(self):
        self.assertTrue(validate_message(''))


class LanguageTests(unittest.TestCase):
    def test_english_and_typography(self):
        self.assertFalse(non_latin_letters('English documentation - 26 C; state -> verified.'))

    def test_non_latin_detection(self):
        self.assertTrue(non_latin_letters('\u4e2d\u6587'))
        self.assertTrue(non_latin_letters('\u65e5\u672c\u8a9e'))

    def test_text_file_validation(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'README.md'
            path.write_text('# English\n', encoding='utf-8')
            self.assertEqual(check_file(path), [])
            path.write_text('# English', encoding='utf-8')
            self.assertTrue(check_file(path))


if __name__ == '__main__':
    unittest.main()
