import { Text, View } from "@origingame/engine/ui";

export interface RibbonProps {
  /** The ribbon's words (NEW, SALE). */
  text: string;
  /** Where it sits: topRight (empty), topLeft, bottomRight, bottomLeft or center (a stamp). */
  corner: string;
  /** Its tone: primary (empty), success, warn, danger or info. */
  tone: string;
}

/**
 * A strip across a corner of whatever it lies over (or a stamp across
 * its middle). Its box only holds and cuts the strip: it draws nothing
 * of its own, so the card under it shows. A game puts the box in the
 * corner it names of a positioned card that clips (`overflow: hidden`),
 * and the card's own corner cuts the strip's ends.
 */
export function Ribbon(props: RibbonProps) {
  return (
    <View class={`group/corner text-fg w-40 h-28 pointer-events-none p-0 overflow-hidden disabled:text-fg-subtle`} data-top-left={props.corner === 'topLeft'} data-bottom-right={props.corner === 'bottomRight'} data-bottom-left={props.corner === 'bottomLeft'} data-center={props.corner === 'center'}>
      <View class={`bg-info text-info-fg font-bold text-caption absolute top-4 left-12 w-40 items-center rotate-45 py-0.5 px-0 gap-0 data-tone-primary:text-accent group-data-bottom-right/corner:top-16 group-data-bottom-right/corner:-rotate-45 group-data-bottom-left/corner:top-16 group-data-bottom-left/corner:-left-12 group-data-bottom-left/corner:rotate-45 group-data-center/corner:top-10 group-data-center/corner:left-4 group-data-center/corner:w-32 group-data-center/corner:-rotate-8 group-data-top-left/corner:-left-12 group-data-top-left/corner:-rotate-45 data-play-stamp:animate-stamp transition duration-240 ease-out starting:opacity-0 starting:translate-y-2`} role="status" label={props.text} data-play-stamp={(props.corner === 'center' ? 'stamp' : 'enter') === "stamp"} data-play-enter={(props.corner === 'center' ? 'stamp' : 'enter') === "enter"} data-tone-primary={(props.tone || 'primary') === "primary"}>
        <Text class={`max-w-16 whitespace-nowrap text-ellipsis`}>{props.text}</Text>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<RibbonProps>> = {
  default: {"text":"New","corner":"","tone":"danger"},
  topLeft: {"text":"Sale","corner":"topLeft","tone":"success"},
  bottomRight: {"text":"−50%","corner":"bottomRight","tone":"warn"},
  bottomLeft: {"text":"Rank 1","corner":"bottomLeft","tone":"info"},
  stamp: {"text":"Sold out","corner":"center","tone":"primary"},
  overflow: {"text":"Limited edition of the Ninefold Gate","corner":"","tone":"danger"},
};
