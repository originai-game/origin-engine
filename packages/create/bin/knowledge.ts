// The knowledge package's art and audio briefs, which create re-exports and
// reads (MUSIC_STYLES, profileOf). It is the knowledge package's own
// scripts/brief.ts, never a copy: packages/knowledge in a checkout, the
// engine package's knowledge/ beside create/ once packed (the release
// archive's engine/ and create/, or node_modules/@origingame/ after an npm
// install).
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const file = [join(here, "..", "..", "knowledge", "scripts", "brief.ts"), join(here, "..", "..", "engine", "knowledge", "scripts", "brief.ts")].find((f) => existsSync(f));
if (!file) throw new Error(`no art brief beside ${join(here, "..")}: expected ../knowledge (the engine repository) or ../engine/knowledge (the engine package); reinstall the engine distribution`);
const brief: typeof import("../../knowledge/scripts/brief.ts") = await import(file);

export const { ART_KINDS, ART_RESOURCES, artBrief, audioBrief, authoredArt, colourName, MOODS, MUSIC_STYLES, parseLook, parsePalette, presetLook, profileOf, readAudioChoices, readLook, RIG_TEMPLATES, rigParts, SFX_PRESETS, styleArtBriefs } = brief;
export type { ArtBlocks, ArtBrief, ArtKind, ArtResource, ArtValue, AudioBrief, AudioBriefInput, AudioFormat, AuthoredArt, BriefLook, BriefOptions, BriefPalette, BriefReference, BriefStyle, ImageParams, ModelParams, Mood, MusicCueBrief, MusicRole, MusicStyle, PaletteColour, ReadLookOptions, RigJoint, RigPart, RigTemplate, SfxBrief, SfxPreset, SfxPresetBrief, StyleArtBrief, StyleArtRequests } from "../../knowledge/scripts/brief.ts";
