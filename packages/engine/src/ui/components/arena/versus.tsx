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
const PORTRAIT = "shrink-0 size-16 p-0.75 rounded-panel bg-linear-to-b from-rim-strong via-rim via-45% to-rim-soft group-data-compact/versus:size-10";
const NAME = "min-w-0 text-start text-lg leading-tight font-bold text-fg";
const CAPTION = "min-w-0 text-start text-caption leading-snug font-semibold text-fg-muted";
const YOU = `flex-none self-start py-0 px-1 text-xs leading-normal font-bold tracking-label uppercase whitespace-nowrap ${packClass("Chip", "frame")}`;

function Side(props: { name?: string; team: "a" | "b"; fighters?: ReadonlyArray<VersusFighter> }) {
  const heading = () => (props.name ?? "") != "" ? props.name : props.team == "a" ? t("arena.team.a") : t("arena.team.b");
  const fighters = () => props.fighters ?? [];
  const ink = () => props.team == "a" ? "text-team1" : "text-team2";
  const glow = () => props.team == "a" ? "shadow-glow-lg shadow-team1 inset-shadow-underline inset-shadow-team1" : "shadow-glow-lg shadow-team2 inset-shadow-underline inset-shadow-team2";
  const row = () => props.team == "a"
    ? "border-l-thick border-team1"
    : "flex-row-reverse border-r-thick border-team2";
  return (
    <View class="flex-1 min-w-0 flex-col gap-y-1 compact:flex-none compact:gap-y-1" role="group" label={heading()}>
      <Text class={`pb-1 max-w-full ${props.team == "b" ? "text-end " : "text-start "}${ink()} ${glow()} text-display-xl leading-none font-bold uppercase tracking-sm group-data-compact/versus:text-display-sm whitespace-normal`}>{heading()}</Text>
      <Show when={fighters().length} fallback={
        <Text class="p-4 text-center text-base text-fg-muted text-shadow-tag" role="status">{t("arena.versus.waiting")}</Text>
      }>
        <View class="flex-col gap-y-1" role="list" label={heading()}>
          <For each={fighters()} key={(f) => f.id}>
            {(f) => (
              <View
                class={`group/fighter flex-row items-center justify-start gap-x-3 py-1 px-3 ${row()} compact:gap-x-1 compact:py-0.5 compact:px-1`}
                role="listitem"
                data-local={f().local === true}
              >
                <View class={PORTRAIT}>
                  <View class="size-full rounded-card flex-row items-center justify-center overflow-hidden">
                    <Show when={(f().portrait ?? 0) != 0} fallback={
                      <Icon class="size-8 text-fg" name={f().icon ?? ""} />
                    }>
                      <Image class="size-full" src={f().portrait ?? 0} />
                    </Show>
                  </View>
                </View>
                <View class="flex-1 min-w-0 flex-col gap-y-0.5">
                  <Text class={`${NAME} ${props.team == "b" ? "text-end" : ""}`}>{f().name}</Text>
                  <Show when={(f().character ?? "") != ""}>
                    <Text class={`${CAPTION} ${props.team == "b" ? "text-end" : ""} group-data-local/fighter:text-fg`}>{f().character}</Text>
                  </Show>
                  <Show when={(f().rank ?? "") != ""}>
                    <Text class={`min-w-0 text-start text-caption leading-snug font-semibold text-gold ${props.team == "b" ? "text-end" : ""} group-data-local/fighter:text-fg`}>{f().rank}</Text>
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

/**
 * A versus reveal: each side's team name and fighters facing each other,
 * replayed when its id changes. An empty side reads Waiting for opponents.
 * Read only.
 */
export function ArenaVersus(props: ArenaVersusProps) {
  return (
    <View class={`group/versus w-full max-h-full flex-row items-stretch p-4 compact:flex-col compact:p-2 ${packClass("Panel", "frame")}`} label={t("arena.versus.name")} data-compact={props.compact === true}>
      <View class="w-full min-w-0 flex-row items-stretch gap-x-4 compact:flex-col compact:items-stretch compact:gap-x-0 compact:gap-y-1">
        <Side {...(props.nameA ? { name: props.nameA } : {})} team="a" fighters={props.sideA ?? []} />
        <View class="flex-none self-center w-22 p-0.75 rounded-panel bg-gold shadow-glow-lg shadow-gold compact:w-13 compact:self-center" aria-hidden="true">
          <View class="relative w-full h-25 rounded-card bg-gold-fg overflow-hidden flex-row items-center justify-center compact:h-15">
            <View class="absolute -inset-x-4 -top-6 bottom-0 nine-glow bg-gold opacity-dim" />
            <Text class="text-6xl leading-none font-bold italic text-gold compact:text-3xl">{t("arena.versus.vs")}</Text>
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
