<meta name="component" content="HeroButton">
<meta name="parts" content="HeroButton.base HeroButton.label HeroButton.key">
<meta name="prop:label" content="string" data-doc="Its words (End turn).">
<meta name="prop:tone" content="string" data-doc="primary (empty), premium, secondary or danger.">
<meta name="prop:disabled" content="bool" data-doc="Shown but inert (Waiting).">
<meta name="prop:key" content="string" data-doc="A keycap beside the words (E); empty is none.">
<meta name="action:press" content="Press">
<meta name="fixture:default" content='{"label":"End turn","tone":"","disabled":false,"key":"E"}'>

<button id="HeroButton" part="HeroButton.base" class:premium={tone == 'premium'} class:secondary={tone == 'secondary'} class:danger={tone == 'danger'} disabled={disabled} onclick={press}>
  <span id="Label" part="HeroButton.label">{label}</span>
  <kbd id="Key" part="HeroButton.key" hidden={!key}>{key}</kbd>
</button>

<style>
[part="HeroButton.base"] { --face: var(--primary); min-width: var(--measure-xs); min-height: var(--size-lg); padding: var(--space-md) var(--space-xl); gap: var(--space-md); border-radius: var(--radius-panel); border: var(--stroke-regular) solid var(--sheen); background: linear-gradient(180deg, var(--sheen), transparent 45%, var(--shadow-30)), var(--face); box-shadow: 0 var(--stroke-thick) 0 var(--shadow-45), var(--elevation-floating); transition: scale var(--motion-duration-instant) var(--motion-easing-out), translate var(--motion-duration-instant) var(--motion-easing-out); }
[part="HeroButton.base"].premium { --face: var(--gold); }
[part="HeroButton.base"].secondary { --face: var(--control-strong); }
[part="HeroButton.base"].danger { --face: var(--danger); }
[part="HeroButton.base"]:hover { scale: 1.03; }
[part="HeroButton.base"]:active { translate: 0 var(--stroke-regular); box-shadow: 0 var(--stroke-hairline) 0 var(--shadow-45); }
[part="HeroButton.base"]:disabled { opacity: var(--state-disabled-opacity); filter: grayscale(1); }
[part="HeroButton.label"] { font: var(--text-lg); color: var(--on-primary); text-transform: uppercase; white-space: nowrap; }
.premium [part="HeroButton.label"] { color: var(--ink-inverse); }
.secondary [part="HeroButton.label"] { color: var(--ink); }
[part="HeroButton.key"] { font: var(--text-caption); }
</style>
