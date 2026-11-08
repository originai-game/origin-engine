<meta name="component" content="styleSheet">

<column id="StyleSheet">
  <text id="Title">Style pack</text>
  <text id="Body">Text, surfaces and controls</text>
  <button id="Primary" variant="primary">Primary</button>
  <button id="Secondary">Secondary</button>
  <button id="Disabled" disabled={true}>Disabled</button>
</column>

<style>
#StyleSheet { width: stretch; height: stretch; padding: 24px; gap: 16px; background: var(--backdrop); }
#Title { font: var(--text-xl2); }
</style>
