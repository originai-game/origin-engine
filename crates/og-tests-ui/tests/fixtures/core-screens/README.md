# Core-screen fixture content

The snippet files supply game-owned content to the generic screens during the
core-screen render test. They are fixture content, not engine templates.

`coast.png` and `logo.png` are original generated fixture art, produced with
Origin Generate (`gpt-image-2.6-sunburst`) and resized with Pillow Lanczos.
The coastal scene has no text; the transparent logo reads NORTHLIGHT.

- Coast generation: `c93b3aeb-9920-44d6-9c72-5487aec4bf8e`, 1536×1024 → 960×540 crop.
- `thumbnail.png`: a 256×256 crop of the same coast for square item/emblem slots.
- Logo generation: `cbacba1a-82ef-4f42-9d4b-582891d64ab8`, 2172×724 → 720×240.

Missing-art behavior is tested separately from populated screen captures.
