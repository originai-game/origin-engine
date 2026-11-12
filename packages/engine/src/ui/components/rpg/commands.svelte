<meta name="component" content="RpgCommands">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgCommands.frame RpgCommands.band RpgCommands.title RpgCommands.cell">
<meta name="prop:commands" content="list" data-doc="The commands in order, keyed rows of { id, label, icon, enabled }: label as the game words it (Attack, Skills, Spells, Items, Defend, Flee).">
<meta name="prop:selected" content="string" data-doc="The command the focus rests on; it takes the primary plate.">
<meta name="fixture:default" content='{"commands":[{"id":"attack","label":"Attack","icon":"sword","enabled":true},{"id":"flee","label":"Flee","icon":"back","enabled":false}],"selected":"attack"}'>

<!--
  A turn's commands as a row of plates, each an icon over its label; the
  selected one is the primary plate. Reports choose (a command's key).
-->
<section id="RpgCommands" part="RpgCommands.frame" aria-label={t("rpg.commands.name")}>
  <header class="small" part="RpgCommands.band"><h3 part="RpgCommands.title">{t("rpg.commands.name")}</h3></header>
  <div class="plates" role="toolbar">
    {#each commands as c (c.id)}
      <button part="RpgCommands.cell" class:primary={c.id == selected} aria-label={c.label} aria-selected={c.id == selected} disabled={!c.enabled} onclick={choose}><div class="face"><og-icon class="command-mark" icon={c.icon}></og-icon><span class="command-label">{c.label}</span></div></button>
    {/each}
  </div>
</section>

<style>
[part="RpgCommands.frame"] { display: flex; flex-direction: column; width: stretch; gap: var(--space-xs); }
[part="RpgCommands.band"] { display: flex; width: stretch; align-items: center; justify-content: center; flex: none; }
.plates { display: grid; grid-auto-flow: column; grid-auto-columns: 1fr; gap: var(--space-xs); width: stretch; }
[part="RpgCommands.cell"] { width: stretch; height: calc(var(--size-xl) * 1.2); padding: 0; }
.face { display: flex; flex-direction: column; width: stretch; height: stretch; align-items: center; justify-content: center; gap: var(--space-xxs); }
.command-mark { width: var(--size-lg); height: var(--size-lg); }
@media (width < 600px), (height < 500px) { .plates { grid-auto-flow: row; grid-template-columns: repeat(3, 1fr); } }
</style>
