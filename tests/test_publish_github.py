"""Offline tests for publishing guards. No live GitHub actions are performed."""
import contextlib
import io
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import publish_github


class PublishGuardTests(unittest.TestCase):
    def test_default_target(self):
        self.assertEqual(publish_github.validate_repo_name('arrenxxxxx/homeverb'), ('arrenxxxxx', 'homeverb'))

    def test_invalid_target(self):
        for value in ('https://github.com/owner/name', '--public', 'owner/../name', 'owner/..', 'owner/name;id'):
            with self.subTest(value=value), self.assertRaises(publish_github.PublishError):
                publish_github.validate_repo_name(value)

    def test_matching_origins(self):
        for origin in ('https://github.com/arrenxxxxx/homeverb.git', 'git@github.com:arrenxxxxx/homeverb.git'):
            self.assertTrue(publish_github.origin_matches(origin, 'arrenxxxxx/homeverb'))

    def test_unrelated_origin_rejected(self):
        for origin in ('/tmp/homeverb.bundle', 'https://github.com/other/homeverb.git', 'https://github.com.evil.test/arrenxxxxx/homeverb'):
            self.assertFalse(publish_github.origin_matches(origin, 'arrenxxxxx/homeverb'))

    def test_private_metadata(self):
        publish_github.require_private({'nameWithOwner': 'arrenxxxxx/homeverb', 'visibility': 'PRIVATE'}, 'arrenxxxxx/homeverb')

    def test_public_or_wrong_repository_rejected(self):
        for meta in ({'nameWithOwner': 'arrenxxxxx/homeverb', 'visibility': 'PUBLIC'},
                     {'nameWithOwner': 'other/homeverb', 'visibility': 'PRIVATE'}, {}):
            with self.subTest(meta=meta), self.assertRaises(publish_github.PublishError):
                publish_github.require_private(meta, 'arrenxxxxx/homeverb')

    def test_dry_run_has_no_subprocess_or_remote_calls(self):
        with patch.object(publish_github, 'run', side_effect=AssertionError('No subprocess allowed')):
            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                self.assertEqual(publish_github.main(['--dry-run']), 0)
            self.assertIn('visibility=PRIVATE', output.getvalue())


if __name__ == '__main__':
    unittest.main()
