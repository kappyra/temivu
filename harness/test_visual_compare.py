"""Tests for the reviewed component-sheet comparison limits."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

try:
    from PIL import Image
except ImportError:
    Image = None

MODULE_PATH = Path(__file__).resolve().parent / 'visual-review' / 'compare.py'
if Image is not None:
    spec = importlib.util.spec_from_file_location('visual_compare', MODULE_PATH)
    compare = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(compare)


@unittest.skipIf(Image is None, 'visual comparison requires Pillow')
class VisualCompareTests(unittest.TestCase):
    def test_pixel_and_size_limits(self):
        with tempfile.TemporaryDirectory() as directory:
            expected, actual = (Path(directory) / name for name in ('expected.png', 'actual.png'))
            Image.new('RGBA', (12, 1), (10, 20, 30, 255)).save(expected)
            image = Image.open(expected).copy()
            for x in range(10):
                image.putpixel((x, 0), (12, 20, 30, 255))
            image.save(actual)
            self.assertIsNone(compare.compare(expected, actual))
            image.putpixel((10, 0), (12, 20, 30, 255))
            image.save(actual)
            self.assertIn('more than 10', compare.compare(expected, actual))
            image.putpixel((0, 0), (13, 20, 30, 255))
            image.save(actual)
            self.assertIn('exceeds 2', compare.compare(expected, actual))
            Image.new('RGBA', (1, 1)).save(actual)
            self.assertIn('size mismatch', compare.compare(expected, actual))


if __name__ == '__main__':
    unittest.main()
