#!/usr/bin/env python3
"""Writes words.bin, the static word-embedding model `og api find` ranks with
(crates/og-idl/src/embed.rs reads it), from minishlab/potion-base-8M
(model2vec, MIT; third_party/potion-base-8M/).

    python3 make-model.py <potion-base-8M dir> <out words.bin> [dim]

The source directory holds the model's model.safetensors and tokenizer.json
from https://huggingface.co/minishlab/potion-base-8M at revision
bf8b056651a2c21b8d2565580b8569da283cab23 (sha256 below). Each token's vector
keeps its first `dim` components (the model is PCA-ordered, so they carry
the most variance) and is stored as int8 with a per-token f32 scale, which
keeps the token's norm: model2vec bakes its Zipf weighting into the norms.

Layout, little-endian: b"OGWORDS1", u32 tokens, u32 dim, u32 unk id; per
token a u8 length and its UTF-8 bytes; tokens f32 scales; tokens x dim int8.
"""

import hashlib, json, struct, sys

import numpy as np
from safetensors.numpy import load_file

SHA256 = {
    "model.safetensors": "f65d0f325faadc1e121c319e2faa41170d3fa07d8c89abd48ca5358d9a223de2",
    "tokenizer.json": "e67e803f624fb4d67dea1c730d06e1067e1b14d830e2c2202569e3ef0f70bb50",
}

src, out = sys.argv[1], sys.argv[2]
dim = int(sys.argv[3]) if len(sys.argv) > 3 else 128
for name, want in SHA256.items():
    got = hashlib.sha256(open(f"{src}/{name}", "rb").read()).hexdigest()
    if got != want:
        sys.exit(f"{name}: sha256 {got}, expected {want}")

tok = json.load(open(f"{src}/tokenizer.json", encoding="utf-8"))
vocab = tok["model"]["vocab"]
words = [None] * len(vocab)
for w, i in vocab.items():
    words[i] = w
emb = load_file(f"{src}/model.safetensors")["embeddings"][:, :dim].astype(np.float32)
assert emb.shape[0] == len(words)
scale = np.abs(emb).max(axis=1) / 127.0
scale[scale == 0] = 1.0
q = np.clip(np.round(emb / scale[:, None]), -127, 127).astype(np.int8)

with open(out, "wb") as f:
    f.write(b"OGWORDS1")
    f.write(struct.pack("<III", len(words), dim, vocab[tok["model"]["unk_token"]]))
    for w in words:
        b = w.encode("utf-8")
        f.write(struct.pack("<B", len(b)))
        f.write(b)
    f.write(scale.astype("<f4").tobytes())
    f.write(q.tobytes())
print(f"{out}: {len(words)} tokens x {dim}")
