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
  The round on a plate lit along its top, its time left beside it, and
  the score with the player's side in its blue against the other's red.
  The game owns the clock and the score; read only.
-->
<section id="ShooterRound" part="ShooterRound.frame" class="hud bare" data-layout="row" role="group" aria-label={theirs >= 0 ? t("shooter.round.score", { ours: ours, theirs: theirs }) : t("shooter.round.name")} class:urgent={urgent}>
  {#if round > 0 || clock != ''}
  <div id="Plate" part="ShooterRound.plate" data-layout="row" data-gap="tight">
    {#if round > 0}<span part="ShooterRound.legend" class="legend">{t("shooter.round.label", { round: round })}</span>{/if}
    {#if clock != ''}<span part="ShooterRound.clock" class="clock" role="timer">{clock}</span>{/if}
  </div>
  {/if}
  <span class="spacer" data-fill></span>
  {#if theirs >= 0}
  <div id="Score" part="ShooterRound.score" data-layout="row" data-gap="tight">
    <span part="ShooterRound.ours" class="ours">{formatNumber(ours)}</span>
    <span class="dash"></span>
    <span part="ShooterRound.theirs" class="theirs">{formatNumber(theirs)}</span>
  </div>
  {/if}
</section>

<style>
:host { align-items: center; column-gap: var(--space-md); max-inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
/* The plate: smoked glass, two corners cut, a lit top edge in the accent. */
#Plate { flex: 0 1 auto; min-inline-size: 0; align-items: baseline; padding: calc(3 * var(--font-step-px)) var(--space-sm); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-lit-rim); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
.urgent #Plate { box-shadow: var(--bar-critical-rim); }
.legend { min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; -webkit-text-stroke-width: 0; text-shadow: none; }
.clock { flex: none; font-size: var(--hud-size-sm); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; -webkit-text-stroke-width: 0; text-shadow: none; }
.urgent .clock { color: var(--hud-empty-ink); }
.spacer { min-inline-size: 0; }
#Score { flex: none; align-items: baseline; }
.ours, .theirs { font-size: var(--hud-size-md); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.ours { color: var(--hud-ally); }
.theirs { color: var(--hud-foe); }
.dash { align-self: center; inline-size: calc(10 * var(--font-step-px)); block-size: calc(2 * var(--font-step-px)); background: var(--hud-ink-dim); }
@media (width < 600px) {
  :host { flex-wrap: wrap; row-gap: var(--space-xxs); justify-content: flex-start; }
  .spacer { display: none; }
  #Plate { flex-wrap: wrap; }
  .ours, .theirs { font-size: var(--hud-size-sm); }
  .clock { font-size: var(--hud-size-xs); }
}
</style>
