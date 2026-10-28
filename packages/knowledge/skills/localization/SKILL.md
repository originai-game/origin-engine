---
name: localization
description: The craft of localizing a game on Origin Engine, beside og-i18n's API — when localization starts (the first string), keys that name meanings, keeping every table complete in the same change, translating honestly, text that must not be baked into art, choosing fonts and their size cost, designing for right to left before shipping it, and the per-locale check at every milestone. Use whenever player-facing text is added or changed, and before a build ships; og-i18n has the tables, messages, fonts and right-to-left API.
---

The market is global and a player may not read the language the code was
written in. Localization is part of making the game, not a translation pass
at the end: every player-facing string is a key from the first screen on,
and the tables stay complete as the game grows.

This skill is the discipline; og-i18n is the API it uses (the `type
strings` tables, `og.i18n.t` and the message forms, font chains and
subsets, kinsoku, right to left, the player's choice, the playtest) and is
not repeated here.

## 1. One key per string, from the first line

- Player-facing text (HUD, menus, prompts, results, the template words
  og.ui would otherwise show in English) is never a literal in code. It is
  a key in a table (og-i18n §1), even while the game ships one language:
  adding the second is then a table, not a hunt through systems.
- Keys name the meaning and the screen, not the English
  (`results.won`, not `results.you_made_it`). Changing the English never
  changes the key.
- Numbers, names and units are arguments of one message, never
  concatenated fragments: word order differs between languages, and a
  translator can reorder `"{distance} m · {time} s"` but not code that
  glues pieces together (og-i18n §2).
- Plurals and gendered forms are the message's plural and select forms,
  never `+ "s"`.
- Text painted into art (a sign, a logo) is an asset decision, recorded
  with the asset. Text a player must read to play is never baked into a
  texture: it would ship untranslated in every other locale.

## 2. Keep every table complete in the same change

- Adding `hud.run` to the base table without `zh-CN` ships an English
  string inside a Chinese UI. Add the key to every table in the change that
  adds it.
- Translate it yourself when you can do it well. When you cannot, leave the
  key out of that table, so the cook warns and the base text shows, and say
  which keys are untranslated; never copy the English into the other table
  silently, which hides the gap from the cook and from `og polish`.
- A new locale is a new table with every key, its font chain, and its
  frames checked (§5).
- Treat a cook warning about keys as a bug, never as noise.

## 3. Fonts are a decision with a size

- The project template ships a Simplified Chinese subset, so `zh-CN`
  renders out of the box; it is a template file installed with the
  project, never retyped or fetched. Other scripts need their own face,
  subset to what the tables use (og-i18n §3); never fetch fonts at run time
  and never take Japanese from the Chinese face.
- Record each added font's licence and provenance beside it in the project
  (`assets/fonts/`), as with any third-party asset.
- Fonts dominate a small build. Subset to the game's own tables, rerun the
  subset after strings change, and note the measured build size before and
  after (`og build`, og-build §6).
- A character outside every font is a visible defect: capture the screen
  in that locale and look, rather than trusting that no warning appeared.

## 4. Right to left, designed for before it ships

The engine shapes text and mirrors og.ui for right-to-left locales
(og-i18n §4); what stays the game's decision:

- Lay out UI by containers, anchors and alignment, not hard-coded
  positions, so the mirror has something to mirror.
- Mirror UI, not the world: a track or a board does not flip because the
  language did. Directional icons in the UI (back, next) need their
  mirrored twins; icons of objects do not.
- Never reverse strings by hand; numbers and Latin words inside
  right-to-left text are the shaper's job.

Until a game ships a right-to-left locale these rules cost nothing, and
adding one later is a table and a capture, not a layout rewrite.

## 5. The check, at every milestone

At every content milestone and before a build ships, for each shipped
locale: run the locale's playtest or a scripted run (`locale:<tag>` in the
command script, og-i18n §6), capture the main screens (title, the play HUD,
results, every menu) and open the frames. Check overflow (German and
Russian run 30 to 40 % longer than English), clipping in fixed-size
buttons, boxes for missing glyphs and mirrored layouts for right to left.
Compare the tables: every key in every locale. Keep the captures as
evidence with the locale and the build they came from (og-build §4).
