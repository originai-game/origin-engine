// The controls og-ui implements natively (crates/og-ui/src/widgets/controls,
// docs/ui.md §11): the switch, the checkbox, the radio group, the segmented
// control, the slider, the number stepper and the dropdown. One set, which
// documents, the inspector and TypeScript all call; each is one host call
// with its desc as JSON.

import type * as Api from "@origingame/engine";
import { boundHost } from "../../runtime.ts";

function abi(): NonNullable<ReturnType<typeof boundHost>>["abi"] {
  const host = boundHost();
  if (!host) throw new Error("no engine host is loaded: start the game through @origingame/engine's web or node loader");
  return host.abi;
}

const parse = <T>(text: string): T => JSON.parse(text) as T;
const json = (value: unknown): string | undefined => (value === undefined ? undefined : JSON.stringify(value));

export function toggleSwitch(id: string, on: boolean, desc?: Api.ToggleSwitchDesc): Api.SwitchResult {
  return parse(abi().og_ui_toggle_switch(id, on, json(desc)));
}

export function checkField(id: string, checked: boolean, desc?: Api.CheckFieldDesc): Api.SwitchResult {
  return parse(abi().og_ui_check_field(id, checked, json(desc)));
}

export function radioGroup(id: string, desc?: Api.RadioGroupDesc): Api.ChoiceIndexResult {
  return parse(abi().og_ui_radio_group(id, json(desc)));
}

export function segmented(id: string, desc?: Api.SegmentedDesc): Api.ChoiceIndexResult {
  return parse(abi().og_ui_segmented(id, json(desc)));
}

export function valueSlider(id: string, value: number, desc?: Api.ValueSliderDesc): Api.ValueSliderResult {
  return parse(abi().og_ui_value_slider(id, value, json(desc)));
}

export function numberStepper(id: string, value: number, desc?: Api.NumberStepperDesc): Api.NumberResult {
  return parse(abi().og_ui_number_stepper(id, value, json(desc)));
}

export function dropdown(id: string, desc?: Api.DropdownDesc): Api.DropdownResult {
  return parse(abi().og_ui_dropdown(id, json(desc)));
}
