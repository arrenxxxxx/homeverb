#!/usr/bin/env python3
"""Create a private GitHub repository and publish this existing Git history.

Requires locally authenticated GitHub CLI. Never embeds a token, force-pushes,
changes repository visibility, or adopts an unrelated existing repository.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_REPO = 'arrenxxxxx/homeverb'
DESCRIPTION = ('Cross-brand appliance control with an embedded core, CLI, mobile app, '
               'and local MCP. No persistent service.')


class PublishError(RuntimeError):
    """A prerequisite, safety check, or remote operation failed."""


def validate_repo_name(value: str) -> tuple[str, str]:
    if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9-]{0,38}/[A-Za-z0-9_.-]{1,100}', value):
        raise PublishError('Repository must be a GitHub OWNER/NAME, not a URL or command option.')
    owner, name = value.split('/', 1)
    if name in {'.', '..'}:
        raise PublishError('Invalid repository name.')
    return owner, name


def origin_matches(url: str, repo: str) -> bool:
    values = {f'https://github.com/{repo}', f'https://github.com/{repo}.git',
              f'git@github.com:{repo}', f'git@github.com:{repo}.git',
              f'ssh://git@github.com/{repo}', f'ssh://git@github.com/{repo}.git'}
    return url.strip().lower() in {value.lower() for value in values}


def run(*args: str, allow_failure: bool = False) -> subprocess.CompletedProcess[str]:
    env = dict(os.environ, GH_HOST='github.com', GIT_TERMINAL_PROMPT='0')
    result = subprocess.run(args, cwd=ROOT, env=env, capture_output=True, text=True, check=False)
    if result.returncode and not allow_failure:
        # No credentials are passed as arguments; gh manages authentication locally.
        detail = result.stderr.strip() or result.stdout.strip() or 'No diagnostic output.'
        raise PublishError(f'{args[0]} {args[1]} failed (exit {result.returncode}): {detail}')
    return result


def gh_json(*args: str) -> dict:
    try:
        value = json.loads(run('gh', *args).stdout)
    except json.JSONDecodeError as exc:
        raise PublishError('GitHub CLI returned invalid JSON.') from exc
    if not isinstance(value, dict):
        raise PublishError('Expected a JSON object from GitHub CLI.')
    return value


def require_private(metadata: dict, repo: str) -> None:
    if str(metadata.get('nameWithOwner', '')).lower() != repo.lower():
        raise PublishError('GitHub returned a different repository; refusing to continue.')
    if metadata.get('visibility') != 'PRIVATE':
        raise PublishError('The target is not PRIVATE. No files will be pushed.')


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', default=DEFAULT_REPO, help='Target OWNER/NAME; defaults to the requested private project.')
    parser.add_argument('--dry-run', action='store_true', help='Print the plan without GitHub access or filesystem changes.')
    args = parser.parse_args(argv)
    owner, _ = validate_repo_name(args.repo)
    if args.dry_run:
        print(f'DRY RUN: target={args.repo}; visibility=PRIVATE; branch=main')
        print('Require a clean existing Git checkout and an authenticated matching owner.')
        print('Create a private repository, or resume only with an exact matching origin.')
        print('Verify private visibility; push without force; verify the main commit; apply English labels.')
        print('No remote calls or local changes were performed.')
        return 0

    for program in ('git', 'gh'):
        if shutil.which(program) is None:
            raise PublishError(f'Required executable not found: {program}. See docs/publishing.md.')
    top = Path(run('git', 'rev-parse', '--show-toplevel').stdout.strip()).resolve()
    if top != ROOT.resolve():
        raise PublishError('Run from the distributed Git checkout, not an unrelated parent repository.')
    if run('git', 'branch', '--show-current').stdout.strip() != 'main':
        raise PublishError('Check out the intended main branch before publishing.')
    if run('git', 'status', '--porcelain').stdout.strip():
        raise PublishError('The working tree or index has uncommitted changes; review and commit them first.')
    local_sha = run('git', 'rev-parse', 'HEAD').stdout.strip()
    for script, arguments in [('validate_docs.py', []), ('validate_repository.py', []), ('validate_commits.py', ['--all'])]:
        print(run(sys.executable, str(ROOT / 'scripts' / script), *arguments).stdout.strip())
    run('gh', 'auth', 'status', '--hostname', 'github.com')
    login = run('gh', 'api', '--hostname', 'github.com', 'user', '--jq', '.login').stdout.strip()
    if login.lower() != owner.lower():
        raise PublishError(f'Authenticated owner is {login}, not {owner}; refusing to publish to another account.')

    origin = run('git', 'remote', 'get-url', 'origin', allow_failure=True)
    if origin.returncode == 0:
        if not origin_matches(origin.stdout, args.repo):
            raise PublishError('Existing origin is unrelated to this target. No remote was changed.')
        print('Using the existing, exact target origin after verification.')
    else:
        print(f'Creating PRIVATE repository {args.repo}.')
        run('gh', 'repo', 'create', args.repo, '--private', '--source', str(ROOT),
            '--remote', 'origin', '--description', DESCRIPTION)
        created_origin = run('git', 'remote', 'get-url', 'origin').stdout
        if not origin_matches(created_origin, args.repo):
            raise PublishError('Created origin does not match the requested target.')

    metadata = gh_json('repo', 'view', args.repo, '--json', 'nameWithOwner,visibility,url')
    require_private(metadata, args.repo)
    push_urls = run('git', 'remote', 'get-url', '--push', '--all', 'origin').stdout.splitlines()
    if len(push_urls) != 1 or not origin_matches(push_urls[0], args.repo):
        raise PublishError('Unexpected push URL configuration; refusing to push.')
    print('Private visibility confirmed. Pushing main without force.')
    run('git', '-c', 'credential.helper=', '-c', 'credential.helper=!gh auth git-credential',
        'push', '--set-upstream', 'origin', 'main')
    ref = gh_json('api', '--hostname', 'github.com', f'repos/{args.repo}/git/ref/heads/main')
    if ref.get('object', {}).get('sha') != local_sha:
        raise PublishError('Remote main SHA does not match local HEAD; publication is not verified.')
    run('gh', 'repo', 'edit', args.repo, '--default-branch', 'main')

    labels = json.loads((ROOT / '.github' / 'labels.json').read_text(encoding='utf-8'))
    for label in labels:
        run('gh', 'label', 'create', label['name'], '--repo', args.repo,
            '--color', label['color'], '--description', label['description'], '--force')
    final = gh_json('repo', 'view', args.repo, '--json', 'nameWithOwner,visibility,url,defaultBranchRef')
    require_private(final, args.repo)
    if final.get('defaultBranchRef', {}).get('name') != 'main':
        raise PublishError('Default branch could not be verified as main.')
    print(json.dumps({'repository': final['nameWithOwner'], 'url': final['url'],
                      'visibility': final['visibility'], 'branch': 'main', 'commit': local_sha,
                      'labels_applied': len(labels), 'remote_ci': 'not checked'}, indent=2))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (PublishError, OSError, ValueError) as exc:
        print(f'PUBLISH FAILED: {exc}', file=sys.stderr)
        print('Any successfully created private repository is left intact; no deletion or force push was attempted.', file=sys.stderr)
        raise SystemExit(1)
