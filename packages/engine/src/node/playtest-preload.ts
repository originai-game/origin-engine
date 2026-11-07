// Preloaded into every playtest file's `bun test` (playtest-files.ts): a
// test that made no assertion fails, as a playtest without a check always
// has. runPlaytest's own checks (expect: steps, invariants, require) and a
// frame reported unchecked each count as one.
import { beforeEach, expect } from "bun:test";

beforeEach(() => {
  expect.hasAssertions();
});
