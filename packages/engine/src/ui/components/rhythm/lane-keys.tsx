import { For, Text, View, t } from "@origingame/engine/ui";
import "./rhythm.en.ogt";

export interface RhythmLaneKeysProps {
  /** The lanes left to right, keyed rows of { id, key, held }: key the input that plays the lane as the game names it (S, LB); held the player holds it now (the cap lights). */
  lanes: ReadonlyArray<{ id: string; key: string; held: boolean }>;
}

/**
 * The lanes' key caps under the judgement line, spread across the field's
 * width, one per lane, each lit while the player holds it. The game reads
 * input and names the keys; read only.
 */
export function RhythmLaneKeys(props: RhythmLaneKeysProps) {
  return (
    <View class={`flex-row items-center gap-1 w-full flex-1`} role="list" label={t("rhythm.lanes.name")}>
      <For each={props.lanes ?? []} key={(l) => l.id}>
        {(l) => (
          <View class={`flex-1 min-w-0 flex-row items-center justify-center`} role="listitem">
            <View
              role="text"
              label={l().held ? t("rhythm.lanes.held", { key: l().key }) : l().key}
              class={`min-w-[3.143rem] compact:min-w-[2.143rem] min-h-[2.429rem] compact:min-h-[2rem] px-1 flex-row items-center justify-center bg-linear-to-b from-white to-[#c9d1e2] rounded-[4.576px] shadow-[0px_2.288px_0px_#0000008c,inset_0px_-2.288px_0px_#0000002e] data-held:bg-linear-to-b data-held:from-[#dffbff] data-held:to-[#38e8ff] data-held:shadow-[0px_0px_14px_#38e8ff,inset_0px_-2.288px_0px_#0000002e]`}
              data-held={l().held}
            >
              <Text class={`text-[1.143rem] compact:text-[0.929rem] font-bold text-[#0b0f18] text-center`}>{l().key}</Text>
            </View>
          </View>
        )}
      </For>
    </View>
  );
}

export const fixtures: Record<string, Partial<RhythmLaneKeysProps>> = {
  default: {"lanes":[{"id":"l1","key":"S","held":true},{"id":"l2","key":"D","held":false},{"id":"l3","key":"F","held":false},{"id":"l4","key":"J","held":true},{"id":"l5","key":"K","held":false},{"id":"l6","key":"L","held":false}]},
  four: {"lanes":[{"id":"a","key":"D","held":false},{"id":"b","key":"F","held":false},{"id":"c","key":"J","held":false},{"id":"d","key":"K","held":false}]},
  overflow: {"lanes":[{"id":"p1","key":"Left Shoulder","held":true},{"id":"p2","key":"Left Trigger","held":false},{"id":"p3","key":"Dpad Up","held":false},{"id":"p4","key":"Y","held":false},{"id":"p5","key":"Right Trigger","held":false},{"id":"p6","key":"Right Shoulder","held":true},{"id":"p7","key":"Space","held":false},{"id":"p8","key":"Enter","held":false}]},
};
