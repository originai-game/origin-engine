<meta name="component" content="hud">
<meta name="prop:kmh" content="number" data-doc="The car's speed, whole km/h.">
<meta name="fixture:default" content='{"kmh":42}'>

<div id="Speed" role="status">
  <span id="Value">{$kmh}</span>
  <span id="Unit">km/h</span>
</div>

<style>
#Speed {
  position: fixed;
  right: calc(env(safe-area-inset-right) + 24px);
  bottom: calc(env(safe-area-inset-bottom) + 24px);
  width: max-content;
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
  padding: 10px 16px;
  background: var(--panel);
  border-radius: var(--radius-panel);
  -og-frame: card;
}
#Value { font: var(--text-display-sm); font-family: var(--font-mono); white-space: nowrap; }
#Unit { font: var(--text-meta); color: var(--ink-muted); }
</style>
