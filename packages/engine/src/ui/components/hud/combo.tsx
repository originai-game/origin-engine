import { Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import { Progress } from "../../controls/progress";
import "./combo.en.strings.ts";

export interface ComboProps {
  /** The game's combo count, shown at zero too. Default 0. */
  count?: number;
  /** The game's multiplier; it is not derived from count. Default 1. */
  multiplier?: number;
  /** Remaining window, 0 to 1; negative hides the timer. The game advances it. Default -1. */
  remaining?: number;
  /** idle, active or broken; zero remaining does not determine gameplay results. Default idle. */
  state?: string;
  /** A custom heading; empty uses Combo. Default "". */
  label?: string;
  /** Hide the plate completely while idle with zero combo. Default false. */
  hideWhenIdle?: boolean;
}

/**
 * The combo readout on its HUD plate: the heading, the count and its
 * multiplier as figures, a broken-chain cue when broken, and the window's
 * progress while it runs. A status: its changes announce. Read only; the
 * game owns the count and the window.
 */
export function Combo(props: ComboProps) {
  const state = () => props.state ?? "idle";
  const hidden = () => (props.hideWhenIdle ?? false) && state() === "idle" && (props.count ?? 0) <= 0;
  return (
    <Show when={!hidden()}>
      <View
        class={`group/combo flex-col max-w-full data-idle:opacity-muted ${packClass("Combo", "plate")}`}
        role="status"
        data-active={state() === "active"}
        data-broken={state() === "broken"}
        data-idle={state() === "idle"}
      >
        <Text class="text-xs font-semibold tracking-label uppercase text-fg-muted text-shadow-tag">{props.label || t("hud.combo.name")}</Text>
        <View class="flex-row flex-wrap items-baseline gap-x-2">
          <Text class={`${packClass("Count", "roll")} ${packClass("Combo", "count")}`}>{formatNumber(Math.max(props.count ?? 0, 0))}</Text>
          <Text class={packClass("Combo", "multiplier")}>{t("hud.combo.multiplier", { value: props.multiplier ?? 1 })}</Text>
        </View>
        <Show when={state() === "broken"}>
          <Text class="text-caption font-semibold uppercase tracking-label text-danger text-shadow-tag">
            {t("hud.combo.broken")}
          </Text>
        </Show>
        <Show when={(props.remaining ?? -1) >= 0}>
          <Progress value={Math.min(Math.max(props.remaining ?? 0, 0), 1) * 100} label={t("hud.combo.window")} class="w-full" />
        </Show>
      </View>
    </Show>
  );
}

export const fixtures: Record<string, Partial<ComboProps>> = {
  default: {"count":17,"multiplier":2.5,"remaining":0.65,"state":"active","label":""},
  idle: {"count":0,"multiplier":1,"remaining":-1,"state":"idle"},
  broken: {"count":17,"multiplier":1,"remaining":0,"state":"broken"},
  overflow: {"count":1234567,"multiplier":12.5,"remaining":0.05,"state":"active","label":"Uninterrupted expedition chain"},
};
