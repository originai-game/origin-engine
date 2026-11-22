import {For, Icon, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import { Art } from "../controls/art";
import { Button } from "../controls/button";
import "./credits.en.ogt";

export interface CreditsScreenProps {
  /** The sections: { title, names: [{ name, role }] }. */
  sections: ReadonlyArray<{ id: string; names: ReadonlyArray<{ id: string; name: string; role: string }>; title: string }>;
  /** How far the roll has run, CSS px. */
  offset: number;
  /** The key art's texture under the roll's veil (og.load.texture; 0: the style's dark ground). */
  keyArt: Texture | 0;
  /** The game's logo texture at the head of the roll (og.load.texture; 0: the logo slot's placeholder). */
  logo: Texture | 0;
  /** Content the caller supplies (was the end snippet). */
  end?: (() => JSX.Element) | undefined;
  /** Content the caller supplies (was the licences snippet). */
  licences?: (() => JSX.Element) | undefined;
  /** The back event, with the row's key where it comes from a row. */
  onBack?: (e: UiEvent, key?: string) => void;
  /** The faster event, with the row's key where it comes from a row. */
  onFaster?: (e: UiEvent, key?: string) => void;
}

/**
 * The credits: the sections' titles and names rolling over the key art's
 * veil, the logo at the head, and the end and licences the caller supplies.
 * The game moves the roll (offset); it reports onBack and onFaster.
 */
export function CreditsScreen(props: CreditsScreenProps) {
  return (
    <View class={`grow shrink basis-0 text-fg p-0 gap-0 overflow-hidden`} onPress={(e) => props.onFaster?.(e)} onAction={{ confirm: (e) => props.onFaster?.(e) }}>
      <Show when={props.keyArt !== 0}>
        <Art class="absolute object-cover inset-0 size-full" kind="keyArt" src={props.keyArt} />
      </Show>
      <View class={`absolute inset-0 compact:bg-linear-to-b compact:from-shade-deep compact:via-shade-deep compact:via-40% compact:to-shade-deep compact:bg-transparent`} />
      <View class="flex-row items-center py-4 px-8 gap-2 compact:py-2 compact:px-3">
        <Button class={`shrink ${packClass("Button", "plate")}`} onPress={(e) => props.onBack?.(e)}>
          <View class="flex-row items-center gap-x-1">
            <Icon class="size-4 rtl:-scale-x-100 rtl:scale-y-100 shrink" name="chevronLeft" />
            <Text class="shrink">{t("screen.common.back")}</Text>
          </View>
        </Button>
      </View>
      <View class="grow shrink basis-0 overflow-hidden">
        <View class="absolute top-0 items-center gap-y-12 py-12 px-4 inset-x-0 compact:gap-y-8 compact:py-6 compact:px-4" style={{ translate: `0 ${-props.offset}px` }}>
          <Art class="w-full max-w-115 aspect-12/5 h-auto object-contain" kind="logo" src={props.logo} />
          <For each={props.sections}>
            {(s) => (
            <View class="w-full max-w-140 items-center text-center gap-y-4">
              <View class="flex-row items-center justify-center gap-x-3 w-full">
                <Text class={`shrink basis-10 min-w-2 h-px bg-linear-to-r from-transparent to-hud-accent`} />
                <Text class={`tracking-kicker uppercase text-hud-accent-ink shrink text-caption font-bold m-0 compact:tracking-label`}>
                  {s().title}
                </Text>
                <Text class={`shrink basis-10 min-w-2 h-px bg-linear-to-r from-transparent to-hud-accent -scale-x-100 scale-y-100`} />
              </View>
              <For each={s().names}>
                {(n) => (
                <View class="items-center text-center gap-y-0.5">
                  <Text class={`text-shadow-xs text-shadow-bg text-4xl leading-tight font-semibold m-0 compact:text-2xl`}>
                    {n().name}
                  </Text>
                  <Show when={n().role}>
                    <Text class={`text-caption font-semibold tracking-xl uppercase text-fg-muted m-0`}>
                      {n().role}
                    </Text>
                  </Show>
                </View>
                )}
              </For>
            </View>
            )}
          </For>
          <View class="w-full max-w-140">{props.end?.()}</View>
          <View class="w-full max-w-140">{props.licences?.()}</View>
        </View>
      </View>
      <View class="flex-row absolute right-6 bottom-4 items-center gap-x-2 text-caption text-fg-muted pointer-coarse:hidden compact:hidden">
        <Text class="text-xs leading-loose font-bold text-hud-accent-fg bg-linear-to-b from-key-hi to-key-lo shadow-key py-0 px-1 rounded-control shrink">
          {t("screen.credits.fasterKey")}
        </Text>
        <Text class="shrink">{t("screen.credits.faster")}</Text>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<CreditsScreenProps>> = {
  default: {"keyArt":0,"logo":0,"offset":0,"sections":[{"id":"item-0","names":[{"id":"item-0","name":"Ada Lin","role":""}],"title":"Directed by"},{"id":"item-1","names":[{"id":"item-0","name":"Tomas Reyes","role":"Lead artist"},{"id":"item-1","name":"Mira Okafor","role":"Environment"}],"title":"Art"},{"id":"item-2","names":[{"id":"item-0","name":"Jun Sato","role":"Composer"}],"title":"Music"}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Faster","back":"Back"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"credits","layer":"menu","focus":"","under":"hide","input":"menu","pause":false,"capture":true,"remember":true,"back":"pop"} as const;
