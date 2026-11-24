import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, formatNumber, t} from "@origingame/engine/ui";
import { Button } from "../../controls/button";
import "./idle.en.ogt";

export interface IdlePrestigeProps {
  /** The panel's heading; empty takes the group's (Prestige). */
  title: string;
  /** The prestige currency's name (Souls). */
  currency: string;
  /** The prestige currency's icon. */
  icon: string;
  /** How much of it the player holds now, as the game formats it. */
  held: string;
  /** How much a reset now would give, as the game formats it. */
  gain: string;
  /** What changes, keyed rows of { id, icon, label, now, after }: now the value before the reset and after the value it would be after, both as the game words them. */
  rows?: ReadonlyArray<{ id: string; icon: string; label: string; now: string; after: string }>;
  /** What a reset needs, as the game words it (Reach 1M gold); empty hides it. */
  requirement: string;
  /** Toward the requirement, 0 to 1, for its bar; negative hides the bar. Default -1. */
  progress?: number;
  /** That progress in words (640K / 1M); empty shows the percent. */
  progressText: string;
  /** The requirement is met: Prestige is enabled and the requirement reads Ready (else Not yet). */
  met?: boolean;
  /** The game is asking the player to confirm (set after request): Reset and Cancel stand in for Prestige. */
  confirming?: boolean;
  /** The cancel event, with the row's key where it comes from a row. */
  onCancel?: (e: UiEvent, key?: string) => void;
  /** The confirm event, with the row's key where it comes from a row. */
  onConfirm?: (e: UiEvent, key?: string) => void;
  /** The request event, with the row's key where it comes from a row. */
  onRequest?: (e: UiEvent, key?: string) => void;
}

/**
 * The prestige panel of an idle game: the currency in a socket glowing in
 * the epic purple, what the player holds and, in that purple, what a reset
 * would give; each changed bonus now and after the reset; and the
 * requirement with its leaning bar, green once met, and Ready or Not yet in
 * words. Prestige reports request; the game then sets confirming and the
 * panel asks inline, Reset (confirm) and Cancel (cancel, or Escape and pad
 * B while asking). Cancel comes first so no key resets by accident. The
 * game decides whether a reset is allowed and what it gives.
 */
export function IdlePrestige(props: IdlePrestigeProps) {
  return (
    <View class={`grow shrink basis-0 w-full h-auto max-h-full p-4 gap-3 disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title !== '' ? props.title : t("idle.prestige.name")} data-confirming={props.confirming === true} data-met={props.met === true}>
      <View class="absolute left-4 right-3/10 top-0 h-px bg-linear-to-r from-sheen to-transparent to-70% pointer-events-none" />
      <View class={`min-h-12 border-b border-border-muted w-full items-start py-1 px-4 gap-0`}>
        <Text class={`whitespace-nowrap text-ellipsis text-xs font-semibold tracking-kicker uppercase text-fg-muted overflow-hidden`}>
          {t("idle.prestige.kicker")}
        </Text>
        <Text class={`font-display text-base font-bold leading-tight text-fg max-w-full m-0`}>
          {props.title !== '' ? props.title : t("idle.prestige.name")}
        </Text>
      </View>
      <Scroll class="shrink gap-0 ">
        <View class="w-full gap-3">
          <View class="flex-row items-center w-full gap-3">
            <View class={`justify-center items-center bg-tile shadow-plate inset-ring-regular inset-ring-border gap-2 size-16 rounded-full`}>
              <Icon class="size-7/10" name={props.icon} />
            </View>
            <View class="grow shrink basis-0 gap-0">
              <Text class={`tracking-xs text-fg-muted max-w-full text-xs font-semibold`}>
                {t("idle.prestige.held", { currency: props.currency })}
              </Text>
              <Text class={`tabular-nums text-fg max-w-full text-display-md font-bold`}>
                {props.held}
              </Text>
            </View>
            <View class="items-end max-w-1/2 gap-0 shrink">
              <Text class={`tracking-xs text-fg-muted max-w-full text-xs font-semibold`}>
                {t("idle.prestige.gain")}
              </Text>
              <Text class={`tabular-nums text-fg max-w-full text-display-md font-bold data-tag-gain:text-xp data-tag-gain:text-end`} data-tag-gain={true}>
                {props.gain}
              </Text>
            </View>
          </View>
          <Show when={(props.rows ?? []).length}>
            <View class="w-full p-0 m-0 gap-0.5" label={t("idle.prestige.bonus")}>
              <For each={props.rows}>
                {(r) => (
                <View class={`flex-row items-center text-fg transition duration-control w-full min-h-8 py-0.5 px-2 gap-1 data-selected:text-fg disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-dim compact:flex-wrap compact:py-0.5 compact:px-1 ${packClass("Row", "frame")}`} label={t("idle.prestige.rowName", { label: r().label, now: r().now, after: r().after })}>
                  <Icon class="size-6" name={r().icon} />
                  <Text class={`grow shrink basis-0 text-fg text-start font-semibold`}>
                    {r().label}
                  </Text>
                  <View class="flex-row items-center justify-end gap-1 compact:grow compact:shrink-0 compact:basis-full">
                    <Text class={`whitespace-nowrap font-bold tabular-nums text-fg-muted shrink`}>
                      {r().now}
                    </Text>
                    <Icon class="text-fg-muted size-4 rtl:-scale-x-100 rtl:scale-y-100" name="play" />
                    <Text class={`whitespace-nowrap font-bold tabular-nums text-fg-muted data-tag-after:text-xp data-tag-after:text-lg shrink`} data-tag-after={true}>
                      {r().after}
                    </Text>
                  </View>
                </View>
                )}
              </For>
            </View>
          </Show>
          <Show when={props.requirement !== ''}>
            <View class={`bg-tile inset-ring-hairline inset-ring-border p-2 gap-1`} data-met={props.met === true}>
              <View class="flex-row items-center w-full gap-1">
                <Text class="grow shrink basis-0 text-start text-fg">{props.requirement}</Text>
                <View class={`flex-row items-center text-xs font-semibold tracking-label uppercase text-fg-muted py-0 px-1 gap-0.5 data-met:bg-success data-met:bg-none data-met:nine-none data-met:text-success-fg ${packClass("Chip", "frame")}`} data-met={props.met === true}>
                  <Icon class="text-shadow-tag size-3" name={props.met ? 'check' : 'lock'} />
                  <Text class="whitespace-nowrap text-shadow-tag shrink">
                    {props.met ? t("idle.prestige.met") : t("idle.prestige.notMet")}
                  </Text>
                </View>
              </View>
              <Show when={(props.progress ?? -1) >= 0}>
                <View class="flex-row items-center w-full gap-1">
                  <View class={`grow shrink basis-0 min-w-12 bg-track shadow-sunk h-2.5 rounded-capsule overflow-hidden`} role="progressbar" label={t("idle.prestige.progress")} valueMin={0} valueMax={100} valueNow={(props.progress ?? -1) * 100} valueText={props.progressText !== '' ? props.progressText : `${formatNumber((props.progress ?? -1) * 100, { decimals: 0 })}%`}>
                    <View class={`bg-hud-accent h-full data-met:bg-none data-met:bg-success`} data-met={props.met === true} style={{ width: `${(100 * ((props.progress ?? -1) > 1 ? 1 : (props.progress ?? -1)))}%` }} />
                  </View>
                  <Text class="max-w-1/2 whitespace-nowrap text-ellipsis text-xs font-bold tabular-nums text-fg-muted overflow-hidden">
                    {props.progressText !== '' ? props.progressText : `${formatNumber((props.progress ?? -1) * 100, { decimals: 0 })}%`}
                  </Text>
                </View>
              </Show>
            </View>
          </Show>
        </View>
      </Scroll>
      <Show when={props.confirming} fallback={
          <View class="flex-row flex-wrap items-center w-full justify-end gap-2">
            <Button variant="primary" class={`min-h-11 min-w-30 gap-1 compact:min-w-0 compact:grow compact:shrink compact:basis-0 pointer-coarse:grow pointer-coarse:shrink pointer-coarse:basis-0`} disabled={!props.met} onPress={(e) => props.onRequest?.(e)}>
              <Icon class="size-4" name={props.icon} />
              <Text>{t("idle.prestige.request")}</Text>
            </Button>
          </View>
      }>
        <View class={`bg-tile shadow-glow-lg shadow-warn inset-ring-regular inset-ring-warn p-3 gap-3`} role="alertdialog" label={t("idle.prestige.ask", { gain: props.gain, currency: props.currency })} onAction={{ back: (e) => props.onCancel?.(e) }}>
          <Text class="text-fg m-0">{t("idle.prestige.ask", { gain: props.gain, currency: props.currency })}</Text>
          <View class="flex-row flex-wrap items-center w-full justify-end gap-2">
            <Button class={`min-h-11 min-w-30 gap-1 compact:min-w-0 compact:grow compact:shrink compact:basis-0 pointer-coarse:grow pointer-coarse:shrink pointer-coarse:basis-0`} onPress={(e) => props.onCancel?.(e)}>
              <Text>{t("idle.prestige.cancel")}</Text>
            </Button>
            <Button class={`text-danger min-h-11 min-w-30 gap-1 compact:min-w-0 compact:grow compact:shrink compact:basis-0 pointer-coarse:grow pointer-coarse:shrink pointer-coarse:basis-0`} onPress={(e) => props.onConfirm?.(e)}>
              <Text>{t("idle.prestige.confirm")}</Text>
            </Button>
          </View>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<IdlePrestigeProps>> = {
  default: {"title":"","currency":"Souls","icon":"skull","held":"12","gain":"+48","rows":[{"id":"income","icon":"coin","label":"Income bonus","now":"×1.12","after":"×1.60"},{"id":"speed","icon":"bolt","label":"Cycle speed","now":"+5%","after":"+24%"}],"requirement":"Reach 1M gold","progress":1,"progressText":"1.2M / 1M","met":true,"confirming":false},
  disabled: {"title":"","currency":"Souls","icon":"skull","held":"0","gain":"+0","rows":[{"id":"income","icon":"coin","label":"Income bonus","now":"×1.00","after":"×1.00"}],"requirement":"Reach 1M gold","progress":0.64,"progressText":"640K / 1M","met":false,"confirming":false},
  confirming: {"title":"","currency":"Souls","icon":"skull","held":"12","gain":"+48","rows":[{"id":"income","icon":"coin","label":"Income bonus","now":"×1.12","after":"×1.60"},{"id":"speed","icon":"bolt","label":"Cycle speed","now":"+5%","after":"+24%"}],"requirement":"Reach 1M gold","progress":1,"progressText":"1.2M / 1M","met":true,"confirming":true},
  empty: {"title":"","currency":"Souls","icon":"skull","held":"0","gain":"+1","rows":[],"requirement":"","progress":-1,"progressText":"","met":true,"confirming":false},
  overflow: {"title":"Ascension Beyond the Veil of the Seventh Celestial Sphere","currency":"Fragments of Forgotten Eternity","icon":"star","held":"1.234 quattuordecillion","gain":"+98.76 quindecillion","rows":[{"id":"r1","icon":"coin","label":"Multiplier applied to every single generator in every realm","now":"×1,234,567","after":"×98,765,432,100"},{"id":"r2","icon":"bolt","label":"Speed","now":"+5%","after":"+24%"},{"id":"r3","icon":"gem","label":"Gem chance","now":"1%","after":"3%"},{"id":"r4","icon":"heart","label":"Offline cap","now":"8h","after":"12h"},{"id":"r5","icon":"crown","label":"Starting gold","now":"0","after":"1M"}],"requirement":"Reach one undecillion gold in a single run without prestiging","progress":0.42,"progressText":"420 decillion / 1 undecillion","met":false,"confirming":false},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"back":"Cancel"} as const;
