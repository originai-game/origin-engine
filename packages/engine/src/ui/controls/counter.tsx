import { Text, formatCompact, formatNumber } from "@origingame/engine/ui";

export interface CounterProps {
  /** The number it shows. */
  value: number;
  /** How it writes the number: grouped (1,234, the default), compact (1.2K) or percent (0 to 1). */
  format?: "grouped" | "compact" | "percent" | undefined;
  /** Digits after the point. */
  decimals?: number | undefined;
  /** Where its first count starts (a score counting up as its screen opens); the count is the motion head's. */
  from?: number | string | undefined;
  /** How long a count takes, in ms; the motion head's. */
  duration?: number | string | undefined;
  class?: string | undefined;
}

/**
 * A number that counts to its value as it changes. The count itself is the
 * motion head's (a value-change transition on the text); until it lands the
 * number shows its value at once.
 */
export function Counter(props: CounterProps) {
  const text = () => {
    const v = props.value;
    if (props.format === "percent") return `${formatNumber(v * 100, { decimals: props.decimals ?? 0 })}%`;
    if (props.format === "compact") return formatCompact(v);
    return formatNumber(v, { decimals: props.decimals ?? 0 });
  };
  return <Text class={`tabular-nums ${props.class ?? ""}`}>{text()}</Text>;
}
