import {Scroll, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Button } from "../controls/button";
import "./confirm.en.ogt";

export interface ConfirmProps {
  /** Shown; the game writes it false to close (onCancel does on a dismissal). */
  open?: boolean;
  /** Its question, the dialog's heading and accessible name. */
  title: string;
  /** What follows from it; empty: none. */
  body: string;
  /** The confirming button's words (Quit, Delete). */
  confirm: string;
  /** The cancelling button's words. */
  cancel: string;
  /** Confirming destroys something: the confirm button is the danger variant. */
  danger?: boolean;
  /** The cancel event, with the row's key where it comes from a row. */
  onCancel?: (e: UiEvent, key?: string) => void;
  /** The confirm event, with the row's key where it comes from a row. */
  onConfirm?: (e: UiEvent, key?: string) => void;
}

/**
 * A confirmation dialog on its own panel over the pack's scrim, which dims
 * the screen while the panel pops in: the question as its heading, what
 * follows from it, and Cancel and Confirm (the danger variant when
 * confirming destroys something). The game closes it.
 */
export function Confirm(props: ConfirmProps) {
  return (
    <Show when={props.open}>
      <View class={packClass("Modal", "scrim")} />
      <View class={`group/confirm shrink min-h-0 m-auto w-115 max-w-full h-auto pt-6 pb-4 nine-panel-cut bg-plate-hi border border-rim text-fg px-6 gap-2 short:min-w-105 short:w-fit short:max-w-180 compact:p-4 ${packClass("Panel", "frame")} ${packClass("Modal", "frame")}`} modal={true} label={props.title} data-danger={props.danger === true} onAction={{ back: (e) => props.onCancel?.(e) }}>
      <Show when={props.danger}>
        <View class={`items-center gap-x-2 text-xs font-bold tracking-label uppercase text-danger`}>
          <Text class="rotate-45 bg-danger size-2" />
          <Text class="shrink">{t("screen.confirm.undone")}</Text>
        </View>
      </Show>
      <Text class={`font-display text-3xl font-bold leading-tight tracking-xs text-fg text-shadow-xs text-shadow-bg m-0 compact:text-2xl`}>
        {props.title}
      </Text>
      <Scroll class="min-h-5.75 shrink gap-2 data-hidden:hidden" data-hidden={props.body === ""}>
        <Text class={`text-base leading-normal text-fg-muted m-0`}>
          {props.body}
        </Text>
      </Scroll>
      <View class={`flex-row flex-wrap-reverse items-center mt-3 gap-2`}>
        <Button class={`text-fg nine-plate-cut grow shrink max-w-full min-h-11 py-2 disabled:border-transparent active:bg-transparent active:border-transparent hover:bg-transparent hover:border-transparent data-pressed:hover:border-transparent ${packClass("Button", "plate")}`} onPress={(e) => props.onCancel?.(e)} autofocus>
          <Text class="whitespace-normal">{props.cancel}</Text>
        </Button>
        <Button class={`nine-plate-cut grow shrink max-w-full min-h-11 py-2 text-accent ${packClass("Button", "plate", "primary")}`} onPress={(e) => props.onConfirm?.(e)}>
          <Text class="whitespace-normal">{props.confirm}</Text>
        </Button>
      </View>
    </View>
    </Show>
  );
}

export const fixtures: Record<string, Partial<ConfirmProps>> = {
  default: {"open":true,"title":"Quit to the menu?","body":"Progress since the last save is lost.","confirm":"Quit","cancel":"Keep playing","danger":true},
  overflow: {"open":true,"title":"Delete the save of the Ninefold Gate campaign for good?","body":"Chapter 7, 61 hours played, every companion and every item in the vault. This cannot be undone, and the cloud copy is deleted with it.","confirm":"Delete everything","cancel":"Keep the save","danger":true},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Confirm","cancel":"Cancel"} as const;
