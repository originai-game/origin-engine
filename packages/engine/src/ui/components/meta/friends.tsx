import { For, Icon, Scroll, Show, Text, type UiEvent, View, t } from "@origingame/engine/ui";
import { Button } from "../../controls/button";
import "./general.en.ogt";

export interface FriendsProps {
  /** The panel's heading; empty takes the group's (Friends). */
  title: string;
  /** Friend requests waiting for an answer: keyed rows of { id, name, avatar, note }: avatar an icon of the style's atlas; note a second line as the game words it (3 mutual friends), empty hides it. Empty hides the section. */
  requests: ReadonlyArray<{ id: string; name: string; avatar: string; note: string }>;
  /** The friends in the game's order: keyed rows of { id, name, avatar, status, activity, canInvite, invited }: status online, ingame, away or offline, shown in words and by a mark; activity what they are doing (Ranked match, Last seen 2 d ago), empty hides it; canInvite the game will send an invite; invited one is pending. */
  friends: ReadonlyArray<{ id: string; name: string; avatar: string; status: string; activity: string; canInvite: boolean; invited: boolean }>;
  /** The friend the game marks as chosen (data-selected, aria-current); empty is none. */
  selected: string;
  /** The accept event, with the row's key where it comes from a row. */
  onAccept?: (e: UiEvent, key?: string) => void;
  /** The add event, with the row's key where it comes from a row. */
  onAdd?: (e: UiEvent, key?: string) => void;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
  /** The decline event, with the row's key where it comes from a row. */
  onDecline?: (e: UiEvent, key?: string) => void;
  /** The invite event, with the row's key where it comes from a row. */
  onInvite?: (e: UiEvent, key?: string) => void;
}

/**
 * The friends panel: a band with the title and Add friend, the friend
 * requests with Accept and Decline, then the friends with an avatar, the
 * name, the status in words beside a mark (online, in game, away, offline),
 * what they are doing and Invite. The game owns the list, presence and
 * whether an invite may go; the panel reports add, accept and decline (a
 * request's key), choose and invite (a friend's key). It fills its height
 * and scrolls inside it.
 */
export function Friends(props: FriendsProps) {
  return (
    <View class={`grow shrink basis-0 border-rim bg-linear-to-b from-glass-hi to-glass-lo shadow-panel text-fg p-4 gap-2 rounded-panel border disabled:text-fg-subtle`} label={props.title !== '' ? props.title : t("general.friends.name")}>
      <View class={`flex-row flex-wrap items-center min-h-12 border-b border-rim-soft w-full py-1 px-4 gap-3`}>
        <Text class={`grow shrink font-display text-base font-bold leading-tight tracking-[-0.01em] text-fg m-0`}>
          {props.title !== '' ? props.title : t("general.friends.name")}
        </Text>
        <Button class="min-h-11" onPress={(e) => props.onAdd?.(e)}>
          <Icon class="size-4" name="plus" />
          <Text>{t("general.friends.add")}</Text>
        </Button>
      </View>
      <Scroll class="gap-3">
        <Show when={props.requests.length}>
          <View class="w-full gap-1" label={t("general.friends.requests")}>
            <Text class={`tracking-[0.14em] uppercase text-gold font-bold text-caption py-0.5 px-1 m-0`}>
              {t("general.friends.requests")}
            </Text>
            <View class="w-full p-0 m-0 gap-0.5">
              <For each={props.requests}>
                {(r) => (
                <View class={`flex-row flex-wrap items-center rounded-tl-[11.424px] rounded-tr-none rounded-br-[11.424px] rounded-bl-none text-fg transition duration-317 shadow-[inset_0px_0px_0px_1px_#ffcf5a73,inset_3.424px_0px_0px_#ffcf5a] w-full py-1 px-2 gap-2 data-selected:text-fg disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45`} label={r().name}>
                  <View class={`flex-row items-center shadow-[inset_0px_0px_0px_1px_#ffcf5a99] justify-center text-gold gap-0 size-12 rounded-capsule`}>
                    <Icon class="size-6 shrink" name={r().avatar} />
                  </View>
                  <View class="grow shrink basis-0 gap-0">
                    <Text class={`font-semibold text-lg text-fg text-start whitespace-nowrap text-ellipsis m-0 overflow-hidden`}>
                      {r().name}
                    </Text>
                    <Show when={r().note}>
                      <Text class={`text-caption text-fg-muted text-start whitespace-nowrap text-ellipsis overflow-hidden`}>
                        {r().note}
                      </Text>
                    </Show>
                  </View>
                  <View class="flex-row items-center ml-auto gap-1">
                    <Button class="text-accent min-h-11 min-w-11" label={t("general.friends.acceptName", { name: r().name })} onPress={(e) => props.onAccept?.(e, r().id)}>
                      <Icon class="size-4" name="check" />
                      <Text class="compact:hidden">{t("general.friends.accept")}</Text>
                    </Button>
                    <Button class="min-h-11 min-w-11" label={t("general.friends.declineName", { name: r().name })} onPress={(e) => props.onDecline?.(e, r().id)}>
                      <Icon class="size-4" name="close" />
                      <Text class="compact:hidden">{t("general.friends.decline")}</Text>
                    </Button>
                  </View>
                </View>
                )}
              </For>
            </View>
          </View>
        </Show>
        <View class="w-full gap-1" label={t("general.friends.list")}>
          <Show when={props.requests.length && props.friends.length}>
            <Text class={`tracking-[0.14em] uppercase text-gold font-bold text-caption py-0.5 px-1 m-0`}>
              {t("general.friends.list")}
            </Text>
          </Show>
          <Show when={props.friends.length} fallback={
              <View class="w-full items-center text-fg-muted py-6 px-0 gap-2" role="status">
                <Icon class="size-12" name="person" />
                <Text class="m-0">{t("general.friends.empty")}</Text>
              </View>
          }>
            <View class="w-full p-0 m-0 gap-0.5" label={t("general.friends.list")}>
              <For each={props.friends}>
                {(f) => (
                <View class="flex-row items-center w-full gap-2">
                  <View class={`grow shrink basis-0 min-h-11 rounded-tl-[11.424px] rounded-tr-none rounded-br-[11.424px] rounded-bl-none text-fg transition duration-317 py-1 px-2 hover:bg-transparent data-selected:bg-transparent data-selected:bg-linear-to-r data-selected:from-[#38e8ff3d] data-selected:via-[#38e8ff14] data-selected:via-55% data-selected:to-[#38e8ff08] data-selected:text-fg data-selected:shadow-[inset_3.424px_0px_0px_#38e8ff,inset_0px_1px_0px_#ffffff2e] disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45`} role="button" current={f().id === props.selected} label={f().name} onPress={(e) => props.onChoose?.(e, f().id)} data-status-away={(f().status) === 'away'} data-status-ingame={(f().status) === 'ingame'} data-status-offline={(f().status) === 'offline'} data-status-online={(f().status) === 'online'} data-selected={f().id === props.selected}>
                    <View class="flex-row items-center w-full gap-2">
                      <View class="group/avatar-wrap" data-status-away={(f().status) === 'away'} data-status-ingame={(f().status) === 'ingame'} data-status-offline={(f().status) === 'offline'} data-status-online={(f().status) === 'online'}>
                        <View class={`flex-row items-center shadow-[inset_0px_0px_0px_1px_#ffffff14,inset_0px_1px_0px_#ffffff1f] justify-center text-fg gap-0 size-12 rounded-capsule group-data-status-away/avatar-wrap:opacity-45`} data-status-away={(f().status) === 'away'} data-status-ingame={(f().status) === 'ingame'} data-status-offline={(f().status) === 'offline'} data-status-online={(f().status) === 'online'}>
                          <Icon class="size-6 shrink" name={f().avatar} />
                        </View>
                        <View class={`bg-current text-fg absolute right-[-1.5px] bottom-[-1.5px] rotate-45 shadow-[inset_0px_0px_0px_1px_#ffffff14,inset_0px_1px_0px_#ffffff1f] size-2 data-status-offline:bg-none data-status-offline:bg-fg-subtle data-tone-success:bg-none data-tone-success:bg-success data-tone-success:text-bg data-tone-warn:bg-none data-tone-warn:bg-warn data-tone-warn:text-bg data-tone-info:bg-none data-tone-info:bg-info data-tone-info:text-bg group-data-status-offline/avatar-wrap:opacity-45`} data-status-away={(f().status) === 'away'} data-status-ingame={(f().status) === 'ingame'} data-status-offline={(f().status) === 'offline'} data-status-online={(f().status) === 'online'} data-tone-success={(f().status === 'online' ? 'success' : f().status === 'ingame' ? 'info' : f().status === 'away' ? 'warn' : '') === "success"} data-tone-info={(f().status === 'online' ? 'success' : f().status === 'ingame' ? 'info' : f().status === 'away' ? 'warn' : '') === "info"} data-tone-warn={(f().status === 'online' ? 'success' : f().status === 'ingame' ? 'info' : f().status === 'away' ? 'warn' : '') === "warn"} />
                      </View>
                      <View class="grow shrink basis-0 gap-0">
                        <Text class={`font-semibold text-lg text-fg text-start whitespace-nowrap text-ellipsis m-0 overflow-hidden`}>
                          {f().name}
                        </Text>
                        <View class="flex-row items-baseline w-full gap-2">
                          <Text class={`tracking-xs text-fg-muted whitespace-nowrap font-bold m-0 data-status-online:text-success data-status-ingame:text-info data-status-away:text-[#ff8a2e] data-status-offline:text-fg-muted`} data-status-away={(f().status) === 'away'} data-status-ingame={(f().status) === 'ingame'} data-status-offline={(f().status) === 'offline'} data-status-online={(f().status) === 'online'}>
                            {f().status === 'ingame' ? t("general.friends.ingame") : f().status === 'online' ? t("general.friends.online") : f().status === 'away' ? t("general.friends.away") : t("general.friends.offline")}
                          </Text>
                          <Show when={f().activity}>
                            <Text class={`text-caption text-fg-muted text-start whitespace-nowrap text-ellipsis overflow-hidden shrink`}>
                              {f().activity}
                            </Text>
                          </Show>
                        </View>
                      </View>
                    </View>
                  </View>
                  <Show when={f().canInvite || f().invited}>
                    <Button class="min-h-11 min-w-11" data-invited={f().invited} disabled={f().invited} label={f().invited ? t("general.friends.invitedName", { name: f().name }) : t("general.friends.inviteName", { name: f().name })} onPress={(e) => props.onInvite?.(e, f().id)}>
                      <Icon class="size-4" name={f().invited ? 'clock' : 'plus'} />
                      <Text class="compact:hidden">
                        {f().invited ? t("general.friends.invited") : t("general.friends.invite")}
                      </Text>
                    </Button>
                  </Show>
                </View>
                )}
              </For>
            </View>
          </Show>
        </View>
      </Scroll>
    </View>
  );
}

export const fixtures: Record<string, Partial<FriendsProps>> = {
  default: {"title":"","requests":[{"id":"r1","name":"Idris","avatar":"person","note":"2 mutual friends"}],"friends":[{"id":"aerin","name":"Aerin","avatar":"person","status":"ingame","activity":"Ranked match","canInvite":false,"invited":false},{"id":"lys","name":"Lys","avatar":"person","status":"online","activity":"In the lobby","canInvite":true,"invited":false},{"id":"maren","name":"Maren","avatar":"person","status":"away","activity":"","canInvite":true,"invited":true},{"id":"sela","name":"Sela","avatar":"person","status":"offline","activity":"Last seen 2 d ago","canInvite":false,"invited":false}],"selected":"lys"},
  empty: {"title":"","requests":[],"friends":[],"selected":""},
  requests: {"title":"","requests":[{"id":"r1","name":"Idris","avatar":"person","note":"2 mutual friends"},{"id":"r2","name":"Ysolde","avatar":"person","note":""}],"friends":[],"selected":""},
  overflow: {"title":"Companions and Allies Across the Realm","requests":[{"id":"r1","name":"Bartholomew Fitzgerald-Montgomery the Third","avatar":"person","note":"Sent you a request a very long time ago, with many mutual friends"}],"friends":[{"id":"f1","name":"Maximilian the Unbreakable Lord of the Northern Wastes","avatar":"person","status":"ingame","activity":"Ranked match — Season 12 grand final, round 3 of 5","canInvite":false,"invited":false},{"id":"f2","name":"Ana","avatar":"person","status":"online","activity":"","canInvite":true,"invited":false},{"id":"f3","name":"Bo","avatar":"person","status":"online","activity":"Lobby","canInvite":true,"invited":false},{"id":"f4","name":"Cy","avatar":"person","status":"away","activity":"","canInvite":true,"invited":false},{"id":"f5","name":"Di","avatar":"person","status":"offline","activity":"Last seen 1 h ago","canInvite":false,"invited":false},{"id":"f6","name":"Ed","avatar":"person","status":"offline","activity":"Last seen 3 d ago","canInvite":false,"invited":false},{"id":"f7","name":"Flo","avatar":"person","status":"offline","activity":"","canInvite":false,"invited":false},{"id":"f8","name":"Gus","avatar":"person","status":"offline","activity":"","canInvite":false,"invited":false}],"selected":"f2"},
};
