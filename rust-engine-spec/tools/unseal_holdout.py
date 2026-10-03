#!/usr/bin/env python3
"""Decrypt the sealed holdout archive for the holdout runner. Needs the key file; run only where the implementer cannot read the output.

  unseal_holdout.py --archive sealed/holdout-<date>.tar.enc --key-file KEY --out DIR
"""
import argparse, subprocess, tarfile, tempfile, os
ap = argparse.ArgumentParser()
ap.add_argument("--archive", required=True); ap.add_argument("--key-file", required=True); ap.add_argument("--out", required=True)
a = ap.parse_args()
with tempfile.NamedTemporaryFile(delete=False) as t: tmp = t.name
subprocess.check_call(["openssl", "enc", "-d", "-aes-256-cbc", "-pbkdf2", "-in", a.archive, "-out", tmp, "-pass", "file:" + a.key_file])
os.makedirs(a.out, exist_ok=True)
with tarfile.open(tmp) as tf: tf.extractall(a.out)
os.unlink(tmp)
print("unsealed to", a.out)
