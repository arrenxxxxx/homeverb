#!/usr/bin/env python3
"""Validate English Conventional Commit messages without contacting a remote."""
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

from validate_repository import non_latin_letters

ROOT = Path(__file__).resolve().parents[1]
HEADER = re.compile(r'^(feat|fix|docs|test|refactor|perf|ci|build|chore|style|revert)(\([a-z0-9][a-z0-9/-]*\))?!?: \S.*$')


def validate_message(message: str) -> list[str]:
    lines = message.strip().splitlines()
    if not lines:
        return ['empty commit message']
    errors: list[str] = []
    if not HEADER.fullmatch(lines[0]):
        errors.append('subject must use an approved Conventional Commit type and an English summary')
    if len(lines[0]) > 100:
        errors.append('subject exceeds 100 characters')
    if non_latin_letters(message):
        errors.append('commit subject and body must be English')
    if len(lines) > 1 and lines[1].strip():
        errors.append('separate the subject from the body with a blank line')
    return errors


def run_git(*args: str) -> str:
    result = subprocess.run(['git', '-C', str(ROOT), *args], text=True, capture_output=True, check=True)
    return result.stdout


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--all', action='store_true', help='Validate all local branch and tag history.')
    mode.add_argument('--range', dest='rev_range', help='Validate an explicit Git revision range.')
    mode.add_argument('--message-file', type=Path, help='Validate a Git commit-message file.')
    mode.add_argument('--pr-title', action='store_true', help='Validate the PR_TITLE environment variable.')
    args = parser.parse_args()
    messages: list[tuple[str, str]]
    if args.message_file:
        raw = args.message_file.read_text(encoding='utf-8')
        # Git strips comment lines in the default commit-message cleanup mode.
        messages = [('new commit', '\n'.join(line for line in raw.splitlines() if not line.startswith('#')))]
    elif args.pr_title:
        messages = [('pull request title', os.environ.get('PR_TITLE', ''))]
    else:
        revision = '--all' if args.all else args.rev_range
        if not revision or (not args.all and revision.startswith('-')):
            parser.error('revision range must not be a Git option')
        shas = run_git('rev-list', revision, '--').splitlines()
        messages = [(sha[:12], run_git('show', '-s', '--format=%B', sha)) for sha in shas]
    errors = [f'{name}: {error}' for name, message in messages for error in validate_message(message)]
    if errors:
        print('\n'.join(errors), file=sys.stderr)
        return 1
    print(f'PASS: {len(messages)} English Conventional Commit message(s).')
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, subprocess.CalledProcessError) as exc:
        print(f'FAIL: cannot validate messages: {exc}', file=sys.stderr)
        raise SystemExit(2)
