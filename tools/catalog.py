#!/usr/bin/env python3
"""Fetch the immutable catalog pinned by catalog.lock.json, verifying before use."""
import argparse, gzip, hashlib, json, os, tempfile, urllib.request
from pathlib import Path

def fetch(destination, lock):
    if lock['format_version'] != 1:
        raise ValueError('Unsupported catalog lock version')
    with urllib.request.urlopen(lock['url'], timeout=120) as response:
        compressed=response.read()
    if hashlib.sha256(compressed).hexdigest() != lock['sha256']:
        raise ValueError('Catalog SHA-256 mismatch; output left unchanged')
    data=gzip.decompress(compressed)
    records=data.splitlines()
    if len(records) != lock['records']:
        raise ValueError('Catalog record count mismatch')
    for line in records:
        if json.loads(line)['schema_version'] != lock['schema_version']:
            raise ValueError('Catalog schema version mismatch')
    destination.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary=tempfile.mkstemp(prefix=destination.name+'.',dir=destination.parent)
    try:
        with os.fdopen(fd,'wb') as output: output.write(data)
        os.replace(temporary,destination)
    finally:
        if os.path.exists(temporary): os.unlink(temporary)
    return len(records)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['fetch'])
    parser.add_argument('destination',type=Path)
    parser.add_argument('--lock',type=Path,default=Path(__file__).resolve().parents[1]/'catalog.lock.json')
    args=parser.parse_args()
    print(f"Fetched {fetch(args.destination,json.loads(args.lock.read_text()))} IR cards")

if __name__=='__main__': main()
