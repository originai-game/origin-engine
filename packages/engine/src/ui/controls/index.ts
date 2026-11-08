// The controls (docs/ui.md §2): the hold button, the switch, the stepper,
// the dropdown and the text fields, merged into og.ui by the package entry.
// Each is declared in crates/og-api/api/ui-controls.ogt; those built here
// are `host false`, the rest are og-ui's native controls (natives.ts).

import { holdButton } from "./buttons.ts";
import { inputArea, inputField } from "./fields.ts";
import { dropdown, numberStepper, toggleSwitch } from "./natives.ts";

export const controls = Object.freeze({
  holdButton,
  toggleSwitch,
  numberStepper,
  inputField,
  inputArea,
  dropdown,
});
