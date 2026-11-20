<meta name="component" content="Cooldown">
<meta name="prop:remaining" content="number" data-doc="Seconds left; 0 is ready.">
<meta name="prop:total" content="number" data-default="1" data-doc="The whole cooldown in seconds.">
<meta name="prop:icon" content="string" data-doc="An icon of the atlas under the sweep.">
<meta name="prop:label" content="string" data-doc="Its accessible name (Dash).">
<meta name="action:use" content="Use">
<meta name="fixture:default" content='{"remaining":3.2,"total":8,"icon":"bolt","label":"Dash"}'>

<button id="Cooldown" part="cell" data-motion={remaining <= 0 ? 'pop' : null} aria-label={label} class:ready={remaining <= 0} disabled={remaining > 0} onclick={use}>
  <div id="Face" layout="overlay" gap="none">
    <og-icon layout="center" id="Icon" part="icon" anchor="fill" icon={icon}></og-icon>
    {#if remaining > 0}<svg id="Sweep" anchor="fill" viewBox="0 0 56 56">
      <circle part="sweep" cx="28" cy="28" r="14" fill="none" stroke-width="28" pathLength="1" stroke-dasharray={(remaining / total) + " 1"} transform="rotate(-90 28 28) scale(-1 1) translate(-56 0)"></circle>
    </svg>
    <span layout="center" id="Seconds" class="seconds" part="figure" anchor="fill">{remaining < 10 ? formatNumber(remaining) : formatCountdown(remaining)}</span>{/if}
  </div>
</button>

<style>
#Cooldown { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
#Face { inline-size: 100%; block-size: 100%; }
/* Over the scene it is the style's scene cell. */
#Cooldown { width: var(--size-lg); height: var(--size-lg); padding: 0; overflow: hidden; --cell-fill: var(--cell-scene-fill); --cell-rim: var(--cell-scene-rim); --cell-shadow: var(--cell-scene-shadow); --cell-empty-shadow: var(--cell-scene-shadow); }
#Icon { width: var(--size-md); height: var(--size-md); }
#Sweep { inline-size: 100%; block-size: 100%; }
#Seconds { text-align: center; --figure-size: var(--font-step-xl); --figure-shadow: var(--tag-halo); }
</style>
