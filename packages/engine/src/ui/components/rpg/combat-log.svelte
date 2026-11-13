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
  <scroll class="lines">
    {#each lines as l (l.id)}
      <div part="RpgCombatLog.row" data-tone={l.tone}><span class="line-text">{l.text}</span><span class="line-detail" hidden={!l.detail}>{l.detail}</span></div>
    {/each}
  </scroll>
</section>

<style>
[part="RpgCombatLog.frame"][hidden], [part="RpgCombatLog.frame"] [hidden] { display: none; }
[part="RpgCombatLog.frame"] { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-xs); }
[part="RpgCombatLog.band"] { display: flex; width: stretch; align-items: center; flex: none; }
.lines { width: stretch; flex: 1; min-height: 0; }
[part="RpgCombatLog.row"] { display: flex; flex-direction: column; padding: 0 var(--space-sm); }
.line-detail { padding-inline-start: var(--space-md); }
</style>
