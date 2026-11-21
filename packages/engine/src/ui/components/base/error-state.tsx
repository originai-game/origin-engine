import { Icon, Show, Text, View, t, type UiEvent } from "@origingame/engine/ui";
import { Spinner } from "./spinner";
import "./states.en.ogt";

export interface ErrorStateProps {
  /** Heading; empty uses Something went wrong. */
  title?: string;
  /** The game's explanation and recovery advice. */
  body?: string;
  /** Optional support reference; never a raw exception or credential. */
  code?: string;
  /** Retry wording; empty uses Try again. */
  retryLabel?: string;
  /** Offer the retry action. Default true. */
  canRetry?: boolean;
  /** Retry is in progress; both actions are disabled and a wait is announced. Default false. */
  busy?: boolean;
  /** Optional secondary action; empty hides it. */
  dismissLabel?: string;
  /** The dismiss event, with the row's key where it comes from a row. */
  onDismiss?: (e: UiEvent, key?: string) => void;
  /** The retry event, with the row's key where it comes from a row. */
  onRetry?: (e: UiEvent, key?: string) => void;
}

/**
 * EmptyState's recess with the failure on a disc tinted and ringed in
 * danger (quieter than the action it leads to): what went
 * wrong, the support reference set apart as a code, then retry (the
 * primary action) and the way out.
 */
export function ErrorState(props: ErrorStateProps) {
  return (
    <View class={`bg-tile shadow-sunk text-fg items-center text-center p-6 gap-4 rounded-block disabled:text-fg-subtle transition duration-240 ease-out starting:opacity-0 starting:translate-y-2`} role="alert" busy={(props.busy ?? false)} label={props.title || t("base.error.title")}>
      <View class={`border-danger bg-danger text-bg justify-center items-center gap-4 size-16 rounded-capsule border-[1.5px] animate-pop`}>
        <Icon class={`text-bg size-8`} name="close" />
      </View>
      <View class="items-center max-w-105 gap-1">
        <Text class={`font-semibold text-lg text-fg m-0`}>
          {props.title || t("base.error.title")}
        </Text>
        <Show when={props.body}>
          <Text class={`text-center m-0`}>{props.body}</Text>
        </Show>
        <Show when={props.code}>
          <Text class={`text-caption tabular-nums bg-raised-lo shadow-[inset_0px_0px_0px_1px_#4e586c] text-fg transition duration-317 self-center w-fit mt-1 mb-0 max-w-full py-0 px-2 mx-0 rounded-control`}>
            {props.code}
          </Text>
        </Show>
      </View>
      <Show when={(props.canRetry ?? true) || props.dismissLabel}>
        <View class={`flex-row flex-wrap items-center justify-center gap-4`}>
          <Show when={(props.canRetry ?? true)}>
            <View class={`flex-row items-center justify-center gap-1 text-accent min-h-11 max-w-full shrink`} role="button" disabled={(props.busy ?? false)} onPress={(e) => props.onRetry?.(e)}>
              <Show when={(props.busy ?? false)}>
                <Spinner size={16} name={t("base.error.busy")} />
              </Show>
              <Text class="whitespace-normal">
                {(props.busy ?? false) ? t("base.error.busy") : props.retryLabel || t("base.error.retry")}
              </Text>
            </View>
          </Show>
          <Show when={props.dismissLabel}>
            <View class={`flex-row items-center justify-center gap-1 min-h-11 max-w-full shrink`} role="button" disabled={(props.busy ?? false)} onPress={(e) => props.onDismiss?.(e)}>
              <Text class="whitespace-normal">{props.dismissLabel}</Text>
            </View>
          </Show>
        </View>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<ErrorStateProps>> = {
  default: {"title":"Could not load your builds","body":"Your saved builds are safe. Check your connection and try again.","code":"NET-204","canRetry":true,"busy":false,"dismissLabel":"Go back"},
  busy: {"body":"Connecting to the server…","busy":true,"dismissLabel":"Go back"},
  terminal: {"title":"This replay is no longer available","body":"It was recorded on an older game version.","canRetry":false,"dismissLabel":"Return to replays"},
  overflow: {"title":"We could not retrieve the shared expedition report","body":"The host may have left the session. Your completed objectives are saved, and you can return to the lobby without losing rewards.","code":"EXPEDITION-REPORT-UNAVAILABLE","dismissLabel":"Return to the expedition lobby"},
};

/** The input actions it answers, with the label a hint shows for each. */
export const actions = {"retry":"Retry the failed operation","dismiss":"Leave the error state"} as const;
