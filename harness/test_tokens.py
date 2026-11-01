import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location('extract_colors', Path(__file__).parent / 'port' / 'extract_colors.py')
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class TokenExtractionTests(unittest.TestCase):
    def test_extracts_complete_schemes_and_rejects_missing_field(self):
        fields = ''.join(f'    {key}: Color(0xFF010203),\n' for key in sorted(MODULE.FIELDS))
        source = ''.join(f'static const {name} = FColors._(\n{fields}  );\n' for name in ('neutralLight', 'neutralDark')).encode()
        first = MODULE.extract(source)
        self.assertEqual(first['schemes']['neutralLight']['background'], 'FF010203')
        self.assertEqual(first, MODULE.extract(source))
        with self.assertRaises(ValueError):
            MODULE.extract(source.replace(b'background:', b'missing:'))


if __name__ == '__main__':
    unittest.main()
