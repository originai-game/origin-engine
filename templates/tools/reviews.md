# Reviewing a render

`og render` (docs/design.md §2.9) leaves three things beside the video;
read them in this order, cheapest first.

1. **The contact sheet** (`<name>.contact-sheet.png`, `--contact-sheet 6`):
   eighteen frames spaced over the range, each labelled with its time and
   tick. One image answers most review questions: does the scene start
   where the script meant, does it move, is anything missing, black or
   frozen, does the UI read. Name what each cell shows; a cell that looks
   like its neighbour where motion was expected is the first finding.
2. **frames.json**: every frame's tick, time and the SHA-256 of its pixels,
   and the audio's. Runs of one script are byte-identical, so comparing two
   frames.json files (before and after a change) lists exactly the frames
   the change moved; a run of identical hashes where the script moves
   something is a frozen stretch. Open those frames (`--frames png`, or
   `--out <dir>`) at full size.
3. **The video** (`.mp4` H.264 or `.webm` VP9, with the capture mixer's
   audio under `--audio`): for timing and sound, which a still cannot
   show. `ffprobe` gives its frame count and duration: they equal
   `(to - from) * fps` and `to - from`.

Write the review as findings with their evidence: the cell or frame
(time and tick), what it shows, what was expected, and the fix. A render
is evidence only of the script it played: say which `--commands` (or
`--timeline`) it was.

The renders in the engine's own evidence follow this:
`tests/evidence/depth/video/NOTES.md`.
