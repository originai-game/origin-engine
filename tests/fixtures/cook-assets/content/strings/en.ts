// The template's strings (og.i18n): every word the menu shows
// is a key here, never a literal in a system. This is the base table, in
// the game's source language; a locale is its own file beside it
// (`zh-CN.ogt`, with the same keys and its `fonts` line), loaded in `start`.
import { file } from '@origingame/engine/author';
export default file.strings({
  "base": true,
  "direction": "ltr",
  "entries": {
    "menu.prompt": "Press Start to begin",
    "menu.spinning": "Spinning  {speed} rad/s",
    "menu.start": "Start",
    "menu.started": "Started",
    "menu.title": "Origin Engine"
  },
  "fonts": [],
  "locale": "en"
});
