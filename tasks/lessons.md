# Lessons

## 2026-10-24 — Development-key capability contract

The owner defines the platform development key as full-featured. When an
engine acceptance flow fails with that key, verify and fix the development
credential contract rather than narrowing it to a single-operation exception
or substituting an unrelated account key. Keep ordinary platform/channel
credentials distinct; full development capability does not imply upgrading
every platform key.

## 2026-10-25 — Regenerate copied assets before recording references

An ignored game-local atlas can survive a source-pack change and make a
reference recording irreproducible on a clean clone. Regenerate the game's
assets from the committed source before recording. Rumpus's frame suite now
compares its atlas and icons with the source pack and refuses both comparison
and recording when either copy is stale.
