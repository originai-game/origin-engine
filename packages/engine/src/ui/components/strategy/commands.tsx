import type { Texture } from "@origingame/engine";
import { For, Icon, Image, packClass, Show, t, Text, View, type UiEvent } from "@origingame/engine/ui";
import "./strategy.en.ogt";

export interface StrategyCommandsProps {
  /** The command card in grid order, four to a row, keyed rows of { id, name, icon, glyph, key, action, enabled, affordable, cost, costIcon, cooldown, time, active, detail, cooldownText, readyText, reason }: name for a reader and the info card's title, empty draws an empty cell; icon the command's picture (og.load.texture), 0 draws glyph (an icon of the atlas); key the hotkey as the game names it (A), or action the input action whose bound glyph the cap shows (og.input.defineAction), both empty hide it, hidden under a touch screen; enabled false greys it and it sends nothing; affordable false greys it and shows cost on a red tag; cost and costIcon the price as the game words it (200) and the resource's picture; cooldown 0 to 1 still to run, a dark sweep over the art, with time its figure (18); active a toggle that is on or the order in hand, its edge lit; detail, cooldownText and readyText the info card's words (Cooldown 45 s, Ready in 18 s); reason why it cannot be used now (Need a Blacksmith), on the info card. */
  commands?: ReadonlyArray<{ id: string; name: string; icon: Texture | 0; glyph: string; key: string; action: string; enabled: boolean; affordable: boolean; cost: string; costIcon: Texture | 0; cooldown: number; time: string; active: boolean; detail: string; cooldownText: string; readyText: string; reason: string }>;
  /** The command whose info card stands over the grid (the one under the pointer, as the game decides); empty shows none. A phone shows none. */
  focused: string;
  /** Command. */
  onCommand?: (e: UiEvent, id: string) => void;
}

/**
 * The command card at the bottom right: abilities, orders and buildings
 * as square tiles four to a row, each its picture with a lit edge, the
 * hotkey in its corner, a cooldown as a dark sweep with the seconds left,
 * a command that cannot be paid for greyed with its price on a red tag, a
 * disabled one greyed, the order in hand lit; the info card of the
 * command in focus over the grid (its name and key, what it does, its
 * price, cooldown and when it is ready, or why it cannot be used).
 * Reports command (the tile's key; a disabled or unaffordable tile sends
 * nothing); the game owns every cost, cooldown and rule.
 */
export function StrategyCommands(props: StrategyCommandsProps) {
  return (
    <View class={`w-fit max-w-full min-w-0 flex-col items-stretch text-fg gap-y-2`}>
      <For each={props.commands ?? []} key={(c) => c.id}>
        {(c) => (
        <Show when={c().id === props.focused && c().name}>
          <View class={`flex-col gap-y-1 w-59.25 max-w-full min-w-0 py-2 px-3 nine-panel-cut bg-plate-hi compact:hidden ${packClass("Panel", "frame")}`} role="tooltip">
            <View class="flex-row items-center justify-between gap-x-2 min-w-0">
              <Text class={`shrink min-w-0 text-reading font-bold whitespace-nowrap overflow-hidden`}>
                {c().name}
              </Text>
              <Show when={c().key}>
                <Text class={`shrink min-w-5 max-w-2/5 py-0 px-1.25 leading-normal text-xs font-bold text-center text-hud-accent-fg bg-linear-to-b from-key-hi to-key-lo rounded-control shadow-key whitespace-nowrap overflow-hidden pointer-coarse:hidden`}>
                  {c().key}
                </Text>
              </Show>
            </View>
            <Show when={c().detail}>
              <Text class={`text-caption leading-snug text-fg-muted whitespace-normal`}>
                {c().detail}
              </Text>
            </Show>
            <Show when={c().cost || c().cooldownText || c().readyText}>
              <View class="flex-row flex-wrap items-center gap-x-2 gap-y-1 min-w-0">
                <Show when={c().cost}>
                  <View class={`shrink flex-row items-center gap-x-1 data-short:text-danger`} data-short={!c().affordable}>
                    <Show when={c().costIcon}>
                      <Image class={`shrink size-4`} src={c().costIcon} />
                    </Show>
                    <Text class={`text-base font-bold tabular-nums whitespace-nowrap data-short:text-danger`} data-short={!c().affordable}>
                      {c().cost}
                    </Text>
                  </View>
                </Show>
                <Show when={c().cooldownText}>
                  <Text class={`shrink min-w-0 py-0 px-1.5 leading-normal text-xs font-bold whitespace-nowrap overflow-hidden nine-chip-cut bg-well ${packClass("Chip", "frame")}`}>
                    {c().cooldownText}
                  </Text>
                </Show>
                <Show when={c().readyText}>
                  <Text class={`shrink min-w-0 py-0 px-1.5 leading-normal text-xs font-bold whitespace-nowrap overflow-hidden text-gold nine-chip-cut bg-well ${packClass("Chip", "frame")}`}>
                    {c().readyText}
                  </Text>
                </Show>
              </View>
            </Show>
            <Show when={c().reason}>
              <View class={`text-caption leading-snug font-semibold text-danger whitespace-normal`} role="status">
                <Text>{c().reason}</Text>
              </View>
            </Show>
          </View>
        </Show>
        )}
      </For>
      <View class={`group/grid grid grid-cols-4 auto-rows-[3.657rem] gap-1 justify-start self-end p-2.5 nine-panel-cut bg-plate-hi compact:grid-cols-8 compact:justify-between compact:self-stretch compact:p-1.25 ${packClass("Panel", "frame")}`} role="toolbar" label={t("strategy.commands.name")}>
        <For each={props.commands ?? []} key={(c) => c.id}>
          {(c) => (
          <Show when={c().name} fallback={
              <View class={`nine-plate-cut bg-sheen`} />
          }>
            <View class={`flex-row items-center justify-center gap-1 group/cmd flex p-0.5 nine-plate-cut bg-plate-hi data-active:bg-hud-accent disabled:opacity-45 text-fg ${packClass("Button", "plate")}`} role="button" label={c().name} description={c().reason ? c().reason : c().readyText ? c().readyText : c().detail} pressed={c().active} disabled={!c().enabled || !c().affordable} data-cooling={c().cooldown > 0} data-active={c().active} onPress={(e) => props.onCommand?.(e, c().id)}>
              <View class="relative size-full overflow-hidden">
                <Show when={c().icon} fallback={
                    <View class="absolute inset-0 flex-row items-center justify-center bg-linear-to-b from-plate-hi to-plate-lo">
                      <Icon class={`size-11/20`} name={c().glyph} />
                    </View>
                }>
                  <Image class={`absolute inset-0 size-full group-disabled/cmd:opacity-45`} src={c().icon} />
                </Show>
                <Show when={c().cooldown > 0}>
                  <View
                    class="absolute inset-0 nine-tile bg-shade-deep"
                    wipe={Math.min(1, c().cooldown)}
                    wipeStart={(1 - Math.min(1, c().cooldown)) * 2 * Math.PI}
                  />
                </Show>
                <Show when={c().cooldown > 0 && c().time}>
                  <View class="absolute inset-x-0 top-1/2 -translate-y-1/2 flex-row justify-center">
                    <Text class={`text-title leading-flat font-bold tabular-nums whitespace-nowrap text-stroke-3 text-stroke-bg`}>
                      {c().time}
                    </Text>
                  </View>
                </Show>
                <Show when={c().action || c().key}>
                  <Text class={`absolute left-1 top-0.5 max-w-47/50 py-0 px-0 leading-tight text-xs font-bold whitespace-nowrap overflow-hidden text-stroke-3 text-stroke-bg pointer-coarse:hidden`} glyph={c().key ? undefined : c().action}>
                    {c().key}
                  </Text>
                </Show>
                <Show when={!c().affordable && c().cost}>
                  <Text class={`absolute right-0.75 bottom-0.75 max-w-97/100 py-0 px-1 leading-snug text-xs font-bold tabular-nums whitespace-nowrap overflow-hidden text-danger nine-chip-cut bg-well ${packClass("Chip", "frame")}`}>
                    {c().cost}
                  </Text>
                </Show>
              </View>
            </View>
          </Show>
          )}
        </For>
      </View>
    </View>
  );
}

export const fixtures: Record<string, Partial<StrategyCommandsProps>> = {
  default: {"focused":"rally","commands":[{"id":"attack","name":"Attack move","icon":0,"glyph":"sword","key":"A","action":"","enabled":true,"affordable":true,"cost":"","costIcon":0,"cooldown":0,"time":"","active":true,"detail":"Move, attacking enemies on the way.","cooldownText":"","readyText":"","reason":""},{"id":"hold","name":"Hold ground","icon":0,"glyph":"shield","key":"H","action":"","enabled":true,"affordable":true,"cost":"","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"Stand and fight where you are.","cooldownText":"","readyText":"","reason":""},{"id":"rally","name":"Rally banner","icon":0,"glyph":"flag","key":"R","action":"","enabled":true,"affordable":true,"cost":"250","costIcon":0,"cooldown":0.6,"time":"18","active":false,"detail":"Nearby troops move 15% faster and hold formation for 20 s.","cooldownText":"Cooldown 45 s","readyText":"Ready in 18 s","reason":""},{"id":"upgrade","name":"Forge upgrade","icon":0,"glyph":"gear","key":"U","action":"","enabled":true,"affordable":true,"cost":"","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"Better weapons for every Man-at-Arms.","cooldownText":"","readyText":"","reason":""},{"id":"barracks","name":"Barracks","icon":0,"glyph":"home","key":"B","action":"","enabled":true,"affordable":true,"cost":"150","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"Trains infantry.","cooldownText":"","readyText":"","reason":""},{"id":"tower","name":"Watchtower","icon":0,"glyph":"flag","key":"T","action":"","enabled":true,"affordable":false,"cost":"200","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"Shoots at enemies in range.","cooldownText":"","readyText":"","reason":"Need 200 stone"},{"id":"archer","name":"Archer","icon":0,"glyph":"bolt","key":"E","action":"","enabled":true,"affordable":true,"cost":"60","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"A ranged soldier.","cooldownText":"","readyText":"","reason":""},{"id":"blank","name":"","icon":0,"glyph":"","key":"","action":"","enabled":false,"affordable":true,"cost":"","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"","cooldownText":"","readyText":"","reason":""}]},
  unaffordable: {"focused":"castle","commands":[{"id":"castle","name":"Castle","icon":0,"glyph":"crown","key":"C","action":"","enabled":true,"affordable":false,"cost":"650","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"A stronghold that trains knights.","cooldownText":"","readyText":"","reason":"Need 650 stone"},{"id":"wall","name":"Stone wall","icon":0,"glyph":"shield","key":"W","action":"","enabled":true,"affordable":false,"cost":"40","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"","cooldownText":"","readyText":"","reason":"Need 40 stone"},{"id":"house","name":"House","icon":0,"glyph":"home","key":"Q","action":"","enabled":true,"affordable":true,"cost":"50","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"","cooldownText":"","readyText":"","reason":""},{"id":"age","name":"Next age","icon":0,"glyph":"star","key":"","action":"","enabled":false,"affordable":true,"cost":"800","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"","cooldownText":"","readyText":"","reason":"Needs two Castle Age buildings"}]},
  cooling: {"focused":"charge","commands":[{"id":"charge","name":"Charge","icon":0,"glyph":"bolt","key":"C","action":"","enabled":false,"affordable":true,"cost":"","costIcon":0,"cooldown":0.95,"time":"57","active":false,"detail":"Run at the enemy, knocking them down.","cooldownText":"Cooldown 60 s","readyText":"Ready in 57 s","reason":""},{"id":"heal","name":"Mend","icon":0,"glyph":"potion","key":"M","action":"","enabled":true,"affordable":true,"cost":"","costIcon":0,"cooldown":0.25,"time":"4","active":false,"detail":"","cooldownText":"","readyText":"","reason":""},{"id":"stop","name":"Stop","icon":0,"glyph":"pause","key":"S","action":"","enabled":true,"affordable":true,"cost":"","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"","cooldownText":"","readyText":"","reason":""}]},
  overflow: {"focused":"banner","commands":[{"id":"move","name":"Move","icon":0,"glyph":"play","key":"M","action":"","enabled":true,"affordable":true,"cost":"","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"","cooldownText":"","readyText":"","reason":""},{"id":"stop","name":"Stop","icon":0,"glyph":"pause","key":"S","action":"","enabled":true,"affordable":true,"cost":"","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"","cooldownText":"","readyText":"","reason":""},{"id":"hold","name":"Hold position","icon":0,"glyph":"shield","key":"H","action":"","enabled":true,"affordable":true,"cost":"","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"","cooldownText":"","readyText":"","reason":""},{"id":"attack","name":"Attack move","icon":0,"glyph":"sword","key":"Left Shoulder + A","action":"","enabled":true,"affordable":true,"cost":"","costIcon":0,"cooldown":0,"time":"","active":true,"detail":"","cooldownText":"","readyText":"","reason":""},{"id":"patrol","name":"Patrol","icon":0,"glyph":"map","key":"P","action":"","enabled":true,"affordable":true,"cost":"","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"","cooldownText":"","readyText":"","reason":""},{"id":"banner","name":"Raise the Banner of the Old Kingdom over the Northern Expeditionary Host","icon":0,"glyph":"flag","key":"B","action":"","enabled":false,"affordable":false,"cost":"12,800,000","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"Every unit within forty paces of the banner fights as if it were at full health for as long as the banner stands, and routed units return to the field.","cooldownText":"Cooldown 10 min","readyText":"","reason":"Requires the Hall of Ancestors and 400 renown"},{"id":"rally","name":"Rally","icon":0,"glyph":"crown","key":"R","action":"","enabled":false,"affordable":true,"cost":"","costIcon":0,"cooldown":0.95,"time":"1:54","active":false,"detail":"","cooldownText":"","readyText":"","reason":""},{"id":"heal","name":"Mend","icon":0,"glyph":"potion","key":"Z","action":"","enabled":true,"affordable":true,"cost":"","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"","cooldownText":"","readyText":"","reason":""},{"id":"form","name":"Formation","icon":0,"glyph":"gear","key":"F","action":"","enabled":true,"affordable":true,"cost":"","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"","cooldownText":"","readyText":"","reason":""},{"id":"split","name":"Split","icon":0,"glyph":"plus","key":"X","action":"","enabled":true,"affordable":true,"cost":"","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"","cooldownText":"","readyText":"","reason":""},{"id":"garrison","name":"Garrison","icon":0,"glyph":"home","key":"G","action":"","enabled":true,"affordable":true,"cost":"","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"","cooldownText":"","readyText":"","reason":""},{"id":"delete","name":"Delete","icon":0,"glyph":"close","key":"Del","action":"","enabled":true,"affordable":true,"cost":"","costIcon":0,"cooldown":0,"time":"","active":false,"detail":"","cooldownText":"","readyText":"","reason":""}]},
};
