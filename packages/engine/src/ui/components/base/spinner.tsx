import { View, t } from "@origingame/engine/ui";
import "./spinner.en.ogt";

export interface SpinnerProps {
  /** Its width and height, CSS px. Default 28. */
  size?: number;
  /** What a screen reader calls it; empty is Loading. */
  name: string;
}

/**
 * A ring that is always there and an arc that runs round it. `size` is
 * its width; a host that holds nothing else while it waits (a portrait,
 * a slot) sets `--spinner-size` on the box round it instead, so the ring
 * is in proportion to it.
 */
export function Spinner(props: SpinnerProps) {
  return (
    <View role="progressbar" busy={true} label={props.name || t("component.spinner.name")} style={{ width: String((props.size ?? 28) + "px"), height: String((props.size ?? 28) + "px") }}>
      <View class="size-full rounded-full border-4 border-rim border-t-accent animate-spin" />
    </View>
  );
}

export const fixtures: Record<string, Partial<SpinnerProps>> = {
  default: {"size":28,"name":""},
};
