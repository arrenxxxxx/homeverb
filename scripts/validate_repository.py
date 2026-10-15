#!/usr/bin/env python3
"""Validate English-only text conventions and basic repository hygiene offline."""
from __future__ import annotations

import re
import sys
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SKIP_PARTS = {'.git', '.idea', '.venv', 'venv', '__pycache__', 'node_modules', 'target', 'build'}
TEXT_SUFFIXES = {'.md', '.mmd', '.svg', '.json', '.yml', '.yaml', '.py', '.sh', '.txt', '.toml', '.rs', '.h', '.mjs', '.java', '.xml', '.pro'}
TEXT_NAMES = {'.gitignore', '.gitattributes', '.editorconfig', 'commit-msg'}


def non_latin_letters(text: str) -> bool:
    """A conservative language regression guard, not an English grammar check."""
    return any(c.isalpha() and not c.isascii() and 'LATIN' not in unicodedata.name(c, '') for c in text)


def check_file(path: Path) -> list[str]:
    errors: list[str] = []
    try:
        text = path.read_text(encoding='utf-8')
    except (OSError, UnicodeError) as exc:
        return [f'{path.name}: cannot read UTF-8 text: {exc}']
    for number, line in enumerate(text.splitlines(), 1):
        translation_catalog = path == ROOT / 'packages/presentation/locales/zh-CN.json'
        if non_latin_letters(line) and not translation_catalog:
            errors.append(f'{path.name}:{number}: non-Latin letters violate the English documentation policy')
    if text and not text.endswith('\n'):
        errors.append(f'{path.name}: missing final newline')
    return errors


def main() -> int:
    errors: list[str] = []
    checked = 0
    for path in sorted(ROOT.rglob('*')):
        if not path.is_file() or set(path.relative_to(ROOT).parts) & SKIP_PARTS:
            continue
        rel = path.relative_to(ROOT)
        if path.is_symlink():
            errors.append(f'{rel}: unexpected symlink')
            continue
        if path.suffix in TEXT_SUFFIXES or path.name in TEXT_NAMES:
            checked += 1
            errors.extend(f'{rel.parent}/{error}' for error in check_file(path))
        if path.name == '.env' or path.suffix in {'.db', '.sqlite', '.sqlite3', '.pem', '.p12'}:
            errors.append(f'{rel}: runtime or credential-bearing file must not be distributed')
    if errors:
        print('\n'.join(errors), file=sys.stderr)
        return 1
    print(f'PASS: {checked} maintained text files satisfy language and basic hygiene checks.')
    print('This is not a grammar checker, comprehensive secret scanner, or security audit.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
