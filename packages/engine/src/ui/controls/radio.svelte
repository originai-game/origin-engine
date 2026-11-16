<meta name="control" content="input[type=radio]">
<meta name="parts" content="option ring dot">
<meta name="prop:text" content="string" data-doc="The caption (its label's text).">
<meta name="prop:description" content="string" data-doc="The line under the caption (its label's <small>).">
<meta name="prop:side" content="string" data-doc="end when the caption is written after the ring.">

<div id="Option" part="option">
  <div id="Ring" part="ring"><div id="Dot" part="dot"></div></div>
  {#if text || description}
    <div id="Text" class="text">
      <span id="Label">{text}</span>
      {#if description}<small id="Description" class="description">{description}</small>{/if}
    </div>
  {/if}
</div>

<style>
:host { display: inline-block; cursor: pointer; }
:host(:disabled) { cursor: default; }
:host(:focus-visible) [part="option"] { outline: var(--state-focus-ring-width) solid var(--focus); outline-offset: var(--state-focus-ring-offset); }
[part="option"] {
  display: flex; align-items: center; gap: var(--space-md);
  padding: var(--space-xxs) var(--space-xs); border-radius: var(--radius-control);
  color: var(--ink);
  transition: background var(--motion-duration-fast) var(--motion-easing-out);
}
:host([description]) [part="option"] { align-items: start; }
:host(:not([side="end"])) .text { order: -1; }
:host(:hover) [part="option"] { background: var(--radio-option-hover-fill, var(--checkbox-row-hover-fill, color-mix(in srgb, var(--ink) 6%, transparent))); }
:host(:disabled) [part="option"] { opacity: var(--state-disabled-opacity); background: transparent; }
[part="ring"] {
  display: flex; align-items: center; justify-content: center; flex: none;
  inline-size: var(--radio-size); block-size: var(--radio-size);
  box-sizing: border-box;
  border: var(--stroke-hairline) solid var(--radio-ring-rim, var(--line-strong));
  border-radius: var(--radius-capsule);
  corner-shape: var(--radio-corner-shape, round);
  background: var(--radio-ring-fill, var(--well));
  box-shadow: var(--radio-ring-shadow, var(--shadow-well));
  transition: background var(--motion-duration-fast) var(--motion-easing-out),
    border-color var(--motion-duration-fast) var(--motion-easing-out),
    box-shadow var(--motion-duration-fast) var(--motion-easing-out),
    scale var(--motion-duration-fast) var(--motion-easing-out);
}
:host(:hover) [part="ring"] { border-color: var(--radio-ring-hover-rim, var(--checkbox-box-hover-rim, var(--primary))); }
:host(:active) [part="ring"] { scale: var(--state-control-press-scale); }
:host(:checked) [part="ring"] {
  border-color: var(--radio-checked-rim, var(--primary));
  background: var(--radio-checked-fill, var(--primary));
  box-shadow: var(--radio-checked-shadow, var(--shadow-sheen));
}
:host(:disabled) [part="ring"] {
  background: var(--checkbox-disabled-fill, var(--control));
  border-color: var(--checkbox-disabled-rim, var(--line));
  box-shadow: none;
}
[part="dot"] {
  inline-size: calc(var(--radio-size) * var(--radio-dot-ratio));
  block-size: calc(var(--radio-size) * var(--radio-dot-ratio));
  border-radius: var(--radius-capsule);
  corner-shape: var(--radio-corner-shape, round);
  background: var(--radio-dot-fill, var(--on-primary));
  opacity: 0; scale: 0;
  transition: opacity var(--motion-duration-instant) var(--motion-easing-exit),
    scale var(--motion-duration-instant) var(--motion-easing-exit);
}
:host(:checked) [part="dot"] {
  opacity: 1; scale: 1;
  transition: opacity var(--motion-duration-fast) var(--motion-easing-out),
    scale var(--motion-duration-spring) var(--motion-easing-spring);
}
.text { display: flex; flex-direction: column; gap: var(--space-xxs); min-inline-size: 0; }
.description { color: var(--ink-muted); font: var(--text-sm); }
</style>
