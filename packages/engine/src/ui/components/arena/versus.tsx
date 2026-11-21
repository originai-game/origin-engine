import { For, Icon, Image, Show, Text, View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import "./arena.en.ogt";

export interface VersusFighter {
  /** The fighter's key. */
  id: string;
  /** What a reader calls them. */
  name: string;
  /** The fighter's portrait texture (0 shows the icon). */
  portrait?: Texture | 0;
  /** An icon name, shown while no portrait is set. */
  icon?: string;
  /** The champion's name (empty hides). */
  character?: string;
  /** The rank as the game words it (Gold II, 1840 SR; empty hides). */
  rank?: string;
  /** Marks the local player (You). */
  local?: boolean;
}

export interface ArenaVersusProps {
  /** The first side's team name; empty takes the kit's (Blue team). */
  nameA?: string;
  /** The second side's team name; empty takes the kit's (Red team). */
  nameB?: string;
  /** The first side's fighters in the game's order. */
  sideA?: ReadonlyArray<VersusFighter>;
  /** The second side's fighters, keyed as sideA. An empty side reads Waiting for opponents. */
  sideB?: ReadonlyArray<VersusFighter>;
  /** The reveal's id: a new value replays the sides' entrance. */
  at?: number;
  /** Its place is narrow (under about 300 px): the cards and portraits tighten. The place decides, not the screen. */
  compact?: boolean;
}

/**
 * The match reveal: the two sides facing each other across a VS seal, each
 * fighter a card with portrait (or icon), name, champion and rank as the
 * game words it; the local player is marked You. A side the game has not
 * filled reads Waiting for opponents. A new reveal id (at) replays the
 * entrance, which reduced motion skips. Read only. Approximations: the
 * seal's hexagon is rounded and its face a flat gold gradient, the reveal
 * and seal entrances are dropped (the replay with the reveal's id with
 * them), the local row's layered wash is a flat tint, the team inks are
 * the default blue and red (the game's seed-team colours have no class
 * surface), the headings' and fighter names' size clamps to their side,
 * the condensed stretch and the fit-down of a too-long team name are
 * dropped, and a phone stacks the sides with the seal between them.
 */
const PORTRAIT = "shrink-0 size-[4.571rem] p-[3px] rounded-[0.857rem] bg-linear-to-b from-white/75 via-white/22 via-45% to-white/14 data-compact:size-[2.857rem]";
const NAME = "min-w-0 text-start text-[1.286rem] leading-[1.15] font-bold text-fg";
const CAPTION = "min-w-0 text-start text-[0.929rem] leading-[1.3] font-semibold text-white/66";
const YOU = `flex-none self-start py-0 px-1 text-[0.857rem] leading-[1.5] font-bold tracking-[0.14em] uppercase text-fg nine-chip-cut bg-[#1b1f29e0] whitespace-nowrap ${packClass("Chip", "frame")}`;

function Side(props: { name?: string; team: "a" | "b"; fighters?: ReadonlyArray<VersusFighter> }) {
  const heading = () => (props.name ?? "") != "" ? props.name : props.team == "a" ? t("arena.team.a") : t("arena.team.b");
  const fighters = () => props.fighters ?? [];
  const ink = () => props.team == "a" ? "text-[#50aaff]" : "text-[#ff6a5e]";
  const glow = () => props.team == "a" ? "shadow-[0px_0px_1rem_rgb(80_170_255/0.5),inset_0px_-2px_0px_0px_#50aaff]" : "shadow-[0px_0px_1rem_rgb(255_106_94/0.5),inset_0px_-2px_0px_0px_#ff6a5e]";
  const row = () => props.team == "a"
    ? "bg-linear-to-r from-[rgb(80_170_255/0.16)] to-[#1a1e29b8] to-40% shadow-[inset_3px_0px_0px_0px_#50aaff]"
    : "flex-row-reverse bg-linear-to-l from-[rgb(255_106_94/0.16)] to-[#1a1e29b8] to-40% shadow-[inset_-3px_0px_0px_0px_#ff6a5e]";
  return (
    <View class="flex-1 min-w-0 flex-col gap-y-1 compact:flex-none compact:gap-y-1" role="group" label={heading()}>
      <Text class={`pb-1 max-w-full ${props.team == "b" ? "text-end " : "text-start "}${ink()} ${glow()} text-[3.286rem] leading-none font-bold uppercase tracking-[0.04em] data-compact:text-[2rem] whitespace-normal`}>{heading()}</Text>
      <Show when={fighters().length} fallback={
        <Text class="p-4 text-center text-[1.143rem] text-white/66 text-shadow-2xs" role="status">{t("arena.versus.waiting")}</Text>
      }>
        <View class="flex-col gap-y-1" role="list" label={heading()}>
          <For each={fighters()} key={(f) => f.id}>
            {(f) => (
              <View
                class={`flex-row items-center justify-start gap-x-3 py-1 px-3 ${row()} data-local:bg-none data-local:bg-[#38e8ff26] compact:gap-x-1 compact:py-0.5 compact:px-1`}
                role="listitem"
                data-local={f().local === true}
              >
                <View class={PORTRAIT}>
                  <View class="size-full rounded-[0.714rem] flex-row items-center justify-center overflow-hidden">
                    <Show when={(f().portrait ?? 0) != 0} fallback={
                      <Icon class="size-8 text-fg" glyph={f().icon ?? ""} />
                    }>
                      <Image class="size-full" src={f().portrait ?? 0} />
                    </Show>
                  </View>
                </View>
                <View class="flex-1 min-w-0 flex-col gap-y-0.5">
                  <Text class={`${NAME} ${props.team == "b" ? "text-end" : ""}`}>{f().name}</Text>
                  <Show when={(f().character ?? "") != ""}>
                    <Text class={`${CAPTION} ${props.team == "b" ? "text-end" : ""} data-local:text-white/78`}>{f().character}</Text>
                  </Show>
                  <Show when={(f().rank ?? "") != ""}>
                    <Text class={`min-w-0 text-start text-[0.929rem] leading-[1.3] font-semibold text-[#ffcf5a] ${props.team == "b" ? "text-end" : ""} data-local:text-white/78`}>{f().rank}</Text>
                  </Show>
                  <Show when={f().local}>
                    <Text class={`${YOU} ${props.team == "b" ? "self-end" : ""}`}>{t("arena.you")}</Text>
                  </Show>
                </View>
              </View>
            )}
          </For>
        </View>
      </Show>
    </View>
  );
}

export function ArenaVersus(props: ArenaVersusProps) {
  return (
    <View class="w-full max-h-full flex-row items-stretch text-fg compact:flex-col" label={t("arena.versus.name")} data-compact={props.compact === true}>
      <View class="w-full min-w-0 flex-row items-stretch gap-x-4 compact:flex-col compact:items-stretch compact:gap-x-0 compact:gap-y-1">
        <Side {...(props.nameA ? { name: props.nameA } : {})} team="a" fighters={props.sideA ?? []} />
        <View class="flex-none self-center w-[6.286rem] p-[3px] rounded-[0.857rem] bg-[#ffcf5a] shadow-[0px_0px_1.857rem_rgb(255_207_90/0.5)] compact:w-[3.714rem] compact:self-center" aria-hidden="true">
          <View class="w-full h-[7.143rem] rounded-[0.714rem] bg-[radial-gradient(circle_at_50%_30%,#7a5b16,#2a2008)] flex-row items-center justify-center compact:h-[4.286rem]">
            <Text class="text-[3.143rem] leading-none font-bold italic text-[#ffcf5a] compact:text-[1.857rem]">{t("arena.versus.vs")}</Text>
          </View>
        </View>
        <Side {...(props.nameB ? { name: props.nameB } : {})} team="b" fighters={props.sideB ?? []} />
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<ArenaVersusProps>> = {
  default: {"nameA":"","nameB":"","at":1,"sideA":[{"id":"you","name":"Rhea","portrait":0,"icon":"sword","character":"Vex","rank":"Gold II","local":true}],"sideB":[{"id":"rook","name":"Rook","portrait":0,"icon":"shield","character":"Brenna","rank":"Gold I","local":false}]},
  teams: {"nameA":"Iron Wolves","nameB":"Ember Court","at":2,"sideA":[{"id":"kai","name":"Kai","portrait":0,"icon":"shield","character":"Brenna","rank":"Platinum III","local":false},{"id":"you","name":"Rhea","portrait":0,"icon":"sword","character":"Vex","rank":"Gold II","local":true},{"id":"ana","name":"Ana","portrait":0,"icon":"heart","character":"Sela","rank":"Gold IV","local":false}],"sideB":[{"id":"rook","name":"Rook","portrait":0,"icon":"bolt","character":"Idris","rank":"Gold I","local":false},{"id":"wren","name":"Wren","portrait":0,"icon":"leaf","character":"Maren","rank":"","local":false},{"id":"quill","name":"Quill","portrait":0,"icon":"star","character":"Quill","rank":"Silver I","local":false}]},
  overflow: {"nameA":"The Unbroken Wardens of the Ninefold Gate","nameB":"Ember Court Invitational Reserve","at":4,"sideA":[{"id":"a1","name":"Maximilian Vandenberghe-Okonkwo","portrait":0,"icon":"crown","character":"Ysolde the Unbroken, Warden of the Ninefold Gate","rank":"Grandmaster · 3,912 SR · Top 0.1%","local":false},{"id":"a2","name":"Rhea","portrait":0,"icon":"sword","character":"Vex","rank":"Diamond I","local":true},{"id":"a3","name":"Ana","portrait":0,"icon":"heart","character":"Sela","rank":"Diamond II","local":false},{"id":"a4","name":"Kai","portrait":0,"icon":"shield","character":"Brenna","rank":"Diamond III","local":false},{"id":"a5","name":"Jo","portrait":0,"icon":"music","character":"Tamsin","rank":"Master","local":false}],"sideB":[{"id":"b1","name":"Abernathy-Whitcombe","portrait":0,"icon":"bolt","character":"Idris","rank":"Grandmaster","local":false},{"id":"b2","name":"Wren","portrait":0,"icon":"leaf","character":"Maren","rank":"Master","local":false},{"id":"b3","name":"Rook","portrait":0,"icon":"flag","character":"Kestrel","rank":"Diamond I","local":false},{"id":"b4","name":"Moreau","portrait":0,"icon":"fire","character":"Ashen Pyre","rank":"Diamond IV","local":false},{"id":"b5","name":"Laurent","portrait":0,"icon":"gem","character":"Hollow","rank":"","local":false}]},
};
