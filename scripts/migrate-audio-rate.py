#!/usr/bin/env python3
"""Run from the repository root; rename rate boundaries, not musical pitch."""
from pathlib import Path

def edit(name, pairs):
    path = Path(name)
    before = text = path.read_text()
    for old, new in pairs:
        text = text.replace(old, new)
    if text != before:
        path.write_text(text)
        print(path)

for name in ["audio", "ui-core-motion"]:
    edit(f"crates/og-api/api/{name}.ogt", [("field pitch f32", "field playbackRate f32")])
edit("crates/og-audio/src/schema.rs", [
    ('#[reflect(min = 0.0625, max = 16, default = 1, tunable)]\n    pub pitch:',
     '#[reflect(name = "playbackRate", min = 0.0625, max = 16, default = 1, tunable)]\n    pub pitch:'),
    ('/// The pitch last sent.\n    pub pitch:', '/// The playback rate last sent.\n    #[reflect(name = "playbackRate")]\n    pub pitch:'),
])
for name in [
    "crates/og-runtime/src/bind/timeline/evaluate.rs",
    "crates/og-runtime/src/bind/video.rs",
    "crates/og-runtime/src/present/vfx.rs",
    "crates/og-runtime/src/bind/modules/motion.rs",
    "crates/og-runtime/src/bind/modules/ui_desc.rs",
    "crates/og-timeline/src/model/text.rs",
    "crates/og-timeline/src/model/tracks.rs",
]:
    edit(name, [('"pitch"', '"playbackRate"'),
                ('pitch is a playback rate', 'playbackRate is a rate multiplier'),
                ('sound, volume, pitch or bus', 'sound, volume, playbackRate or bus')])
edit("crates/og-bundle/src/vfx.rs", [
    ('("pitch", VfxKind::Sound', '("playbackRate", VfxKind::Sound'),
    ('pitch <p>', 'playbackRate <rate>'),
    ('pitch 0.9 1.1', 'playbackRate 0.9 1.1'),
    ('# a pitch picked', '# a playback rate picked'),
])
edit("crates/og-api/api/timeline.ogt", [('(sound, volume, pitch, bus)', '(sound, volume, playbackRate, bus)')])
edit("crates/og-api/api/vfx.ogt", [('`pitch <min> <max>`', '`playbackRate <min> <max>`')])
edit("tests/fixtures/vehicle-terrain/src/main.ts", [('AudioSource.pitch', 'AudioSource.playbackRate'), ('pitch:', 'playbackRate:')])
