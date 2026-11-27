import { Show, View } from "@origingame/engine/ui";
import { BossBar, CardFace, RpgMinimap, RpgVitals } from "@origingame/engine/ui/components";
import { fixtures as vitals, type RpgVitalsProps } from "@origingame/engine/ui/components/rpg/vitals.tsx";
import { fixtures as minimap, type RpgMinimapProps } from "@origingame/engine/ui/components/rpg/minimap.tsx";
import { PauseScreen } from "@origingame/engine/ui/screens/pause.tsx";
import { MainMenuScreen } from "@origingame/engine/ui/screens/main-menu.tsx";
import { fixtures as menu, type MainMenuScreenProps } from "@origingame/engine/ui/screens/main-menu.tsx";
import type { Texture, UiMaterial } from "@origingame/engine";

export interface ReelCard {
  readonly id: string;
  readonly name: string;
  readonly cost: string;
  readonly type: string;
  readonly typeLabel: string;
  readonly rarity: string;
  readonly text: string;
  readonly art: Texture | 0;
  readonly material: { shader: UiMaterial; params: Record<string, number | readonly number[]> } | undefined;
  readonly lift: number;
}

export interface ReelProps {
  mode: string;
  cards: ReadonlyArray<ReelCard>;
  burn: number;
  portrait: Texture | 0;
  keyArt: Texture | 0;
}

const press = () => {};

/** A card's UI material; the burning card's progress follows the clock. */
const finish = (c: ReelCard, burn: number) =>
  c.material && (c.id === "burn" ? { ...c.material, params: { ...c.material.params, progress: burn } } : c.material);

/** The page's UI reel over the wanderer's walk: a hand of cards, an RPG HUD, the pause screen or the main menu. */
export function Reel(props: ReelProps) {
  return (
    <View class="absolute inset-0">
      <Show when={props.mode === "cards" || props.mode === "plain"}>
        <View class="absolute inset-x-0 bottom-0 flex-row justify-center items-end gap-5 pb-24">
          {props.cards.map((c) => (
            <View class={`w-46 h-66 ${c.lift > 0 ? "mb-6" : ""}`} material={props.mode === "plain" ? undefined : finish(c, props.burn)}>
              <CardFace name={c.name} cost={c.cost} icon="fire" art={c.art} type={c.type} typeLabel={c.typeLabel} rarity={c.rarity} text={c.text} />
            </View>
          ))}
        </View>
      </Show>
      <Show when={props.mode === "rpg"}>
        <View class="absolute top-4 left-4 w-80">
          <RpgVitals {...(vitals.default as RpgVitalsProps)} name="Wanderer" portrait={props.portrait} health={612} />
        </View>
        <View class="absolute bottom-24 inset-x-0 items-center">
          <View class="w-120"><BossBar name="The Drowned Keeper" epithet="Warden of the Reef" value={72} max={100} /></View>
        </View>
        <View class="absolute top-44 right-4 w-56 h-56">
          <RpgMinimap {...(minimap.default as RpgMinimapProps)} place="Lantern Coast" clock="17:54" />
        </View>
      </Show>
      <Show when={props.mode === "pause"}>
        <PauseScreen objective="Reach the lighthouse" played={4520} canSave={true}
          onResume={press} onSettings={press} onControls={press} onSave={press} onLoad={press} onQuitToMenu={press} onQuit={press} />
      </Show>
      <Show when={props.mode === "menu"}>
        <MainMenuScreen {...(menu.default as MainMenuScreenProps)} name="The Last Light" keyArt={props.keyArt} news="The lighthouse is lit" saveLine="Chapter 1 · The cliff path" />
      </Show>
    </View>
  );
}
