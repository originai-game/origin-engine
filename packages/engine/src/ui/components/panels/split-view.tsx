import { View, packClass, t, type UiEvent } from "@origingame/engine/ui";
import "./split-view.en.ogt";

export interface SplitViewProps {
  /** The start pane's share of the room, 0 to 1; the game keeps it from the divider's drag events. Default 0.5. */
  ratio?: number;
  /** One pane over the other; false is side by side. */
  column?: boolean;
  /** What a screen reader calls the divider; empty is Resize. */
  label: string;
  /** Content the caller supplies (was the end snippet). */
  end?: (() => JSX.Element) | undefined;
  /** Content the caller supplies (was the start snippet). */
  start?: (() => JSX.Element) | undefined;
  /** The resize event, with the row's key where it comes from a row. */
  onResize?: (e: UiEvent, key?: string) => void;
}

/**
 * Two panes on the HUD plate, the divider between them a faint groove
 * with its grip lit in the accent.
 *
 * Dropped against the .svelte: nothing.
 */
export function SplitView(props: SplitViewProps) {
  const ratio = () => props.ratio ?? 0.5;
  const share = { [props.column ? "height" : "width"]: `${100 * ratio()}%` };
  return (
    <View class={`group/splitview relative flex-row nine-panel-cut bg-glass text-fg w-140 max-w-full h-90 p-0 gap-0 overflow-hidden data-column:flex-col disabled:text-fg-subtle ${packClass("Panel", "frame")}`} data-column={props.column}>
      <View class={`bg-tile shadow-sunk rounded-control overflow-auto group-data-column/splitview:w-full shrink`} style={share}>
        {props.start?.()}
      </View>
      <View class={`bg-lit-soft hover:bg-lit justify-center items-center w-2 gap-0 group-data-column/splitview:w-full group-data-column/splitview:h-2`} role="separator" label={props.label || t("component.split-view.name")} valueNow={ratio() * 100} valueMin={0} valueMax={100} onPointerDown={(e) => props.onResize?.(e)} onPointerMove={(e) => props.onResize?.(e)}>
        <View class={`bg-hud-accent w-0.5 h-3 shadow-[0px_0px_9.136px_#38e8ff] rounded-capsule group-data-column/splitview:w-3 group-data-column/splitview:h-0.5`} />
      </View>
      <View class={`grow shrink basis-0 overflow-auto`}>{props.end?.()}</View>
    </View>
  );
}

export const fixtures: Record<string, Partial<SplitViewProps>> = {
  default: {"ratio":0.35,"column":false,"label":""},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"resize":"Resize"} as const;
