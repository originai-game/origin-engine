import { For, Icon, Text, type UiEvent, View } from "@origingame/engine/ui";
import { Button } from "../controls/button";

export interface NotificationsScreenProps {
  /** The game's own notifications, newest last, a keyed collection of { id, title, body, tone (info, success, warn, danger) }; the game removes one when it is read or times out. The toasts og.ui.toast queued ($toasts) follow them, with nothing to pass: each leaves on its own and a press dismisses it. */
  items: ReadonlyArray<{ id: string; title: string; body: string; tone: string }>;
  /** The toasts og.ui.toast queued, as the game's own keyed rows of { id, tone (info, success, warn, danger), text, leaving (its leave is playing): the game removes one when it leaves and a press dismisses it. */
  toasts?: ReadonlyArray<{ id: string; tone: string; text: string; leaving?: boolean }>;
  /** The dismiss event, with the row's key where it comes from a row. */
  onDismiss?: (e: UiEvent, key?: string) => void;
  /** The dismissToast event, with the row's key where it comes from a row. */
  onDismissToast?: (e: UiEvent, key?: string) => void;
}

/**
 * The notification stack in the HUD's top-right corner. A notification is
 * a glass plate with two corners cut, a rail down its leading edge and a
 * wash from it in its tone, the tone's mark in an octagon socket, its
 * title in small capitals of the tone and its body in white, and a quiet
 * close. A toast is the compact form: its mark and its words on a plate
 * that slides back out when it leaves. Each tone has its own hue (warn an
 * amber well apart from gold). Under reduced motion they appear in place.
 */
export function NotificationsScreen(props: NotificationsScreenProps) {
  return (
    <View class="items-end gap-2" role="log" live="polite">
      <For each={props.items}>
        {(n) => (
        <View class={`group/frame pr-1 pl-[16.571px] border-rim rounded-tl-[11.424px] rounded-tr-none rounded-br-[11.424px] rounded-bl-none shadow-[inset_0px_0px_0px_1px_#ffffff1f,inset_0px_1px_0px_#ffffff2e] text-fg items-center w-90 max-w-full py-2 gap-3 border data-tone-success:bg-transparent data-tone-warn:bg-transparent data-tone-danger:bg-transparent disabled:text-fg-subtle compact:w-full`} data-tone-danger={(n().tone) === 'danger'} data-tone-success={(n().tone) === 'success'} data-tone-warn={(n().tone) === 'warn'}>
          <View class="absolute left-0 w-[0.286rem] bg-info inset-y-0 group-data-tone-success/frame:bg-none group-data-tone-success/frame:bg-success group-data-tone-warn/frame:bg-none group-data-tone-warn/frame:bg-[#ff8a2e] group-data-tone-danger/frame:bg-none group-data-tone-danger/frame:bg-danger" />
          <View class={`bg-linear-to-b from-white/50 via-white/12 via-50% to-white/6 p-[1.5px] size-12`}>
            <View class="items-center justify-center w-full text-info group-data-tone-success/frame:text-success group-data-tone-warn/frame:text-[#ff8a2e] group-data-tone-danger/frame:text-danger shrink">
              <Icon class="size-4 shrink" name={n().tone === 'success' ? 'check' : n().tone === 'warn' ? 'bolt' : n().tone === 'danger' ? 'close' : 'star'} />
            </View>
          </View>
          <View class="grow shrink basis-0 gap-0.5">
            <Text class={`text-caption font-bold tracking-[0.14em] uppercase text-info m-0 group-data-tone-success/frame:text-success group-data-tone-warn/frame:text-[#ff8a2e] group-data-tone-danger/frame:text-danger`}>
              {n().title}
            </Text>
            <Text class={`font-semibold text-fg m-0`}>{n().body}</Text>
          </View>
          <Button class="text-fg-subtle self-start" onPress={(e) => props.onDismiss?.(e, n().id)} label={n().title}>
            <Icon class="size-4" name="close" />
          </Button>
        </View>
        )}
      </For>
      <For each={props.toasts}>
        {(t) => (
        <View class={`group/frame pr-0 pl-[0.286rem] border-rim rounded-tl-[11.424px] rounded-tr-none rounded-br-[11.424px] rounded-bl-none shadow-[inset_0px_0px_0px_1px_#ffffff1f,inset_0px_1px_0px_#ffffff2e] text-fg items-center w-auto max-w-full py-0 gap-3 border transition duration-240 ease-in data-leaving:translate-x-8 data-leaving:opacity-0 data-tone-success:bg-transparent data-tone-warn:bg-transparent data-tone-danger:bg-transparent disabled:text-fg-subtle compact:w-auto`} data-leaving={t().leaving === true} data-tone-danger={(t().tone) === 'danger'} data-tone-success={(t().tone) === 'success'} data-tone-warn={(t().tone) === 'warn'}>
          <View class="absolute left-0 w-[0.286rem] bg-info inset-y-0 group-data-tone-success/frame:bg-none group-data-tone-success/frame:bg-success group-data-tone-warn/frame:bg-none group-data-tone-warn/frame:bg-[#ff8a2e] group-data-tone-danger/frame:bg-none group-data-tone-danger/frame:bg-danger" />
          <Button class="w-full text-start gap-1 shrink" onPress={(e) => props.onDismissToast?.(e, t().id)}>
            <Icon class="text-info size-4 group-data-tone-success/frame:text-success group-data-tone-warn/frame:text-[#ff8a2e] group-data-tone-danger/frame:text-danger" name={t().tone === 'success' ? 'check' : t().tone === 'warn' ? 'bolt' : t().tone === 'danger' ? 'close' : 'star'} />
            <Text>{t().text}</Text>
          </Button>
        </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<NotificationsScreenProps>> = {
  default: {"items":[{"id":"1","title":"Achievement","body":"First blood","tone":"success"}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"dismiss":"Dismiss"} as const;

/** How the screen mounts: its layer, first focus, what lies under it, input, pause and back. */
export const screen = {"name":"notifications","layer":"toast"} as const;
