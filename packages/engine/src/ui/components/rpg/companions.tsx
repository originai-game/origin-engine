import type { Texture } from "@origingame/engine";
import { For, packClass, Show, Text, View, t } from "@origingame/engine/ui";
import { HudBar } from "../hud/bar";
import { HudPortrait } from "../hud/portrait";
import "./rpg.en.ogt";

export interface RpgCompanionsProps {
  /** The leader's companions in the game's order, keyed rows of { id, name, portrait, health, healthMax, ghost, mana, manaMax, key, fallen }: portrait a texture (0: the person glyph); ghost the health before the latest loss; manaMax 0 hides the mana bar; key the input that switches to them as the game names it (2), on a key cap at the plate's end, empty for none; fallen out of the fight (on data-fallen: the plate dims, the portrait greys and a Fallen tag stands beside the name). */
  members?: ReadonlyArray<{ id: string; name: string; portrait: Texture | 0; health: number; healthMax: number; ghost: number; mana: number; manaMax: number; key: string; fallen: boolean }>;
  /** The share of health under which a companion is low: the fill turns to danger and the rim glows (0.25). */
  low?: number;
}

/**
 * The leader's companions under the leader's vitals: each a small octagon
 * portrait, the name over a thin leaning health bar with its ghost and a
 * thinner mana bar, and the key that switches to them on a cap at the
 * plate's end. A fallen companion's plate dims with a Fallen tag. The
 * game owns every value and state; read only.
 */

function isLow(props: RpgCompanionsProps, m: { fallen: boolean; health: number; healthMax: number }) {
  return !m.fallen && m.healthMax > 0 && m.health <= m.healthMax * (props.low ?? 0.25);
}

/**
 * The leader's companions as small portraits with health bars, a companion
 * under the low share turning to danger. Read only.
 */
export function RpgCompanions(props: RpgCompanionsProps) {
  return (
    <View
      class={`flex-col w-64 max-w-full text-stroke-3 text-stroke-bg p-2 gap-y-2.25 compact:flex-row compact:flex-wrap compact:w-full compact:gap-x-3 compact:gap-y-2 ${packClass("Plate", "frame")}`}
      role="group"
      label={t("rpg.party.name")}
    >
      <For each={props.members ?? []} key={(m) => m.id}>
        {(m) => (
        <View
          class={`group/member flex-row gap-x-2.5 items-center min-w-0 compact:flex-col compact:items-stretch compact:gap-y-0.75 data-fallen:opacity-muted`}
          role="group"
          label={m().fallen ? t("rpg.companions.member", { name: m().name, state: t("rpg.companions.fallen") }) : m().name}
          data-fallen={m().fallen}
          data-low={isLow(props, m())}
        >
          <View class={`shrink-0 self-center compact:hidden`}>
            <HudPortrait portrait={m().portrait} dimmed={m().fallen} />
          </View>
          <View class={`flex-1 min-w-0 flex-col gap-y-1`}>
            <View class={`flex-row items-baseline min-w-0 gap-1`}>
              <Text class={`shrink text-reading font-semibold whitespace-nowrap overflow-hidden text-ellipsis compact:text-caption`}>
                {m().name}
              </Text>
              <Show when={m().fallen}>
                <Text class={`shrink-0 text-xs font-semibold tracking-label uppercase text-danger whitespace-nowrap compact:tracking-normal`}>
                  {t("rpg.companions.fallen")}
                </Text>
              </Show>
            </View>
            <HudBar value={m().health} max={m().healthMax} ghost={m().ghost} low={isLow(props, m())} />
            <Show when={m().manaMax > 0 && !m().fallen}>
              <HudBar value={m().mana} max={m().manaMax} kind="mana" />
            </Show>
          </View>
          <Show when={m().key}>
            <Text
              class={`shrink-0 self-center min-w-5.5 max-w-18 px-1 text-xs leading-normal font-bold text-center text-key-fg bg-linear-to-b from-key-hi to-key-lo rounded-control shadow-key whitespace-nowrap overflow-hidden text-ellipsis pointer-coarse:hidden`}
            >
              {m().key}
            </Text>
          </Show>
        </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<RpgCompanionsProps>> = {
  default: {"low":0.25,"members":[{"id":"brannoch","name":"Brannoch","portrait":0,"health":91,"healthMax":100,"ghost":0,"mana":40,"manaMax":100,"key":"2","fallen":false},{"id":"sylvenne","name":"Sylvenne","portrait":0,"health":18,"healthMax":100,"ghost":44,"mana":88,"manaMax":100,"key":"3","fallen":false},{"id":"caelum","name":"Caelum","portrait":0,"health":0,"healthMax":100,"ghost":0,"mana":0,"manaMax":100,"key":"4","fallen":true}]},
  fallen: {"low":0.25,"members":[{"id":"caelum","name":"Caelum","portrait":0,"health":0,"healthMax":100,"ghost":12,"mana":0,"manaMax":100,"key":"4","fallen":true}]},
  critical: {"low":0.25,"members":[{"id":"sylvenne","name":"Sylvenne","portrait":0,"health":2,"healthMax":100,"ghost":30,"mana":0,"manaMax":0,"key":"","fallen":false}]},
  overflow: {"low":0.25,"members":[{"id":"long","name":"Brannoch Hollowmere, Sworn Shield of the Ninefold Gate","portrait":0,"health":123456,"healthMax":999999,"ghost":200000,"mana":99999,"manaMax":99999,"key":"Left Shoulder + Dpad Up","fallen":false},{"id":"down","name":"Caelum of the Long Ashen Road","portrait":0,"health":0,"healthMax":10,"ghost":0,"mana":0,"manaMax":10,"key":"4","fallen":true}]},
};
