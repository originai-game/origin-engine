<meta name="component" content="StrategySelection">
<meta name="strings" content="./strategy.en.ogt">
<meta name="parts" content="StrategySelection.frame StrategySelection.band StrategySelection.title StrategySelection.badge StrategySelection.cell StrategySelection.portrait StrategySelection.meter StrategySelection.detail StrategySelection.name StrategySelection.grid StrategySelection.command StrategySelection.key StrategySelection.reason">
<meta name="prop:title" content="string" data-doc="The panel's heading; empty takes the kit's (Selection).">
<meta name="prop:summary" content="string" data-doc="A line beside the heading as the game words it (14 units, 2 buildings); empty hides it.">
<meta name="prop:units" content="list" data-doc="What is selected, in the game's order, keyed rows of { id, name, kind, line, status, portrait, icon, count, health, healthMax }: kind unit or building (data-kind); line the class line (Light infantry); status the game's word for what it is doing (Idle, Under construction), empty none; portrait a texture (og.load.texture), 0 draws icon from the atlas instead; count units of that type in the group (shown above 1); health over healthMax fills its bar.">
<meta name="prop:focused" content="string" data-doc="The selected row whose detail and commands are shown (the active subgroup); empty is none.">
<meta name="prop:commands" content="list" data-doc="The focused row's command card, in grid order, keyed rows of { id, label, icon, action, hotkey, enabled, reason, cooldown, cooldownText, active }: action the input action whose glyph the key cap shows (og.input.defineAction), hotkey a cap label instead of the glyph, both empty hide the cap; enabled false disables it and reason says why in words (Need 50 gold); cooldown 0 to 1 still to wait shades it, cooldownText the time left as the game words it (4s); active marks a toggle that is on (data-active).">
<meta name="fixture:default" content='{"title":"","summary":"9 units","focused":"spear","units":[{"id":"spear","name":"Spearmen","kind":"unit","line":"Light infantry","status":"Holding","portrait":0,"icon":"sword","count":6,"health":340,"healthMax":420},{"id":"archer","name":"Archers","kind":"unit","line":"Ranged","status":"","portrait":0,"icon":"bolt","count":2,"health":150,"healthMax":160},{"id":"captain","name":"Captain Ilse","kind":"unit","line":"Hero","status":"","portrait":0,"icon":"crown","count":1,"health":96,"healthMax":300}],"commands":[{"id":"move","label":"Move","icon":"play","action":"cmdMove","hotkey":"","enabled":true,"reason":"","cooldown":0,"cooldownText":"","active":false},{"id":"stop","label":"Stop","icon":"pause","action":"cmdStop","hotkey":"","enabled":true,"reason":"","cooldown":0,"cooldownText":"","active":false},{"id":"hold","label":"Hold","icon":"shield","action":"cmdHold","hotkey":"","enabled":true,"reason":"","cooldown":0,"cooldownText":"","active":true},{"id":"attack","label":"Attack","icon":"sword","action":"cmdAttack","hotkey":"","enabled":true,"reason":"","cooldown":0,"cooldownText":"","active":false},{"id":"charge","label":"Charge","icon":"bolt","action":"","hotkey":"C","enabled":false,"reason":"","cooldown":0.45,"cooldownText":"12s","active":false},{"id":"rally","label":"Rally","icon":"flag","action":"","hotkey":"R","enabled":false,"reason":"Needs the Banner","cooldown":0,"cooldownText":"","active":false}]}'>
<meta name="fixture:single" content='{"title":"","summary":"","focused":"keep","units":[{"id":"keep","name":"Stone Keep","kind":"building","line":"Town centre","status":"Training","portrait":0,"icon":"home","count":1,"health":1850,"healthMax":2400}],"commands":[{"id":"villager","label":"Villager","icon":"person","action":"","hotkey":"Q","enabled":true,"reason":"","cooldown":0,"cooldownText":"","active":false},{"id":"rally","label":"Set rally","icon":"flag","action":"","hotkey":"W","enabled":true,"reason":"","cooldown":0,"cooldownText":"","active":false},{"id":"age","label":"Next age","icon":"crown","action":"","hotkey":"E","enabled":false,"reason":"Need 800 gold","cooldown":0,"cooldownText":"","active":false}]}'>
<meta name="fixture:empty" content='{"title":"","summary":"","focused":"","units":[],"commands":[]}'>
<meta name="fixture:disabled" content='{"title":"","summary":"4 units","focused":"miner","units":[{"id":"miner","name":"Miners","kind":"unit","line":"Workers","status":"Carrying ore","portrait":0,"icon":"gem","count":4,"health":110,"healthMax":160}],"commands":[{"id":"build","label":"Build","icon":"home","action":"","hotkey":"B","enabled":false,"reason":"No space here","cooldown":0,"cooldownText":"","active":false},{"id":"repair","label":"Repair","icon":"gear","action":"","hotkey":"R","enabled":false,"reason":"Nothing damaged","cooldown":0,"cooldownText":"","active":false},{"id":"burrow","label":"Burrow","icon":"drop","action":"","hotkey":"D","enabled":false,"reason":"","cooldown":0.8,"cooldownText":"40s","active":false},{"id":"return","label":"Return","icon":"back","action":"","hotkey":"G","enabled":true,"reason":"","cooldown":0,"cooldownText":"","active":false}]}'>
<meta name="fixture:overflow" content='{"title":"Selection of the Northern Expeditionary Host","summary":"48 units, 3 buildings in 14 groups","focused":"u7","units":[{"id":"u1","name":"Spearmen","kind":"unit","line":"Light infantry","status":"","portrait":0,"icon":"sword","count":12,"health":900,"healthMax":960},{"id":"u2","name":"Archers","kind":"unit","line":"Ranged","status":"","portrait":0,"icon":"bolt","count":8,"health":500,"healthMax":640},{"id":"u3","name":"Knights","kind":"unit","line":"Heavy cavalry","status":"","portrait":0,"icon":"shield","count":6,"health":1100,"healthMax":1500},{"id":"u4","name":"Healers","kind":"unit","line":"Support","status":"","portrait":0,"icon":"potion","count":4,"health":220,"healthMax":320},{"id":"u5","name":"Scouts","kind":"unit","line":"Light cavalry","status":"","portrait":0,"icon":"map","count":3,"health":40,"healthMax":300},{"id":"u6","name":"Rams","kind":"unit","line":"Siege","status":"","portrait":0,"icon":"bomb","count":2,"health":600,"healthMax":1400},{"id":"u7","name":"Grand Marshal Aurelia von Hohenstaufen-Wittelsbach the Unyielding","kind":"unit","line":"Hero · Commander of the Northern Expeditionary Host","status":"Channelling the Banner of the Old Kingdom","portrait":0,"icon":"crown","count":1,"health":2,"healthMax":1800},{"id":"u8","name":"Crossbowmen","kind":"unit","line":"Ranged","status":"","portrait":0,"icon":"bolt","count":6,"health":480,"healthMax":480},{"id":"u9","name":"Pikemen","kind":"unit","line":"Infantry","status":"","portrait":0,"icon":"sword","count":5,"health":510,"healthMax":600},{"id":"u10","name":"Workers","kind":"unit","line":"Economy","status":"","portrait":0,"icon":"gear","count":1,"health":40,"healthMax":40},{"id":"b1","name":"Watchtower","kind":"building","line":"Defence","status":"","portrait":0,"icon":"flag","count":2,"health":700,"healthMax":900},{"id":"b2","name":"Barracks","kind":"building","line":"Production","status":"","portrait":0,"icon":"home","count":1,"health":1500,"healthMax":1500},{"id":"u11","name":"Mages","kind":"unit","line":"Caster","status":"","portrait":0,"icon":"fire","count":3,"health":150,"healthMax":270},{"id":"u12","name":"Wolves","kind":"unit","line":"Summoned","status":"","portrait":0,"icon":"skull","count":4,"health":200,"healthMax":400}],"commands":[{"id":"move","label":"Move","icon":"play","action":"cmdMove","hotkey":"","enabled":true,"reason":"","cooldown":0,"cooldownText":"","active":false},{"id":"stop","label":"Stop","icon":"pause","action":"cmdStop","hotkey":"","enabled":true,"reason":"","cooldown":0,"cooldownText":"","active":false},{"id":"hold","label":"Hold position","icon":"shield","action":"cmdHold","hotkey":"","enabled":true,"reason":"","cooldown":0,"cooldownText":"","active":false},{"id":"attack","label":"Attack move","icon":"sword","action":"cmdAttack","hotkey":"","enabled":true,"reason":"","cooldown":0,"cooldownText":"","active":false},{"id":"patrol","label":"Patrol","icon":"map","action":"","hotkey":"P","enabled":true,"reason":"","cooldown":0,"cooldownText":"","active":false},{"id":"banner","label":"Raise the Banner of the Old Kingdom","icon":"flag","action":"","hotkey":"B","enabled":false,"reason":"Requires the Hall of Ancestors and 400 renown","cooldown":0,"cooldownText":"","active":false},{"id":"rally","label":"Rally","icon":"crown","action":"","hotkey":"R","enabled":false,"reason":"","cooldown":0.95,"cooldownText":"1:54","active":false},{"id":"heal","label":"Mend","icon":"potion","action":"","hotkey":"H","enabled":true,"reason":"","cooldown":0,"cooldownText":"","active":true},{"id":"form","label":"Formation","icon":"gear","action":"","hotkey":"F","enabled":true,"reason":"","cooldown":0,"cooldownText":"","active":false},{"id":"split","label":"Split","icon":"plus","action":"","hotkey":"S","enabled":true,"reason":"","cooldown":0,"cooldownText":"","active":false}]}'>

<!--
  The selection panel: what is selected as portrait cells (a count when a
  group holds several, a health bar), the focused row's name, line, status
  and health, and its command card as a grid of plates, each with its key
  cap, a cooldown shade and, when disabled, the reason in words. Reports
  choose (a cell's key: focus that subgroup) and command (a plate's key).
  The game binds the hotkeys and decides every flag, figure and reason.
-->
<section id="StrategySelection" part="StrategySelection.frame" aria-label={title != '' ? title : t("strategy.selection.title")}>
  <header part="StrategySelection.band"><h2 part="StrategySelection.title">{title != '' ? title : t("strategy.selection.title")}</h2><span part="StrategySelection.badge" class="summary" hidden={summary == ''}>{summary}</span></header>
  {#if units.length}
    <div class="body">
      <div class="left">
        <scroll class="roster">
          <div class="units" role="listbox" aria-label={t("strategy.selection.units")}>
            {#each units as u (u.id)}
              <button part="StrategySelection.cell" role="option" aria-selected={u.id == focused} aria-label={u.name} aria-description={t("strategy.selection.unitValue", { count: u.count, now: u.health, max: u.healthMax })} data-kind={u.kind} onclick={choose}><div class="face">
                <div part="StrategySelection.portrait" class="picture"><img class="art" src={u.portrait} alt="" hidden={!u.portrait}><og-icon class="mark" icon={u.icon} hidden={!!u.portrait}></og-icon><span part="StrategySelection.badge" class="count" hidden={!(u.count > 1)}>{t("strategy.selection.count", { count: u.count })}</span></div>
                <div part="StrategySelection.meter" class="health" role="meter" aria-label={t("strategy.selection.health")} aria-valuemin="0" aria-valuemax={u.healthMax} aria-valuenow={u.health} style:--fill={u.healthMax > 0 ? u.health / u.healthMax : 0}><div class="fill"></div></div>
              </div></button>
            {/each}
          </div>
        </scroll>
        {#each units as u (u.id)}
          {#if u.id == focused}
            <div part="StrategySelection.detail" class="detail" data-kind={u.kind}>
              <span part="StrategySelection.name" class="unit-name">{u.name}</span>
              <span class="line" hidden={u.line == ''}>{u.line}</span>
              <span class="status" hidden={u.status == ''}>{u.status}</span>
              <div class="vital"><StatBar value={u.health} max={u.healthMax} kind="health" segment={0} shield={0} shieldMax={0} heal={0} recoverable={0} label="" icon="" showValue={true} valueInside={false} valueText={t("strategy.selection.healthValue", { now: u.health, max: u.healthMax })} low={0} critical={0} bare={true} name={t("strategy.selection.health")} /></div>
            </div>
          {/if}
        {/each}
      </div>
      <div part="StrategySelection.grid" class="commands" role="toolbar" aria-label={t("strategy.selection.commands")} hidden={!commands.length}>
        {#each commands as c (c.id)}
          <button part="StrategySelection.command" aria-label={c.label} aria-description={c.reason != '' ? c.reason : c.cooldownText} aria-pressed={c.active} disabled={!c.enabled} data-active={c.active} data-cooling={c.cooldown > 0} onclick={command}><div class="face">
            <div class="cool" hidden={!(c.cooldown > 0)} style:--cool={c.cooldown}></div>
            <div class="top"><kbd part="StrategySelection.key" class="key" data-action={c.action} hidden={c.action == '' && c.hotkey == ''}>{c.hotkey}</kbd><span class="on" hidden={!c.active}>{t("strategy.selection.active")}</span></div>
            <og-icon class="command-art" icon={c.icon}></og-icon>
            <span class="cool-text" hidden={c.cooldownText == ''}>{c.cooldownText}</span>
            <span class="command-label">{c.label}</span>
            <span part="StrategySelection.reason" class="reason" hidden={c.enabled || c.reason == ''}>{c.reason}</span>
          </div></button>
        {/each}
      </div>
    </div>
  {:else}
    <p class="empty-text" role="status">{t("strategy.selection.empty")}</p>
  {/if}
</section>

<style>
[part="StrategySelection.frame"][hidden], [part="StrategySelection.frame"] [hidden] { display: none; }
[part="StrategySelection.frame"] { display: flex; flex-direction: column; width: stretch; min-width: 0; gap: var(--space-sm); container-type: inline-size; }
[part="StrategySelection.band"] { display: flex; width: stretch; align-items: baseline; gap: var(--space-sm); flex: none; }
[part="StrategySelection.title"] { margin: 0; flex: 1; min-width: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.summary { flex: none; max-width: 50%; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.body { display: flex; width: stretch; gap: var(--space-md); align-items: flex-start; }
.left { display: flex; flex-direction: column; flex: 1; min-width: 0; gap: var(--space-sm); }
/* The roster scrolls past two rows and never shrinks below one. */
.roster { width: stretch; min-height: calc(var(--size-xl) * 1.3); max-height: calc(var(--size-xl) * 2.6); flex: none; contain: inline-size; }
.units { display: flex; flex-wrap: wrap; gap: var(--space-xs); padding: var(--space-xxs); }
[part="StrategySelection.cell"] { flex: none; width: var(--size-xl); padding: var(--space-xxs); }
[part="StrategySelection.cell"] .face { display: flex; flex-direction: column; position: relative; width: stretch; gap: var(--space-xxs); }
[part="StrategySelection.portrait"] { width: stretch; aspect-ratio: 1; align-items: center; justify-content: center; overflow: hidden; }
.art { width: stretch; height: stretch; object-fit: cover; }
.mark { width: 62%; height: 62%; }
.count { position: absolute; inset-inline-end: 0; bottom: 0; padding: 0 var(--space-xxs); white-space: nowrap; font-variant-numeric: tabular-nums; }
[part="StrategySelection.meter"] { width: stretch; height: var(--space-xs); overflow: hidden; }
[part="StrategySelection.meter"] .fill { position: absolute; inset-inline-start: 0; top: 0; width: calc(var(--fill) * 100%); height: stretch; }
[part="StrategySelection.detail"] { display: flex; flex-direction: column; width: stretch; gap: var(--space-xxs); padding: var(--space-xs) var(--space-sm); }
.unit-name, .line, .status { width: stretch; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.vital { width: stretch; max-width: calc(var(--size-xxl) * 3); overflow: hidden; }
/* The command card: plates four across, rows as the game fills them. */
[part="StrategySelection.grid"] { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: var(--space-xs); flex: none; width: calc(var(--size-xl) * 5.6); }
[part="StrategySelection.command"] { width: stretch; min-width: 0; overflow: hidden; min-height: calc(var(--size-xl) * 1.15); padding: var(--space-xxs); }
[part="StrategySelection.command"] .face { display: flex; flex-direction: column; position: relative; width: stretch; height: stretch; align-items: center; gap: 0; }
.command-art { width: var(--size-md); height: var(--size-md); }
.cool { position: absolute; left: 0; bottom: 0; width: stretch; height: calc(var(--cool) * 100%); }
/* The key cap and the On word share a top row, so neither covers the icon. */
.top { display: flex; width: stretch; min-height: var(--size-xs); align-items: center; justify-content: space-between; gap: var(--space-xxs); }
.key { flex: none; }
.on { flex: none; margin-inline-start: auto; padding: 0 var(--space-xxs); white-space: nowrap; }
/* The time left sits over the icon, on the cooldown's shade. */
.cool-text { position: absolute; left: 0; top: calc(var(--size-xs) + var(--space-xxs)); width: stretch; height: var(--size-md); text-align: center; white-space: nowrap; font-variant-numeric: tabular-nums; }
.command-label { width: stretch; text-align: center; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
/* A reason wraps: it is the one line that says why, so it is never cut. */
.reason { width: stretch; min-width: 0; text-align: center; white-space: normal; }
.empty-text { margin: 0; padding: var(--space-md) 0; }
@media (pointer: coarse) { .key { display: none; } }
/* A narrow panel stacks the selection over the command card, which takes the width. */
@container (width < 600px) {
  .body { flex-direction: column; gap: var(--space-sm); }
  .left { width: stretch; }
  .roster { min-height: var(--size-xl); max-height: calc(var(--size-xl) * 1.4); }
  [part="StrategySelection.cell"] { width: var(--size-lg); }
  [part="StrategySelection.grid"] { width: stretch; }
  [part="StrategySelection.command"] { min-height: var(--size-xl); }
}
</style>
