{#snippet treePanelHierarchy()}
<column class="fixture-list">
  <text>Coastal lookout</text>
  <button variant="quiet">Lighthouse</button>
  <button>Lantern_03</button>
  <button variant="quiet">Ocean</button>
</column>
{/snippet}
{#snippet view()}
<art id="FixtureView" kind="tutorial" src="0" />
{/snippet}
{#snippet scrollInspector()}
<column id="FixtureInspector">
  <text heading="true">Transform</text>
  <text>Position: 12, 4, -8</text>
  <text>Rotation: 0, 45, 0</text>
  <text>Scale: 1, 1, 1</text>
  <text heading="true">Light</text>
  <text>Intensity</text>
  <slider value="0.8" min="0" max="1" step="0.1" />
  <toggle checked="true">Cast shadows</toggle>
</column>
{/snippet}
{#snippet cataloguePanelCatalogue()}
<column class="fixture-list">
  <button>Lantern</button>
  <button>Stone wall</button>
  <button>Coastal grass</button>
</column>
{/snippet}
<style>
.fixture-list { width: stretch; gap: 8px; }
#FixtureInspector { width: stretch; gap: 12px; }
#FixtureView { width: stretch; height: stretch; }
</style>
