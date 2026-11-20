<meta name="component" content="RpgCombatLog">
<meta name="strings" content="./combat-log.en.ogt">
<meta name="parts" content="RpgCombatLog.frame RpgCombatLog.band RpgCombatLog.title RpgCombatLog.row RpgCombatLog.fade RpgCombatLog.caption">
<meta name="prop:lines" content="list" data-doc="The log, oldest first, keyed rows of { id, text, tone, detail }: tone damage, heal, status, crit or plain inks the line; detail a quieter second line (→ 42 damage), empty for none. The game words every line.">
<meta name="fixture:default" content='{"lines":[{"id":"l1","text":"Sir Aldric uses Shield Bash on Cultist.","tone":"plain","detail":"→ 42 damage"},{"id":"l2","text":"Cultist is Stunned!","tone":"status","detail":""}]}'>

<!--
  The combat log: the newest lines at the foot, each inked by its tone.
  Scrolls inside the height it is given. Read only.
-->
<section id="RpgCombatLog" part="RpgCombatLog.frame" class="hud" data-layout="shell" role="log" aria-label={t("rpg.log.name")}>
  <header class="small" part="RpgCombatLog.band" data-layout="row"><h3 part="RpgCombatLog.title">{t("rpg.log.name")}</h3></header>
  <div class="scroll-shell" data-layout="overlay" data-fill><div class="lines" data-layout="scroll" data-anchor="fill">
    {#each lines as l (l.id)}
      <div part="RpgCombatLog.row" data-layout="stack" data-gap="hair" data-tone={l.tone}><span class="line-text">{l.text}</span>{#if l.detail}<span class="line-detail" part="RpgCombatLog.caption">{l.detail}</span>{/if}</div>
    {/each}
  </div><div part="RpgCombatLog.fade" data-anchor="bottom-edge"></div></div>
</section>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
[part="RpgCombatLog.band"] { flex: none; }
.scroll-shell { --fade-size: var(--space-lg); }
.lines { padding-block-end: var(--fade-size); }
[part="RpgCombatLog.row"] { flex: none; padding: var(--space-xs) var(--space-sm); }
.line-detail { padding-inline-start: var(--space-md); }
</style>
