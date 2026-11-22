import { For, Icon, Image, Show, Text, View, t, packClass } from "@origingame/engine/ui";
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
    <View class={`flex-col gap-y-2.5 w-55 max-w-full compact:flex-row compact:w-full compact:gap-x-3 text-fg`} role="group" label={t("shooter.squad.name")}>
      <For each={props.members ?? []} key={(m) => m.id}>
        {(m) => {
          const down = () => m().state == "down" || m().state == "out";
          const shieldShare = () => ((m().shield ?? 0) <= 0 ? 0 : (m().shield ?? 0) >= (m().shieldMax ?? 1) ? 1 : (m().shield ?? 0) / (m().shieldMax ?? 1));
          const healthShare = () => ((m().health ?? 0) <= 0 ? 0 : (m().health ?? 0) >= (m().healthMax ?? 1) ? 1 : (m().health ?? 0) / (m().healthMax ?? 1));
          const ghostShare = () => ((m().ghost ?? 0) <= (m().health ?? 0) ? 0 : (m().ghost ?? 0) >= (m().healthMax ?? 1) ? 1 : (m().ghost ?? 0) / (m().healthMax ?? 1));
          const cells = () => ((props.segment ?? 25) > 0 ? Math.floor((m().shieldMax ?? 0) / (props.segment ?? 25)) : 1);
          return (
            <View
              class={`flex-1 flex-col gap-y-1 min-w-0 opacity-100 data-down:opacity-60`}
              role="group"
              label={m().status ? t("shooter.squad.mate", { name: m().name, status: m().status }) : m().name}
              data-down={down()}
            >
              <View class="flex-row items-baseline gap-x-2.5 min-w-0">
                <View class="shrink-0 self-center size-10 compact:size-7 rounded-card bg-linear-to-b from-rim-strong via-rim to-rim-soft">
                  <View class="absolute inset-0.5 rounded-block overflow-hidden bg-plate-hi">
                    <Show when={(m().portrait ?? 0) > 0} fallback={
                      <View class="size-full flex-row items-center justify-center">
                        <Icon class="size-1/2 text-fg-muted" name="person" />
                      </View>
                    }>
                      <Image class="size-full data-out:opacity-45" src={m().portrait ?? 0} data-out={m().state == "out"} />
                    </Show>
                  </View>
                </View>
                <View class="flex-1 flex-row items-baseline gap-x-2 min-w-0">
                  <Text class={`min-w-0 text-reading font-semibold whitespace-nowrap truncate text-fg compact:text-caption py-0.25 px-1.5 nine-chip-cut bg-glass ${packClass("Plate", "frame")}`}>{m().name}</Text>
                  <Show when={(m().status ?? "") != ""}>
                    <Text class={`shrink-0 max-w-full min-w-0 text-xs font-semibold tracking-label uppercase whitespace-nowrap truncate text-fg-muted data-reviving:text-ally-ink data-down:text-danger py-0.25 px-1.5 nine-chip-cut bg-glass ${packClass("Plate", "frame")}`}
                      data-reviving={m().state == "reviving"}
                      data-down={down()}
                    >
                      {m().status}
                    </Text>
                  </Show>
                </View>
              </View>
              <Show when={(m().shieldMax ?? 0) > 0}>
                <View class="relative self-center w-full h-1.5 compact:h-1 overflow-hidden -skew-x-22 bg-track shadow-plate ring ring-rim">
                  <View class="absolute left-0 top-0 h-full bg-ally shadow-glow shadow-ally" style={{ width: `${shieldShare() * 100}%` }}>
                    <View class="absolute left-0 right-0 top-0 h-21/50 bg-linear-to-b from-sheen to-transparent to-42%" />
                  </View>
                  <View class="absolute inset-0 pointer-events-none">
                    <For each={Array.from({ length: Math.max(cells() - 1, 0) }, (_, k) => k)} key={(k) => `c${k}`}>
                      {(k) => <View class="absolute top-0 bottom-0 w-0.5 bg-tile" style={{ left: `${((k() + 1) / cells()) * 100}%` }} />}
                    </For>
                  </View>
                </View>
              </Show>
              <View
                class="relative self-center w-full h-1.5 compact:h-1 overflow-hidden -skew-x-22 bg-track shadow-plate ring ring-rim"
                role="meter"
                label={t("shooter.vitals.health")}
                valueMin={0}
                valueMax={m().healthMax ?? 0}
                valueNow={m().health ?? 0}
              >
                <View class="absolute left-0 top-0 h-full bg-gold" style={{ width: `${ghostShare() * 100}%` }} />
                <View class="absolute left-0 top-0 h-full bg-material-regular" style={{ width: `${healthShare() * 100}%` }}>
                  <View class="absolute left-0 right-0 top-0 h-21/50 bg-linear-to-b from-sheen to-transparent to-42%" />
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
