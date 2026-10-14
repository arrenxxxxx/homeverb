"""Offline checks for the owner-approved project identity."""
from pathlib import Path
import json
import re
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
import publish_github


class ProjectNamingTests(unittest.TestCase):
    def test_readme_uses_approved_brand(self):
        text = (ROOT / 'README.md').read_text(encoding='utf-8')
        self.assertTrue(text.startswith('# HomeVerb\n'))
        self.assertIn('owner-approved project name', text)
        self.assertIn('One language for your appliances. No daemon required.', text)

    def test_publishing_target_uses_approved_slug(self):
        self.assertEqual(publish_github.DEFAULT_REPO, 'arrenxxxxx/homeverb')
        text = (ROOT / 'docs/publishing.md').read_text(encoding='utf-8')
        self.assertIn('`arrenxxxxx/homeverb`', text)

    def test_mcp_configuration_uses_approved_command(self):
        text = (ROOT / 'docs/mcp.md').read_text(encoding='utf-8')
        match = re.search(r'```json\n(.*?)\n```', text, flags=re.S)
        self.assertIsNotNone(match)
        data = json.loads(match.group(1))
        server = data['mcpServers']['homeverb']
        self.assertEqual(server['command'], 'homeverb')
        self.assertEqual(server['args'][:2], ['mcp', '--stdio'])

    def test_diagram_and_decision_use_approved_brand(self):
        svg = (ROOT / 'docs/diagrams/architecture-overview.svg').read_text(encoding='utf-8')
        decision = (ROOT / 'docs/adr/0006-project-name.md').read_text(encoding='utf-8')
        self.assertIn('HomeVerb architecture', svg)
        self.assertIn('HOMEVERB / ARCHITECTURE BASELINE', svg)
        self.assertIn('Status: accepted by the project owner.', decision)
        self.assertIn('`homeverb-core`', decision)


if __name__ == '__main__':
    unittest.main()
