"""Check that contract validation never silently skips timestamp formats."""
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

from jsonschema import FormatChecker

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from validate_contracts import checked_formats


class ContractFormatTests(unittest.TestCase):
    def test_dates_are_actually_checked(self):
        checker = checked_formats()
        self.assertTrue(checker.conforms('2026-10-14T05:00:01Z', 'date-time'))
        self.assertFalse(checker.conforms('not-a-timestamp', 'date-time'))
        self.assertFalse(checker.conforms('2026-02-31T05:00:01Z', 'date-time'))

    def test_missing_optional_format_support_fails_closed(self):
        with patch('validate_contracts.FormatChecker', return_value=FormatChecker(formats=[])):
            with self.assertRaisesRegex(RuntimeError, 'Date-time checks are unavailable'):
                checked_formats()


if __name__ == '__main__':
    unittest.main()
