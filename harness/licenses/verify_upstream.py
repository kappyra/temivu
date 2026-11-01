#!/usr/bin/env python3
"""Check recorded upstream license-file hashes and cached crate VCS metadata.

This is provenance verification, not a license, notice or legal compliance audit.
"""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import sys

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--cache', required=True, type=Path)
parser.add_argument('--licenses', required=True, type=Path)
parser.add_argument('--provenance', required=True, type=Path)
args = parser.parse_args()
cache = args.cache.resolve(strict=True)
licenses = args.licenses.resolve(strict=True)
metadata = json.load(sys.stdin)
recorded = json.loads(args.provenance.read_text())
packages = {}
for package in metadata['packages']:
    key = (package['name'], package['version'])
    if key in packages:
        raise ValueError(f'ambiguous package identity: {key}')
    packages[key] = package
seen = set()
unresolved = []
for item in recorded['packages']:
    key = (item['name'], item['version'])
    if key in seen or key not in packages:
        raise ValueError(f'duplicate or missing package: {key}')
    seen.add(key)
    package = packages[key]
    if package['repository'] != item['repository']:
        raise ValueError(f'repository changed: {key}')
    revision = item['vcs_revision']
    if not re.fullmatch(r'[a-f0-9]{40}', revision):
        raise ValueError(f'invalid revision: {key}')
    path = PurePosixPath(package['manifest_path'])
    if len(path.parts) < 3 or path.parts[:2] != ('/', 'cache') or '..' in path.parts:
        raise ValueError(f'unexpected package path: {key}')
    manifest = cache.joinpath(*path.parts[2:]).resolve(strict=True)
    if not manifest.is_relative_to(cache) or manifest.name != 'Cargo.toml':
        raise ValueError(f'package path escapes cache: {key}')
    vcs = (manifest.parent / '.cargo_vcs_info.json').resolve(strict=True)
    if not vcs.is_relative_to(manifest.parent):
        raise ValueError(f'VCS metadata escapes package: {key}')
    if json.loads(vcs.read_text())['git']['sha1'] != revision:
        raise ValueError(f'VCS revision changed: {key}')
    if not item['files']:
        unresolved.append(key)
    for file in item['files']:
        rel = PurePosixPath(file['path'])
        if rel.is_absolute() or '..' in rel.parts:
            raise ValueError(f'unsafe license path: {key}')
        path = licenses.joinpath(*rel.parts).resolve(strict=True)
        if not path.is_relative_to(licenses) or not path.is_file():
            raise ValueError(f'license path escapes directory: {key}')
        if hashlib.sha256(path.read_bytes()).hexdigest() != file['sha256']:
            raise ValueError(f'license file hash mismatch: {key}')
if len(seen) != 59 or unresolved != [('dispatch', '0.2.0')]:
    raise ValueError('provenance coverage or unresolved package list changed; review it explicitly')
print(f'Upstream provenance matches cached metadata for {len(seen)} packages; dispatch 0.2.0 has no collected file. This is not a compliance audit.')
