<meta name="component" content="RpgCombatLog">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgCombatLog.frame RpgCombatLog.band RpgCombatLog.title RpgCombatLog.row">
<meta name="prop:lines" content="list" data-doc="The log, oldest first, keyed rows of { id, text, tone, detail }: tone damage, heal, status, crit or plain inks the line; detail a quieter second line (→ 42 damage), empty for none. The game words every line.">
<meta name="fixture:default" content='{"lines":[{"id":"l1","text":"Sir Aldric uses Shield Bash on Cultist.","tone":"plain","detail":"→ 42 damage"},{"id":"l2","text":"Cultist is Stunned!","tone":"status","detail":""}]}'>

<!--
  The combat log: the newest lines at the foot, each inked by its tone.
  Scrolls inside the height it is given. Read only.
-->
<section id="RpgCombatLog" part="RpgCombatLog.frame" class="hud" role="log" aria-label={t("rpg.log.name")}>
  <header class="small" part="RpgCombatLog.band"><h3 part="RpgCombatLog.title">{t("rpg.log.name")}</h3></header>
  <div class="lines">
    {#each lines as l (l.id)}
      <div part="RpgCombatLog.row" data-tone={l.tone}><span class="line-text">{l.text}</span>{#if l.detail}<span class="line-detail">{l.detail}</span>{/if}</div>
    {/each}
  </div>
</section>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); }
:host { display: flex; flex-direction: column; inline-size: stretch; block-size: stretch; gap: var(--space-sm); }
[part="RpgCombatLog.band"] { display: flex; align-items: center; flex: none; }
.lines { display: flex; flex-direction: column; gap: var(--space-sm); flex: 1; min-block-size: 0; overflow-y: auto; }
[part="RpgCombatLog.row"] { display: flex; flex: none; flex-direction: column; gap: var(--space-xxs); padding: var(--space-xs) var(--space-sm); border-inline-start: var(--stroke-regular) solid var(--line); }
[part="RpgCombatLog.row"][data-tone=damage] { border-color: var(--danger); }
[part="RpgCombatLog.row"][data-tone=heal] { border-color: var(--success); }
[part="RpgCombatLog.row"][data-tone=crit] { border-color: var(--warn); }
.line-detail { padding-inline-start: var(--space-md); color: var(--ink-muted); font: var(--text-caption); }
</style>
