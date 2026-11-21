import { For, Show, Text, View, createSignal, t, type UiEvent } from "@origingame/engine/ui";
import "./action-hints.en.ogt";

export interface ActionHintsProps {
  /** The hints, a keyed collection of { action, hint, hold } as og.ui.actionBar answers them: action the og-input action whose glyph its cap shows on the device used last, hint what it does (Buy), hold true when it fires on a hold (its fill grows while the action or a press holds it). */
  hints?: ReadonlyArray<{ action: string; hint: string; hold?: boolean }>;
  /** What a screen reader calls the bar; empty is Actions. */
  label?: string;
  /** Do. */
  onPress?: (e: UiEvent, action: string) => void;
  /** Hold, completing after 0.8 s; the hold hint's fill runs while it holds. */
  onHold?: (e: UiEvent, action: string) => void;
}

/**
 * A bar of input hints: each a key cap showing its action's binding glyph
 * and what it does, one that fires on a hold keeping the hold and running
 * its fill as it holds.
 */
export function ActionHints(props: ActionHintsProps) {
  return (
    <View class={`flex-row flex-wrap items-center justify-end gap-4 pointer-coarse:justify-center pointer-coarse:gap-2`} role="toolbar" label={props.label || t("component.action-hints.name")}>
      <For each={props.hints ?? []} key={(h) => h.action}>
        {(h) => (
          <View class="flex-col max-w-full">
            <HintRow
              {...(h().hold ? { hold: true } : {})}
              hint={h().hint}
              action={h().action}
              {...(props.onPress ? { onPress: props.onPress } : {})}
              {...(props.onHold ? { onHold: props.onHold } : {})}
            />
          </View>
        )}
      </For>
    </View>
  );
}

/** One hint row: a press hint or a hold one, each its own button. */
function HintRow(props: { hold?: boolean; hint: string; action: string; onPress?: (e: UiEvent, action: string) => void; onHold?: (e: UiEvent, action: string) => void }) {
  const progress = createSignal(0);
  const held = () => progress() > 0;
  return (
    <View
      class={`relative flex-row items-center gap-2 min-h-10 max-w-full py-0.5 px-2 overflow-hidden transition duration-317 text-fg bg-raised-lo shadow-[inset_0px_0px_0px_1px_#4e586c,inset_0px_-12px_12px_-8px_#00000038,inset_0px_1px_0px_0px_#ffffff1f,0px_1px_2px_0px_#00000080] active:bg-none active:bg-raised-pressed pointer-coarse:min-h-11 pointer-coarse:px-3`}
      role="button"
      label={props.hint}
      {...(props.hold
        ? { hold: 0.8, onHold: (e: UiEvent) => progress.set(e.value === "0" ? 0 : Math.min(Number(e.value), 1)) }
        : {})}
      onPress={(e) => {
        progress.set(0);
        (props.hold ? props.onHold : props.onPress)?.(e, props.action);
      }}
      data-action={props.action}
    >
      <Text class={`shrink-0 size-6 min-w-6 text-center text-[0.929rem] leading-[1.6] font-bold text-[#262b38] bg-linear-to-b from-white to-[#d8dbe2] rounded-[0.286rem] shadow-[0px_0.143rem_0px_#0000008c,inset_0px_-0.143rem_0px_#0000002e] text-shadow-none`} glyph={props.action} />
      <Text class="min-w-0 text-start text-[0.857rem] font-semibold tracking-[0.14em] uppercase text-fg-muted whitespace-nowrap truncate">{props.hint}</Text>
      <Show when={held()}>
        <View class="absolute bottom-0 left-0 h-[2px] bg-fg" style={{ width: `${progress() * 100}%` }} />
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ActionHintsProps>> = {
  default: {"label":"","hints":[{"action":"confirm","hint":"Buy","hold":false},{"action":"back","hint":"Back","hold":false},{"action":"details","hint":"Sell all","hold":true}]},
};
