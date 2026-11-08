{#snippet tree()}
<column id="FixtureSettings">
  <text heading="true">Audio</text>
  <text>Master volume</text>
  <slider id="FixtureVolume" value="0.75" min="0" max="1" step="0.05" />
  <text heading="true">Accessibility</text>
  <toggle id="FixtureCaptions" checked="true">Subtitles</toggle>
  <toggle id="FixtureMotion" checked="false">Reduce motion</toggle>
  <button id="FixtureReset" variant="quiet">Restore defaults</button>
</column>
{/snippet}
<style>
#FixtureSettings { width: stretch; gap: 16px; }
</style>
