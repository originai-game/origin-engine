import {Show, Text, View, t, packClass} from "@origingame/engine/ui";
import "./hud-editor.en.ogt";

export interface HudEditorScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
  /** The hint at the top shows (false once the player has moved something). */
  showHint?: boolean;
  /** Content the caller supplies (was the editorEditor snippet). */
  editorEditor?: (() => JSX.Element) | undefined;
}

/**
 * The HUD editor's frame: a hint at the top until the player moves
 * something, over the editor the caller supplies.
 */
export function HudEditorScreen(props: HudEditorScreenProps) {
  return (
    <View class={`absolute inset-0 ${packClass("Screen", "frame")} ${props.class ?? ""}`}>
      <Show when={props.showHint}>
        <View class={`absolute w-max top-6 bg-plate-hi backdrop-blur-regular nine-panel-cut py-3 px-6 mx-auto inset-x-0 compact:top-12 ${packClass("Panel", "frame")}`}>
          <Text class={`text-center line-clamp-2 text-reading font-semibold text-fg m-0`}>
            {t("screen.hud-editor.hint")}
          </Text>
        </View>
      </Show>
      <View class={`absolute w-max max-w-full bottom-6 mx-auto inset-x-0 narrow:left-4 narrow:right-0 narrow:w-auto narrow:bottom-8 narrow:mx-0 short:left-24 short:right-0 short:w-auto short:mx-0`}>
        {props.editorEditor?.()}
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<HudEditorScreenProps>> = {
  default: {"showHint":true},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"back":"Done"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"hud-editor","layer":"modal","focus":"","under":"none","input":"menu","pause":true,"capture":true,"remember":true,"back":"back"} as const;
