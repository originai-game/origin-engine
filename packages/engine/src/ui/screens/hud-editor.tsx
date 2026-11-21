import { Show, Text, View, t } from "@origingame/engine/ui";
import "./hud-editor.en.ogt";

export interface HudEditorScreenProps {
  /** The hint at the top shows (false once the player has moved something). */
  showHint?: boolean;
  /** Content the caller supplies (was the editorEditor snippet). */
  editorEditor?: (() => JSX.Element) | undefined;
}

export function HudEditorScreen(props: HudEditorScreenProps) {
  return (
    <View>
      <Show when={props.showHint}>
        <View class="absolute w-max top-6 bg-linear-to-b from-[#171f30d6] to-[#080c16e0] shadow-[inset_0px_0px_0px_1px_#ffffff2e,inset_0px_1px_0px_#ffffff8c] backdrop-blur-[22px] rounded-tl-[18.288px] rounded-tr-none rounded-br-[18.288px] rounded-bl-none py-3 px-6 mx-auto inset-x-0 compact:top-12">
          <Text class={`text-center line-clamp-2 text-[1.071rem] font-semibold text-accent-hover m-0`}>
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
