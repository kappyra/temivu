import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('extract_localizations', Path(__file__).parent / 'port' / 'extract_localizations.py')
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def english_messages():
    """A minimal valid English ARB message map for every extracted key."""
    data = {}
    for key in MODULE.KEYS:
        if key in MODULE.PLACEHOLDERS:
            data[key] = ' '.join('{%s}' % name for name in MODULE.PLACEHOLDERS[key])
        else:
            data[key] = key
    data.update({
        'paginationPreviousSemanticsLabel': 'Previous',
        'paginationNextSemanticsLabel': 'Next',
        'paginationPageSemanticsLabel': 'Page {page}',
        'progressSemanticsLabel': 'Loading',
    })
    return data


class LocalizationExtractionTests(unittest.TestCase):
    def test_extraction_accepts_all_keys_and_is_deterministic(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'f_en.arb').write_text(json.dumps(english_messages()))
            first = MODULE.extract(root)
            self.assertEqual(first['locales']['en']['paginationPageSemanticsLabel'], 'Page {page}')
            self.assertEqual(first['locales']['en']['barrierOnTapHint'], '{modalRouteContentName}')
            self.assertEqual(set(first['locales']['en']), set(MODULE.KEYS))
            self.assertEqual(first, MODULE.extract(root))

    def test_untranslated_key_falls_back_to_english(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'f_en.arb').write_text(json.dumps(english_messages()))
            # Every non-English `barrierOnTapHint` is not an ARB template, so it
            # is deliberately not extracted; English supplies the fallback.
            untranslated = english_messages()
            untranslated.pop('barrierOnTapHint')
            (root / 'f_fr.arb').write_text(json.dumps(untranslated))
            extracted = MODULE.extract(root)
            self.assertNotIn('barrierOnTapHint', extracted['locales']['fr'])

    def test_reordered_translated_placeholders_are_accepted(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            messages = english_messages()
            messages['bottomNavigationBarTabSemanticsLabel'] = '{tabCount} 個のタブのうち {tabIndex} 個目'
            (root / 'f_en.arb').write_text(json.dumps(messages))
            extracted = MODULE.extract(root)
            self.assertEqual(
                extracted['locales']['en']['bottomNavigationBarTabSemanticsLabel'],
                '{tabCount} 個のタブのうち {tabIndex} 個目',
            )

    def test_rejects_plural_select_and_broken_templates(self):
        invalid = {
            'plural message': ('paginationPageSemanticsLabel', '{page, plural, other{pages}}'),
            'select message': (
                'bottomNavigationBarTabSemanticsLabel',
                '{tabCount, select, other{{tabIndex}}}',
            ),
            'missing placeholder': ('bottomNavigationBarTabSemanticsLabel', 'Tab {tabIndex}'),
            'unexpected plain-string brace': ('progressSemanticsLabel', 'Loading {progress}'),
            'nested brace': ('paginationPageSemanticsLabel', 'Page {{page}}'),
            'dart interpolation leak': ('barrierOnTapHint', 'Close $modalRouteContentName'),
        }
        for name, (key, value) in invalid.items():
            with self.subTest(name), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                messages = english_messages()
                messages[key] = value
                (root / 'f_en.arb').write_text(json.dumps(messages))
                with self.assertRaises(ValueError):
                    MODULE.extract(root)

    def test_rejects_non_string_and_missing_english_messages(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            messages = english_messages()
            messages['selectHint'] = 42
            (root / 'f_en.arb').write_text(json.dumps(messages))
            with self.assertRaises(ValueError):
                MODULE.extract(root)
            messages = english_messages()
            messages.pop('selectHint')
            (root / 'f_en.arb').write_text(json.dumps(messages))
            with self.assertRaises(ValueError):
                MODULE.extract(root)

    def test_rejects_missing_arb_files(self):
        with tempfile.TemporaryDirectory() as temporary:
            with self.assertRaises(ValueError):
                MODULE.extract(Path(temporary))


if __name__ == '__main__':
    unittest.main()
