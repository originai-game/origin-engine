// A test fixture, not a template (tests/fixtures/README.md): data tables
// (og.table) that name each other's rows, read as the game's components.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "data-tables",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1
});
