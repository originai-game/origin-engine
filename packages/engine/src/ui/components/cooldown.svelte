<meta name="component" content="Cooldown">
<meta name="parts" content="Cooldown.cell Cooldown.icon Cooldown.figure">
<meta name="prop:remaining" content="number" data-doc="Seconds left; 0 is ready.">
<meta name="prop:total" content="number" data-doc="The whole cooldown in seconds.">
<meta name="prop:icon" content="string" data-doc="An icon of the atlas under the sweep.">
<meta name="prop:label" content="string" data-doc="Its accessible name (Dash).">
<meta name="action:use" content="Use">
<meta name="fixture:default" content='{"remaining":3.2,"total":8,"icon":"bolt","label":"Dash"}'>

<button id="Cooldown" part="Cooldown.cell" aria-label={label} class:ready={remaining <= 0} disabled={remaining > 0} onclick={use}>
  <div id="Face">
    <og-icon id="Icon" part="Cooldown.icon" icon={icon}></og-icon>
    <svg id="Sweep" viewBox="0 0 56 56" hidden={remaining <= 0}>
      <circle cx="28" cy="28" r="14" fill="none" stroke-width="28" pathLength="1" stroke-dasharray={(remaining / total) + " 1"} transform="rotate(-90 28 28) scale(-1 1) translate(-56 0)"></circle>
    </svg>
    <span id="Seconds" part="Cooldown.figure" hidden={remaining <= 0}>{remaining < 10 ? formatNumber(remaining) : formatCountdown(remaining)}</span>
  </div>
</button>

<style>
#Face { width: stretch; height: stretch; align-items: center; justify-content: center; }
#Cooldown { width: var(--size-lg); height: var(--size-lg); padding: 0; overflow: hidden; }
#Cooldown.ready { animation-name: ready; animation-duration: var(--motion-duration-slow); animation-timing-function: var(--motion-easing-out); box-shadow: 0 0 var(--space-sm) var(--primary); }
#Icon { width: var(--size-md); height: var(--size-md); }
#Sweep { position: absolute; inset-inline-start: 0; top: 0; width: stretch; height: stretch; stroke: var(--scrim); }
#Seconds { position: absolute; inset-inline-start: 0; top: var(--space-md); width: stretch; text-align: center; font-size: var(--font-step-xl); }
@keyframes ready { 0% { scale: 1.12; } 100% { scale: 1; } }
</style>
