<meta name="component" content="Cooldown">
<meta name="prop:remaining" content="number" data-doc="Seconds left; 0 is ready.">
<meta name="prop:total" content="number" data-doc="The whole cooldown in seconds.">
<meta name="prop:icon" content="string" data-doc="An icon of the atlas under the sweep.">
<meta name="prop:label" content="string" data-doc="Its accessible name (Dash).">
<meta name="action:use" content="Use">
<meta name="fixture:default" content='{"remaining":3.2,"total":8,"icon":"bolt","label":"Dash"}'>

<button id="Cooldown" aria-label={label} class:ready={remaining <= 0} disabled={remaining > 0} onclick={use}>
  <div id="Face">
    <og-icon id="Icon" icon={icon}></og-icon>
    <svg id="Sweep" viewBox="0 0 56 56" hidden={remaining <= 0}>
      <circle cx="28" cy="28" r="14" fill="none" stroke="#000000b3" stroke-width="28" pathLength="1" stroke-dasharray={(remaining / total) + " 1"} transform="rotate(-90 28 28) scale(-1 1) translate(-56 0)"></circle>
    </svg>
    <span id="Seconds" hidden={remaining <= 0}>{remaining < 10 ? formatNumber(remaining) : formatCountdown(remaining)}</span>
  </div>
</button>

<style>
#Face { width: stretch; height: stretch; align-items: center; justify-content: center; }
#Cooldown { width: 56px; height: 56px; padding: 0; border-radius: 8px; overflow: hidden; background: var(--control-strong); border: 1px solid var(--line-strong); }
#Cooldown.ready { animation: ready 380ms ease-out; box-shadow: 0 0 8px var(--primary); }
#Icon { width: 36px; height: 36px; }
#Sweep { position: absolute; inset-inline-start: 0; top: 0; width: 56px; height: 56px; }
#Seconds { position: absolute; inset-inline-start: 0; top: 16px; width: stretch; text-align: center; font: var(--text-lg); text-shadow: 0 1px 2px #000000ff; }
@keyframes ready { 0% { scale: 1.12; } 100% { scale: 1; } }
</style>
