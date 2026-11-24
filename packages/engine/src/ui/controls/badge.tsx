import { Show, Text, View, formatNumber, packClass } from "@origingame/engine/ui";

export type BadgeTone = "neutral" | "accent" | "success" | "warn" | "danger" | "info" | "gold";

export interface BadgeProps {
  /** Its colour: `neutral` (the default), `accent`, `success`, `warn`, `danger`, `info` or `gold`. */
  tone?: BadgeTone | undefined;
  /** A count to show (unread mail, new items); 0 hides the badge unless `showZero`. */
  count?: number | undefined;
  /** Counts above this read `max+` (99+). Default 99. */
  max?: number | undefined;
  /** Show a count of 0 rather than hiding. */
  showZero?: boolean | undefined;
  /** Only a dot, no words (something new here). */
  dot?: boolean | undefined;
  /** What a screen reader says (3 unread); the visible text when absent. */
  label?: string | undefined;
  /** Classes after its own (a placement on a corner). */
  class?: string | undefined;
  /** Words instead of a count (New, -20%). */
  children?: any;
}

/**
 * A small status tag: words, a count capped at `max+`, or a lone dot, in
 * one of seven tones. The pack dresses it through `Badge.frame` and
 * `Badge.frame.<tone>` (`Badge.dot` for the dot). Read only.
 */
export function Badge(props: BadgeProps) {
  const tone = () => (props.tone && props.tone !== "neutral" ? props.tone : undefined);
  const max = () => props.max ?? 99;
  const hasCount = () => props.count !== undefined;
  const hidden = () => hasCount() && (props.count ?? 0) <= 0 && !props.showZero && !props.children;
  const text = () => ((props.count ?? 0) > max() ? `${formatNumber(max())}+` : formatNumber(props.count ?? 0));
  return (
    <Show when={!hidden()}>
      <Show
        when={!props.dot}
        fallback={<View class={`${packClass("Badge", "dot", tone())} ${props.class ?? ""}`} role="status" label={props.label} />}
      >
        <View class={`flex-row items-center justify-center ${packClass("Badge", "frame", tone())} ${props.class ?? ""}`} role="status" label={props.label}>
          <Show when={hasCount()} fallback={props.children}>
            <Text class="tabular-nums">{text()}</Text>
          </Show>
        </View>
      </Show>
    </Show>
  );
}

export const ground = "panel";

export const fixtures: Record<string, Partial<BadgeProps>> = {
  words: { children: () => <Text>New</Text> },
  accent: { tone: "accent", children: () => <Text>Beta</Text> },
  count: { tone: "danger", count: 3, label: "3 unread" },
  capped: { tone: "danger", count: 240 },
  gold: { tone: "gold", children: () => <Text>-20%</Text> },
  dot: { tone: "accent", dot: true, label: "New" },
};
