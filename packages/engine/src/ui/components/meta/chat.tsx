import {For, Icon, Input, Scroll, Show, Text, packClass, type UiEvent, View, formatNumber, t} from "@origingame/engine/ui";
import { Button } from "../../controls/button";
import "./general.en.ogt";

export interface ChatProps {
  /** The channels, keyed rows of { id, label, unread }: unread the count the game keeps, 0 hides its badge. Empty shows no tab row. */
  channels: ReadonlyArray<{ id: string; label: string; unread: number }>;
  /** The channel shown. */
  channel: string;
  /** The shown channel's messages, oldest first: keyed rows of { id, kind, sender, time, text }: kind player, own (the player's) or system; time as the game formats it; text wraps, and a long word breaks. */
  messages: ReadonlyArray<{ id: string; kind: string; sender: string; time: string; text: string }>;
  /** The message being written, two-way: value={draft} shows it; the field reports every change through onDraft with the text trimmed to maxLength, and the game clears it after a send. */
  draft: string;
  /** The longest message the game takes, in characters. */
  maxLength: number;
  /** The game takes a message now; false disables the field and Send (muted, slow mode). */
  canSend?: boolean;
  /** A line over the field as the game words it (Slow mode: 3 s, You are muted); empty hides it. */
  notice: string;
  /** The field's text changed (onDraft), trimmed to maxLength. */
  onDraft?: (e: UiEvent, text: string) => void;
  /** The choosetab event, with the row's key where it comes from a row. */
  onChooseTab?: (e: UiEvent, key?: string) => void;
  /** The send event, with the row's key where it comes from a row. */
  onSend?: (e: UiEvent, key?: string) => void;
}

/**
 * A chat panel: channel tabs with unread badges, the message log (system
 * lines, the player's own messages and others' with sender and time, long
 * text wrapping and long words breaking) and an input row. The game owns
 * the channels, the messages, moderation and the clock; the panel reports
 * choosetab (a channel's key), send
 * (the Send button) and onDraft (the field's text, trimmed to maxLength).
 * The log fills the height it is given and scrolls inside it, newest at
 * the foot.
 */
export function Chat(props: ChatProps) {
  return (
    <View class={`grow shrink basis-0 border-rim bg-linear-to-b from-glass-hi to-glass-lo shadow-panel text-fg p-4 gap-2 rounded-panel border disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={t("general.chat.name")}>
      <Show when={props.channels.length}>
        <View class="w-full pb-0.5" role="tablist" label={t("general.chat.channels")}>
          <Scroll class="w-full">
            <View class="flex-row items-center gap-0.5">
              <For each={props.channels}>
                {(c) => (
                <View class={`flex-row items-center justify-center gap-1 text-fg-muted transition duration-control min-h-11 nine-plate-cut bg-layer-2 py-0 px-3 data-selected:text-fg data-selected:bg-surface-hover data-selected:border-l-thick data-selected:border-hud-accent hover:bg-none hover:bg-surface-hover`} role="tab" selected={c().id === props.channel} label={c().unread > 0 ? t("general.chat.unread", { channel: c().label, count: c().unread }) : c().label} onPress={(e) => props.onChooseTab?.(e, c().id)} data-selected={c().id === props.channel}>
                  <View class="flex-row items-center gap-1">
                    <Text class="whitespace-nowrap max-w-60 text-ellipsis font-bold overflow-hidden shrink">
                      {c().label}
                    </Text>
                    <Show when={c().unread > 0}>
                      <Text class={`font-bold text-caption tabular-nums nine-chip-cut bg-gold text-gold-fg transition duration-control min-w-4 text-center py-0 px-0.5 m-0 shrink ${packClass("Chip", "frame")}`}>
                        {formatNumber(c().unread)}
                      </Text>
                    </Show>
                  </View>
                </View>
                )}
              </For>
            </View>
          </Scroll>
        </View>
      </Show>
      <View class={`grow shrink basis-0 nine-plate-cut bg-glass w-full py-1 px-0.5 gap-0 ${packClass("Field", "frame")}`}>
        <Scroll class={`flex-col-reverse grow shrink basis-0 w-full gap-0`} role="log" label={t("general.chat.messages")} live="polite">
          <Show when={props.messages.length} fallback={
              <Text class="text-center text-fg-muted p-4 m-0" role="status">{t("general.chat.empty")}</Text>
          }>
            <View class="w-full p-2 m-0 gap-1">
              <For each={props.messages}>
                {(m) => (
                <View class={`group/message w-full py-0.5 px-0 gap-0.5 data-kind-own:items-end`} data-kind-own={(m().kind) === 'own'} data-kind-system={(m().kind) === 'system'}>
                  <Show when={m().kind !== 'system'} fallback={
                      <View class="flex-row items-baseline w-full gap-2 group-data-kind-own/message:justify-end">
                        <Icon class="text-gold size-4" name="bolt" />
                        <Text class={`tracking-xs text-info whitespace-nowrap text-ellipsis font-bold m-0 overflow-hidden data-kind-system:text-fg-muted shrink`} data-kind-system={true}>
                          {t("general.chat.system")}
                        </Text>
                        <Text class={`text-caption text-fg-subtle whitespace-nowrap tabular-nums m-0`}>
                          {m().time}
                        </Text>
                      </View>
                  }>
                    <View class="flex-row items-baseline w-full gap-2 group-data-kind-own/message:justify-end">
                      <Text class={`tracking-xs text-info whitespace-nowrap text-ellipsis font-bold m-0 overflow-hidden data-kind-own:text-gold data-kind-system:text-fg-muted shrink`} data-kind-own={(m().kind) === 'own'} data-kind-system={(m().kind) === 'system'}>
                        {m().kind === 'own' ? t("general.chat.you") : m().sender}
                      </Text>
                      <Text class={`text-caption text-fg-subtle whitespace-nowrap tabular-nums m-0`}>
                        {m().time}
                      </Text>
                    </View>
                  </Show>
                  <Text class={`text-start text-fg m-0 group-data-kind-own/message:text-end group-data-kind-own/message:bg-none group-data-kind-own/message:bg-surface-hover group-data-kind-system/message:text-fg-muted group-data-kind-system/message:max-w-full group-data-kind-system/message:p-0 data-bubble:max-w-22/25 data-bubble:bg-glass data-bubble:bg-layer-2 data-bubble:nine-plate-cut data-bubble:py-1 data-bubble:px-2`} data-bubble={m().kind !== 'system'} data-kind-own={(m().kind) === 'own'} data-kind-system={(m().kind) === 'system'}>
                    {m().text}
                  </Text>
                </View>
                )}
              </For>
            </View>
          </Show>
        </Scroll>
      </View>
      <Show when={props.notice}>
        <Text class={`tracking-xs text-warn m-0`} role="status">
          {props.notice}
        </Text>
      </Show>
      <View class={`flex-row w-full gap-1`} onAction={{ submit: (e) => props.onSend?.(e) }}>
        <Input class="grow shrink basis-0 min-h-11" label={t("general.chat.placeholder")} placeholder={t("general.chat.placeholder")} disabled={!props.canSend} value={props.draft} onChange={(e) => props.onDraft?.(e, e.value.slice(0, props.maxLength))} />
        <Button variant="primary" class={`min-h-11 min-w-11`} label={t("general.chat.send")} disabled={!props.canSend} onPress={(e) => props.onSend?.(e)}>
          <View class="flex-row items-center gap-1">
            <Icon class="size-4 shrink" name="play" />
            <Text class="compact:hidden shrink">{t("general.chat.send")}</Text>
          </View>
        </Button>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<ChatProps>> = {
  default: {"channels":[{"id":"world","label":"World","unread":0},{"id":"guild","label":"Guild","unread":3},{"id":"party","label":"Party","unread":0}],"channel":"world","messages":[{"id":"m1","kind":"system","sender":"","time":"20:14","text":"Welcome to the World channel. Be kind."},{"id":"m2","kind":"player","sender":"Aerin","time":"20:15","text":"Anyone up for the raid tonight?"},{"id":"m3","kind":"own","sender":"Brenna","time":"20:16","text":"Count me in, I will bring potions."},{"id":"m4","kind":"player","sender":"Lys","time":"20:16","text":"Same here!"}],"draft":"","maxLength":200,"canSend":true,"notice":""},
  empty: {"channels":[{"id":"world","label":"World","unread":0}],"channel":"world","messages":[],"draft":"","maxLength":200,"canSend":true,"notice":""},
  disabled: {"channels":[{"id":"world","label":"World","unread":0}],"channel":"world","messages":[{"id":"m1","kind":"system","sender":"","time":"20:20","text":"You are muted for 5 minutes."}],"draft":"","maxLength":200,"canSend":false,"notice":"You are muted for 5 minutes."},
  overflow: {"channels":[{"id":"world","label":"World","unread":0},{"id":"guild","label":"The Order of the Silver Dawn","unread":128},{"id":"party","label":"Party","unread":9},{"id":"trade","label":"Trade","unread":0},{"id":"whisper","label":"Whispers","unread":1}],"channel":"guild","messages":[{"id":"m1","kind":"system","sender":"","time":"19:58","text":"Guild message of the day: the raid starts at nine, bring your best gear and enough food for three hours."},{"id":"m2","kind":"player","sender":"Maximilian the Unbreakable","time":"20:01","text":"Supercalifragilisticexpialidocious-wordthatdoesnotbreaknaturallyanywhereatallsoitmustbreak"},{"id":"m3","kind":"own","sender":"Brenna","time":"20:02","text":"A rather long message from me that wraps across several lines because I had a lot to say about the raid plan and the loot rules."},{"id":"m4","kind":"player","sender":"Lys","time":"20:03","text":"ok"},{"id":"m5","kind":"player","sender":"Aerin","time":"20:04","text":"See you there."},{"id":"m6","kind":"player","sender":"Idris","time":"20:05","text":"Who has the key?"},{"id":"m7","kind":"own","sender":"Brenna","time":"20:05","text":"I do."},{"id":"m8","kind":"player","sender":"Maren","time":"20:06","text":"Great, then we are set."},{"id":"m9","kind":"system","sender":"","time":"20:07","text":"Sela joined the guild."},{"id":"m10","kind":"player","sender":"Sela","time":"20:07","text":"Hello everyone!"}],"draft":"A draft that is already fairly long and keeps going","maxLength":200,"canSend":true,"notice":"Slow mode: one message every 3 s"},
};
