import gzip, hashlib, json, tempfile, unittest
from pathlib import Path
from unittest.mock import patch
import catalog

class CatalogTests(unittest.TestCase):
    def test_checksum_and_record_validation_precede_atomic_replacement(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);asset=root/'catalog.gz';destination=root/'nested/cards.jsonl'
            data=b'{"schema_version":191}\n'
            compressed=gzip.compress(data,mtime=0);asset.write_bytes(compressed)
            lock=dict(format_version=1,url=asset.as_uri(),sha256=hashlib.sha256(compressed).hexdigest(),records=1,schema_version=191)
            self.assertEqual(catalog.fetch(destination,lock),1)
            self.assertEqual(destination.read_bytes(),data)
            for invalid in [dict(lock,sha256='0'*64),dict(lock,records=2),dict(lock,schema_version=190)]:
                with self.assertRaises(ValueError):catalog.fetch(destination,invalid)
                self.assertEqual(destination.read_bytes(),data)
            with patch.object(catalog.os,'replace',side_effect=OSError('read only')):
                with self.assertRaises(OSError):catalog.fetch(destination,lock)
            self.assertEqual(destination.read_bytes(),data)
            self.assertEqual(list(destination.parent.iterdir()),[destination])

if __name__=='__main__': unittest.main()
