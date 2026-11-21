import { Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import { Progress } from "../../controls/progress";
import "./combo.en.ogt";

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
}

/**
 * The combo readout on its HUD plate: the heading, the count and its
 * multiplier as figures, what state the chain is in, and the window's
 * progress while it runs. A status: its changes announce. Read only; the
 * game owns the count and the window.
 */
export function Combo(props: ComboProps) {
  const state = props.state ?? "idle";
  return (
    <View
      class="flex-col gap-y-1 border-rim bg-linear-to-b from-[#1e2637e5] to-plate-lo shadow-plate max-w-full p-2 rounded-block"
      role="status"
      data-active={state == "active"}
      data-broken={state == "broken"}
      data-idle={state == "idle"}
    >
      <Text class="text-base text-fg text-shadow-2xs">{props.label || t("hud.combo.name")}</Text>
      <View class="flex-row flex-wrap items-baseline gap-x-2">
        <Text class="text-[22.88px] font-bold tabular-nums text-fg text-shadow-2xs">{formatNumber(Math.max(props.count ?? 0, 0))}</Text>
        <Text class="text-[18.24px] font-bold tabular-nums text-fg text-shadow-2xs">{t("hud.combo.multiplier", { value: props.multiplier ?? 1 })}</Text>
      </View>
      <Text class="text-caption text-fg-muted data-active:text-success data-broken:text-danger text-shadow-2xs">
        {state == "broken" ? t("hud.combo.broken") : state == "active" ? t("hud.combo.active") : t("hud.combo.idle")}
      </Text>
      <Show when={(props.remaining ?? -1) >= 0}>
        <Progress value={Math.min(Math.max(props.remaining ?? 0, 0), 1)} label={t("hud.combo.window")} class="w-full" />
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ComboProps>> = {
  default: {"count":17,"multiplier":2.5,"remaining":0.65,"state":"active","label":""},
  idle: {"count":0,"multiplier":1,"remaining":-1,"state":"idle"},
  broken: {"count":17,"multiplier":1,"remaining":0,"state":"broken"},
  overflow: {"count":1234567,"multiplier":12.5,"remaining":0.05,"state":"active","label":"Uninterrupted expedition chain"},
};
