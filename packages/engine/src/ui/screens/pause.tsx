import {For, Show, Text, packClass, type UiEvent, View, formatDuration, t} from "@origingame/engine/ui";
import { MenuEntry, MenuList } from "../controls/menu-list";
import "./pause.en.strings.ts";

export interface PauseScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
  /** The current objective (empty: none). */
  objective: string;
  /** Seconds played this run. */
  played: number;
  /** Saving is allowed here. */
  canSave?: boolean;
  /** The game's own rows (Restart stage), each drawn as a built-in row and reported by onRow with its key: in the main group after the built-in ones, or after the row `after` names (a built-in row's key: resume, settings, controls, save, load, quitToMenu, quit; or an earlier row of the game's); a `minor` row goes in the group set apart below them, with Quit to menu and Quit game. */
  rows?: ReadonlyArray<PauseRow>;
  /** A row of the game's (props.rows) pressed, with its key. */
  onRow?: ((e: UiEvent, key: string) => void) | undefined;
  /** Content the caller supplies (was the side snippet). */
  side?: (() => JSX.Element) | undefined;
  /** The controls event, with the row's key where it comes from a row. */
  onControls?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The load event, with the row's key where it comes from a row. */
  onLoad?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The quit event, with the row's key where it comes from a row. */
  onQuit?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The quitToMenu event, with the row's key where it comes from a row. */
  onQuitToMenu?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The resume event, with the row's key where it comes from a row. */
  onResume?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The save event, with the row's key where it comes from a row. */
  onSave?: ((e: UiEvent, key?: string) => void) | undefined;
  /** The settings event, with the row's key where it comes from a row. */
  onSettings?: ((e: UiEvent, key?: string) => void) | undefined;
}

/** A row of the pause menu: a game's own (PauseScreenProps.rows), drawn as the built-in ones are. */
export interface PauseRow {
  /** Its stable key, reported by onRow. */
  key: string;
  /** What it says. */
  label: string;
  /** The key of the row it follows; empty or unknown: the end of its group. */
  after?: string;
  /** In the group set apart below, with Quit to menu and Quit game, without a chevron. */
  minor?: boolean;
  /** Tinted as a danger when hovered, as Quit game is. */
  danger?: boolean;
  /** Drawn grey and takes no press. */
  disabled?: boolean;
}

type Entry = PauseRow & { name: string; apart: boolean; press: (e: UiEvent) => void };

/**
 * The rows drawn, in order: each built-in row whose handler the game passes,
 * then the game's own rows in their places. No handler, no row.
 */
function pauseRows(props: PauseScreenProps): Entry[] {
  const own = (key: string, label: string, handler: ((e: UiEvent) => void) | undefined, more: Partial<PauseRow> = {}) =>
    handler ? [{ key, label, name: key.charAt(0).toUpperCase() + key.slice(1), apart: false, press: handler, ...more }] : [];
  const main: Entry[] = [
    ...own("resume", t("screen.pause.resume"), props.onResume && ((e) => props.onResume!(e))),
    ...own("settings", t("screen.pause.settings"), props.onSettings && ((e) => props.onSettings!(e))),
    ...own("controls", t("screen.pause.controls"), props.onControls && ((e) => props.onControls!(e))),
    // Saving greys out where the game says it is not allowed.
    ...own("save", t("screen.pause.save"), props.onSave && ((e) => props.onSave!(e)), { disabled: !props.canSave }),
    ...own("load", t("screen.pause.load"), props.onLoad && ((e) => props.onLoad!(e))),
  ];
  const minor: Entry[] = [
    ...own("quitToMenu", t("screen.pause.quitToMenu"), props.onQuitToMenu && ((e) => props.onQuitToMenu!(e)), { minor: true }),
    ...own("quit", t("screen.pause.quit"), props.onQuit && ((e) => props.onQuit!(e)), { minor: true, danger: true }),
  ];
  for (const row of props.rows ?? []) {
    const group = row.minor ? minor : main;
    const entry: Entry = { ...row, name: row.key, apart: false, press: (e) => props.onRow?.(e, row.key) };
    const at = row.after ? group.findIndex((r) => r.key === row.after) : -1;
    if (at < 0) group.push(entry);
    else group.splice(at + 1, 0, entry);
  }
  if (main.length > 0 && minor.length > 0) minor[0] = { ...minor[0]!, apart: true };
  return [...main, ...minor];
}

/**
 * The pause menu: Resume, Settings, Controls, Save (grey when not
 * allowed), Load, Quit to menu and Quit game, each drawn only when the game
 * passes its handler, and the game's own rows among them, with the current
 * objective and the time played beside it. Each reports its event.
 */
export function PauseScreen(props: PauseScreenProps) {
  return (
    <View class={`absolute inset-0 flex-row text-fg overflow-hidden ${packClass("Screen", "frame")} ${props.class ?? ""}`} onAction={{ back: (e) => props.onResume?.(e) }}>
      <View class={`absolute inset-0 ${packClass("Screen", "veil")}`} />
      <View class={`flex-col gap-5 w-120 max-w-full h-full pt-[calc(env(hud-inset-top)+3rem)] pb-[calc(env(hud-inset-bottom)+3rem)] pl-[calc(env(hud-inset-left)+4rem)] pr-6 transition duration-base ease-out starting:opacity-0 starting:-translate-x-6 portrait:w-full portrait:items-stretch portrait:pt-[calc(env(hud-inset-top)+5rem)] portrait:pl-[calc(env(hud-inset-left)+2rem)] portrait:pr-[calc(env(hud-inset-right)+2rem)] compact:w-full compact:gap-3 compact:pt-[calc(env(hud-inset-top)+2rem)] compact:pb-[calc(env(hud-inset-bottom)+1.5rem)] compact:pl-[calc(env(hud-inset-left)+1.25rem)] compact:pr-[calc(env(hud-inset-right)+1.25rem)] short:pt-3 short:pb-3`}>
        <Text class={`m-0 ${packClass("Screen", "heading")} portrait:self-center portrait:text-center compact:self-center compact:text-center`}>
          {t("screen.pause.title")}
        </Text>
        <View class={`flex-col gap-y-1 self-start w-full py-3 px-5 short:hidden ${packClass("Plate", "frame")}`}>
          <Show when={props.objective !== ""}>
            <Text class="text-2xs font-bold tracking-kicker uppercase text-fg-muted m-0">
              {t("screen.pause.objectiveLabel")}
            </Text>
            <Text class="text-lg leading-snug font-semibold line-clamp-2 m-0 compact:text-base">
              {props.objective}
            </Text>
          </Show>
          <Text class="m-0 text-sm tabular-nums text-fg-muted">
            {t("screen.pause.Played", { played: formatDuration(props.played) })}
          </Text>
        </View>
        <View class="grow min-h-2" />
        <MenuList wrap label={t("screen.pause.title")} class={`w-full p-3 ${packClass("Panel", "frame")} short:grid short:grid-cols-[1fr_1fr] short:gap-x-3`}>
          <For each={pauseRows(props)} key={(r) => r.key}>
            {(r) => (
              <>
                <Show when={r().apart}>
                  <View class="h-3 short:h-1" />
                </Show>
                <MenuEntry id={r().name} size="md" label={r().label} minor={r().minor} danger={r().danger} autofocus={r().key === "resume"} disabled={r().disabled === true} onPress={(e) => r().press(e)} />
              </>
            )}
          </For>
        </MenuList>
      </View>
      <View class="grow shrink basis-0 items-center justify-center p-8 portrait:hidden compact:hidden">{props.side?.()}</View>
    </View>
  );
}

/** The fixtures' handler: a row is drawn only where the game passes one. */
const press = () => {};

export const fixtures: Record<string, Partial<PauseScreenProps>> = {
  default: {"canSave":true,"objective":"Reach the lighthouse","played":4520,"onResume":press,"onSettings":press,"onControls":press,"onSave":press,"onLoad":press,"onQuitToMenu":press,"onQuit":press},
  // Star Focus's menu: three of the built-in rows and a row of its own; the rows it passes no handler for are not drawn.
  ownRows: {"canSave":false,"objective":"","played":0,"onResume":press,"onSettings":press,"onQuitToMenu":press,"onControls":undefined,"onSave":undefined,"onLoad":undefined,"onQuit":undefined,"rows":[{"key":"restart","label":"Restart stage","after":"resume"}],"onRow":press},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Select","back":"Resume"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"pause","layer":"menu","focus":"Panel/Menu/Resume","under":"blur","input":"menu","pause":true,"capture":true,"remember":true,"back":"resume"} as const;
