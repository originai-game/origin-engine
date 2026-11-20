<meta name="component" content="RpgCombatLog">
<meta name="strings" content="./combat-log.en.ogt">
<meta name="prop:lines" content="list" data-doc="The log, oldest first, keyed rows of { id, text, tone, detail }: tone damage, heal, status, crit or plain inks the line; detail a quieter second line (→ 42 damage), empty for none. The game words every line.">
<meta name="fixture:default" content='{"lines":[{"id":"l1","text":"Sir Aldric uses Shield Bash on Cultist.","tone":"plain","detail":"→ 42 damage"},{"id":"l2","text":"Cultist is Stunned!","tone":"status","detail":""}]}'>

<!--
  The combat log: the newest lines at the foot, each inked by its tone.
  Scrolls inside the height it is given. Read only.
-->
<section id="RpgCombatLog" part="frame" variant="hud" layout="stack" fill role="log" aria-label={t("rpg.log.name")}>
  <header class="band" variant="small" part="band" layout="row"><h3 part="title">{t("rpg.log.name")}</h3></header>
  <div class="scroll-shell" layout="overlay" fill><div class="lines" layout="scroll" anchor="fill">
    {#each lines as l (l.id)}
      <div class="row" part="row" layout="stack" gap="hair" data-tone={l.tone}><span class="line-text">{l.text}</span>{#if l.detail}<span class="line-detail" part="caption">{l.detail}</span>{/if}</div>
    {/each}
  </div><div part="fade" anchor="bottom-edge"></div></div>
</section>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
#RpgCombatLog { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
.band { flex: none; }
.scroll-shell { --fade-size: var(--space-lg); }
.lines { padding-block-end: var(--fade-size); }
.row { flex: none; padding: var(--space-xs) var(--space-sm); }
.line-detail { padding-inline-start: var(--space-md); }
</style>
