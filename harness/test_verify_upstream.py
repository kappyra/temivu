import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).parent / 'licenses' / 'verify_upstream.py'


class UpstreamProvenanceTests(unittest.TestCase):
    def test_hash_revision_and_paths_are_checked(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            cache = root / 'cache'
            licenses = root / 'licenses'
            cache.mkdir()
            licenses.mkdir()
            (licenses / 'LICENSE').write_text('sample text')
            file = {'path': 'LICENSE', 'sha256': hashlib.sha256(b'sample text').hexdigest()}
            metadata, records = [], []
            for i in range(59):
                name, version = ('dispatch', '0.2.0') if i == 0 else (f'crate-{i}', '1.0.0')
                crate = cache / name
                crate.mkdir()
                (crate / 'Cargo.toml').write_text('')
                (crate / '.cargo_vcs_info.json').write_text(json.dumps({'git': {'sha1': 'a' * 40}}))
                metadata.append({'name': name, 'version': version, 'repository': 'repo', 'manifest_path': f'/cache/{name}/Cargo.toml'})
                records.append({'name': name, 'version': version, 'repository': 'repo', 'vcs_revision': 'a' * 40, 'files': [] if i == 0 else [file.copy()]})
            provenance = root / 'provenance.json'
            command = [sys.executable, str(SCRIPT), '--cache', str(cache), '--licenses', str(licenses), '--provenance', str(provenance)]
            def run():
                provenance.write_text(json.dumps({'packages': records}))
                return subprocess.run(command, input=json.dumps({'packages': metadata}), capture_output=True, text=True, check=False)
            valid = run()
            self.assertEqual(valid.returncode, 0, valid.stderr)
            records[1]['files'][0]['sha256'] = '0' * 64
            self.assertNotEqual(run().returncode, 0)
            records[1]['files'][0] = file.copy()
            records[1]['files'][0]['path'] = '../outside'
            self.assertNotEqual(run().returncode, 0)
            records[1]['files'][0] = file.copy()
            outside = root / 'outside'
            outside.write_text('sample text')
            (licenses / 'LINK').symlink_to(outside)
            records[1]['files'][0]['path'] = 'LINK'
            self.assertNotEqual(run().returncode, 0)
            records[1]['files'][0] = file.copy()
            records[1]['vcs_revision'] = 'b' * 40
            self.assertNotEqual(run().returncode, 0)
            records[1]['vcs_revision'] = 'a' * 40
            records.pop()
            self.assertNotEqual(run().returncode, 0)


if __name__ == '__main__':
    unittest.main()
