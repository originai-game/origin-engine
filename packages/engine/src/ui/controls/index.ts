// The controls (docs/ui.md §11): buttons, toggles and choices, ranges,
// fields, pickers, the key-binding row and the on-screen keyboard, merged
// into og.ui by the package entry. Each is declared in
// crates/og-api/api/ui.ogt (module `ui`, `host false`).

import { holdButton, iconButton, pushButton } from "./buttons.ts";
import { checkField, cycler, radioGroup, segmented, toggleSwitch } from "./choices.ts";
import { numberStepper, rangeSlider, valueSlider } from "./ranges.ts";

export const controls = Object.freeze({
  pushButton,
  iconButton,
  holdButton,
  toggleSwitch,
  checkField,
  radioGroup,
  segmented,
  cycler,
  valueSlider,
  rangeSlider,
  numberStepper,
});
