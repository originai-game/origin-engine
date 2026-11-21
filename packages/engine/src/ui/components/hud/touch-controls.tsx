import { For, Show, View, t } from "@origingame/engine/ui";
import { ActionButton } from "./action-button";
import "./touch-controls.en.ogt";

/** One action button at the end side, as ActionButton takes it. */
export interface TouchAction {
  id: string;
  action: string;
  label: string;
  icon?: string;
  primary?: boolean;
  /** A charge round its rim, 0 to 1; negative is none. Default -1. */
  charge?: number;
  disabled?: boolean;
}

export interface TouchControlsProps {
  /** The two-axis og-input action the stick at the start side feeds (move); empty leaves the stick out. */
  move?: string;
  /** A d-pad in the stick's place: eight ways, each axis -1, 0 or 1. */
  pad?: boolean;
  /** A second stick at the end side and the two-axis action it feeds (look); empty leaves it out. */
  look?: string;
  /** The sticks' dead zone as a part of their radius, up to 0.95; 0 is the engine's 0.15. */
  deadZone?: number;
  /** The buttons at the end side, the first nearest the corner: keyed rows of { id, action, label, icon, primary, charge, disabled }, each as ActionButton takes them. */
  actions?: ReadonlyArray<TouchAction>;
  /** Shown under a mouse too (a touch laptop, a preview). Otherwise the controls show only where the primary pointer is a finger. */
  always?: boolean;
  /** Every control feeds nothing and shows faded (a cutscene, a menu over the HUD). */
  disabled?: boolean;
}

/**
 * The on-screen controls of a game played with thumbs: a stick or a d-pad
 * at the start side, the action buttons at the end side with the first
 * nearest the corner (a row of three, the rest in rows above), and a
 * second stick beside them when the game looks with one. Each control is
 * an input source of the og-input action it names (its box is a control
 * zone: touchStick, touchPad or touchButton), so the game reads the same
 * actions as from keys or a pad and handles no event; the component
 * reports none. It shows only where the primary pointer is a finger,
 * unless always is set, and a control that is not shown or is disabled
 * feeds nothing. The page places it: it is as wide as its box and stands
 * on its bottom edge.
 */

const ENGINE_DEAD_ZONE = 0.15;

/** A touch stick: its ring and its knob, whose box is the action's stick zone. */
function Stick(props: { action: string; deadZone: number; disabled: boolean; label: string }) {
  return (
    <View
      class="relative size-40 compact:size-28 rounded-full border-rim bg-tile shadow-plate active:border-rim-strong disabled:opacity-45"
      role="slider"
      label={props.label}
      valueMin={-1}
      valueMax={1}
      valueText={props.label}
      disabled={props.disabled}
      touchStick={props.action}
      deadZone={props.deadZone}
    >
      <View knob class="absolute size-[40%] rounded-full nine-disc bg-fg/45 shadow-float -translate-x-1/2 -translate-y-1/2 left-1/2 top-1/2" />
    </View>
  );
}

/** A touch d-pad: its cross and its box, which is the action's d-pad zone. */
function Pad(props: { action: string; disabled: boolean; label: string }) {
  const arm = "absolute rounded-[0.286rem] bg-tile border border-rim-soft";
  return (
    <View class="relative size-40 compact:size-28 active:scale-97 disabled:opacity-45" role="group" label={props.label} disabled={props.disabled} touchPad={props.action}>
      <View class={`${arm} bottom-[38%] left-[30%] right-[30%] top-0`} />
      <View class={`${arm} top-[38%] left-[30%] right-[30%] bottom-0`} />
      <View class={`${arm} right-[38%] top-[30%] bottom-[30%] left-0`} />
      <View class={`${arm} left-[38%] top-[30%] bottom-[30%] right-0`} />
    </View>
  );
}

export function TouchControls(props: TouchControlsProps) {
  const dz = () => Math.min(Math.max(props.deadZone ?? 0, 0), 0.95) || ENGINE_DEAD_ZONE;
  const actions = () => props.actions ?? [];
  return (
    <View
      class={`flex-row items-end justify-between gap-6 w-full p-4 compact:gap-4 compact:p-3 ${props.always ? "flex" : "hidden pointer-coarse:flex"}`}
      role="group"
      label={t("component.touch-controls.name")}
    >
      <Show when={(props.move ?? "") !== ""}>
        <Show when={props.pad === true} fallback={
          <Stick action={props.move ?? ""} deadZone={dz()} disabled={props.disabled === true} label={t("component.touch-controls.move")} />
        }>
          <Pad action={props.move ?? ""} disabled={props.disabled === true} label={t("component.touch-controls.move")} />
        </Show>
      </Show>
      <View class="flex-row items-end justify-end gap-4 min-w-0">
        <Show when={actions().length > 0}>
          <View class="flex-row-reverse flex-wrap-reverse content-end justify-end gap-3 min-w-0" style={{ maxWidth: "min(100%, 14rem)" }}>
            <For each={actions()} key={(a) => a.id}>
              {(a) => (
                <ActionButton
                  action={a().action}
                  label={a().label}
                  {...((a().icon ?? "") !== "" ? { icon: a().icon } : {})}
                  {...(a().primary ? { primary: true } : {})}
                  {...(((a().charge ?? -1) >= 0) ? { charge: a().charge } : {})}
                  disabled={props.disabled === true || a().disabled === true}
                />
              )}
            </For>
          </View>
        </Show>
        <Show when={(props.look ?? "") !== ""}>
          <Stick action={props.look ?? ""} deadZone={dz()} disabled={props.disabled === true} label={t("component.touch-controls.look")} />
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<TouchControlsProps>> = {
  default: {"move":"move","pad":false,"look":"","deadZone":0,"always":true,"disabled":false,"actions":[{"id":"jump","action":"jump","label":"Jump","icon":"bolt","primary":true,"charge":-1,"disabled":false},{"id":"dash","action":"dash","label":"Dash","icon":"fire","primary":false,"charge":-1,"disabled":false},{"id":"use","action":"use","label":"Use","icon":"key","primary":false,"charge":-1,"disabled":false}]},
  pad: {"move":"move","pad":true,"look":"","deadZone":0,"always":true,"disabled":false,"actions":[{"id":"a","action":"confirm","label":"A","icon":"","primary":true,"charge":-1,"disabled":false},{"id":"b","action":"back","label":"B","icon":"","primary":false,"charge":-1,"disabled":false}]},
  twin: {"move":"move","pad":false,"look":"look","deadZone":0.2,"always":true,"disabled":false,"actions":[{"id":"fire","action":"fire","label":"Fire","icon":"bomb","primary":true,"charge":-1,"disabled":false},{"id":"ult","action":"ultimate","label":"Ultimate","icon":"star","primary":false,"charge":0.6,"disabled":true}]},
  many: {"move":"move","pad":false,"look":"","deadZone":0,"always":true,"disabled":false,"actions":[{"id":"attack","action":"attack","label":"Attack","icon":"sword","primary":true,"charge":-1,"disabled":false},{"id":"block","action":"block","label":"Block","icon":"shield","primary":false,"charge":-1,"disabled":false},{"id":"dodge","action":"dodge","label":"Dodge","icon":"bolt","primary":false,"charge":-1,"disabled":false},{"id":"heal","action":"heal","label":"Heal","icon":"potion","primary":false,"charge":-1,"disabled":false},{"id":"bag","action":"bag","label":"Bag","icon":"bag","primary":false,"charge":-1,"disabled":false}]},
  disabled: {"move":"move","pad":false,"look":"","deadZone":0,"always":true,"disabled":true,"actions":[{"id":"jump","action":"jump","label":"Jump","icon":"bolt","primary":true,"charge":-1,"disabled":false}]},
};
