# effekseer-magicalxspiral

101 Effekseer sample effects by **MAGICALxSPIRAL**. Source: https://effekseer.github.io/en/contributes/MAGICALxSPIRAL/index.html
(the Effekseer sample effects, CC0 1.0). Credit: "Effekseer and MAGICALxSPIRAL". Install with
`og vfx add effekseer-magicalxspiral`; declarations land in `content/vfx/effekseer-magicalxspiral/`.

Converted by `og vfx import` into the engine's own `file.vfx` effects,
emitters and effect materials; spawn one as `base/vfx/effekseer-magicalxspiral/<Effect>`.
Textures in `textures/` are copied from the samples (PNGs losslessly
recompressed, pixels unchanged); what did not convert is listed as
`// effekseer:` comments at the end of each effect's .ts. Each file's
source in its contributor's zip is in `assets.json`.

## Capturing short or small effects

`foot_smoke4` is a sparse orange/yellow additive spray, not an opaque smoke
cloud. Its source uses 0.3-sized particles, a 10-frame start delay, a
30-frame particle life and 10-frame fades, at 60 Hz. At spawn scale 0.6 it
can read as a faint shimmer in a distant third-person view. For a close-up,
use scale 1, a 50-degree camera about 1.5 m from the spawn, and capture
around 0.4 s after spawning. With a spawn at `(0, 0.6, 0)`, a camera at
`(0, 0.405, 1.5)` looking at the spawn shows the individual glowing motes.

`Magic2_earth` has a yellow sigil lasting 0.25 s; its particles finish
before 0.5 s. Capture at 0.1–0.2 s after spawning, not after a one-second
settle. At scale 1, a 50-degree camera at `(0, 1.404, 5.2)` looking at a
spawn at `(0, 0.6, 0)` frames the whole sigil and surrounding sparks.
Moving that camera to 1.5 m clips the sigil. A 2 fps contact sheet can miss
this short flash; review full-resolution frames from a 30 fps recording.

These views were checked on native Vulkan on an NVIDIA L4. Both effects
use unlit additive particles: daylight is not needed to light them, and a
bright background reduces their contrast. Keep their source timing,
blending and textures; frame the effect rather than changing its content
to make a gallery tile larger.

| Effect | Category | Contributor | Source in the zip |
| --- | --- | --- | --- |
| `content/A_Gohlem1.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/A_Gohlem1.efkproj` |
| `content/A_Salamander1.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/A_Salamander1.efkproj` |
| `content/A_Salamander2.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/A_Salamander2.efkproj` |
| `content/A_Salamander4.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/A_Salamander4.efkproj` |
| `content/A_Sylph1.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/A_Sylph1.efkproj` |
| `content/A_Sylph2.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/A_Sylph2.efkproj` |
| `content/A_Undine1.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/A_Undine1.efkproj` |
| `content/A_Undine2.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/A_Undine2.efkproj` |
| `content/AquaPoint.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/AquaPoint.efkproj` |
| `content/Attack1.ts` | slash | MAGICALxSPIRAL | `MAGICALxSPIRAL/Attack1.efkproj` |
| `content/Attack2.ts` | slash | MAGICALxSPIRAL | `MAGICALxSPIRAL/Attack2.efkproj` |
| `content/Attack5.ts` | slash | MAGICALxSPIRAL | `MAGICALxSPIRAL/Attack5.efkproj` |
| `content/Attack6.ts` | slash | MAGICALxSPIRAL | `MAGICALxSPIRAL/Attack6.efkproj` |
| `content/Attack_Impact.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Attack_Impact.efkproj` |
| `content/Breakdown.ts` | explosion | MAGICALxSPIRAL | `MAGICALxSPIRAL/Breakdown.efkproj` |
| `content/FirePot1.ts` | fire | MAGICALxSPIRAL | `MAGICALxSPIRAL/FirePot1.efkproj` |
| `content/FirePot2.ts` | fire | MAGICALxSPIRAL | `MAGICALxSPIRAL/FirePot2.efkproj` |
| `content/Gohlem1.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Gohlem1.efkproj` |
| `content/Gohlem11.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Gohlem11.efkproj` |
| `content/Gohlem5.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Gohlem5.efkproj` |
| `content/Gohlem7.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Gohlem7.efkproj` |
| `content/Gohlem_Shield1.ts` | shield | MAGICALxSPIRAL | `MAGICALxSPIRAL/Gohlem_Shield1.efkproj` |
| `content/Gohlem_Shield2.ts` | shield | MAGICALxSPIRAL | `MAGICALxSPIRAL/Gohlem_Shield2.efkproj` |
| `content/Holy1.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Holy1.efkproj` |
| `content/Holy2.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Holy2.efkproj` |
| `content/Icicle1.ts` | magic-projectile | MAGICALxSPIRAL | `MAGICALxSPIRAL/Icicle1.efkproj` |
| `content/Icicle2.ts` | magic-projectile | MAGICALxSPIRAL | `MAGICALxSPIRAL/Icicle2.efkproj` |
| `content/Icicle3.ts` | magic-projectile | MAGICALxSPIRAL | `MAGICALxSPIRAL/Icicle3.efkproj` |
| `content/IcicleBroken.ts` | explosion | MAGICALxSPIRAL | `MAGICALxSPIRAL/IcicleBroken.efkproj` |
| `content/IcicleLocus.ts` | magic-projectile | MAGICALxSPIRAL | `MAGICALxSPIRAL/IcicleLocus.efkproj` |
| `content/Impact.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Impact.efkproj` |
| `content/Lance3.ts` | magic-projectile | MAGICALxSPIRAL | `MAGICALxSPIRAL/Lance3.efkproj` |
| `content/Lance4.ts` | magic-projectile | MAGICALxSPIRAL | `MAGICALxSPIRAL/Lance4.efkproj` |
| `content/Lance5.ts` | magic-projectile | MAGICALxSPIRAL | `MAGICALxSPIRAL/Lance5.efkproj` |
| `content/Magic1_earth.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Magic1_earth.efkproj` |
| `content/Magic1_fire.ts` | fire | MAGICALxSPIRAL | `MAGICALxSPIRAL/Magic1_fire.efkproj` |
| `content/Magic1_holy.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Magic1_holy.efkproj` |
| `content/Magic1_water.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Magic1_water.efkproj` |
| `content/Magic1_wind.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Magic1_wind.efkproj` |
| `content/Magic2_earth.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Magic2_earth.efkproj` |
| `content/Magic2_fire.ts` | fire | MAGICALxSPIRAL | `MAGICALxSPIRAL/Magic2_fire.efkproj` |
| `content/Magic2_holy.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Magic2_holy.efkproj` |
| `content/Magic2_water.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Magic2_water.efkproj` |
| `content/Magic2_wind.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Magic2_wind.efkproj` |
| `content/MagicArea.ts` | portal | MAGICALxSPIRAL | `MAGICALxSPIRAL/MagicArea.efkproj` |
| `content/Rock1.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Rock1.efkproj` |
| `content/Rock2.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Rock2.efkproj` |
| `content/Salamander1.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Salamander1.efkproj` |
| `content/Salamander11.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Salamander11.efkproj` |
| `content/Salamander12.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Salamander12.efkproj` |
| `content/Salamander14.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Salamander14.efkproj` |
| `content/Salamander15.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Salamander15.efkproj` |
| `content/Salamander2.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Salamander2.efkproj` |
| `content/Salamander3.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Salamander3.efkproj` |
| `content/Salamander4.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Salamander4.efkproj` |
| `content/Salamander5.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Salamander5.efkproj` |
| `content/Salamander6.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Salamander6.efkproj` |
| `content/Salamander7.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Salamander7.efkproj` |
| `content/Snowman1.ts` | weather | MAGICALxSPIRAL | `MAGICALxSPIRAL/Snowman1.efkproj` |
| `content/StairBroken.ts` | explosion | MAGICALxSPIRAL | `MAGICALxSPIRAL/StairBroken.efkproj` |
| `content/StairBroken3.ts` | explosion | MAGICALxSPIRAL | `MAGICALxSPIRAL/StairBroken3.efkproj` |
| `content/Sylph1.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Sylph1.efkproj` |
| `content/Sylph10.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Sylph10.efkproj` |
| `content/Sylph11.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Sylph11.efkproj` |
| `content/Sylph13.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Sylph13.efkproj` |
| `content/Sylph15.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Sylph15.efkproj` |
| `content/Sylph2.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Sylph2.efkproj` |
| `content/Sylph3.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Sylph3.efkproj` |
| `content/Sylph4.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Sylph4.efkproj` |
| `content/Sylph5.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Sylph5.efkproj` |
| `content/Sylph6.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Sylph6.efkproj` |
| `content/Sylph7.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Sylph7.efkproj` |
| `content/Sylph8.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Sylph8.efkproj` |
| `content/Thunder1.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Thunder1.efkproj` |
| `content/Thunder2.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Thunder2.efkproj` |
| `content/Thunder3.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Thunder3.efkproj` |
| `content/Topwater1.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Topwater1.efkproj` |
| `content/Undine1.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Undine1.efkproj` |
| `content/Undine11.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Undine11.efkproj` |
| `content/Undine13.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Undine13.efkproj` |
| `content/Undine2.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Undine2.efkproj` |
| `content/Undine3.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Undine3.efkproj` |
| `content/Undine4.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Undine4.efkproj` |
| `content/Undine5.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Undine5.efkproj` |
| `content/Undine6.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Undine6.efkproj` |
| `content/Undine8.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Undine8.efkproj` |
| `content/WHead1.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/WHead1.efkproj` |
| `content/WHead2.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/WHead2.efkproj` |
| `content/WHead3.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/WHead3.efkproj` |
| `content/Water_Impact.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Water_Impact.efkproj` |
| `content/Wind_Impact.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/Wind_Impact.efkproj` |
| `content/foot_smoke1.ts` | smoke | MAGICALxSPIRAL | `MAGICALxSPIRAL/foot_smoke1.efkproj` |
| `content/foot_smoke2.ts` | smoke | MAGICALxSPIRAL | `MAGICALxSPIRAL/foot_smoke2.efkproj` |
| `content/foot_smoke3.ts` | smoke | MAGICALxSPIRAL | `MAGICALxSPIRAL/foot_smoke3.efkproj` |
| `content/foot_smoke4.ts` | smoke | MAGICALxSPIRAL | `MAGICALxSPIRAL/foot_smoke4.efkproj` |
| `content/foot_smoke5.ts` | smoke | MAGICALxSPIRAL | `MAGICALxSPIRAL/foot_smoke5.efkproj` |
| `content/foot_smoke6.ts` | smoke | MAGICALxSPIRAL | `MAGICALxSPIRAL/foot_smoke6.efkproj` |
| `content/foot_smoke7.ts` | smoke | MAGICALxSPIRAL | `MAGICALxSPIRAL/foot_smoke7.efkproj` |
| `content/foot_smoke8.ts` | smoke | MAGICALxSPIRAL | `MAGICALxSPIRAL/foot_smoke8.efkproj` |
| `content/meteo1.ts` | magic-projectile | MAGICALxSPIRAL | `MAGICALxSPIRAL/meteo1.efkproj` |
| `content/salamander13.ts` | magic-impact | MAGICALxSPIRAL | `MAGICALxSPIRAL/salamander13.efkproj` |
