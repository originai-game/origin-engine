<meta name="control" content="input[type=checkbox]">
<meta name="parts" content="row box mark">
<meta name="prop:indeterminate" content="bool" data-doc="Neither on nor off: the mark is a dash.">
<meta name="prop:description" content="string" data-doc="The line under the caption (its label's <small>).">
<meta name="prop:side" content="string" data-doc="end when the caption is written after the box.">
<meta name="prop:text" content="string" data-doc="The caption (its label's text).">

<div id="Row" part="row">
  <div id="Box" part="box"><og-icon id="Mark" part="mark" aria-label="" icon={indeterminate ? "minus" : "tick"}></og-icon></div>
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
:host(:focus-visible) [part="row"] { outline: var(--state-focus-ring-width) solid var(--focus); outline-offset: var(--state-focus-ring-offset); }
[part="row"] {
  display: flex; align-items: center; gap: var(--space-md);
  padding: var(--space-xxs) var(--space-xs); border-radius: var(--radius-control);
  color: var(--ink);
  transition: background var(--motion-duration-fast) var(--motion-easing-out);
}
:host([description]) [part="row"] { align-items: start; }
:host(:not([side="end"])) .text { order: -1; }
:host(:hover) [part="row"] { background: var(--checkbox-row-hover-fill, color-mix(in srgb, var(--ink) 6%, transparent)); }
:host(:disabled) [part="row"] { opacity: var(--state-disabled-opacity); background: transparent; }
[part="box"] {
  display: flex; align-items: center; justify-content: center; flex: none;
  inline-size: var(--checkbox-size); block-size: var(--checkbox-size);
  box-sizing: border-box;
  border: var(--stroke-hairline) solid var(--checkbox-box-rim, var(--line-strong));
  border-radius: var(--checkbox-radius, calc(var(--checkbox-size) * var(--checkbox-radius-ratio)));
  corner-shape: var(--checkbox-corner-shape, round);
  background: var(--checkbox-box-fill, var(--well));
  box-shadow: var(--checkbox-box-shadow, var(--shadow-well));
  transition: background var(--motion-duration-fast) var(--motion-easing-out),
    border-color var(--motion-duration-fast) var(--motion-easing-out),
    box-shadow var(--motion-duration-fast) var(--motion-easing-out),
    scale var(--motion-duration-fast) var(--motion-easing-out);
}
:host(:hover) [part="box"] {
  border-color: var(--checkbox-box-hover-rim, var(--primary));
  background: var(--checkbox-box-hover-fill, var(--checkbox-box-fill, var(--well)));
}
:host(:active) [part="box"] { scale: var(--state-control-press-scale); }
:host(:checked) [part="box"], :host(:indeterminate) [part="box"] {
  border-color: var(--checkbox-checked-rim, var(--primary));
  background: var(--checkbox-checked-fill, var(--primary));
  box-shadow: var(--checkbox-checked-shadow, var(--shadow-sheen));
}
:host(:hover:checked) [part="box"], :host(:hover:indeterminate) [part="box"] { background: var(--checkbox-checked-hover-fill, var(--checkbox-checked-fill, var(--primary-hover))); }
:host(:invalid) [part="box"] { border: var(--field-border-width) solid var(--field-invalid-rim); }
:host(:disabled) [part="box"] {
  background: var(--checkbox-disabled-fill, var(--control));
  border-color: var(--checkbox-disabled-rim, var(--line));
  box-shadow: none;
}
[part="mark"] {
  inline-size: calc(var(--checkbox-size) * var(--checkbox-mark-ratio));
  block-size: calc(var(--checkbox-size) * var(--checkbox-mark-ratio));
  color: var(--checkbox-mark-ink);
  opacity: 0; scale: 0.4;
  transition: opacity var(--motion-duration-instant) var(--motion-easing-exit),
    scale var(--motion-duration-instant) var(--motion-easing-exit);
}
:host(:checked) [part="mark"], :host(:indeterminate) [part="mark"] {
  opacity: 1; scale: 1;
  transition: opacity var(--motion-duration-fast) var(--motion-easing-out),
    scale var(--motion-duration-spring) var(--motion-easing-spring);
}
.text { display: flex; flex-direction: column; gap: var(--space-xxs); min-inline-size: 0; }
.description { color: var(--ink-muted); font: var(--text-sm); }
</style>
