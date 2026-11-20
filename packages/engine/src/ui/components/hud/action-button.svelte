<meta name="component" content="ActionButton">
<meta name="prop:action" content="string" data-doc="The og-input action it feeds while a touch holds it (jump).">
<meta name="prop:label" content="string" data-doc="Its words, its accessible name; shown when it has no icon.">
<meta name="prop:icon" content="string" data-doc="An icon of the atlas on its face; empty shows the label.">
<meta name="prop:primary" content="bool" data-doc="The main action: the primary fill and a larger disc.">
<meta name="prop:charge" content="number" data-default="-1" data-doc="A charge round its rim, 0 to 1 (an ultimate); negative is none.">
<meta name="prop:disabled" content="bool" data-doc="It feeds nothing and shows faded.">
<meta name="fixture:default" content='{"action":"jump","label":"Jump","icon":"bolt","primary":true,"charge":0.7,"disabled":false}'>

<button id="ActionButton" part="disc" data-og-touch={action} aria-label={label} class:primary={primary} disabled={disabled}>
  <div id="Face" layout="overlay">
    {#if charge >= 0}<svg id="Charge" anchor="fill" viewBox="0 0 80 80">
      <circle id="ChargeTrack" cx="40" cy="40" r="37" stroke-width="4"></circle>
      <circle id="ChargeFill" cx="40" cy="40" r="37" stroke-width="4" stroke-linecap="round" pathLength="1" stroke-dasharray={charge + " 1"} transform="rotate(-90 40 40)"></circle>
    </svg>{/if}
    {#if icon}<og-icon id="Icon" part="icon" icon={icon}></og-icon>
    {:else}<span id="Label" part="text">{label}</span>{/if}
  </div>
</button>

<style>
#Face { inline-size: 100%; block-size: 100%; justify-content: center; align-items: center; --layout-anchor-inset: 0; }
#ActionButton { width: var(--action-size); height: var(--action-size); padding: 0; border-radius: var(--radius-capsule); border: var(--stroke-regular) solid var(--line-strong); background: radial-gradient(circle at 50% 35%, var(--sheen), var(--shadow-30)), var(--control-strong); box-shadow: var(--shadow-float); transition: scale var(--motion-ui-duration) var(--motion-ui-easing); }
.primary { width: calc(var(--action-size) * 1.25); height: calc(var(--action-size) * 1.25); background: radial-gradient(circle at 50% 35%, var(--sheen), var(--shadow-30)), var(--primary); }
#ActionButton:active { scale: var(--state-press-scale); }
#ActionButton:disabled { opacity: var(--state-touch-disabled-opacity); }
#Charge { inline-size: 100%; block-size: 100%; pointer-events: none; }
#ChargeTrack { fill: none; stroke: var(--scrim); }
#ChargeFill { fill: none; stroke: var(--gold); }
#Icon { width: var(--size-md); height: var(--size-md); color: var(--ink); }
.primary #Icon { color: var(--on-primary); }
#Label { font: var(--text-sm); white-space: nowrap; }
.primary #Label { color: var(--on-primary); }
</style>
