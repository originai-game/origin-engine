# potion-base-8M

minishlab/potion-base-8M (https://huggingface.co/minishlab/potion-base-8M,
revision `bf8b056651a2c21b8d2565580b8569da283cab23`, licence MIT per its
model card), a model2vec static embedding model distilled from
BAAI/bge-base-en-v1.5, with that model's uncased WordPiece vocabulary.
`LICENSE` is model2vec's MIT licence (MinishLab/model2vec), under which the
potion models are published.

Used by `og api find` (`crates/og-idl/src/embed.rs`): the token table cut to
its first 128 (PCA-ordered) dimensions and stored as int8 with one f32 scale
per token in `crates/og-api/search/words.bin`, written by
`crates/og-api/search/make-model.py`, which checks the sha256 of the
downloaded `model.safetensors` and `tokenizer.json`. The WordPiece tokenizer
is a port of the BERT basic and WordPiece tokenizers the model's
`tokenizer.json` names.
