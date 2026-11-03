// The UI kit: og.ui's game widget set (host calls and their callback sugar),
// merged into og.ui by the package entry. Every member is declared in
// crates/og-api/api/ui.ogt (module `ui`).

import { widgets } from "./bind.ts";

export const uiKit = Object.freeze({
  ...widgets,
});
