<meta name="component" content="HoldButton">
<meta name="prop:label" content="string" data-doc="Its words and accessible name (Delete save).">
<meta name="prop:holding" content="string" data-doc="What it says while it is held (Keep holding…); empty keeps its label.">
<meta name="prop:hint" content="string" data-doc="A line under it, what to do (Hold to delete); empty for none.">
<meta name="prop:icon" content="string" data-doc="An icon of the atlas before the label; empty for none.">
<meta name="prop:action" content="string" data-doc="An og-input action whose hold holds it too (confirm); empty for none.">
<meta name="prop:variant" content="string" data-doc="Its role: danger (the default, a destructive confirm), primary or secondary.">
<meta name="prop:disabled" content="bool" data-doc="Shown but inert.">
<meta name="action:confirm" content="Confirm">
<meta name="fixture:default" content='{"label":"Delete save","holding":"Keep holding…","hint":"Hold to delete","icon":"close","action":"","variant":"danger","disabled":false}'>

<div id="HoldButton">
  <button id="Button" data-hold="1200" data-action={action} onhold={confirm} aria-label={label} class:primary={variant == "primary"} class:secondary={variant == "secondary"} disabled={disabled}>
    <icon id="Icon" icon={icon} hidden={!icon}></icon>
    <span id="Label" class="rest">{label}</span>
    <span id="Holding" class="held" hidden={!holding}>{holding}</span>
  </button>
  <span id="Hint" hidden={!hint}>{hint}</span>
</div>

<style>
#HoldButton { flex-direction: column; align-items: center; gap: 4px; }
#Button { min-height: 44px; padding: 0 20px; gap: 8px; align-items: center; justify-content: center; border-radius: 8px; border: 1px solid var(--danger); background: var(--panel-raised); color: var(--danger); }
#Button::-og-hold { background: #e5484d59; }
#Button.primary { border-width: 0; background: var(--primary); color: var(--on-primary); }
#Button.primary::-og-hold { background: #ffffff59; }
#Button.secondary { border-width: 0; background: var(--control-strong); color: var(--ink); }
#Button.secondary::-og-hold { background: #ffffff40; }
#Button:hover { filter: brightness(1.1); }
#Button:disabled { opacity: 0.45; }
#Icon { width: 18px; height: 18px; }
#Label, #Holding { font: var(--text-body); white-space: nowrap; }
#Button:not(:active) #Holding { display: none; }
#Button:active #Label { display: none; }
#Hint { font: var(--text-caption); color: var(--ink-muted); white-space: nowrap; }
</style>
