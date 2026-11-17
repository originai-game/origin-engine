<meta name="component" content="Crosshair">
<meta name="strings" content="./crosshair.en.ogt">
<meta name="prop:spread" content="number" data-doc="The weapon's current spread, 0 (settled) to 1 (widest): the arms move out from the centre.">
<meta name="prop:hit" content="string" data-default="none" data-doc="The last hit to confirm: none, hit, headshot, kill or shieldBreak. The game clears it to none when its mark should go.">
<meta name="prop:hitAt" content="number" data-doc="The id of that hit (its tick): a new id plays the mark again.">
<meta name="prop:aimed" content="bool" data-doc="The aim rests on an enemy: the reticle takes the target ink.">
<meta name="prop:shape" content="string" data-default="cross" data-doc="cross (four arms and a dot), dot, or circle (a ring and a dot).">
<meta name="parts" content="Crosshair.arm Crosshair.ring Crosshair.dot Crosshair.hit">
<meta name="fixture:default" content='{"spread":0.2,"hit":"none","hitAt":0,"aimed":false,"shape":"cross"}'>
<meta name="fixture:headshot" content='{"spread":0.45,"hit":"headshot","hitAt":12,"aimed":true,"shape":"cross"}'>
<meta name="fixture:kill" content='{"spread":0,"hit":"kill","hitAt":30,"aimed":false,"shape":"dot"}'>
<meta name="fixture:circle" content='{"spread":0.6,"hit":"shieldBreak","hitAt":4,"aimed":false,"shape":"circle"}'>

<div id="Crosshair" role="image" aria-label={t("shooter.crosshair.name")} class:circle={shape == 'circle'} class:dotOnly={shape == 'dot'} style:--spread={spread}>
  <div id="Up" class="arm vertical" part="Crosshair.arm" class:aimed={aimed}></div>
  <div id="Down" class="arm vertical" part="Crosshair.arm" class:aimed={aimed}></div>
  <div id="Left" class="arm horizontal" part="Crosshair.arm" class:aimed={aimed}></div>
  <div id="Right" class="arm horizontal" part="Crosshair.arm" class:aimed={aimed}></div>
  <div id="Ring" part="Crosshair.ring" class:aimed={aimed}></div>
  <div id="Dot" part="Crosshair.dot" class:aimed={aimed}></div>
  {#key hitAt}
    <div id="Hit" class:shown={hit != 'none'}>
      <div id="HitNE" class="tick" part="Crosshair.hit" class:headshot={hit == 'headshot'} class:kill={hit == 'kill'} class:shieldBreak={hit == 'shieldBreak'}></div>
      <div id="HitSE" class="tick" part="Crosshair.hit" class:headshot={hit == 'headshot'} class:kill={hit == 'kill'} class:shieldBreak={hit == 'shieldBreak'}></div>
      <div id="HitSW" class="tick" part="Crosshair.hit" class:headshot={hit == 'headshot'} class:kill={hit == 'kill'} class:shieldBreak={hit == 'shieldBreak'}></div>
      <div id="HitNW" class="tick" part="Crosshair.hit" class:headshot={hit == 'headshot'} class:kill={hit == 'kill'} class:shieldBreak={hit == 'shieldBreak'}></div>
    </div>
  {/key}
</div>

<style>
/* Geometry is the reticle's structure: arms of --arm length, --gap from the
   centre plus the spread's travel, --weight thick; the roles give the ink
   and the dark keyline that holds it over sky and snow alike. */
:host {
  position: relative;
  --arm: calc(10 * var(--font-step-px));
  --weight: calc(2 * var(--font-step-px));
  --gap: calc(var(--space-sm) + var(--spread, 0) * var(--size-md));
  --tick: calc(14 * var(--font-step-px));
  width: var(--size-xxl); height: var(--size-xxl);
  pointer-events: none;
  color: var(--hud-tick);
}
.arm { position: absolute; background: var(--hud-tick); box-shadow: var(--hud-reticle-rim); transition: translate var(--motion-ui-duration) var(--motion-ui-easing); }
.arm.aimed, #Dot.aimed { background: var(--hud-foe); }
.vertical { left: calc(50% - var(--weight) / 2); width: var(--weight); height: var(--arm); }
.horizontal { top: calc(50% - var(--weight) / 2); width: var(--arm); height: var(--weight); }
#Up { bottom: calc(50% + var(--gap)); }
#Down { top: calc(50% + var(--gap)); }
#Left { right: calc(50% + var(--gap)); }
#Right { left: calc(50% + var(--gap)); }
.dotOnly .arm, .circle .arm { display: none; }
#Ring { display: none; position: absolute; left: calc(50% - var(--gap) - var(--arm) / 2); top: calc(50% - var(--gap) - var(--arm) / 2); width: calc(var(--gap) * 2 + var(--arm)); height: calc(var(--gap) * 2 + var(--arm)); background: var(--hud-tick); mask-image: radial-gradient(closest-side, transparent calc(100% - var(--weight)), #000 calc(100% - var(--weight))); }
.circle #Ring { display: block; }
#Ring.aimed { background: var(--hud-foe); }
#Dot { position: absolute; left: calc(50% - var(--weight)); top: calc(50% - var(--weight)); width: calc(var(--weight) * 2); height: calc(var(--weight) * 2); border-radius: var(--hud-radar-radius); background: var(--hud-tick); box-shadow: var(--hud-reticle-rim); }
/* Hit marks: four short diagonal ticks just outside the arms, shown while
   the game holds `hit`, popped in again by a new `hitAt`: white for a
   hit, red with a glow for a headshot and a kill (a kill's thicker), the
   shield's blue for a broken shield. */
#Hit { position: absolute; inset: 0; opacity: 0; transition: opacity var(--motion-exit); }
#Hit.shown { opacity: 1; scale: 1; transition: opacity var(--motion-enter), scale var(--motion-duration-spring) var(--motion-easing-spring); }
@starting-style { #Hit.shown { opacity: 0; scale: 1.5; } }
.tick { position: absolute; width: var(--tick); height: calc(3 * var(--font-step-px)); background: var(--hud-tick); box-shadow: var(--hud-reticle-rim); }
.tick.headshot, .tick.kill { background: var(--hud-hit); box-shadow: var(--hud-hit-glow); }
.tick.kill { height: calc(4 * var(--font-step-px)); }
.tick.shieldBreak { background: var(--hud-ally); box-shadow: var(--hud-hit-shield-glow); }
#HitNE { left: calc(50% + var(--gap) + var(--space-xs)); top: calc(50% - var(--gap) - var(--space-xs) - var(--tick) / 2); rotate: -45deg; }
#HitSE { left: calc(50% + var(--gap) + var(--space-xs)); top: calc(50% + var(--gap) + var(--space-xs) + var(--tick) / 2); rotate: 45deg; }
#HitSW { left: calc(50% - var(--gap) - var(--space-xs) - var(--tick)); top: calc(50% + var(--gap) + var(--space-xs) + var(--tick) / 2); rotate: -45deg; }
#HitNW { left: calc(50% - var(--gap) - var(--space-xs) - var(--tick)); top: calc(50% - var(--gap) - var(--space-xs) - var(--tick) / 2); rotate: 45deg; }
</style>
