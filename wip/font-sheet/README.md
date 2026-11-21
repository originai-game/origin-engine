# WIP: text measurement sheet (not for landing)

From the paint thread T-e2c409a7, on main 06a9cc02d, handed to the text rebuild thread.

- `project/`: a TSX game that draws three lines (Latin, Chinese, digits) at 12, 14, 16, 20, 28 and 48 px. Each row is
  `round(size*1.3)` px tall, at 1200x620. The command `pack:<default|cartoon|fantasy|scifi>` loads a style pack and adds
  `assets/fonts/NotoSansSC-Regular.ttf` as `fallbackFonts`; without it, no TSX text draws CJK. The command `sheet` mounts the sheet.
  `og cook project && og run project --headless --no-check --size 1200x620 --out /tmp/fs-<pack> --commands "0.02:pack:<pack>;0.3:sheet;1.5:screenshot:sheet.png;1.6:quit"`
  og loads the addon from packages/engine/native/, so copy a fresh libog_host_node.so there first. A stale addon is silent.
- `ref/`: `python3 make.py` writes `<pack>.html`, the same layout in Chrome using the faces og actually draws:
  default InstrumentSans[wdth,wght], cartoon Fredoka-Medium, fantasy Newsreader-Medium,
  scifi BarlowSemiCondensed-SemiBold, and NotoSansSC-Regular for CJK. Take the screenshot at 1200x620, DPR 1.
- `compare.sh <og.png> <chrome.png> <out.png> <label>`: stacks the two sheets and adds 4x crops of the 12-16 px rows.
