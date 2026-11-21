import { For, Icon, Image, Show, Text, View, t } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./shooter.en.ogt";

export interface SquadMember {
  id: string;
  /** What a reader calls them. */
  name: string;
  /** Their portrait, a texture (og.load.texture); 0 draws the person glyph. */
  portrait?: Texture | 0;
  /** The state as the game words it (Reviving), empty for none. */
  status?: string;
  /** ok, down, reviving or out. */
  state?: string;
  /** Shield now. */
  shield?: number;
  /** Most shield; 0 hides the shield bar. */
  shieldMax?: number;
  /** Health now. */
  health?: number;
  /** Most health. */
  healthMax?: number;
  /** The health before the latest loss. */
  ghost?: number;
}

export interface ShooterSquadProps {
  /** The player's squadmates in the game's order. The game owns every value and state; read only. */
  members?: ReadonlyArray<SquadMember>;
  /** Shield per cell of each shield bar (25). */
  segment?: number;
}

/**
 * The squad at the screen's top corner: each squadmate's portrait in an
 * octagon, the name and a state in words, a thin shield bar cut into cells
 * over a thin health bar with its ghost. The game owns every value and
 * state; read only. Approximations: the portrait's octagon cut is rounded,
 * the out state's grey-out is opacity, the cells' mask is hairline
 * dividers, and a phone's side-by-side reflow keeps the plate's stack.
 */
export function ShooterSquad(props: ShooterSquadProps) {
  return (
    <View class={`flex-col gap-y-[0.714rem] w-[15.714rem] max-w-full compact:flex-row compact:w-full compact:gap-x-3 text-fg`} role="group" label={t("shooter.squad.name")}>
      <For each={props.members ?? []} key={(m) => m.id}>
        {(m) => {
          const down = () => m().state == "down" || m().state == "out";
          const shieldShare = () => ((m().shield ?? 0) <= 0 ? 0 : (m().shield ?? 0) >= (m().shieldMax ?? 1) ? 1 : (m().shield ?? 0) / (m().shieldMax ?? 1));
          const healthShare = () => ((m().health ?? 0) <= 0 ? 0 : (m().health ?? 0) >= (m().healthMax ?? 1) ? 1 : (m().health ?? 0) / (m().healthMax ?? 1));
          const ghostShare = () => ((m().ghost ?? 0) <= (m().health ?? 0) ? 0 : (m().ghost ?? 0) >= (m().healthMax ?? 1) ? 1 : (m().ghost ?? 0) / (m().healthMax ?? 1));
          const cells = () => ((props.segment ?? 25) > 0 ? Math.floor((m().shieldMax ?? 0) / (props.segment ?? 25)) : 1);
          return (
            <View
              class={`flex-1 flex-col gap-y-[0.286rem] min-w-0 opacity-100 data-down:opacity-60`}
              role="group"
              label={m().status ? t("shooter.squad.mate", { name: m().name, status: m().status }) : m().name}
              data-down={down()}
            >
              <View class="flex-row items-baseline gap-x-[0.714rem] min-w-0">
                <View class="shrink-0 self-center size-[2.857rem] compact:size-[2rem] rounded-[0.714rem] bg-linear-to-b from-white/75 via-white/22 to-white/14">
                  <View class="absolute inset-[0.143rem] rounded-[0.571rem] overflow-hidden bg-plate-hi">
                    <Show when={(m().portrait ?? 0) > 0} fallback={
                      <View class="size-full flex-row items-center justify-center">
                        <Icon class="size-[50%] text-fg/66" name="person" />
                      </View>
                    }>
                      <Image class="size-full data-out:opacity-45" src={m().portrait ?? 0} data-out={m().state == "out"} />
                    </Show>
                  </View>
                </View>
                <View class="flex-1 flex-row items-baseline gap-x-2 min-w-0">
                  <Text class="min-w-0 text-[1.071rem] font-semibold whitespace-nowrap truncate text-fg text-shadow-2xs text-stroke-3 text-stroke-bg compact:text-[0.929rem]">{m().name}</Text>
                  <Show when={(m().status ?? "") != ""}>
                    <Text
                      class={`shrink-0 max-w-full min-w-0 text-[0.857rem] font-semibold tracking-[0.14em] uppercase whitespace-nowrap truncate text-fg/78 text-shadow-2xs text-stroke-3 text-stroke-bg data-reviving:text-[#4fd8ff] data-down:text-danger`}
                      data-reviving={m().state == "reviving"}
                      data-down={down()}
                    >
                      {m().status}
                    </Text>
                  </Show>
                </View>
              </View>
              <Show when={(m().shieldMax ?? 0) > 0}>
                <View class="relative self-center w-full h-[0.429rem] compact:h-[0.286rem] overflow-hidden -skew-x-[22deg] bg-tile shadow-[0px_0px_0px_1px_rgb(255_255_255/0.22),inset_0px_1px_0px_0px_rgb(255_255_255/0.14),0px_0.143rem_0.286rem_rgb(0_0_0/0.45)]">
                  <View class="absolute left-0 top-0 h-full bg-[rgb(79_216_255)] shadow-[0.143rem_0px_0.714rem_rgb(79_216_255/0.8)]" style={{ width: `${shieldShare() * 100}%` }}>
                    <View class="absolute left-0 right-0 top-0 h-[42%] bg-linear-to-b from-white/55 to-transparent" />
                  </View>
                  <View class="absolute inset-0 pointer-events-none">
                    <For each={Array.from({ length: Math.max(cells() - 1, 0) }, (_, k) => k)} key={(k) => `c${k}`}>
                      {(k) => <View class="absolute top-0 bottom-0 w-[0.143rem] bg-tile" style={{ left: `${((k() + 1) / cells()) * 100}%` }} />}
                    </For>
                  </View>
                </View>
              </Show>
              <View
                class="relative self-center w-full h-[0.429rem] compact:h-[0.286rem] overflow-hidden -skew-x-[22deg] bg-tile shadow-[0px_0px_0px_1px_rgb(255_255_255/0.22),inset_0px_1px_0px_0px_rgb(255_255_255/0.14),0px_0.143rem_0.286rem_rgb(0_0_0/0.45)]"
                role="meter"
                label={t("shooter.vitals.health")}
                valueMin={0}
                valueMax={m().healthMax ?? 0}
                valueNow={m().health ?? 0}
              >
                <View class="absolute left-0 top-0 h-full bg-[linear-gradient(90deg,rgb(255_207_90/0.15),rgb(255_207_90/0.85))]" style={{ width: `${ghostShare() * 100}%` }} />
                <View class="absolute left-0 top-0 h-full bg-[rgb(244_246_251)]" style={{ width: `${healthShare() * 100}%` }}>
                  <View class="absolute left-0 right-0 top-0 h-[42%] bg-linear-to-b from-white/55 to-transparent" />
                </View>
              </View>
            </View>
          );
        }}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<ShooterSquadProps>> = {
  default: {"segment":25,"members":[{"id":"moraine","name":"Moraine","portrait":0,"status":"Reviving","state":"reviving","shield":50,"shieldMax":100,"health":38,"healthMax":100,"ghost":64},{"id":"quill","name":"Quill","portrait":0,"status":"","state":"ok","shield":100,"shieldMax":100,"health":100,"healthMax":100,"ghost":0}]},
  down: {"segment":25,"members":[{"id":"tarn","name":"Tarn","portrait":0,"status":"Down","state":"down","shield":0,"shieldMax":100,"health":0,"healthMax":100,"ghost":20},{"id":"sable","name":"Sable","portrait":0,"status":"Eliminated","state":"out","shield":0,"shieldMax":100,"health":0,"healthMax":100,"ghost":0},{"id":"ashby","name":"Ashby","portrait":0,"status":"","state":"ok","shield":25,"shieldMax":100,"health":12,"healthMax":100,"ghost":30}]},
  empty: {"segment":25,"members":[]},
  overflow: {"segment":50,"members":[{"id":"long","name":"xX_Midnight_Sniper_Of_The_Northern_Relay_Xx","portrait":0,"status":"Reviving a squadmate under heavy fire","state":"reviving","shield":250,"shieldMax":300,"health":9999,"healthMax":12000,"ghost":11000},{"id":"none","name":"Quill","portrait":0,"status":"","state":"ok","shield":0,"shieldMax":0,"health":100,"healthMax":100,"ghost":0}]},
};
