<meta name="component" content="SplitView">
<meta name="strings" content="./split-view.en.ogt">
<meta name="parts" content="SplitView.frame SplitView.start SplitView.divider SplitView.grip SplitView.end">
<meta name="prop:ratio" content="number" data-doc="The start pane's share of the room, 0 to 1; the game keeps it from the divider's drag events.">
<meta name="prop:column" content="bool" data-doc="One pane over the other; false is side by side.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the divider; empty is Resize.">
<meta name="action:resize" content="Resize">
<meta name="fixture:default" content='{"ratio":0.35,"column":false,"label":""}'>

<div id="SplitView" part="SplitView.frame" class:column={column} style:--ratio={ratio}>
  <div id="Start" part="SplitView.start">{@render start?.()}</div>
  <div id="Divider" part="SplitView.divider" role="separator" aria-label={label || t("component.split-view.name")} aria-valuenow={ratio * 100} aria-valuemin="0" aria-valuemax="100" onpointerdown={resize} onpointermove={resize}>
    <div id="Grip" part="SplitView.grip"></div>
  </div>
  <div id="End" part="SplitView.end">{@render end?.()}</div>
</div>

<style>
[part="SplitView.frame"] { display: flex; width: var(--measure-xl); height: var(--measure-md); padding: 0; overflow: hidden; }
[part="SplitView.frame"].column { flex-direction: column; }
[part="SplitView.start"] { width: calc(var(--ratio) * 100%); height: stretch; background: var(--well); }
.column [part="SplitView.start"] { width: stretch; height: calc(var(--ratio) * 100%); }
[part="SplitView.end"] { width: stretch; height: stretch; }
[part="SplitView.divider"] { width: var(--space-sm); height: stretch; align-items: center; justify-content: center; background: var(--line); cursor: ew-resize; }
.column [part="SplitView.divider"] { width: stretch; height: var(--space-sm); cursor: ns-resize; }
[part="SplitView.divider"]:hover { background: var(--primary); }
[part="SplitView.grip"] { width: var(--stroke-regular); height: var(--size-sm); border-radius: var(--radius-capsule); background: var(--ink-muted); }
.column [part="SplitView.grip"] { width: var(--size-sm); height: var(--stroke-regular); }
</style>
