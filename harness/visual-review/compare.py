#!/usr/bin/env python3
"""Compare a reviewed component sheet against new renders; not a parity test."""
import argparse
from pathlib import Path
import sys

from PIL import Image


def compare(expected, actual):
    with Image.open(expected) as a, Image.open(actual) as b:
        if a.size != b.size:
            return f'size mismatch: {a.size} != {b.size}'
        # Allow tiny renderer rounding differences in at most ten pixels.
        left_bytes, right_bytes = a.convert('RGBA').tobytes(), b.convert('RGBA').tobytes()
        changed = 0
        for offset in range(0, len(left_bytes), 4):
            delta = max(abs(left_bytes[offset + i] - right_bytes[offset + i]) for i in range(4))
            if delta > 2:
                return f'channel difference exceeds 2: {delta}'
            if delta:
                changed += 1
                if changed > 10:
                    return f'more than 10 pixels changed: {changed}'
        return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('render_dir', type=Path)
    args = parser.parse_args()
    baselines = Path(__file__).resolve().parent / 'baselines'
    for rendered in sorted(args.render_dir.glob('*.png')):
        if not (baselines / rendered.name).exists():
            print(f'{rendered.name}: rendered image has no reviewed baseline', file=sys.stderr)
            return 1
    for baseline in sorted(baselines.glob('*.png')):
        if not (args.render_dir / baseline.name).exists():
            print(f'{baseline.name}: reviewed baseline has no matching render', file=sys.stderr)
            return 1
    for baseline in sorted(baselines.glob('*.png')):
        try:
            error = compare(baseline, args.render_dir / baseline.name)
        except (OSError, ValueError) as exc:
            print(f'{baseline.name}: {exc}', file=sys.stderr)
            return 1
        if error:
            print(f'{baseline.name}: {error}', file=sys.stderr)
            return 1
        print(f'{baseline.stem}: matches reviewed baseline')

    return 0


if __name__ == '__main__':
    sys.exit(main())
