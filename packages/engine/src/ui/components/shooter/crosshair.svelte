<meta name="component" content="Crosshair">
<meta name="strings" content="./shooter.en.ogt">
<meta name="prop:spread" content="number" data-doc="The weapon's current spread, 0 (settled) to 1 (widest): the arms move out from the centre.">
<meta name="prop:hit" content="string" data-default="none" data-doc="The last hit to confirm: none, hit, headshot, kill or shieldBreak. The game clears it to none when its mark should go.">
<meta name="prop:hitAt" content="number" data-doc="The id of that hit (its tick): a new id plays the mark again.">
<meta name="prop:aimed" content="bool" data-doc="The aim rests on an enemy: the reticle takes the target ink.">
<meta name="prop:shape" content="string" data-default="cross" data-doc="cross (four arms and a dot), dot, or circle (a ring and a dot).">
<meta name="parts" content="Crosshair.arm Crosshair.dot Crosshair.hit">
<meta name="fixture:default" content='{"spread":0.2,"hit":"none","hitAt":0,"aimed":false,"shape":"cross"}'>
<meta name="fixture:headshot" content='{"spread":0.45,"hit":"headshot","hitAt":12,"aimed":true,"shape":"cross"}'>
<meta name="fixture:kill" content='{"spread":0,"hit":"kill","hitAt":30,"aimed":false,"shape":"dot"}'>
<meta name="fixture:circle" content='{"spread":0.6,"hit":"shieldBreak","hitAt":4,"aimed":false,"shape":"circle"}'>

<div id="Crosshair" role="image" aria-label={t("shooter.crosshair.name")} class:circle={shape == 'circle'} class:dotOnly={shape == 'dot'} style:--spread={spread}>
  <div id="Up" class="arm vertical" part="Crosshair.arm" class:aimed={aimed}></div>
  <div id="Down" class="arm vertical" part="Crosshair.arm" class:aimed={aimed}></div>
  <div id="Left" class="arm horizontal" part="Crosshair.arm" class:aimed={aimed}></div>
  <div id="Right" class="arm horizontal" part="Crosshair.arm" class:aimed={aimed}></div>
  <div id="Ring" class:aimed={aimed}></div>
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
   centre plus the spread's travel; the dressing gives the ink and keyline. */
#Crosshair {
  --arm: var(--size-xs);
  --line: var(--stroke-thick);
  --gap: calc(var(--space-xs) + var(--spread, 0) * var(--size-md));
  width: var(--size-xxl); height: var(--size-xxl);
  pointer-events: none;
}
.arm { position: absolute; background: var(--scene-ink); border-radius: var(--radius-xs); transition: translate var(--motion-ui-duration) var(--motion-ui-easing); }
.vertical { left: calc(50% - var(--line) / 2); width: var(--line); height: var(--arm); }
.horizontal { top: calc(50% - var(--line) / 2); width: var(--arm); height: var(--line); }
#Up { bottom: calc(50% + var(--gap)); }
#Down { top: calc(50% + var(--gap)); }
#Left { right: calc(50% + var(--gap)); }
#Right { left: calc(50% + var(--gap)); }
.dotOnly .arm, .circle .arm { display: none; }
#Ring { display: none; position: absolute; left: calc(50% - var(--gap) - var(--arm) / 2); top: calc(50% - var(--gap) - var(--arm) / 2); width: calc(var(--gap) * 2 + var(--arm)); height: calc(var(--gap) * 2 + var(--arm)); border-radius: var(--radius-pill); border: var(--line) solid var(--scene-ink); }
.circle #Ring { display: block; }
#Ring.aimed { border-color: var(--danger); }
#Dot { position: absolute; left: calc(50% - var(--line)); top: calc(50% - var(--line)); width: calc(var(--line) * 2); height: calc(var(--line) * 2); border-radius: var(--radius-pill); background: var(--scene-ink); }
/* Hit marks: four short diagonal ticks just outside the arms, shown while
   the game holds `hit`, popped in again by a new `hitAt`. */
#Hit { position: absolute; left: 0; top: 0; width: stretch; height: stretch; opacity: 0; transition: opacity var(--motion-exit); }
#Hit.shown { opacity: 1; scale: 1; transition: opacity var(--motion-enter), scale var(--motion-duration-spring) var(--motion-easing-spring); }
@starting-style { #Hit.shown { opacity: 0; scale: 1.5; } }
.tick { position: absolute; width: var(--line); height: var(--space-md); background: var(--scene-ink); }
#HitNE { left: calc(50% + var(--space-md)); top: calc(50% - var(--space-md) * 2); rotate: 45deg; }
#HitSE { left: calc(50% + var(--space-md)); top: calc(50% + var(--space-md)); rotate: -45deg; }
#HitSW { left: calc(50% - var(--space-md)); top: calc(50% + var(--space-md)); rotate: 45deg; }
#HitNW { left: calc(50% - var(--space-md)); top: calc(50% - var(--space-md) * 2); rotate: -45deg; }
</style>
