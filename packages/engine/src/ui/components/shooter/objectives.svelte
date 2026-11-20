<meta name="component" content="ShooterObjectives">
<meta name="strings" content="./shooter.en.ogt">
<meta name="prop:label" content="string" data-doc="The block's heading as the game words it; empty takes the component's Objective.">
<meta name="prop:goals" content="list" data-doc="The objectives in the game's order, keyed rows of { id, text, tag, current }: text the objective as the game words it; tag its progress or time on a tag (2 / 3, 0:45), empty for none; current the one in play (lit; the others are dimmed). The game owns progress and completion.">
<meta name="fixture:default" content='{"label":"","goals":[{"id":"relay","text":"Secure the relay tower","tag":"2 / 3","current":true},{"id":"catwalk","text":"Hold the catwalk","tag":"0:45","current":false}]}'>
<meta name="fixture:one" content='{"label":"Contract","goals":[{"id":"extract","text":"Reach the extraction point","tag":"","current":true}]}'>
<meta name="fixture:empty" content='{"label":"","goals":[]}'>
<meta name="fixture:overflow" content='{"label":"Primary and secondary objectives of the operation","goals":[{"id":"long","text":"Secure the relay tower on the northern ridge before the storm wall closes in","tag":"12,345 / 99,999","current":true},{"id":"two","text":"Hold the catwalk above the old refinery until reinforcements arrive","tag":"1:02:45","current":false},{"id":"three","text":"Recover the black box","tag":"","current":false}]}'>

<!--
  The objectives at the screen's top corner: a heading in the HUD's
  accent, the objective in play lit with its progress on a tag, the others
  dimmed under it with theirs (a phone shows the one in play alone). Read
  only.
-->
<section id="ShooterObjectives" part="frame" variant="hud" layout="stack" role="group" aria-label={label != '' ? label : t("shooter.objective.label")}>
  {#if goals.length}
  <span part="legend" class="legend">{label != '' ? label : t("shooter.objective.label")}</span>
  {#each goals as g (g.id)}
  <div part="goal" class="goal" layout="row" gap="tight" class:current={g.current}>
    <div part="marker" class="marker"></div>
    <span part="text" class="text">{g.text}</span>
    {#if g.tag}<span part="progress" class="tag">{g.tag}</span>{/if}
  </div>
  {/each}
  {/if}
</section>

<style>
#ShooterObjectives { row-gap: calc(6 * var(--font-step-px)); inline-size: calc(280 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.legend { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-accent); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; min-inline-size: 0; }
.goal { min-inline-size: 0; opacity: 0.66; }
.goal.current { opacity: 1; }
/* A diamond: gold for the objective in play, dim for the rest. */
.marker { flex: none; inline-size: calc(8 * var(--font-step-px)); block-size: calc(8 * var(--font-step-px)); margin-inline: calc(2 * var(--font-step-px)); transform: rotate(45deg); background: var(--hud-ink-dim); box-shadow: var(--hud-marker-rim); }
.current .marker { inline-size: calc(10 * var(--font-step-px)); block-size: calc(10 * var(--font-step-px)); margin-inline: calc(1 * var(--font-step-px)); background: var(--hud-gold); }
.text { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.current .text { font-size: var(--hud-size-sm); }
.tag { flex: none; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; -webkit-text-stroke-width: 0; text-shadow: none; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
#ShooterObjectives:size(compact) { inline-size: stretch; }
.legend:size(compact), .goal:size(compact) { display: none; }
.goal.current:size(compact) { display: flex; }
</style>
