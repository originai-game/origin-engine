<meta name="component" content="HoldButton">
<meta name="parts" content="HoldButton.base HoldButton.icon HoldButton.label HoldButton.hint">
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

<div id="HoldButton">
  <button id="Button" part="HoldButton.base" data-hold="1200" data-action={action} onhold={confirm} aria-label={label} class:primary={variant == "primary"} class:secondary={variant == "secondary"} class:hasHolding={!!holding} disabled={disabled}>
    {#if icon}<og-icon id="Icon" part="HoldButton.icon" icon={icon}></og-icon>{/if}
    <span id="Label" part="HoldButton.label" class="rest">{label}</span>
    {#if holding}<span id="Holding" part="HoldButton.label" class="held">{holding}</span>{/if}
  </button>
  {#if hint}<span id="Hint" part="HoldButton.hint">{hint}</span>{/if}
</div>

<style>
:host { display: flex; flex-direction: column; align-items: center; gap: var(--space-sm); max-inline-size: 100%; }
[part="HoldButton.base"] { min-height: var(--touch-target); max-inline-size: 100%; padding: var(--space-xs) var(--space-lg); gap: var(--space-sm); align-items: center; justify-content: center; border-radius: var(--button-radius); border: var(--stroke-hairline) solid var(--danger); background: var(--panel-raised); color: var(--danger); }
[part="HoldButton.base"]::-og-hold { background: var(--danger-45); }
[part="HoldButton.base"].primary { border-width: 0; background: var(--primary); color: var(--on-primary); }
[part="HoldButton.base"].primary::-og-hold { background: var(--primary-active); }
[part="HoldButton.base"].secondary { border-width: 0; background: var(--control-strong); color: var(--ink); }
[part="HoldButton.base"].secondary::-og-hold { background: var(--sheen); }
[part="HoldButton.base"]:hover { filter: brightness(1.1); }
[part="HoldButton.base"]:disabled { opacity: var(--state-disabled-opacity); }
[part="HoldButton.icon"] { width: var(--size-xs); height: var(--size-xs); }
[part="HoldButton.icon"], [part="HoldButton.label"] { color: var(--danger); }
.primary [part="HoldButton.icon"], .primary [part="HoldButton.label"] { color: var(--on-primary); }
.secondary [part="HoldButton.icon"], .secondary [part="HoldButton.label"] { color: var(--ink); }
[part="HoldButton.label"] { font: var(--text-body); text-align: center; }
#Button:not(:active) #Holding { display: none; }
#Button.hasHolding:active #Label { display: none; }
[part="HoldButton.hint"] { font: var(--text-caption); color: var(--ink-muted); text-align: center; }
</style>
