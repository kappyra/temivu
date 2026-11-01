#!/usr/bin/env python3
"""Extract Forui neutral Color literals into a source-hash-keyed JSON IR.

This narrow tokenizer is not a Dart parser, analyzer, or semantic translator.
Fail on incomplete source rather than silently generating partial color maps.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re

FIELDS = {'barrier', 'background', 'foreground', 'primary', 'primaryForeground',
          'secondary', 'secondaryForeground', 'muted', 'mutedForeground',
          'destructive', 'destructiveForeground', 'error', 'errorForeground',
          'card', 'border'}


def extract(data):
    text = data.decode('utf-8')
    schemes = {}
    for name in ('neutralLight', 'neutralDark'):
        match = re.search(r'static const ' + name + r' = FColors\._\((.*?)\n  \);', text, re.S)
        if not match:
            raise ValueError(f'missing {name}')
        pairs = re.findall(r'\b(\w+):\s*Color\(0x([0-9A-Fa-f]{8})\)', match.group(1))
        colors = dict(pairs)
        if len(pairs) != len(colors) or set(colors) != FIELDS:
            raise ValueError(f'{name}: unexpected or duplicate color fields')
        schemes[name] = {key: colors[key].upper() for key in sorted(colors)}
    return {'schema': 1, 'source_sha256': hashlib.sha256(data).hexdigest(), 'schemes': schemes}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    result = extract(args.source.read_bytes())
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + '\n')


if __name__ == '__main__':
    main()
