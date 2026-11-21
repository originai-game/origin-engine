import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { EmptyState } from "../base/empty-state";
import { ErrorState } from "../base/error-state";
import { Spinner } from "../base/spinner";
import "./meta.en.ogt";

export interface InboxProps {
  /** Heading; empty uses Inbox. */
  title?: string;
  /** The game's count or capacity wording, such as 2 unread · 8 / 50. */
  summary?: string;
  /** Keyed { id, subject, sender, date, preview, body, unread, hasAttachments, claimable, claimed, expired, canDelete } messages. Text is plain, not HTML. The game updates read/claim/delete facts after intents. Default []. */
  messages?: ReadonlyArray<{ id: string; subject: string; sender: string; date: string; preview: string; body: string; unread: boolean; hasAttachments: boolean; claimable: boolean; claimed: boolean; expired: boolean; canDelete: boolean }>;
  /** Keyed { id, message, name, icon, amount } rewards; message is the owning message id. Default []. */
  attachments?: ReadonlyArray<{ id: string; message: string; name: string; icon: string; amount: string }>;
  /** Selected message id, owned by the game; empty shows a choose-message prompt. */
  selected?: string;
  /** Loading messages; no message actions are available. Default false. */
  loading?: boolean;
  /** A claim/delete operation is pending; prevent duplicate mutations. Default false. */
  busy?: boolean;
  /** Loading failed; show recovery advice and retry. */
  error?: string;
  /** The choose event, with the row's key where it comes from a row. */
  onChoose?: (e: UiEvent, key?: string) => void;
  /** The claim event, with the row's key where it comes from a row. */
  onClaim?: (e: UiEvent, key?: string) => void;
  /** The delete event, with the row's key where it comes from a row. */
  onDelete?: (e: UiEvent, key?: string) => void;
}

export function Inbox(props: InboxProps) {
  return (
    <View class={`grow shrink basis-0 border-rim bg-linear-to-b from-glass-hi to-glass-lo shadow-panel text-fg h-auto p-4 gap-3 rounded-panel border disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title || t("meta.inbox.title")} busy={(props.loading ?? false) || (props.busy ?? false)}>
      <View class={`flex-row flex-wrap items-center min-h-12 border-b border-rim-soft py-1 px-4 gap-2`}>
        <Text class={`grow shrink basis-0 font-display text-base font-bold leading-tight tracking-[-0.01em] text-fg m-0`}>
          {props.title || t("meta.inbox.title")}
        </Text>
        <Show when={props.summary}>
          <Text class={`text-caption text-fg bg-[#060910b8] border border-[#ffffff14] nine-chip-cut py-0.5 px-2 shrink ${packClass("Chip", "frame")}`}>
            {props.summary}
          </Text>
        </Show>
      </View>
      <Show when={(props.loading ?? false)} fallback={
          <Show when={props.error} fallback={
              <Show when={!(props.messages ?? []).length} fallback={
                  <View class={`grid grid-cols-[minmax(0,1fr)_minmax(0,1.5fr)] compact:grid-cols-[minmax(0,1fr)]`}>
                    <Scroll class={`max-h-140 p-0.5 m-0 gap-1 compact:min-h-[11.429rem] compact:max-h-48`} label={t("meta.inbox.messages")}>
                      <For each={(props.messages ?? [])}>
                        {(message) => (
                        <View>
                          <View class={`group/row nine-plate-cut text-fg transition duration-317 border border-[#4e586c] w-full text-start py-2 px-3 data-selected:bg-transparent data-selected:text-fg disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45`} role="button" pressed={message().id === props.selected} disabled={(props.busy ?? false)} onPress={(e) => props.onChoose?.(e, message().id)} data-not-unread={!(message().unread)} data-unread={message().unread} data-selected={message().id === props.selected}>
                            <View class="items-start w-full gap-2">
                              <Text class={`text-caption text-fg-muted`}>
                                {message().sender}
                              </Text>
                              <Text class={`font-bold text-lg text-fg w-full line-clamp-1 text-ellipsis group-data-not-unread/row:text-fg-muted`} label={message().subject}>
                                {message().subject}
                              </Text>
                              <Text class={`text-caption text-fg-muted`}>
                                {message().preview}
                              </Text>
                              <View class="flex-row flex-wrap items-center gap-2">
                                <Text class={`text-caption text-fg-muted shrink`}>
                                  {message().date}
                                </Text>
                                <Text class={`font-bold text-caption tabular-nums bg-raised-lo shadow-[inset_0px_0px_0px_1px_#4e586c] text-fg-subtle transition duration-317 tracking-[0.14em] uppercase py-0 px-2 rounded-capsule group-data-unread/row:text-gold shrink`}>
                                  {message().unread ? t("meta.inbox.unread") : t("meta.inbox.read")}
                                </Text>
                                <Show when={message().hasAttachments}>
                                  <Icon class={`text-gold size-4`} name="bag" />
                                </Show>
                              </View>
                            </View>
                          </View>
                        </View>
                        )}
                      </For>
                    </Scroll>
                    <View class="max-h-140 overflow-y-hidden bg-[#060910b8] shadow-[inset_0px_0px_0px_1px_#ffffff14,inset_0px_1px_0px_#ffffff1f] py-3 px-4 rounded-card compact:max-h-none">
                      <Show when={!props.selected}>
                        <Text class={`text-caption text-fg-muted m-0`}>
                          {t("meta.inbox.select")}
                        </Text>
                      </Show>
                      <For each={(props.messages ?? [])}>
                        {(message) => (
                        <Show when={message().id === props.selected}>
                          <View class={`items-start gap-2`} label={message().subject}>
                            <Text class={`font-bold text-lg text-fg w-full line-clamp-1 text-ellipsis m-0`}>
                              {message().subject}
                            </Text>
                            <View class="flex-row flex-wrap items-center gap-2">
                              <Text class={`text-caption text-fg-muted shrink`}>
                                {message().sender}
                              </Text>
                              <Text class={`text-caption text-fg-muted shrink`}>
                                {message().date}
                              </Text>
                            </View>
                            <Text class={`text-fg-muted m-0`}>
                              {message().body}
                            </Text>
                            <Show when={message().hasAttachments}>
                              <View class={`w-full gap-1`} label={t("meta.inbox.attachments")}>
                                <Text class="font-bold text-caption tracking-[0.14em] uppercase text-fg-muted m-0">
                                  {t("meta.inbox.attachments")}
                                </Text>
                                <View class="flex-row flex-wrap p-0 m-0 gap-1">
                                  <For each={(props.attachments ?? [])}>
                                    {(attachment) => (
                                    <Show when={attachment().message === message().id}>
                                      <View class={`flex-row items-center pr-3 pl-2 bg-[#060910b8] border border-[#ffcf5a99] nine-chip-cut py-1 gap-2 shrink ${packClass("Chip", "frame")}`}>
                                        <Icon class={`text-gold size-4`} name={attachment().icon} />
                                        <View class={`shrink`}>
                                          <Text>{attachment().name}</Text>
                                        </View>
                                        <View class={`text-gold font-bold shrink`}>
                                          <Text class="tabular-nums">{attachment().amount}</Text>
                                        </View>
                                      </View>
                                    </Show>
                                    )}
                                  </For>
                                </View>
                                <Text class={`text-caption text-fg-muted data-state-expired:text-danger data-state-claimed:text-success`} data-state-claimed={(message().expired ? 'expired' : message().claimed ? 'claimed' : 'available') === 'claimed'} data-state-expired={(message().expired ? 'expired' : message().claimed ? 'claimed' : 'available') === 'expired'}>
                                  {message().expired ? t("meta.inbox.expired") : message().claimed ? t("meta.claimed") : t("meta.available")}
                                </Text>
                              </View>
                            </Show>
                            <View class={`flex-row flex-wrap items-center gap-2`}>
                              <Show when={message().hasAttachments}>
                                <View class={`text-accent min-h-11 shrink`} role="button" disabled={(props.busy ?? false) || !message().claimable || message().claimed || message().expired} onPress={(e) => props.onClaim?.(e, message().id)}>
                                  <Text class="whitespace-normal">{t("meta.inbox.claim")}</Text>
                                </View>
                              </Show>
                              <Show when={message().canDelete}>
                                <View class={`min-h-11 shrink`} role="button" disabled={(props.busy ?? false)} onPress={(e) => props.onDelete?.(e, message().id)}>
                                  <Text class="whitespace-normal">{t("meta.inbox.delete")}</Text>
                                </View>
                              </Show>
                            </View>
                          </View>
                        </Show>
                        )}
                      </For>
                    </View>
                  </View>
              }>
                <EmptyState title={t("meta.inbox.empty")} icon="bag" />
              </Show>
          }>
            <ErrorState body={props.error ?? ''} />
          </Show>
      }>
        <View class="flex-row items-center p-4 gap-2" role="status">
          <Spinner name={t("meta.loading")} />
          <Text class="shrink">{t("meta.loading")}</Text>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<InboxProps>> = {
  default: {"summary":"1 unread · 2 / 50","selected":"welcome","messages":[{"id":"welcome","subject":"Welcome to the expedition","sender":"Expedition Guild","date":"16 Nov","preview":"Supplies for your first journey.","body":"Welcome, adventurer. These supplies will help you reach the northern camp.\n\nGood luck on your first expedition.","unread":true,"hasAttachments":true,"claimable":true,"claimed":false,"expired":false,"canDelete":true},{"id":"notice","subject":"Scheduled maintenance","sender":"Game team","date":"15 Nov","preview":"Service resumes at 09:00 UTC.","body":"The world will be unavailable during maintenance. All progress is safe.","unread":false,"hasAttachments":false,"claimable":false,"claimed":false,"expired":false,"canDelete":true}],"attachments":[{"id":"gold","message":"welcome","name":"Gold","icon":"coin","amount":"500"},{"id":"potions","message":"welcome","name":"Health potions","icon":"potion","amount":"3"}]},
  empty: {},
  loading: {"loading":true},
  error: {"error":"Messages could not be loaded. Reconnect and try again."},
  expired: {"selected":"old","messages":[{"id":"old","subject":"Rewards from the previous season","sender":"Season team","date":"1 Oct","preview":"The claim window has closed.","body":"These attachments have expired. See you next season.","unread":false,"hasAttachments":true,"claimable":false,"claimed":false,"expired":true,"canDelete":true}]},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"choose":"Read the message by stable key","claim":"Claim the selected message's attachments by stable message key","delete":"Request deleting the selected message by stable key","retry":"Retry loading messages"} as const;
