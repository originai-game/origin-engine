# Long sounds cooked in chunks (P4, cook-anim)

Apple M4 Pro, macOS, 2026-10-20, dev profile.

`sound { stream true chunk <s> }` in a WAV's import settings (or a QOA file
above 1 MiB) cooks the sound in chunks of whole QOA frames: its blob holds
the QOA header and the first chunk, each other chunk is `<id>#chunk<n>`.
The runtime keeps the first chunk and, per playhead, the chunks from 0.5 s
behind to 4 s ahead (and a loop's start), reads the missing ones (natively
at once, with a loader on the mixing thread as a backstop; on the web
fetched like a streaming group's pack) and drops the rest.

## The three-minute track (`crates/og-audio/tests/chunks.rs`)

`chunks.json` (`$OG_EVIDENCE_DIR/M6/chunks.json` from the test):

| | |
|---|---|
| Track | 180.27 s, 48 kHz stereo, 6,989,848 B of QOA |
| Chunks | 46 of 37 QOA frames (3.95 s), 153,032 B each |
| Scene | looped from 20 s on the music bus, a 120 bpm og.Music clock; at 60 s a 2 s crossfade to a second voice of the track started 100 s in, which wraps at 180 s; 190 s in all |
| Whole-file path, resident | 6,989,848 B for the whole run |
| Chunked path, resident peak | 918,200 B (6 chunks: the first, two playheads' windows, the loop start's) |
| Chunk reads / drops | 50 / 46 |
| Beats | 380 on both paths |
| PCM | 9,120,000 stereo frames, bit-identical on both paths (hash 1eafea3643407585) |

The same scene for 70 s with no feeding at all, the mixer pumped every
600 ticks and the chunks read through the loader as voices reach them:
bit-identical to the whole file again.

Broken once each: without a voice holding its chunk and the next, the
crossfade's outgoing voice went silent when its playhead was removed
(first differing sample 5,765,120, at 60.05 s); a chunk of the wrong
length was accepted until the indexer required the chunk's bytes to end
at its last frame.

## Through og run (`packages/engine/tests/sound-chunks.test.ts`)

The idle archetype's music (`orbit.wav`, 26.7 s of 22,050 Hz mono, looped)
cooked whole and with `sound { stream true chunk 2 }` (13 chunks of 9 QOA
frames; 12 chunk files beside the head), each run by `og run --commands
35:quit --audio run.wav` on the native host: the two WAVs have the same
SHA-256 (fb654b4d1ece8a983103083757d4363682e4777802625002fa5a3d45980fda85
in the gate's run), so the chunked music, its loop wrap included, is
sample for sample the whole file's.

No committed template opts in, and none has a WAV whose QOA passes 1 MiB
(the largest, roguelike's `tomb.wav`, 29 s of 22 kHz mono, is 0.25 MiB),
so every template cooks exactly as before.
