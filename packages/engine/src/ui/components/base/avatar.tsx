import type { Texture } from "@origingame/engine";
import { Icon, Image, Show, Text, View, formatNumber, t } from "@origingame/engine/ui";
import { Spinner } from "./spinner";
import "./avatar.en.ogt";

export interface AvatarProps {
  /** The displayed and accessible name; empty uses the localized unnamed-player label. */
  name?: string;
  /** Shows the name caption; false keeps the accessible name for composition beside a heading. Default true. */
  showName?: boolean;
  /** A loaded portrait texture; without one the fallback icon shows. */
  portrait?: Texture | 0 | undefined;
  /** Fallback atlas icon when there is no portrait. Default "person". */
  icon?: string;
  /** Level badge; 0 hides it. Default 0. */
  level?: number;
  /** online, away, busy or offline; empty hides the presence row. It is shown in words as well as colour. */
  status?: string;
  /** The game's wording of presence; empty uses the standard status label. */
  statusText?: string;
  /** A frame key a pack selects through data-frame; the component imposes no decorative frame. */
  frame?: string;
  /** Portrait data is loading; shows a loading label and keeps the name and layout. Default false. */
  loading?: boolean;
}

/**
 * A portrait in a round well: presence is a dot on its rim (and words
 * under the name, so it never rests on colour), the level a badge across
 * its foot.
 */
export function Avatar(props: AvatarProps) {
  return (
    <View class={`text-fg items-center w-24 p-0 m-0 gap-1 disabled:text-fg-subtle`} data-frame={props.frame} data-status={props.status} label={props.name || t("base.avatar.unnamed")} busy={(props.loading ?? false)}>
      <View class={`bg-tile shadow-sunk w-full aspect-square mb-1 gap-2 rounded-capsule`}>
        <View class={`grow shrink basis-0 border-rim justify-center items-center gap-0 size-full rounded-capsule border-[1.5px] overflow-hidden data-muted:opacity-45`}>
          <Show when={(props.loading ?? false)} fallback={
              <Show when={(props.portrait ?? 0)} fallback={
                  <Icon class={`text-fg-muted size-[56%]`} name={(props.icon ?? "person")} />
              }>
                <Image class={`object-cover object-top size-full`} src={props.portrait} />
              </Show>
          }>
            <View class={`grow shrink basis-0 justify-center items-center gap-2 size-full`}>
              <Spinner name={t("base.avatar.loading")} />
            </View>
          </Show>
        </View>
        <Show when={props.frame}>
          <View class={`absolute border-accent-ink pointer-events-none inset-0 rounded-capsule border-3 data-muted:opacity-45`} data-frame={props.frame} />
        </Show>
        <Show when={props.status && !(props.loading ?? false)}>
          <View class={`absolute top-[2%] right-[2%] border-tile bg-accent text-fg size-4 rounded-capsule border-2 data-tone-success:bg-none data-tone-success:bg-success data-tone-success:text-bg data-tone-danger:bg-none data-tone-danger:bg-danger data-tone-danger:text-bg data-tone-warn:bg-none data-tone-warn:bg-warn data-tone-warn:text-bg`} data-tone-success={(props.status === 'online' ? 'success' : props.status === 'away' ? 'warn' : props.status === 'busy' ? 'danger' : null) === "success"} data-tone-warn={(props.status === 'online' ? 'success' : props.status === 'away' ? 'warn' : props.status === 'busy' ? 'danger' : null) === "warn"} data-tone-danger={(props.status === 'online' ? 'success' : props.status === 'away' ? 'warn' : props.status === 'busy' ? 'danger' : null) === "danger"} />
        </Show>
        <Show when={(props.level ?? 0) > 0}>
          <View class={`absolute bottom-0 text-caption bg-raised-lo shadow-[inset_0px_0px_0px_1px_#4e586c] text-fg transition duration-317 justify-center items-end py-0 px-2 inset-x-0 rounded-capsule`} label={t("base.avatar.level", { level: (props.level ?? 0) })}>
            <Text class="tabular-nums">{formatNumber((props.level ?? 0))}</Text>
          </View>
        </Show>
      </View>
      <View class="items-center w-full gap-0">
        <Show when={(props.showName ?? true)}>
          <Text class={`font-semibold text-caption text-fg text-center m-0`}>
            {props.name || t("base.avatar.unnamed")}
          </Text>
        </Show>
        <Show when={(props.loading ?? false)} fallback={
            <Show when={props.status || props.statusText}>
              <Text class={`text-caption text-fg-muted text-center m-0 data-tone-success:text-success data-tone-danger:text-danger data-tone-warn:text-warn`} data-tone-success={(props.status === 'online' ? 'success' : props.status === 'away' ? 'warn' : props.status === 'busy' ? 'danger' : null) === "success"} data-tone-warn={(props.status === 'online' ? 'success' : props.status === 'away' ? 'warn' : props.status === 'busy' ? 'danger' : null) === "warn"} data-tone-danger={(props.status === 'online' ? 'success' : props.status === 'away' ? 'warn' : props.status === 'busy' ? 'danger' : null) === "danger"}>
                {props.statusText || (props.status === 'online' ? t("base.avatar.online") : props.status === 'away' ? t("base.avatar.away") : props.status === 'busy' ? t("base.avatar.busy") : t("base.avatar.offline"))}
              </Text>
            </Show>
        }>
          <Text class={`text-caption text-fg-muted text-center m-0`} role="status">
            {t("base.avatar.loading")}
          </Text>
        </Show>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<AvatarProps>> = {
  default: {"name":"Brenna Vale","portrait":0,"icon":"person","level":12,"status":"online","statusText":"","frame":"","loading":false},
  away: {"name":"Idris","level":8,"status":"away"},
  busy: {"name":"Sela","status":"busy","frame":"veteran"},
  offline: {"name":"Maren","status":"offline"},
  empty: {},
  loading: {"name":"Loading companion","loading":true},
  captionless: {"name":"Brenna Vale","showName":false,"level":12,"status":"online"},
};
