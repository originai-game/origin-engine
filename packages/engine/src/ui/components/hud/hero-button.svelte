<meta name="component" content="HeroButton">
<meta name="prop:label" content="string" data-doc="Its words (End turn).">
<meta name="prop:tone" content="string" data-doc="primary (empty), premium, secondary or danger.">
<meta name="prop:disabled" content="bool" data-doc="Shown but inert (Waiting).">
<meta name="prop:key" content="string" data-doc="A keycap beside the words (E); empty is none.">
<meta name="action:press" content="Press">
<meta name="fixture:default" content='{"label":"End turn","tone":"","disabled":false,"key":"E"}'>

<button class="base" id="HeroButton" part="base" aria-label={label} variant={tone == 'secondary' || tone == 'danger' ? tone : 'primary'} class:premium={tone == 'premium'} class:secondary={tone == 'secondary'} class:danger={tone == 'danger'} disabled={disabled} onclick={press}>
  <span class="label" id="Label" part="label">{label}</span>
  {#if key}<kbd class="key" id="Key" part="key">{key}</kbd>{/if}
</button>

<style>
#HeroButton { --face: var(--primary); min-inline-size: min(100%, var(--measure-xs)); max-inline-size: 100%; min-block-size: var(--size-lg); padding: var(--space-md) var(--space-xl); gap: var(--space-md); border-radius: var(--radius-panel); border: var(--stroke-regular) solid var(--sheen); background: linear-gradient(180deg, var(--sheen), transparent 45%, var(--shadow-30)), var(--face); box-shadow: 0 var(--stroke-thick) 0 var(--shadow-45), var(--elevation-floating); transition: scale var(--motion-duration-instant) var(--motion-easing-out), translate var(--motion-duration-instant) var(--motion-easing-out); }
.base.premium { --face: var(--gold); }
.base.secondary { --face: var(--control-strong); }
.base.danger { --face: var(--danger); }
.base:hover { translate: 0 calc(var(--stroke-hairline) * -1); }
.base:active { translate: 0 var(--stroke-regular); box-shadow: 0 var(--stroke-hairline) 0 var(--shadow-45); }
/* Disabled comes last so it holds the face still under hover and press. */
.base:disabled { translate: none; box-shadow: 0 var(--stroke-thick) 0 var(--shadow-45), var(--elevation-floating); opacity: var(--state-disabled-opacity); filter: grayscale(1); }
.label, .key { font: var(--text-lg); color: var(--on-primary); text-transform: uppercase; text-align: center; }
.premium .label, .premium .key { color: var(--ink-inverse); }
.secondary .label, .secondary .key { color: var(--ink); }
.key { font: var(--text-caption); }
</style>
