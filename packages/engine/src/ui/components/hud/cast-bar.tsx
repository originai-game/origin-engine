import { Icon, Image, Show, Text, View, formatNumber, packClass, t } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./cast-bar.en.ogt";

export interface CastBarProps {
  /** What is being cast (Fireball, Reviving); empty uses Casting. */
  name?: string | undefined;
  /** A texture (og.load.texture); 0 or absent draws `glyph` instead. */
  icon?: Texture | 0 | undefined;
  /** An icon name, in art's place. */
  glyph?: string | undefined;
  /** Seconds elapsed since the cast began, as the game counts them. */
  elapsed?: number | undefined;
  /** The cast's full length in seconds; nonpositive shows an empty bar. */
  duration?: number | undefined;
  /** A channel: the fill drains from full instead of filling. */
  channel?: boolean | undefined;
  /** The cast cannot be interrupted: the fill turns neutral and a shield marks it. */
  locked?: boolean | undefined;
  /** The cast was broken off: the fill turns danger and Interrupted replaces the time. */
  interrupted?: boolean | undefined;
  /** Hide the remaining time. */
  hideTime?: boolean | undefined;
  /** Classes after its own (a width, a placement). */
  class?: string | undefined;
}

/**
 * A cast or channel bar: icon, name and remaining time over a track whose
 * fill follows `elapsed / duration` (drains for a channel). The game owns
 * the timing and the states; the bar shows `locked` and `interrupted`
 * through `data-locked` and `data-interrupted`. The remaining time reads
 * tenths under ten seconds. Dressed by `CastBar.frame`, `CastBar.track` and
 * `CastBar.fill`.
 */
export function CastBar(props: CastBarProps) {
  const duration = () => Math.max(props.duration ?? 0, 0);
  const progress = () => (duration() > 0 ? Math.min(Math.max((props.elapsed ?? 0) / duration(), 0), 1) : 0);
  const share = () => (props.channel ? 1 - progress() : progress());
  const remaining = () => Math.max(duration() - (props.elapsed ?? 0), 0);
  const time = () => formatNumber(remaining(), { decimals: remaining() < 10 ? 1 : 0 });
  const name = () => props.name || t("hud.cast-bar.casting");
  return (
    <View
      class={`min-w-56 ${packClass("CastBar", "frame")} ${props.class ?? ""}`}
      role="progressbar"
      label={name()}
      valueMin={0}
      valueMax={100}
      valueNow={Math.round(progress() * 100)}
      valueText={props.interrupted ? t("hud.cast-bar.interrupted") : t("hud.cast-bar.remaining", { time: time() })}
      data-locked={props.locked === true}
      data-interrupted={props.interrupted === true}
    >
      <View class="flex-row items-center gap-x-2 min-w-0">
        <Show when={props.icon} fallback={
          <Show when={props.glyph}>
            <Icon class="shrink-0 size-4 text-fg" name={props.glyph} />
          </Show>
        }>
          <Image class="shrink-0 size-5 rounded-control" src={props.icon as Texture} />
        </Show>
        <Text class="grow shrink basis-0 min-w-0 truncate text-sm font-semibold">{name()}</Text>
        <Show when={props.locked}>
          <Icon class="shrink-0 size-3.5 text-fg-muted" name="shield" label={t("hud.cast-bar.locked")} />
        </Show>
        <Show when={props.interrupted} fallback={
          <Show when={!props.hideTime}>
            <Text class="shrink-0 text-xs font-semibold tabular-nums text-fg-muted">{t("hud.cast-bar.seconds", { time: time() })}</Text>
          </Show>
        }>
          <Text class="shrink-0 text-xs font-semibold uppercase tracking-label text-danger" role="status">{t("hud.cast-bar.interrupted")}</Text>
        </Show>
      </View>
      <View class={packClass("CastBar", "track")}>
        <View
          class={packClass("CastBar", "fill")}
          data-locked={props.locked === true}
          data-interrupted={props.interrupted === true}
          style={{ width: `${share() * 100}%` }}
        />
      </View>
    </View>
  );
}

export const ground = "world";

export const fixtures: Record<string, Partial<CastBarProps>> = {
  default: { name: "Fireball", glyph: "fire", elapsed: 1.2, duration: 2.5 },
  channel: { name: "Arcane Beam", glyph: "bolt", elapsed: 1, duration: 4, channel: true },
  locked: { name: "Reviving", glyph: "heart", elapsed: 3, duration: 5, locked: true },
  interrupted: { name: "Hearthstone", glyph: "star", elapsed: 6, duration: 10, interrupted: true },
  long: { name: "Ritual of the Drowned Choir beneath the Mere", glyph: "star", elapsed: 2, duration: 40 },
};
