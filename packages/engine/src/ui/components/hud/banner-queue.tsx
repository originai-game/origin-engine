import { For, Icon, Image, Show, Text, View, packClass } from "@origingame/engine/ui";
import type { Texture } from "@origingame/engine";

export type BannerTone = "neutral" | "accent" | "success" | "warn" | "danger" | "epic" | "legendary";

export interface BannerItem {
  /** A stable id; a new id enters, a removed one leaves. */
  id: string;
  /** A texture (og.load.texture); 0 or absent draws `glyph` instead. */
  icon?: Texture | 0;
  /** An icon name, in art's place. */
  glyph?: string;
  /** The small line above the title (Achievement unlocked, New quest). */
  kicker?: string;
  /** The banner's headline. */
  title: string;
  /** A line under the title; empty hides it. */
  subtitle?: string;
  /** The kicker's and icon's colour. */
  tone?: BannerTone;
  /** Set while the banner plays its exit, before the game removes it. */
  leaving?: boolean;
}

export interface BannerQueueProps {
  /** The banners showing, oldest first; the game owns timing and the queue's length. */
  items?: ReadonlyArray<BannerItem> | undefined;
  /** Play the unlock cue as each banner enters. Default true. */
  sound?: boolean | undefined;
  /** Classes after its own (a placement). */
  class?: string | undefined;
}

const toneText: Record<BannerTone, string> = {
  neutral: "text-fg-muted",
  accent: "text-hud-accent",
  success: "text-success",
  warn: "text-warn",
  danger: "text-danger",
  epic: "text-epic",
  legendary: "text-legendary",
};

/**
 * A stack of non-blocking banners at the top centre of the screen
 * (achievements, milestones, a quest unlocked). Each keyed item enters with
 * the unlock cue and plays its exit while `leaving`; the game decides when
 * each one arrives and goes. The stack never takes input. Each banner is
 * dressed by `BannerQueue.item`.
 */
export function BannerQueue(props: BannerQueueProps) {
  return (
    <View
      class={`absolute inset-x-0 top-0 flex-col items-center gap-y-2 px-4 pt-[calc(env(hud-inset-top)+1rem)] pointer-events-none ${props.class ?? ""}`}
      role="log"
      live="polite"
    >
      <For each={props.items ?? []} key={(item) => item.id}>
        {(item) => (
          <View
            class={packClass("BannerQueue", "item")}
            data-leaving={item().leaving === true}
            sound={props.sound === false ? undefined : "unlock"}
            role="status"
          >
            <View class="shrink-0 size-10 rounded-control flex-row items-center justify-center bg-raised-lo">
              <Show when={item().icon} fallback={
                <Show when={item().glyph}>
                  <Icon class={`size-6 ${toneText[item().tone ?? "accent"]}`} name={item().glyph} />
                </Show>
              }>
                <Image class="size-10 rounded-control" src={item().icon as Texture} />
              </Show>
            </View>
            <View class="grow shrink basis-0 min-w-0 flex-col gap-y-0.5">
              <Show when={item().kicker}>
                <Text class={`text-xs font-semibold uppercase tracking-label truncate ${toneText[item().tone ?? "accent"]}`}>{item().kicker}</Text>
              </Show>
              <Text class="text-base font-semibold truncate">{item().title}</Text>
              <Show when={item().subtitle}>
                <Text class="text-sm text-fg-muted truncate">{item().subtitle}</Text>
              </Show>
            </View>
          </View>
        )}
      </For>
    </View>
  );
}

export const ground = "world";

export const fixtures: Record<string, Partial<BannerQueueProps>> = {
  default: { items: [{ id: "a", glyph: "trophy", kicker: "Achievement unlocked", title: "First Blood", subtitle: "Defeat your first foe", tone: "legendary" }] },
  stack: {
    items: [
      { id: "a", glyph: "flag", kicker: "New quest", title: "The Drowned Abbey", tone: "accent" },
      { id: "b", glyph: "star", kicker: "Level up", title: "Level 12", subtitle: "2 skill points", tone: "success" },
      { id: "c", glyph: "gem", kicker: "Epic drop", title: "Tidecaller's Mantle", tone: "epic", leaving: true },
    ],
  },
  long: { items: [{ id: "a", glyph: "trophy", kicker: "Achievement unlocked after a very long time", title: "Keeper of the Seventh Bell beneath the Mere, Last of the Tidebound Choir", subtitle: "Ring every bell in the abbey before the tide turns", tone: "legendary" }] },
};
