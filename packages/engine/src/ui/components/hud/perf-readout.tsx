import { Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import "./perf-readout.en.strings.ts";

export interface PerfReadoutProps {
  /** Frames per second; negative means unavailable, zero is a real reading. */
  fps?: number;
  /** Frame time in milliseconds; negative means unavailable. */
  frameMs?: number;
  /** Round-trip milliseconds; negative means unavailable. */
  ping?: number;
  /** Packet loss as a fraction, 0 to 1; negative means unavailable. */
  loss?: number;
  /** offline, connecting, online or degraded. The game supplies connection state and thresholds; the document samples no clocks or network. */
  status?: string;
}

/**
 * A HUD plate of how the game runs: the connection state in its tone's
 * colour, then four metrics — frames, frame time, ping and packet loss —
 * each its caption over its figure, Unavailable where the game has no
 * reading. The game supplies every value and threshold; the document
 * samples no clocks or network. Read only.
 */
export function PerfReadout(props: PerfReadoutProps) {
  const status = () => props.status ?? "offline";
  const state = () => status() === "degraded" ? "text-warn" : status() === "online" ? "text-success" : "text-fg-muted";
  return (
    <View class={`flex-col max-w-full gap-y-1 p-2 ${packClass("Plate", "frame")}`} role="group" label={t("hud.perf-readout.name")}>
      <Text class={`text-caption text-shadow-tag ${state()}`} role="status">
        {status() === "online" ? t("hud.perf-readout.online") : status() === "connecting" ? t("hud.perf-readout.connecting") : status() === "degraded" ? t("hud.perf-readout.degraded") : t("hud.perf-readout.offline")}
      </Text>
      <View class="flex-row flex-wrap gap-x-3 gap-y-2">
        <View class="flex-col gap-y-0.5">
          <Text class="text-caption text-fg-muted text-shadow-tag">{t("hud.perf-readout.fps")}</Text>
          <Text class="text-caption font-bold tabular-nums text-fg text-shadow-tag">{(props.fps ?? -1) >= 0 ? formatNumber(props.fps ?? 0) : t("hud.perf-readout.unknown")}</Text>
        </View>
        <View class="flex-col gap-y-0.5">
          <Text class="text-caption text-fg-muted text-shadow-tag">{t("hud.perf-readout.frame")}</Text>
          <Text class="text-caption font-bold tabular-nums text-fg text-shadow-tag">{(props.frameMs ?? -1) >= 0 ? t("hud.perf-readout.ms", { value: props.frameMs ?? 0 }) : t("hud.perf-readout.unknown")}</Text>
        </View>
        <View class="flex-col gap-y-0.5">
          <Text class="text-caption text-fg-muted text-shadow-tag">{t("hud.perf-readout.ping")}</Text>
          <Text class="text-caption font-bold tabular-nums text-fg text-shadow-tag">{(props.ping ?? -1) >= 0 ? t("hud.perf-readout.ms", { value: props.ping ?? 0 }) : t("hud.perf-readout.unknown")}</Text>
        </View>
        <View class="flex-col gap-y-0.5">
          <Text class="text-caption text-fg-muted text-shadow-tag">{t("hud.perf-readout.loss")}</Text>
          <Text class="text-caption font-bold tabular-nums text-fg text-shadow-tag">{(props.loss ?? -1) >= 0 ? `${formatNumber(Math.min(props.loss ?? 0, 1) * 100, { decimals: 0 })}%` : t("hud.perf-readout.unknown")}</Text>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<PerfReadoutProps>> = {
  default: {"fps":60,"frameMs":16.7,"ping":42,"loss":0,"status":"online"},
  degraded: {"fps":24,"frameMs":41.7,"ping":240,"loss":0.12,"status":"degraded"},
  offline: {"fps":60,"frameMs":16.7,"ping":-1,"loss":-1,"status":"offline"},
  connecting: {"fps":-1,"frameMs":-1,"ping":-1,"loss":-1,"status":"connecting"},
  unknown: {"fps":-1,"frameMs":-1,"ping":-1,"loss":-1,"status":"offline"},
  zero: {"fps":0,"frameMs":0,"ping":0,"loss":0,"status":"online"},
};
