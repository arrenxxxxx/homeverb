#!/usr/bin/env python3
"""Check relative Markdown targets and JSON syntax, without network access."""
from __future__ import annotations
import json
import re
import sys
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
SKIP_PARTS = {".git", ".venv", "venv", "__pycache__", "node_modules", "target", "build"}
LINK = re.compile(r"!?\[[^\]\n]*\]\(([^\s)]+)(?:\s+[^)]*)?\)")

def main() -> int:
    errors: list[str] = []
    markdown = sorted(p for p in ROOT.rglob("*.md") if not set(p.relative_to(ROOT).parts) & SKIP_PARTS)
    targets = 0
    for path in markdown:
        text = path.read_text(encoding="utf-8")
        # Fenced examples are not actual Markdown hyperlinks.
        clean = re.sub(r"```.*?```", "", text, flags=re.S)
        for value in LINK.findall(clean):
            value = value.strip("<>")
            parsed = urlsplit(value)
            if parsed.scheme or value.startswith(("#", "//")):
                continue
            target = (path.parent / unquote(parsed.path)).resolve()
            targets += 1
            if not target.is_relative_to(ROOT):
                errors.append(f"{path.relative_to(ROOT)}: target escapes repository: {value}")
            elif not target.exists():
                errors.append(f"{path.relative_to(ROOT)}: missing target: {value}")
        if text.count("```") % 2:
            errors.append(f"{path.relative_to(ROOT)}: unbalanced fenced code block")
    json_paths = sorted(p for p in ROOT.rglob("*.json") if not set(p.relative_to(ROOT).parts) & SKIP_PARTS)
    for path in json_paths:
        try:
            json.loads(path.read_text(encoding="utf-8"))
        except (OSError, ValueError) as exc:
            errors.append(f"{path.relative_to(ROOT)}: {exc}")
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print(f"PASS: {len(markdown)} Markdown files; {targets} local links; {len(json_paths)} JSON files.")
    print("External URLs, heading anchors, runtime code and physical devices were not tested.")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
