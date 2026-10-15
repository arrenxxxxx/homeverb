"""Load the native UI schemas with local-only, explicit reference resolution."""

from __future__ import annotations

import json
from pathlib import Path

from jsonschema import Draft202012Validator
from referencing import Registry, Resource
from referencing.exceptions import NoSuchResource

from validate_contracts import checked_formats

ROOT = Path(__file__).resolve().parents[1]


def load(relative: str):
    return json.loads((ROOT / relative).read_text(encoding='utf-8'))


def reject_remote(uri):
    raise NoSuchResource(ref=uri)


def validator(name: str, definition: str | None = None):
    registry = Registry(retrieve=reject_remote)
    for path in sorted((ROOT / 'schemas').glob('*.schema.json')):
        schema = json.loads(path.read_text(encoding='utf-8'))
        Draft202012Validator.check_schema(schema)
        uri = 'urn:homeverb:schema:' + path.name.removesuffix('.schema.json')
        registry = registry.with_resource(uri, Resource.from_contents(schema))
    uri = 'urn:homeverb:schema:' + name
    if definition is not None:
        uri += '#/$defs/' + definition
    return Draft202012Validator({'$ref': uri}, registry=registry, format_checker=checked_formats())
