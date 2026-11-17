<meta name="component" content="HoldButton">
<meta name="parts" content="HoldButton.base HoldButton.icon HoldButton.label HoldButton.hint HoldButton.caption">
<meta name="prop:label" content="string" data-doc="Its words and accessible name (Delete save).">
<meta name="prop:holding" content="string" data-doc="What it says while it is held (Keep holding…); empty keeps its label.">
<meta name="prop:hint" content="string" data-doc="A line under it, what to do (Hold to delete); empty for none.">
<meta name="prop:icon" content="string" data-doc="An icon of the atlas before the label; empty for none.">
<meta name="prop:action" content="string" data-doc="An og-input action whose hold holds it too (confirm); empty for none.">
<meta name="prop:variant" content="string" data-default="danger" data-doc="Its role: danger (the default, a destructive confirm), primary or secondary.">
<meta name="prop:disabled" content="bool" data-doc="Shown but inert.">
<meta name="action:confirm" content="Confirm">
<meta name="fixture:default" content='{"label":"Delete save","holding":"Keep holding…","hint":"Hold to delete","icon":"close","action":"","variant":"danger","disabled":false}'>
<meta name="fixture:label-only" content='{"label":"Secure checkpoint","holding":"","hint":"Hold to secure","icon":"","action":"","variant":"primary","disabled":false}'>
<meta name="fixture:disabled" content='{"label":"Delete save","holding":"Keep holding…","hint":"Unavailable during combat","icon":"close","action":"","variant":"danger","disabled":true}'>
<meta name="fixture:overflow" content='{"label":"Abandon the current expedition","holding":"Keep holding to confirm","hint":"Hold to abandon the expedition","icon":"close","action":"","variant":"danger","disabled":false}'>

<div id="HoldButton" data-layout="stack">
  <button id="Button" part="HoldButton.base" variant={variant} data-hold="1200" data-action={action} onhold={confirm} aria-label={label} class:primary={variant == "primary"} class:secondary={variant == "secondary"} class:hasHolding={!!holding} disabled={disabled}>
    {#if icon}<og-icon id="Icon" part="HoldButton.icon" icon={icon}></og-icon>{/if}
    <span id="Label" part="HoldButton.label" class="rest">{label}</span>
    {#if holding}<span id="Holding" part="HoldButton.label" class="held">{holding}</span>{/if}
  </button>
  {#if hint}<span id="Hint" part="HoldButton.hint HoldButton.caption" class="hint">{hint}</span>{/if}
</div>

<style>
:host { align-items: center; max-inline-size: 100%; }
[part="HoldButton.base"] { min-height: var(--touch-target); max-inline-size: 100%; padding: var(--space-xs) var(--space-lg); align-items: center; justify-content: center; --ink-label: currentColor; }
[part="HoldButton.icon"] { width: var(--size-xs); height: var(--size-xs); }
[part~="HoldButton.label"] { text-align: center; }
#Button:not(:active) #Holding { display: none; }
#Button.hasHolding:active #Label { display: none; }
[part~="HoldButton.hint"] { text-align: center; }
</style>
