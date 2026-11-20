<meta name="component" content="RpgCommands">
<meta name="strings" content="./rpg.en.ogt">
<meta name="prop:commands" content="list" data-doc="The commands in order, keyed rows of { id, label, icon, enabled }: label as the game words it (Attack, Skills, Spells, Items, Defend, Flee).">
<meta name="prop:selected" content="string" data-doc="The command the focus rests on; it takes the primary plate.">
<meta name="fixture:default" content='{"commands":[{"id":"attack","label":"Attack","icon":"sword","enabled":true},{"id":"flee","label":"Flee","icon":"back","enabled":false}],"selected":"attack"}'>

<!--
  A turn's commands as a row of plates, each an icon over its label; the
  selected one is the primary plate. Reports choose (a command's key).
-->
<section id="RpgCommands" part="frame" variant="hud" layout="stack" aria-label={t("rpg.commands.name")}>
  <header class="band" variant="small" part="band" layout="row"><h3 part="title">{t("rpg.commands.name")}</h3></header>
  <div class="plates grid" gap="tight" role="toolbar">
    {#each commands as c (c.id)}
      <button class="cell" part="cell" class:primary={c.id == selected} aria-label={c.label} aria-selected={c.id == selected} disabled={!c.enabled} onclick={choose}><div class="face cover" layout="stack" fill gap="tight"><og-icon class="command-mark" icon={c.icon}></og-icon><span class="command-label" part="headline">{c.label}</span></div></button>
    {/each}
  </div>
</section>

<style>
.cover { justify-content: center; align-items: center; }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(var(--layout-tile), 100%), 1fr)); }
/* Drawn over the scene: the style's tag halo holds its text there. */
#RpgCommands { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
#RpgCommands { inline-size: stretch; }
.band { justify-content: center; flex: none; }
.plates { grid-template-columns: none; grid-auto-flow: column; grid-auto-columns: 1fr; }
.cell { block-size: calc(var(--size-xl) * 1.2); padding: 0; min-inline-size: 0; }
.face { inline-size: 100%; block-size: 100%; }
.command-mark { width: var(--size-lg); height: var(--size-lg); }
@media (width < 600px), (height < 500px) { .plates { grid-auto-flow: row; grid-template-columns: repeat(3, 1fr); } }
</style>
