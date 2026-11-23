import {For, Icon, Scroll, Show, Text, packClass, type UiEvent, View, t} from "@origingame/engine/ui";
import { Button } from "../../controls/button";
import { ProgressBar } from "../hud/progress-bar";
import "./general.en.ogt";

export interface AchievementsProps {
  /** The list's heading; empty takes the group's (Achievements). */
  title: string;
  /** The overall count as the game words it (34 / 120); empty hides it with its bar. */
  summary: string;
  /** The overall share unlocked, 0 to 1, for the summary bar. */
  done: number;
  /** The achievements in the game's order: keyed rows of { id, name, about, icon, state, progress, progressText, date, hidden, rarity, tier, claimable, claimed }: state locked, progress or unlocked; progress 0 to 1 and progressText (3 / 10) for one in progress; date when it was unlocked, as the game formats it, empty says Unlocked alone; hidden a secret one, shown as Hidden achievement until unlocked; rarity the share of players who have it (2.4%), empty hides it; tier common, rare, epic or legendary, shown as a word and as the medal's colour, empty hides the word; claimable a reward waiting; claimed it was taken, which hides Claim. */
  achievements: ReadonlyArray<{ id: string; name: string; about: string; icon: string; state: string; progress: number; progressText: string; date: string; hidden: boolean; rarity: string; tier: string; claimable: boolean; claimed: boolean }>;
  /** The claim event, with the row's key where it comes from a row. */
  onClaim?: (e: UiEvent, key?: string) => void;
}

/**
 * The achievement list: a band with the title, then the overall count as a
 * large figure over a leaning gold bar; then one entry per achievement, a
 * card with two corners cut: its medal (a hexagon rimmed in its tier's
 * colour), its name, description, its state in words (locked, a bar for one
 * in progress, the date for one unlocked) and its tier on a chip; at the end
 * the share of players who have it, large, and a gold Claim while a reward
 * waits (Claimed once taken). A hidden achievement shows as one, hatched,
 * with neither name nor description, until it is unlocked. On a narrow list
 * the rarity takes a line under the words and Claim takes a line of its own. The
 * game owns every number; the list reports claim (an achievement's key). It
 * fills its height and scrolls inside it.
 */
export function Achievements(props: AchievementsProps) {
  return (
    <View class={`grow shrink basis-0 border-rim bg-linear-to-b from-glass-hi to-glass-lo shadow-panel text-fg p-4 gap-2 rounded-panel border disabled:text-fg-subtle ${packClass("Panel", "frame")}`} label={props.title !== '' ? props.title : t("general.achievements.name")}>
      <View class={`flex-row flex-wrap items-center min-h-12 border-b border-rim-soft w-full py-1 px-4 gap-3`}>
        <Text class={`grow shrink basis-0 font-display text-base font-bold leading-tight tracking-tight text-fg m-0`}>
          {props.title !== '' ? props.title : t("general.achievements.name")}
        </Text>
      </View>
      <Show when={props.summary}>
        <View class={`flex-row items-center w-full pt-2 pb-1 px-4 gap-3 compact:py-1 compact:px-3`}>
          <Text class={`text-display-lg font-bold tabular-nums text-fg whitespace-nowrap leading-flat compact:text-display-sm`}>
            {props.summary}
          </Text>
          <ProgressBar value={props.done} valueText={props.summary} />
        </View>
      </Show>
      <Show when={props.achievements.length} fallback={
          <View class="grow shrink basis-0 justify-center items-center w-full gap-2" role="status">
            <Icon class="size-12" name="trophy" />
            <Text class="m-0">{t("general.achievements.empty")}</Text>
          </View>
      }>
        <Scroll class="grow shrink basis-0 gap-2">
          <View class={`w-full pt-0 pb-1 px-0 m-0 gap-1 ${packClass("List", "cascade")}`} label={props.title !== '' ? props.title : t("general.achievements.name")}>
            <For each={props.achievements}>
              {(a) => (
              <View class={`group/row flex-row items-center nine-plate-cut bg-layer-2 text-fg transition duration-317 w-full pr-3 pl-2 py-2 gap-3 compact:items-start compact:gap-y-1 compact:flex-wrap compact:p-2 hover:bg-layer-3 data-ready:bg-layer-3 data-hidden:bg-layer-1 data-selected:text-fg disabled:text-fg-subtle data-done:text-fg-subtle data-down:opacity-45 ${packClass("List", "item")}`} label={a().hidden && a().state !== 'unlocked' ? t("general.achievements.hidden") : a().name} data-ready={a().claimable && !a().claimed} data-hidden={a().hidden && a().state !== 'unlocked'} data-claimed={a().claimed} data-tier-epic={(a().tier) === 'epic'} data-tier-legendary={(a().tier) === 'legendary'} data-tier-rare={(a().tier) === 'rare'} data-state-locked={(a().state) === 'locked'} data-state-unlocked={(a().state) === 'unlocked'}>
                <View class={`w-16 h-17.5 text-fg-muted bg-linear-to-b from-lit via-current via-40% to-entry-lo to-125% gap-2 group-data-claimed/row:opacity-66 compact:w-12 compact:h-13.25 data-tier-rare:text-info data-tier-epic:text-xp data-tier-legendary:text-warn data-state-locked:text-fg-subtle data-state-locked:bg-linear-to-b data-state-locked:from-sheen data-state-locked:to-lit data-state-locked:bg-transparent`} data-tier-epic={(a().tier) === 'epic'} data-tier-legendary={(a().tier) === 'legendary'} data-tier-rare={(a().tier) === 'rare'} data-state-locked={(a().state) === 'locked'} data-state-unlocked={(a().state) === 'unlocked'}>
                  <View class="absolute justify-center items-center inset-0 gap-2">
                    <Icon class="size-6" name={a().hidden && a().state !== 'unlocked' ? 'lock' : a().icon} />
                  </View>
                </View>
                <View class="grow shrink basis-0 gap-0.5 group-data-claimed/row:opacity-66">
                  <Text class={`text-fg text-start font-bold text-lg m-0 group-data-state-locked/row:text-fg-muted`}>
                    {a().hidden && a().state !== 'unlocked' ? t("general.achievements.hidden") : a().name}
                  </Text>
                  <Show when={(a().hidden && a().state !== 'unlocked') || a().about}>
                    <Text class={`text-caption text-fg-muted text-start m-0`}>
                      {a().hidden && a().state !== 'unlocked' ? t("general.achievements.hiddenAbout") : a().about}
                    </Text>
                  </Show>
                  <View class="flex-row flex-wrap items-center w-full gap-1">
                    <Show when={a().state === 'progress'} fallback={
                        <View class={`flex-row items-center tracking-xs text-fg-muted gap-0.5 data-state-unlocked:text-success shrink`} data-state-locked={(a().state) === 'locked'} data-state-unlocked={(a().state) === 'unlocked'}>
                          <Icon class="size-4" name={a().state === 'unlocked' ? 'check' : 'lock'} />
                          <Text class="shrink">
                            {a().state === 'unlocked' ? (a().date ? t("general.achievements.unlockedOn", { date: a().date }) : t("general.achievements.unlocked")) : t("general.achievements.locked")}
                          </Text>
                        </View>
                    }>
                      <View class="w-72 max-w-full shrink">
                        <ProgressBar class="bg-layer-3" value={a().progress} valueText={a().progressText} />
                      </View>
                      <Text class="whitespace-nowrap font-bold tabular-nums text-fg" label={t("general.achievements.progress")}>
                        {a().progressText}
                      </Text>
                    </Show>
                    <Show when={a().tier}>
                      <Text class={`whitespace-nowrap font-bold text-caption tracking-label uppercase text-fg-muted bg-well py-0 px-1 m-0 rounded-capsule data-tier-rare:text-info data-tier-epic:text-xp data-tier-legendary:text-warn`} data-tier-epic={(a().tier) === 'epic'} data-tier-legendary={(a().tier) === 'legendary'} data-tier-rare={(a().tier) === 'rare'}>
                        {a().tier === 'legendary' ? t("general.achievements.tierLegendary") : a().tier === 'epic' ? t("general.achievements.tierEpic") : a().tier === 'rare' ? t("general.achievements.tierRare") : t("general.achievements.tierCommon")}
                      </Text>
                    </Show>
                  </View>
                </View>
                <Show when={a().rarity}>
                  <View class={`group/rarity min-w-24 items-end text-end m-0 group-data-claimed/row:opacity-66 compact:flex-row compact:items-baseline compact:w-full compact:gap-x-1 compact:pl-15`} role="group" label={t("general.achievements.rarity", { percent: a().rarity })} data-tier-epic={(a().tier) === 'epic'} data-tier-legendary={(a().tier) === 'legendary'} data-tier-rare={(a().tier) === 'rare'}>
                    <Text class="whitespace-nowrap text-xl font-bold tabular-nums text-fg group-data-tier-legendary/rarity:text-gold">
                      {a().rarity}
                    </Text>
                    <Text class="whitespace-nowrap text-caption tracking-label uppercase text-fg-subtle">
                      {t("general.achievements.players")}
                    </Text>
                  </View>
                </Show>
                <Show when={a().claimed} fallback={
                    <Show when={a().claimable}>
                      <Button class={`text-accent min-h-11 compact:w-full animate-pop ${packClass("Button", "plate", "primary")}`} onPress={(e) => props.onClaim?.(e, a().id)}>
                        <Text>{t("general.achievements.claim")}</Text>
                      </Button>
                    </Show>
                }>
                  <View class={`flex-row items-center font-bold tracking-label uppercase text-success gap-0.5 compact:w-full compact:justify-end animate-stamp`}>
                    <Icon class="size-4" name="check" />
                    <Text class="whitespace-nowrap shrink">{t("general.achievements.claimed")}</Text>
                  </View>
                </Show>
              </View>
              )}
            </For>
          </View>
        </Scroll>
      </Show>
    </View>
  );
}

export const fixtures: Record<string, Partial<AchievementsProps>> = {
  default: {"title":"","summary":"3 / 5","done":0.6,"achievements":[{"id":"first","name":"First Steps","about":"Finish the tutorial.","icon":"flag","state":"unlocked","progress":1,"progressText":"","date":"12 Nov 2026","hidden":false,"rarity":"92.1%","tier":"common","claimable":true,"claimed":false},{"id":"hunter","name":"Monster Hunter","about":"Defeat 100 monsters.","icon":"sword","state":"progress","progress":0.64,"progressText":"64 / 100","date":"","hidden":false,"rarity":"18.4%","tier":"rare","claimable":false,"claimed":false},{"id":"rich","name":"Tycoon","about":"Hold a million gold at once.","icon":"coin","state":"locked","progress":0,"progressText":"","date":"","hidden":false,"rarity":"1.2%","tier":"legendary","claimable":false,"claimed":false},{"id":"secret","name":"The Truth","about":"Find what lies beneath.","icon":"skull","state":"locked","progress":0,"progressText":"","date":"","hidden":true,"rarity":"0.4%","tier":"epic","claimable":false,"claimed":false},{"id":"pet","name":"Best Friends","about":"Pet the dog.","icon":"heart","state":"unlocked","progress":1,"progressText":"","date":"3 Nov 2026","hidden":false,"rarity":"71.0%","tier":"common","claimable":false,"claimed":true}]},
  empty: {"title":"","summary":"","done":0,"achievements":[]},
  overflow: {"title":"Feats and Accomplishments of the Realm","summary":"1,204 / 9,999","done":0.12,"achievements":[{"id":"a1","name":"An Achievement With an Exceptionally Long Name That Wraps Across Lines","about":"A description that goes on for a good while so it has to wrap onto a second and maybe a third line in narrow layouts.","icon":"trophy","state":"progress","progress":0.33,"progressText":"3,333 / 10,000","date":"","hidden":false,"rarity":"0.01%","tier":"legendary","claimable":false,"claimed":false},{"id":"a2","name":"Collector","about":"Collect every gem.","icon":"gem","state":"unlocked","progress":1,"progressText":"","date":"1 Jan 2026","hidden":false,"rarity":"5.5%","tier":"epic","claimable":true,"claimed":false},{"id":"a3","name":"Secret","about":"","icon":"star","state":"progress","progress":0.5,"progressText":"1 / 2","date":"","hidden":true,"rarity":"","tier":"rare","claimable":false,"claimed":false},{"id":"a4","name":"Sprinter","about":"Run 10 km.","icon":"bolt","state":"progress","progress":0.9,"progressText":"9 / 10 km","date":"","hidden":false,"rarity":"44%","tier":"common","claimable":false,"claimed":false},{"id":"a5","name":"Chef","about":"Cook 50 meals.","icon":"apple","state":"locked","progress":0,"progressText":"","date":"","hidden":false,"rarity":"12%","tier":"common","claimable":false,"claimed":false},{"id":"a6","name":"Angler","about":"Catch 30 fish.","icon":"drop","state":"locked","progress":0,"progressText":"","date":"","hidden":false,"rarity":"9%","tier":"rare","claimable":false,"claimed":false},{"id":"a7","name":"Bard","about":"Play every song.","icon":"music","state":"unlocked","progress":1,"progressText":"","date":"2 Feb 2026","hidden":false,"rarity":"3%","tier":"epic","claimable":false,"claimed":true}]},
};
