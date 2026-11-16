<meta name="control" content="input[type=checkbox][switch]">
<meta name="parts" content="row track knob">
<meta name="prop:text" content="string" data-doc="The caption (its label's text).">
<meta name="prop:description" content="string" data-doc="The line under the caption (its label's <small>).">
<meta name="prop:side" content="string" data-doc="end when the caption is written after the switch.">

<div id="Row" part="row">
  {#if text || description}
    <div id="Text" class="text">
      <span id="Label">{text}</span>
      {#if description}<small id="Description" class="description">{description}</small>{/if}
    </div>
  {/if}
  <div id="Track" part="track">
    <div id="Knob" part="knob"><og-icon id="Tick" class="tick" icon="tick" aria-label=""></og-icon></div>
  </div>
</div>

<style>
:host { display: inline-block; cursor: pointer; }
:host(:disabled) { cursor: default; }
:host(:focus-visible) [part="track"] { outline: var(--state-focus-ring-width) solid var(--focus); outline-offset: var(--state-focus-ring-offset); }
[part="row"] {
  display: flex; align-items: center; justify-content: space-between; gap: var(--space-md);
  padding: var(--space-xxs) var(--space-xs); border-radius: var(--radius-control);
  color: var(--ink);
  transition: background var(--motion-duration-fast) var(--motion-easing-out);
}
:host([side="end"]) .text { order: 1; }
:host(:hover) [part="row"] { background: var(--switch-row-hover-fill, var(--checkbox-row-hover-fill, color-mix(in srgb, var(--ink) 6%, transparent))); }
:host(:disabled) [part="row"] { opacity: var(--state-disabled-opacity); background: transparent; }
[part="track"] {
  --switch-inset: max(2px, calc(var(--switch-height-md) * var(--switch-inset-ratio)));
  position: relative; flex: none;
  inline-size: calc(var(--switch-height-md) * var(--switch-width-ratio));
  block-size: var(--switch-height-md);
  box-sizing: border-box;
  border: var(--stroke-hairline) solid var(--switch-track-rim, transparent);
  border-radius: var(--radius-capsule);
  corner-shape: var(--switch-corner-shape, round);
  background: var(--switch-track-fill, color-mix(in srgb, var(--ink) 12%, var(--control-strong)));
  box-shadow: var(--switch-track-shadow, var(--shadow-well));
  transition: background var(--motion-duration-base) var(--motion-easing-out),
    border-color var(--motion-duration-base) var(--motion-easing-out),
    box-shadow var(--motion-duration-base) var(--motion-easing-out);
}
:host(:checked) [part="track"] {
  border-color: var(--switch-checked-rim, var(--switch-track-rim, transparent));
  background: var(--switch-checked-fill, var(--primary));
  box-shadow: var(--switch-checked-shadow, var(--shadow-sheen));
}
:host(:hover:checked) [part="track"] { background: var(--switch-checked-hover-fill, var(--switch-checked-fill, var(--primary-hover))); }
[part="knob"] {
  position: absolute; top: var(--switch-inset); inset-inline-start: var(--switch-inset);
  block-size: calc(100% - 2 * var(--switch-inset)); aspect-ratio: 1;
  display: flex; align-items: center; justify-content: center;
  box-sizing: border-box;
  border: var(--stroke-hairline) solid var(--switch-knob-rim, transparent);
  border-radius: var(--radius-capsule);
  corner-shape: var(--switch-corner-shape, round);
  background: var(--switch-knob-fill, var(--knob));
  box-shadow: var(--switch-knob-shadow, var(--shadow-raised));
  transition: inset-inline-start var(--motion-duration-spring) var(--motion-easing-spring),
    scale var(--motion-duration-fast) var(--motion-easing-out);
}
:host(:checked) [part="knob"] {
  inset-inline-start: calc(100% - var(--switch-inset) - (var(--switch-height-md) - 2 * var(--stroke-hairline) - 2 * var(--switch-inset)));
}
:host(:active) [part="knob"] { scale: 1.12 1; }
.tick {
  inline-size: 62%; block-size: 62%;
  color: var(--switch-checked-fill, var(--primary));
  opacity: 0; scale: 0.5;
  transition: opacity var(--motion-duration-fast) var(--motion-easing-out),
    scale var(--motion-duration-fast) var(--motion-easing-out);
}
:host(:checked) .tick { opacity: 1; scale: 1; }
.text { display: flex; flex-direction: column; gap: var(--space-xxs); min-inline-size: 0; }
.description { color: var(--ink-muted); font: var(--text-sm); }
</style>
