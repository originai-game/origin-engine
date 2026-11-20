<meta name="component" content="ShooterRound">
<meta name="strings" content="./shooter.en.ogt">
<meta name="parts" content="ShooterRound.frame ShooterRound.plate ShooterRound.legend ShooterRound.clock ShooterRound.score ShooterRound.ours ShooterRound.theirs">
<meta name="prop:round" content="number" data-doc="The round in play; 0 hides the word.">
<meta name="prop:clock" content="string" data-doc="The round's time left as the game words it (1:42); empty hides it.">
<meta name="prop:ours" content="number" data-doc="The player's side's score.">
<meta name="prop:theirs" content="number" data-default="-1" data-doc="The other side's score; negative hides the score.">
<meta name="prop:urgent" content="bool" data-doc="The game says time is short: the plate's lit edge and the clock turn to danger.">
<meta name="fixture:default" content='{"round":2,"clock":"1:42","ours":7,"theirs":5,"urgent":false}'>
<meta name="fixture:urgent" content='{"round":12,"clock":"0:09","ours":12,"theirs":12,"urgent":true}'>
<meta name="fixture:clock" content='{"round":0,"clock":"14:58","ours":0,"theirs":-1,"urgent":false}'>
<meta name="fixture:overflow" content='{"round":123,"clock":"1:02:44","ours":12345,"theirs":67890,"urgent":false}'>

<!--
  The round at the top of the screen: the player's side's score on a plate
  in its blue and the other side's on one in its red, either side of a
  smoked plate lit along its top that holds the round over its time left.
  The game owns the clock and the score; read only.
-->
<section id="ShooterRound" part="ShooterRound.frame" class="hud bare" data-layout="row" role="group" aria-label={theirs >= 0 ? t("shooter.round.score", { ours: ours, theirs: theirs }) : t("shooter.round.name")} class:urgent={urgent}>
  {#if theirs >= 0}<div id="Ours" part="ShooterRound.score" class="side"><span part="ShooterRound.ours" class="ours">{formatNumber(ours)}</span></div>{/if}
  {#if round > 0 || clock != ''}
  <div id="Plate" part="ShooterRound.plate" data-layout="stack" data-gap="none">
    {#if round > 0}<span part="ShooterRound.legend" class="legend">{t("shooter.round.label", { round: round })}</span>{/if}
    {#if clock != ''}<span part="ShooterRound.clock" class="clock" role="timer">{clock}</span>{/if}
  </div>
  {/if}
  {#if theirs >= 0}<div id="Theirs" part="ShooterRound.score" class="side"><span part="ShooterRound.theirs" class="theirs">{formatNumber(theirs)}</span></div>{/if}
</section>

<style>
:host { align-items: stretch; justify-content: center; column-gap: calc(3 * var(--font-step-px)); max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; filter: var(--hud-lift); }
/* The time: smoked glass, a lit top edge in the accent, the round over the clock. */
#Plate { flex: 0 1 auto; min-inline-size: 0; align-items: center; justify-content: center; padding: calc(3 * var(--font-step-px)) var(--space-md) calc(4 * var(--font-step-px)); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-lit-rim); }
.urgent #Plate { box-shadow: var(--bar-critical-rim); animation: var(--pulse-motion); }
.legend { max-inline-size: 100%; min-inline-size: 0; font-size: var(--hud-tag-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; -webkit-text-stroke-width: 0; text-shadow: none; }
.clock { flex: none; font-size: var(--hud-size-md); line-height: 1.05; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.urgent .clock { color: var(--hud-empty-ink); }
/* The sides: a plate each in its side's colour, cut on its outer corners. */
.side { display: flex; flex: 0 1 auto; min-inline-size: calc(48 * var(--font-step-px)); align-items: center; justify-content: center; padding: 0 var(--space-sm); }
#Ours { background: var(--hud-ally-plate); box-shadow: var(--hud-ally-plate-rim); border-radius: var(--hud-plate-cut) 0 0 var(--hud-plate-cut); corner-shape: var(--cut-shape); }
#Theirs { background: var(--hud-foe-plate); box-shadow: var(--hud-foe-plate-rim); border-radius: 0 var(--hud-plate-cut) var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); }
.ours, .theirs { min-inline-size: 0; font-size: var(--hud-size-lg); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
@media (width < 600px), (height < 500px) {
  .side { min-inline-size: calc(34 * var(--font-step-px)); padding: 0 var(--space-xs); }
  #Plate { padding-inline: var(--space-sm); }
  .ours, .theirs { font-size: var(--hud-size-md); }
  .clock { font-size: var(--hud-size-sm); }
}
@media (prefers-reduced-motion: reduce) { .urgent #Plate { animation: none; } }
</style>
