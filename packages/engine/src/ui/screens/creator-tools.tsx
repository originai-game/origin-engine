import {Scroll, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Button } from "../controls/button";
import "./creator-tools.en.ogt";

export interface CreatorToolsScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
  /** select, move, rotate or scale. */
  tool: string;
  /** The world is playing. */
  playing?: boolean;
  /** The selected object's name (empty: none). */
  selected: string;
  /** On a phone, the open panel: hierarchy, inspector or catalogue. */
  panel: string;
  /** Unsaved changes. */
  dirty?: boolean;
  /** Content the caller supplies (was the cataloguePanelCatalogue snippet). */
  cataloguePanelCatalogue?: (() => JSX.Element) | undefined;
  /** Content the caller supplies (was the scrollInspector snippet). */
  scrollInspector?: (() => JSX.Element) | undefined;
  /** Content the caller supplies (was the treePanelHierarchy snippet). */
  treePanelHierarchy?: (() => JSX.Element) | undefined;
  /** Content the caller supplies (was the view snippet). */
  view?: (() => JSX.Element) | undefined;
  /** The play event, with the row's key where it comes from a row. */
  onPlay?: (e: UiEvent, key?: string) => void;
  /** The redo event, with the row's key where it comes from a row. */
  onRedo?: (e: UiEvent, key?: string) => void;
  /** The save event, with the row's key where it comes from a row. */
  onSave?: (e: UiEvent, key?: string) => void;
  /** The undo event, with the row's key where it comes from a row. */
  onUndo?: (e: UiEvent, key?: string) => void;
  /** A tool pressed, with its key (select, move, rotate or scale): the game sets `tool`. */
  onTool?: (e: UiEvent, key?: string) => void;
  /** A panel tab pressed on a phone, with its key (hierarchy, inspector or catalogue): the game sets `panel`. */
  onPanel?: (e: UiEvent, key?: string) => void;
}

/**
 * The creator's tools on the HUD's smoked glass: the four tools a
 * segmented switch lit in the accent, the panels plates with tracked
 * capital titles. On a phone the panel tabs are one strip that scrolls
 * sideways, each a finger's target; a focus move scrolls its tab into view.
 */
export function CreatorToolsScreen(props: CreatorToolsScreenProps) {
  return (
    <View class={`absolute inset-0 text-fg ${packClass("Screen", "frame")} ${props.class ?? ""}`}>
      <View class={`flex-row border-rim bg-linear-to-b from-plate-hi to-plate-lo inset-shadow-lit inset-ring-hairline inset-ring-rim text-fg flex-wrap items-center p-2 gap-2 border disabled:text-fg-subtle ${packClass("Panel", "frame")}`} role="toolbar">
        <Button class={`bg-tile inset-ring-hairline inset-ring-rim-soft h-7 font-semibold tracking-label uppercase px-2.75 narrow:h-8 narrow:py-0 narrow:px-3.75 data-selection-on:bg-none data-selection-on:bg-hud-accent data-selection-on:nine-none data-selection-on:text-hud-accent-fg data-selection-on:border-transparent data-selection-on:inset-shadow-underline data-selection-on:inset-shadow-hud-accent data-selection-on:inset-ring-hairline data-selection-on:inset-ring-rim-soft data-selection-off:border-transparent shrink`} data-selection-off={props.tool !== "select"} data-selection-on={props.tool === "select"} onPress={(e) => props.onTool?.(e, "select")}>
          <Text>{t("screen.creator-tools.select")}</Text>
        </Button>
        <Button class={`bg-tile inset-ring-hairline inset-ring-rim-soft h-7 font-semibold tracking-label uppercase px-2.75 narrow:h-8 narrow:py-0 narrow:px-3.75 data-selection-on:bg-none data-selection-on:bg-hud-accent data-selection-on:nine-none data-selection-on:text-hud-accent-fg data-selection-on:border-transparent data-selection-on:inset-shadow-underline data-selection-on:inset-shadow-hud-accent data-selection-on:inset-ring-hairline data-selection-on:inset-ring-rim-soft data-selection-off:border-transparent shrink`} data-selection-off={props.tool !== "move"} data-selection-on={props.tool === "move"} onPress={(e) => props.onTool?.(e, "move")}>
          <Text>{t("screen.creator-tools.move")}</Text>
        </Button>
        <Button class={`bg-tile inset-ring-hairline inset-ring-rim-soft h-7 font-semibold tracking-label uppercase px-2.75 narrow:h-8 narrow:py-0 narrow:px-3.75 data-selection-on:bg-none data-selection-on:bg-hud-accent data-selection-on:nine-none data-selection-on:text-hud-accent-fg data-selection-on:border-transparent data-selection-on:inset-shadow-underline data-selection-on:inset-shadow-hud-accent data-selection-on:inset-ring-hairline data-selection-on:inset-ring-rim-soft data-selection-off:border-transparent shrink`} data-selection-off={props.tool !== "rotate"} data-selection-on={props.tool === "rotate"} onPress={(e) => props.onTool?.(e, "rotate")}>
          <Text>{t("screen.creator-tools.rotate")}</Text>
        </Button>
        <Button class={`bg-tile inset-ring-hairline inset-ring-rim-soft h-7 font-semibold tracking-label uppercase px-2.75 narrow:h-8 narrow:py-0 narrow:px-3.75 data-selection-on:bg-none data-selection-on:bg-hud-accent data-selection-on:nine-none data-selection-on:text-hud-accent-fg data-selection-on:border-transparent data-selection-on:inset-shadow-underline data-selection-on:inset-shadow-hud-accent data-selection-on:inset-ring-hairline data-selection-on:inset-ring-rim-soft data-selection-off:border-transparent shrink`} data-selection-off={props.tool !== "scale"} data-selection-on={props.tool === "scale"} onPress={(e) => props.onTool?.(e, "scale")}>
          <Text>{t("screen.creator-tools.scale")}</Text>
        </Button>
        <Button class={`h-7 ml-auto px-2.75 narrow:h-8 narrow:py-0 narrow:px-3.75 shrink`} onPress={(e) => props.onUndo?.(e)}>
          <Text>{t("screen.creator-tools.undo")}</Text>
        </Button>
        <Button class={`h-7 px-2.75 narrow:h-8 narrow:py-0 narrow:px-3.75 shrink`} onPress={(e) => props.onRedo?.(e)}>
          <Text>{t("screen.creator-tools.redo")}</Text>
        </Button>
        <Button class={`text-accent h-7 px-2.75 narrow:h-8 narrow:py-0 narrow:px-3.75 data-run-playing:opacity-80 shrink`} onPress={(e) => props.onPlay?.(e)} data-run-stopped={props.playing !== true} data-run-playing={props.playing === true}>
          <Text>{t("screen.creator-tools.play")}</Text>
        </Button>
        <Button variant="primary" class={`h-7 px-2.75 narrow:h-8 narrow:py-0 narrow:px-3.75 shrink`} disabled={!props.dirty} onPress={(e) => props.onSave?.(e)}>
          <Text>{t("screen.common.save")}</Text>
        </Button>
      </View>
      <View class="grow shrink basis-0 grid grid-cols-[auto_1fr_auto] grid-rows-[1fr_auto] narrow:grid-cols-1 narrow:grid-rows-[minmax(0,2fr)_auto_minmax(0,3fr)]">
        <View class={`w-70 narrow:w-auto narrow:min-h-70 border-rim bg-linear-to-b from-plate-hi to-plate-lo inset-shadow-bar inset-shadow-entry-grain inset-ring-hairline inset-ring-rim text-fg p-2 gap-2 border disabled:text-fg-subtle narrow:hidden narrow:data-chosen:flex ${packClass("Panel", "frame")}`} data-chosen={props.panel === 'hierarchy'}>
          <Text class={`tracking-label uppercase text-fg-muted text-xs font-semibold m-0`}>
            {t("screen.creator-tools.hierarchy")}
          </Text>
          <View class="grow shrink basis-0">{props.treePanelHierarchy?.()}</View>
        </View>
        <View>{props.view?.()}</View>
        <Scroll class="hidden p-2 gap-1 narrow:flex narrow:flex-nowrap narrow:min-w-0 narrow:bg-linear-to-b narrow:from-plate-hi narrow:to-plate-lo narrow:bg-transparent narrow:inset-shadow-lit narrow:inset-ring-hairline narrow:inset-ring-rim">
          <Button class={`h-7 px-2.75 narrow:border-transparent narrow:h-11 narrow:grow-0 narrow:shrink-0 narrow:basis-auto narrow:font-semibold narrow:tracking-label narrow:uppercase narrow:py-0 narrow:px-3.75 narrow:rounded-none narrow:data-on:bg-none narrow:data-on:bg-hud-accent narrow:data-on:nine-none narrow:data-on:text-hud-accent-fg narrow:data-on:inset-shadow-underline narrow:data-on:inset-shadow-hud-accent`} data-current={props.panel === "hierarchy"} data-on={props.panel === "hierarchy"} onPress={(e) => props.onPanel?.(e, "hierarchy")}>
            <Text class="narrow:whitespace-nowrap">{t("screen.creator-tools.hierarchy")}</Text>
          </Button>
          <Button class={`h-7 px-2.75 narrow:border-transparent narrow:h-11 narrow:grow-0 narrow:shrink-0 narrow:basis-auto narrow:font-semibold narrow:tracking-label narrow:uppercase narrow:py-0 narrow:px-3.75 narrow:rounded-none narrow:data-on:bg-none narrow:data-on:bg-hud-accent narrow:data-on:nine-none narrow:data-on:text-hud-accent-fg narrow:data-on:inset-shadow-underline narrow:data-on:inset-shadow-hud-accent`} data-current={props.panel === "inspector"} data-on={props.panel === "inspector"} onPress={(e) => props.onPanel?.(e, "inspector")}>
            <Text class="narrow:whitespace-nowrap">{t("screen.creator-tools.inspector")}</Text>
          </Button>
          <Button class={`h-7 px-2.75 narrow:border-transparent narrow:h-11 narrow:grow-0 narrow:shrink-0 narrow:basis-auto narrow:font-semibold narrow:tracking-label narrow:uppercase narrow:py-0 narrow:px-3.75 narrow:rounded-none narrow:data-on:bg-none narrow:data-on:bg-hud-accent narrow:data-on:nine-none narrow:data-on:text-hud-accent-fg narrow:data-on:inset-shadow-underline narrow:data-on:inset-shadow-hud-accent`} data-current={props.panel === "catalogue"} data-on={props.panel === "catalogue"} onPress={(e) => props.onPanel?.(e, "catalogue")}>
            <Text class="narrow:whitespace-nowrap">{t("screen.creator-tools.catalogue")}</Text>
          </Button>
        </Scroll>
        <View class={`w-90 narrow:w-auto narrow:min-h-70 border-rim bg-linear-to-b from-plate-hi to-plate-lo inset-shadow-bar inset-shadow-entry-grain inset-ring-hairline inset-ring-rim text-fg p-2 gap-2 border disabled:text-fg-subtle narrow:hidden narrow:data-chosen:flex ${packClass("Panel", "frame")}`} data-chosen={props.panel === 'inspector'}>
          <Text class={`font-display text-base font-bold leading-tight tracking-tight text-fg line-clamp-1 text-ellipsis m-0`}>
            {props.selected}
          </Text>
          <Scroll class="grow shrink basis-0">
            <View class="pb-4">{props.scrollInspector?.()}</View>
          </Scroll>
        </View>
        <View class={`h-50 narrow:h-auto narrow:min-h-70 border-rim bg-linear-to-b from-plate-hi to-plate-lo inset-shadow-lit inset-ring-hairline inset-ring-rim text-fg p-2 gap-2 border disabled:text-fg-subtle narrow:hidden narrow:data-chosen:flex ${packClass("Panel", "frame")}`} data-chosen={props.panel === 'catalogue'}>
          <Text class={`tracking-label uppercase text-fg-muted text-xs font-semibold m-0`}>
            {t("screen.creator-tools.catalogue")}
          </Text>
          <View class="grow shrink basis-0">{props.cataloguePanelCatalogue?.()}</View>
        </View>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<CreatorToolsScreenProps>> = {
  default: {"dirty":true,"panel":"inspector","playing":false,"selected":"Lantern_03","tool":"move"},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"back":"Exit"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"creator-tools","layer":"menu","focus":"","under":"none","input":"all","pause":false,"capture":true,"remember":true,"back":"exit"} as const;
