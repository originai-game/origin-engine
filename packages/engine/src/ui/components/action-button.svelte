<meta name="component" content="ActionButton">
<meta name="prop:action" content="string" data-doc="The og-input action it feeds while a touch holds it (jump).">
<meta name="prop:label" content="string" data-doc="Its words, its accessible name; shown when it has no icon.">
<meta name="prop:icon" content="string" data-doc="An icon of the atlas on its face; empty shows the label.">
<meta name="prop:primary" content="bool" data-doc="The main action: the primary fill and a larger disc.">
<meta name="prop:charge" content="number" data-doc="A charge round its rim, 0 to 1 (an ultimate); negative is none.">
<meta name="prop:disabled" content="bool" data-doc="It feeds nothing and shows faded.">
<meta name="fixture:default" content='{"action":"jump","label":"Jump","icon":"bolt","primary":true,"charge":0.7,"disabled":false}'>

<button id="ActionButton" data-og-touch={action} aria-label={label} class:primary={primary} disabled={disabled}>
  <div id="Face">
    <svg id="Charge" viewBox="0 0 80 80" hidden={charge < 0}>
      <circle cx="40" cy="40" r="37" fill="none" stroke="#00000080" stroke-width="4"></circle>
      <circle id="ChargeFill" cx="40" cy="40" r="37" fill="none" stroke-width="4" stroke-linecap="round" pathLength="1" stroke-dasharray={charge + " 1"} transform="rotate(-90 40 40)"></circle>
    </svg>
    <og-icon id="Icon" icon={icon} hidden={!icon}></og-icon>
    <span id="Label" hidden={!!icon}>{label}</span>
  </div>
</button>

<style>
#Face { width: stretch; height: stretch; align-items: center; justify-content: center; }
#ActionButton { width: 64px; height: 64px; padding: 0; border-radius: 32px; border: 2px solid #ffffff59; background: radial-gradient(circle at 50% 35%, #ffffff33, #00000059), var(--control-strong); box-shadow: 0 3px 8px #00000080; }
#ActionButton.primary { width: 80px; height: 80px; border-radius: 40px; background: radial-gradient(circle at 50% 35%, #ffffff40, #00000040), var(--primary); }
#ActionButton:active { scale: 0.92; filter: brightness(1.2); }
#ActionButton:disabled { opacity: 0.45; }
#Charge { position: absolute; inset-inline-start: 0; top: 0; width: stretch; height: stretch; }
#ChargeFill { stroke: var(--gold); }
#Icon { width: 32px; height: 32px; color: var(--ink); }
.primary #Icon { color: var(--on-primary); }
#Label { font: var(--text-sm); white-space: nowrap; }
</style>
