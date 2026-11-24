import { Text, formatCompact, formatNumber, packClass, untrack, useClock } from "@origingame/engine/ui";

export interface CounterProps {
  /** The number it shows. */
  value: number;
  /** How it writes the number: grouped (1,234, the default), compact (1.2K) or percent (0 to 1). */
  format?: "grouped" | "compact" | "percent" | undefined;
  /** Digits after the point. */
  decimals?: number | undefined;
  /** Where its first count starts (a score counting up as its screen opens); rolls on the next tick. */
  from?: number | string | undefined;
  /** How long a count takes, in ms or CSS time; overrides Count.roll's duration. */
  duration?: number | string | undefined;
  class?: string | undefined;
}

/**
 * A number that rolls to its value as it changes (`Count.roll`). When `from`
 * is given, it draws `from` on its first tick and rolls to `value` on the
 * next, deterministically on the world clock.
 */
export function Counter(props: CounterProps) {
  const clock = useClock();
  const startTick = untrack(clock).tick;
  let rolled = false;
  const shown = () => {
    if (!rolled && props.from !== undefined && props.from !== "") {
      if (clock().tick === startTick) {
        const f = Number(props.from);
        if (Number.isFinite(f)) return f;
      } else {
        rolled = true;
      }
    }
    return props.value;
  };
  const text = () => {
    const v = shown();
    if (props.format === "percent") return `${formatNumber(v * 100, { decimals: props.decimals ?? 0 })}%`;
    if (props.format === "compact") return formatCompact(v);
    return formatNumber(v, { decimals: props.decimals ?? 0 });
  };
  const durationStyle = () => {
    if (props.duration === undefined || props.duration === "") return undefined;
    const ms = typeof props.duration === "number" ? `${props.duration}ms` : props.duration;
    return { transitionDuration: ms };
  };
  return (
    <Text
      class={`${packClass("Count", "roll")} ${packClass("Counter", "value")} ${props.class ?? ""}`}
      style={durationStyle()}
    >
      {text()}
    </Text>
  );
}
