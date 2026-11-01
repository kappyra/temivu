import copy
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

PORT = Path(__file__).parent / 'port'
sys.path.insert(0, str(PORT))
SPEC = importlib.util.spec_from_file_location('verify_sources', PORT / 'verify_sources.py')
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def english_messages():
    """A minimal valid English ARB message map for every extracted key."""
    extract = MODULE.extract_localizations
    data = {}
    for key in extract.__globals__['KEYS']:
        if key in extract.__globals__['PLACEHOLDERS']:
            data[key] = ' '.join('{%s}' % name for name in extract.__globals__['PLACEHOLDERS'][key])
        else:
            data[key] = key
    data.update({
        'paginationPreviousSemanticsLabel': 'Previous',
        'paginationNextSemanticsLabel': 'Next',
        'paginationPageSemanticsLabel': 'Page {page}',
        'progressSemanticsLabel': 'Loading',
    })
    return data


class SourceVerificationTests(unittest.TestCase):
    def test_dirty_source_and_tampered_ir_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'forui/lib/src/theme').mkdir(parents=True)
            (root / 'forui/lib/l10n').mkdir(parents=True)
            fields = ''.join(f'    {key}: Color(0xFF010203),\n' for key in sorted(MODULE.extract_colors.__globals__['FIELDS']))
            colors = ''.join(f'static const {name} = FColors._(\n{fields}  );\n' for name in ('neutralLight', 'neutralDark'))
            color_path = root / MODULE.PATHS[0]
            color_path.write_text(colors)
            arb = root / 'forui/lib/l10n/f_en.arb'
            arb.write_text(json.dumps(english_messages()))
            env = dict(os.environ, GIT_CONFIG_NOSYSTEM='1', GIT_CONFIG_GLOBAL=os.devnull)
            def git(*args):
                return subprocess.run(['git', '-c', 'user.name=Test', '-c', 'user.email=test@example.invalid', '-C', str(root), *args], env=env, check=True, capture_output=True, text=True).stdout.strip()
            git('init', '-q')
            git('add', 'forui')
            git('commit', '-qm', 'fixture')
            revision = git('rev-parse', 'HEAD')
            color_ir = MODULE.extract_colors(color_path.read_bytes())
            locale_ir = MODULE.extract_localizations(arb.parent)
            self.assertTrue(MODULE.verify(root, color_ir, locale_ir, revision))
            altered = copy.deepcopy(locale_ir)
            altered['locales']['en']['progressSemanticsLabel'] = 'Changed'
            with self.assertRaisesRegex(ValueError, 'stored IR'):
                MODULE.verify(root, color_ir, altered, revision)
            arb.write_text(arb.read_text() + '\n')
            with self.assertRaisesRegex(ValueError, 'input files differ'):
                MODULE.verify(root, color_ir, locale_ir, revision)
            with self.assertRaisesRegex(ValueError, 'unavailable'):
                MODULE.verify(root / 'missing', color_ir, locale_ir, revision)


if __name__ == '__main__':
    unittest.main()
