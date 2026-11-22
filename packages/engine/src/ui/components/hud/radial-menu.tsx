import { For, Icon, Line, Show, Text, View, type UiEvent, t } from "@origingame/engine/ui";
import "./radial-menu.en.ogt";

export interface RadialMenuItem {
  id: string;
  /** Its label (Heal). */
  label: string;
  /** An icon of the atlas. */
  icon?: string;
  /** Its centre as a turn fraction (slice i of n: i / n). */
  at: number;
  /** Its cost as the game words it (300); empty is none. */
  cost?: string;
  /** It cannot be paid for. */
  unaffordable?: boolean;
  disabled?: boolean;
}

export interface RadialMenuProps {
  /** The slices, clockwise from the top. */
  items?: ReadonlyArray<RadialMenuItem>;
  /** One slice's share of the turn (1 / n). */
  span?: number;
  /** The slice the pointer or stick points at (its id); empty is none. */
  chosen?: string;
  /** The centre's line for the chosen slice; empty shows its label. */
  detail?: string;
  /** Aim: the pointer moved. */
  onAim?: (e: UiEvent) => void;
  /** Choose a slice. */
  onChoose?: (e: UiEvent, id: string) => void;
}

/**
 * A wheel of slices over the scene: a disc, each slice a plate wedge with
 * its icon, label and cost upright at its centre, the chosen slice's wedge
 * lit and its line in the centre well. The game owns the aim and the
 * choice; the buttons report theirs. The style's SVG wedges are sampled
 * <Line> polylines here; the buttons sit by their slice's turn with
 * inline transforms, since the turn is data. The wheel is two --size-4xl
 * across and every offset is a share of that.
 */
const TAU = Math.PI * 2;
const R = 105;

function wedgePoints(at: number, span: number): readonly number[] {
  const start = (at - span / 2) * TAU - Math.PI / 2 + 0.0175;
  const sweep = Math.max(span - 0.006, 0.01) * TAU;
  const points: number[] = [];
  const steps = 24;
  for (let k = 0; k <= steps; k++) {
    const theta = start + (k * sweep) / steps;
    points.push(160 + R * Math.cos(theta), 160 + R * Math.sin(theta));
  }
  return points;
}

/**
 * A radial menu: slices clockwise from the top, the one the pointer or
 * stick aims at lit, and its detail in the centre. It reports onAim and
 * onChoose; the game decides what a slice does.
 */
export function RadialMenu(props: RadialMenuProps) {
  const span = () => props.span ?? 0;
  const chosen = () => props.chosen ?? "";
  return (
    <View class="relative size-70 pointer-events-auto" role="menu" label={t("component.radial-menu.name")} onPointerMove={(e) => props.onAim?.(e)}>
      <View class="absolute inset-0 rounded-full bg-hud ring ring-shade-deep" />
      <For each={props.items ?? []} key={(item) => item.id}>
        {(item) => (
          <View class="group/radialmenu absolute inset-0" data-chosen={item().id === chosen()}>
            <View class="absolute inset-0 flex-row items-center justify-center">
              <View class="relative size-70">
                <Line points={wedgePoints(item().at, span())} class="stroke-rim-soft group-data-chosen/radialmenu:stroke-accent stroke-86" />
              </View>
            </View>
            <View
              class="absolute top-4 left-26.25 size-20"
              style={{ transform: `rotate(${item().at * 360}deg)`, transformOrigin: "50% 180%" }}
            >
              <View
                class="size-full flex-col items-center justify-center gap-y-0.5"
                style={{ transform: `rotate(${item().at * -360}deg)` }}
              >
                <View
                  class="group/radialitem flex-col items-center justify-center size-full"
                  role="menuitem"
                  label={item().label}
                  current={item().id === chosen()}
                  disabled={item().disabled}
                  onPress={(e) => props.onChoose?.(e, item().id)}
                  data-chosen={item().id === chosen()}
                  data-disabled={item().disabled === true}
                >
                  <Show when={(item().icon ?? "") !== ""}>
                    <Icon class="size-6 text-fg group-data-chosen/radialitem:text-accent-fg group-data-chosen/radialitem:scale-110 group-data-disabled/radialitem:opacity-50" name={item().icon} />
                  </Show>
                  <Text class="max-w-full text-caption whitespace-nowrap text-fg text-shadow-xs group-data-chosen/radialitem:text-accent-fg group-data-disabled/radialitem:opacity-50">{item().label}</Text>
                  <Show when={(item().cost ?? "") !== ""}>
                    <View class="flex-row items-center gap-x-0.5">
                      <Icon class="size-4 text-fg" name="coin" />
                      <Text class={`${item().unaffordable ? "text-danger" : "text-warn"} text-reading font-bold tabular-nums whitespace-nowrap text-shadow-xs`}>{item().cost}</Text>
                    </View>
                  </Show>
                </View>
              </View>
            </View>
          </View>
        )}
      </For>
      <View class="absolute inset-0 flex-col items-center justify-center p-2 pointer-events-none">
        <Show when={(props.detail ?? "") !== ""} fallback={
          <For each={(props.items ?? []).filter((item) => item.id === chosen())} key={(item) => item.id}>
            {(item) => <Text class="text-center text-caption text-fg-muted text-shadow-xs">{item().label}</Text>}
          </For>
        }>
          <Text class="text-center text-caption text-fg-muted text-shadow-xs">{props.detail}</Text>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RadialMenuProps>> = {
  default: {"span":0.1667,"chosen":"heal","detail":"Restore 40 health","items":[{"id":"heal","label":"Heal","icon":"heart","at":0,"cost":"","unaffordable":false,"disabled":false},{"id":"shield","label":"Ward","icon":"shield","at":0.1667,"cost":"150","unaffordable":false,"disabled":false},{"id":"bomb","label":"Bomb","icon":"bomb","at":0.3333,"cost":"300","unaffordable":true,"disabled":false},{"id":"map","label":"Map","icon":"map","at":0.5,"cost":"","unaffordable":false,"disabled":false},{"id":"bag","label":"Bag","icon":"bag","at":0.6667,"cost":"","unaffordable":false,"disabled":true},{"id":"flag","label":"Rally","icon":"flag","at":0.8333,"cost":"","unaffordable":false,"disabled":false}]},
};
