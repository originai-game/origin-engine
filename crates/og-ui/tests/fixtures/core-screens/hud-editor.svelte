{#snippet editorEditor()}
<panel id="FixtureHealth">
  <text>Health · 75 / 100</text>
  <progress value="0.75" />
</panel>
<panel id="FixtureHud">
  <text heading="true">HUD layout</text>
  <text>Selected: health bar</text>
  <text>Scale</text>
  <slider value="1" min="0.5" max="1.5" step="0.1" />
  <toggle checked="true">Snap to safe area</toggle>
  <row id="FixtureActions">
    <button variant="quiet">Reset layout</button>
    <button variant="primary">Done</button>
  </row>
</panel>
{/snippet}
<style>
#FixtureHealth { position: fixed; inset-inline-start: 24px; top: 160px; width: 240px; padding: 12px; gap: 8px; background: var(--panel); }
#FixtureHud { width: stretch; padding: 16px; gap: 12px; background: var(--panel); }
#FixtureActions { gap: 8px; }
</style>
