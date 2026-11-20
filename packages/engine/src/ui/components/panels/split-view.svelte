<meta name="component" content="SplitView">
<meta name="strings" content="./split-view.en.ogt">
<meta name="prop:ratio" content="number" data-default="0.5" data-doc="The start pane's share of the room, 0 to 1; the game keeps it from the divider's drag events.">
<meta name="prop:column" content="bool" data-doc="One pane over the other; false is side by side.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the divider; empty is Resize.">
<meta name="action:resize" content="Resize">
<meta name="fixture:default" content='{"ratio":0.35,"column":false,"label":""}'>

<!-- Two panes on the HUD plate, the divider between them a faint groove
     with its grip lit in the accent. -->
<div class="frame" id="SplitView" part="frame" layout="row" gap="none" class:column={column} style:--ratio={ratio}>
  <div class="start" id="Start" part="well">{@render start?.()}</div>
  <div class="divider cover" id="Divider" part="divider" layout="stack" fill gap="none" role="separator" aria-label={label || t("component.split-view.name")} aria-valuenow={ratio * 100} aria-valuemin="0" aria-valuemax="100" onpointerdown={resize} onpointermove={resize}>
    <div class="grip" id="Grip" part="grip"></div>
  </div>
  <div class="end" id="End" part="end" fill>{@render end?.()}</div>
</div>

<style>
.cover { justify-content: center; align-items: center; }
#SplitView { inline-size: var(--measure-xl); max-inline-size: 100%; block-size: var(--measure-md); padding: 0; overflow: hidden; align-items: stretch; }
.frame.column { flex-direction: column; }
.start { width: calc(var(--ratio) * 100%); height: stretch; overflow: auto; }
.column .start { width: stretch; height: calc(var(--ratio) * 100%); }
.end { height: stretch; overflow: auto; }
.divider { flex: none; width: var(--space-sm); height: stretch; cursor: ew-resize; }
.column .divider { width: stretch; height: var(--space-sm); cursor: ns-resize; }
.grip { width: var(--stroke-regular); height: var(--size-sm); }
.column .grip { width: var(--size-sm); height: var(--stroke-regular); }
#SplitView { --divider-fill: var(--data-divider-fill); --divider-hover-fill: var(--data-row-pressed-fill); --grip-fill: var(--data-grip); }
.grip { box-shadow: var(--data-grip-glow); }
</style>
