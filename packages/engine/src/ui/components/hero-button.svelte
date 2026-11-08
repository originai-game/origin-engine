<meta name="component" content="HeroButton">
<meta name="prop:label" content="string" data-doc="Its words (End turn).">
<meta name="prop:tone" content="string" data-doc="primary (empty), premium, secondary or danger.">
<meta name="prop:disabled" content="bool" data-doc="Shown but inert (Waiting).">
<meta name="prop:key" content="string" data-doc="A keycap beside the words (E); empty is none.">
<meta name="action:press" content="Press">
<meta name="fixture:default" content='{"label":"End turn","tone":"","disabled":false,"key":"E"}'>

<button id="HeroButton" class:premium={tone == 'premium'} class:secondary={tone == 'secondary'} class:danger={tone == 'danger'} disabled={disabled} onclick={press}>
  <span id="Label">{label}</span>
  <kbd id="Key" hidden={!key}>{key}</kbd>
</button>

<style>
#HeroButton { min-width: 200px; min-height: 56px; padding: 12px 28px; gap: 12px; border-radius: 12px; border: 2px solid #ffffff40; background: linear-gradient(180deg, #ffffff38, #0000 45%, #00000047), var(--primary); box-shadow: 0 4px 0 #00000080, 0 8px 18px #00000066; transition: scale 120ms ease-out, translate 120ms ease-out; }
#HeroButton.premium { background: linear-gradient(180deg, #fff4c2ff, #ffc93cff 45%, #c27d0eff); }
#HeroButton.secondary { background: linear-gradient(180deg, #ffffff22, #0000 45%, #00000047), var(--control-strong); }
#HeroButton.danger { background: linear-gradient(180deg, #ffffff38, #0000 45%, #00000047), var(--danger); }
#HeroButton:hover { scale: 1.03; }
#HeroButton:active { translate: 0 3px; box-shadow: 0 1px 0 #00000080; }
#HeroButton:disabled { opacity: 0.5; filter: grayscale(1); }
#Label { font: var(--text-lg); color: var(--on-primary); text-transform: uppercase; white-space: nowrap; }
.premium #Label { color: #3a2400ff; }
.secondary #Label { color: var(--ink); }
#Key { font: var(--text-caption); }
</style>
