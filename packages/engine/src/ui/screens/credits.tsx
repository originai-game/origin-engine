import {For, Icon, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";
import { Art } from "../controls/art";
import { Backdrop } from "../controls/backdrop";
import { HintBar } from "../controls/hint-bar";
import { Button } from "../controls/button";
import { engineCredits } from "./engine-credits";
import "./credits.en.strings.ts";

export interface CreditsScreenProps {
  /** Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. */
  class?: string | undefined;
  /** The sections: { title, names: [{ name, role }] }. The engine's own credits (its art's authors) follow them. */
  sections: ReadonlyArray<{ id: string; names: ReadonlyArray<{ id: string; name: string; role: string }>; title: string }>;
  /** How far the roll has run, CSS px. */
  offset: number;
  /** The key art's texture under the roll's veil (og.load.texture; 0: the style's dark ground). */
  keyArt: Texture | 0;
  /** The game's logo texture at the head of the roll (og.load.texture; 0: the name set as a wordmark). */
  logo: Texture | 0;
  /** The game's name, set as the pack's wordmark at the head of the roll where there is no logo. */
  name?: string | undefined;
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
    <View class={`absolute inset-0 text-fg overflow-hidden ${packClass("Screen", "frame")} ${props.class ?? ""}`} onPress={(e) => props.onFaster?.(e)} onAction={{ confirm: (e) => props.onFaster?.(e), back: (e) => props.onBack?.(e) }}>
      <Backdrop art={props.keyArt} veil="full" />
      <View class="grow shrink basis-0 overflow-hidden mask-[linear-gradient(to_bottom,transparent,black_6%,black_80%,transparent_90%)]">
        <View class="absolute top-0 inset-x-0 items-center gap-y-14 pt-24 pb-16 px-6 compact:gap-y-10 compact:pt-16" style={{ translate: `0 ${-props.offset}px` }}>
          <Show when={props.logo === 0} fallback={<Art class="w-full max-w-115 aspect-12/5 h-auto object-contain" kind="logo" src={props.logo} />}>
            <Show when={props.name}>
              <Text class={`text-center max-w-full m-0 ${packClass("Title", "wordmark")} compact:text-display-xl`}>{props.name}</Text>
            </Show>
          </Show>
          <For each={[...props.sections, ...engineCredits.map((s) => ({ ...s, title: t(s.title) }))]}>
            {(s) => (
            <View class="w-full max-w-140 items-center text-center gap-y-5">
              <Text class={`m-0 ${packClass("Screen", "note")} text-lg font-bold uppercase tracking-kicker text-fg`}>
                {s().title}
              </Text>
              <For each={s().names}>
                {(n) => (
                <View class="items-center text-center gap-y-1">
                  <Text class={`m-0 ${packClass("Screen", "heading")} text-4xl normal-case compact:text-2xl`}>
                    {n().name}
                  </Text>
                  <Show when={n().role}>
                    <Text class={`m-0 ${packClass("Screen", "note")} text-base uppercase tracking-label font-bold`}>
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
      <View class="absolute inset-x-0 bottom-0 flex-row justify-end pointer-events-none pb-[calc(env(hud-inset-bottom)+1.5rem)] pr-[calc(env(hud-inset-right)+3rem)] compact:pb-[calc(env(hud-inset-bottom)+0.75rem)] compact:pr-[calc(env(hud-inset-right)+1.25rem)]">
        <HintBar hints={[{ action: "back", label: t("screen.common.back") }, { action: "confirm", label: t("screen.credits.faster") }]} />
      </View>
      <View class="absolute top-0 left-0 pt-[calc(env(hud-inset-top)+1rem)] pl-[calc(env(hud-inset-left)+1rem)] hidden pointer-coarse:flex">
        <Button size="sm" onPress={(e) => props.onBack?.(e)}>
          <Icon class="size-4 rtl:-scale-x-100 rtl:scale-y-100" name="chevronLeft" />
          <Text>{t("screen.common.back")}</Text>
        </Button>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<CreditsScreenProps>> = {
  default: {"keyArt":0,"logo":0,"name":"Lantern Coast","offset":0,"sections":[{"id":"item-0","names":[{"id":"item-0","name":"Ada Lin","role":""}],"title":"Directed by"},{"id":"item-1","names":[{"id":"item-0","name":"Tomas Reyes","role":"Lead artist"},{"id":"item-1","name":"Mira Okafor","role":"Environment"}],"title":"Art"},{"id":"item-2","names":[{"id":"item-0","name":"Jun Sato","role":"Composer"}],"title":"Music"}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"confirm":"Faster","back":"Back"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"credits","layer":"menu","focus":"","under":"hide","input":"menu","pause":false,"capture":true,"remember":true,"back":"pop"} as const;
