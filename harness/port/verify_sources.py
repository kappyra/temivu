#!/usr/bin/env python3
"""Verify generated token IR against a clean, pinned Forui checkout."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

from extract_colors import extract as extract_colors
from extract_localizations import extract as extract_localizations

ROOT = Path(__file__).resolve().parents[2]
PINNED = 'eea1e8e651d263f14ef43897ccbb7a2576549da5'
PATHS = ('forui/lib/src/theme/colors.dart', 'forui/lib/l10n')


def verify(source, colors_ir, locales_ir, expected_revision=PINNED):
    source = Path(source)
    if not source.is_dir():
        raise ValueError('pinned Forui checkout is unavailable')
    # Do not run a repository-controlled fsmonitor command while inspecting it.
    command = ['git', '-c', 'core.fsmonitor=false', '-C', str(source)]
    env = dict(os.environ, GIT_CONFIG_NOSYSTEM='1', GIT_TERMINAL_PROMPT='0')
    revision = subprocess.run(command + ['rev-parse', 'HEAD'], env=env, check=True, capture_output=True, text=True).stdout.strip()
    if revision != expected_revision:
        raise ValueError('Forui checkout revision does not match the pin')
    dirty = subprocess.run(command + ['status', '--porcelain', '--untracked-files=all', '--', *PATHS], env=env, check=True, capture_output=True, text=True).stdout
    if dirty:
        raise ValueError('Forui input files differ from the pinned checkout')
    actual_colors = extract_colors((source / PATHS[0]).read_bytes())
    actual_locales = extract_localizations(source / PATHS[1])
    if actual_colors != colors_ir or actual_locales != locales_ir:
        raise ValueError('stored IR differs from extracted pinned source')
    return True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=Path(os.environ.get('TEMIVU_UPSTREAM', 'forui-source')))
    args = parser.parse_args()
    try:
        verify(args.source, json.loads((ROOT / 'harness/port/neutral-colors.json').read_text()),
               json.loads((ROOT / 'harness/port/localizations.json').read_text()))
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f'Source verification failed: {error}', file=sys.stderr)
        return 1
    print(f'IR matches clean Forui revision {PINNED}.')
    return 0


if __name__ == '__main__':
    sys.exit(main())
