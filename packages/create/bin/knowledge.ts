// The knowledge package's art/audio briefs and creation vocabulary, which create
// re-exports and reads without copying them: packages/knowledge in a checkout,
// or the engine package's knowledge/ beside create/ once packed (the release
// archive's engine/ and create/, or node_modules/@origingame/ after an npm
// install).
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
function knowledgeScript(name: string): string {
  const file = [join(here, "..", "..", "knowledge", "scripts", name), join(here, "..", "..", "engine", "knowledge", "scripts", name)].find((f) => existsSync(f));
  if (!file) throw new Error(`no knowledge/scripts/${name} beside ${join(here, "..")}: expected ../knowledge (the engine repository) or ../engine/knowledge (the engine package); reinstall the engine distribution`);
  return file;
}

const brief: typeof import("../../knowledge/scripts/brief.ts") = await import(knowledgeScript("brief.ts"));
const creation: typeof import("../../knowledge/scripts/creation.ts") = await import(knowledgeScript("creation.ts"));

export const { ART_KINDS, ART_RESOURCES, artBrief, audioBrief, authoredArt, colourName, MOODS, MUSIC_STYLES, parseLook, parsePalette, presetLook, profileOf, readAudioChoices, readLook, RIG_TEMPLATES, rigParts, SFX_PRESETS, styleArtBriefs } = brief;
export type { ArtBlocks, ArtBrief, ArtKind, ArtResource, ArtValue, AudioBrief, AudioBriefInput, AudioFormat, AuthoredArt, BriefLook, BriefOptions, BriefPalette, BriefReference, BriefStyle, ImageParams, ModelParams, Mood, MusicCueBrief, MusicRole, MusicStyle, PaletteColour, ReadLookOptions, RigJoint, RigPart, RigTemplate, SfxBrief, SfxPreset, SfxPresetBrief, StyleArtBrief, StyleArtRequests } from "../../knowledge/scripts/brief.ts";

export const { CHOICE_LABELS, CREATION_CHOICES, CREATION_FIELDS, creationBlock, creationSchema, defaultView, FILM_GENRE, FORM_VIEWS, formOf, GENRE_VIEWS, inferCreationChoices, localized, LOOK_ZH, parseCreationInput, VARIANT_ZH } = creation;
export type { CreationChoices, CreationInput, Form, Genre, View } from "../../knowledge/scripts/creation.ts";
