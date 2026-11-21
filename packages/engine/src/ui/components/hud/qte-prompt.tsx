import { Show, Text, View, createSignal, packClass, type UiEvent, t } from "@origingame/engine/ui";
import "./qte-prompt.en.ogt";

export interface QtePromptProps {
  /** The action to attempt; empty uses Respond. */
  label?: string;
  /** The displayed input hint, not an input binding; empty hides it. */
  keyLabel?: string;
  /** An og-input action bound by the game. Pointer, focused keyboard/pad and this action activate the same control. */
  action?: string;
  /** press, hold or mash. Hold emits once after holdMs; mash emits one respond per activation. The game counts responses and judges success. */
  mode?: string;
  /** The minimum hold duration in milliseconds, at least 1. Releasing early cancels without an intent. */
  holdMs?: number;
  /** Time left as a fraction, 0 to 1. The game advances it; no clock runs here and zero does not choose a result. */
  remaining?: number;
  /** Game-supplied completion fraction, 0 to 1; displayed for mash prompts. */
  progress?: number;
  /** active, waiting, success, failure or expired. Only active accepts input; the game owns resolution and expiry. */
  state?: string;
  /** Temporarily unavailable even while active. */
  disabled?: boolean;
  /** Respond; the game owns resolution. */
  onRespond?: (e: UiEvent) => void;
}

/**
 * A keyed prompt over the scene: what to attempt, a big trigger (press,
 * hold or mash — the hold runs holdMs with its fill, and confirms when it
 * completes), its state in words, and the game-supplied time left (and,
 * for a mash, the response progress) as thin bars. Only the active state
 * accepts input; the game owns resolution and expiry. Read only beyond
 * the respond.
 */
export function QtePrompt(props: QtePromptProps) {
  const state = props.state ?? "active";
  const hold = (props.mode ?? "press") === "hold";
  const active = () => state === "active" && !props.disabled;
  const stateInk = () => state === "success" ? "text-[#45e08f]" : state === "failure" || state === "expired" ? "text-[#ff9599]" : "text-fg-muted";
  const holdFill = createSignal(0);
  const held = () => holdFill() > 0;
  return (
    <View
      class={`flex-col max-w-full gap-y-2 border-rim bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-plate rounded-block border p-2 ${packClass("Plate", "frame")}`}
      label={(props.label ?? "") !== "" ? props.label : t("hud.qte-prompt.respond")}
      busy={state === "waiting"}
      data-state-success={state === "success"}
      data-state-failed={state === "failure" || state === "expired"}
    >
      <Text class="text-[0.929rem] font-semibold text-fg text-shadow-xs">{(props.label ?? "") !== "" ? props.label : t("hud.qte-prompt.respond")}</Text>
      <View
        class={`relative flex-row items-center justify-center min-h-11 max-w-full py-2 px-3 gap-x-2 border-rim-strong bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-[0px_0px_0px_1px_#00000073,inset_0px_1px_0px_0px_#ffffff14,0px_6px_18px_-6px_#00000099] rounded-button border overflow-hidden active:bg-none active:bg-tile-pressed active:scale-96 disabled:opacity-50 disabled:scale-100`}
        role="button"
        label={(props.label ?? "") !== "" ? props.label : t("hud.qte-prompt.respond")}
        {...(hold ? { hold: Math.max(props.holdMs ?? 600, 1) / 1000, onHold: (e: UiEvent) => holdFill.set(e.value === "0" ? 0 : Math.min(Number(e.value), 1)) } : {})}
        onPress={(e) => {
          holdFill.set(0);
          props.onRespond?.(e);
        }}
        disabled={!active()}
        data-action={(props.action ?? "") !== "" ? props.action : undefined}
      >
        <Show when={(props.keyLabel ?? "") !== ""}>
          <Text class="shrink-0 text-[0.929rem] leading-[1.6] font-bold text-center text-[#262b38] bg-linear-to-b from-white to-[#d8dbe2] rounded-[0.286rem] shadow-[0px_0.143rem_0px_#0000008c,inset_0px_-0.143rem_0px_#0000002e] whitespace-nowrap text-shadow-none px-1">{props.keyLabel}</Text>
        </Show>
        <Text class="min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg text-shadow-xs">
          {hold ? t("hud.qte-prompt.hold") : (props.mode ?? "press") === "mash" ? t("hud.qte-prompt.mash") : t("hud.qte-prompt.press")}
        </Text>
        <Show when={held()}>
          <View class="absolute bottom-0 left-0 h-[2px] bg-accent" style={{ width: `${holdFill() * 100}%` }} />
        </Show>
      </View>
      <Text class={`text-caption text-shadow-xs ${stateInk()}`} role="status">
        {state === "success" ? t("hud.qte-prompt.success") : state === "failure" ? t("hud.qte-prompt.failure") : state === "expired" ? t("hud.qte-prompt.expired") : state === "waiting" ? t("hud.qte-prompt.waiting") : props.disabled ? t("hud.qte-prompt.disabled") : t("hud.qte-prompt.active")}
      </Text>
      <Text class="text-caption text-fg-muted text-shadow-xs">{t("hud.qte-prompt.time")}</Text>
      <View class="relative w-full h-[0.571rem] overflow-hidden bg-track shadow-[0px_0px_0px_1px_#ffffff38,inset_0px_1px_0px_0px_#ffffff24,0px_0.143rem_0.286rem_#00000073]" role="progressbar" label={t("hud.qte-prompt.time")} valueMin={0} valueMax={100} valueNow={Math.min(Math.max(props.remaining ?? 0, 0), 1) * 100}>
        <View class="absolute left-0 top-0 h-full bg-accent transition-layout duration-300 ease-out" style={{ width: `${Math.min(Math.max(props.remaining ?? 0, 0), 1) * 100}%` }} />
      </View>
      <Show when={(props.mode ?? "press") === "mash"}>
        <Text class="text-caption text-fg-muted text-shadow-xs">{t("hud.qte-prompt.progress")}</Text>
        <View class="relative w-full h-[0.571rem] overflow-hidden bg-track shadow-[0px_0px_0px_1px_#ffffff38,inset_0px_1px_0px_0px_#ffffff24,0px_0.143rem_0.286rem_#00000073]" role="progressbar" label={t("hud.qte-prompt.progress")} valueMin={0} valueMax={100} valueNow={Math.min(Math.max(props.progress ?? 0, 0), 1) * 100}>
          <View class="absolute left-0 top-0 h-full bg-accent transition-layout duration-300 ease-out" style={{ width: `${Math.min(Math.max(props.progress ?? 0, 0), 1) * 100}%` }} />
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<QtePromptProps>> = {
  default: {"label":"Evade the falling gate","keyLabel":"Space","action":"","mode":"press","holdMs":600,"remaining":0.72,"progress":0,"state":"active","disabled":false},
  hold: {"label":"Brace the gate","keyLabel":"E","mode":"hold","holdMs":600,"remaining":0.6,"progress":0,"state":"active","disabled":false},
  mash: {"label":"Break free","keyLabel":"Space","mode":"mash","progress":0.4,"remaining":0.35,"holdMs":600,"state":"active","disabled":false},
  success: {"state":"success","remaining":0.2,"mode":"press","holdMs":600,"progress":0,"disabled":false},
  failure: {"state":"failure","remaining":0.1,"mode":"press","holdMs":600,"progress":0,"disabled":false},
  expired: {"state":"expired","remaining":0,"mode":"press","holdMs":600,"progress":0,"disabled":false},
  waiting: {"state":"waiting","mode":"press","holdMs":600,"remaining":1,"progress":0,"disabled":false},
  disabled: {"disabled":true,"mode":"press","holdMs":600,"remaining":1,"progress":0,"state":"active"},
  overflow: {"label":"Brace the damaged northern observatory mechanism","keyLabel":"Right trigger","mode":"hold","holdMs":600,"remaining":0.5,"progress":0,"state":"active","disabled":false},
};
