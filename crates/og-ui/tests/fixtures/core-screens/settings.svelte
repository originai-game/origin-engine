{#snippet tree()}
<column id="FixtureSettings">
  <text heading="true">Audio</text>
  <text>Master volume</text>
  <input type="range" id="FixtureVolume" value="0.75" min="0" max="1" step="0.05" />
  <text heading="true">Accessibility</text>
  <label>Subtitles <input type="checkbox" switch id="FixtureCaptions" checked="true"></label>
  <label>Reduce motion <input type="checkbox" switch id="FixtureMotion" checked="false"></label>
  <button id="FixtureReset" variant="quiet">Restore defaults</button>
</column>
{/snippet}
<style>
#FixtureSettings { width: stretch; gap: 16px; }
</style>
