<meta name="component" content="SplitView">
<meta name="strings" content="./split-view.en.ogt">
<meta name="parts" content="SplitView.frame SplitView.start SplitView.divider SplitView.grip SplitView.end SplitView.well">
<meta name="prop:ratio" content="number" data-default="0.5" data-doc="The start pane's share of the room, 0 to 1; the game keeps it from the divider's drag events.">
<meta name="prop:column" content="bool" data-doc="One pane over the other; false is side by side.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the divider; empty is Resize.">
<meta name="action:resize" content="Resize">
<meta name="fixture:default" content='{"ratio":0.35,"column":false,"label":""}'>

<!-- Two panes on the HUD plate, the divider between them a faint groove
     with its grip lit in the accent. -->
<div id="SplitView" part="SplitView.frame" data-layout="row" data-gap="none" class:column={column} style:--ratio={ratio}>
  <div id="Start" part="SplitView.start SplitView.well">{@render start?.()}</div>
  <div id="Divider" part="SplitView.divider" data-layout="cover" data-gap="none" role="separator" aria-label={label || t("component.split-view.name")} aria-valuenow={ratio * 100} aria-valuemin="0" aria-valuemax="100" onpointerdown={resize} onpointermove={resize}>
    <div id="Grip" part="SplitView.grip"></div>
  </div>
  <div id="End" part="SplitView.end" data-fill>{@render end?.()}</div>
</div>

<style>
:host { inline-size: var(--measure-xl); max-inline-size: 100%; block-size: var(--measure-md); padding: 0; overflow: hidden; align-items: stretch; }
[part="SplitView.frame"].column { flex-direction: column; }
[part~="SplitView.start"] { width: calc(var(--ratio) * 100%); height: stretch; overflow: auto; }
.column [part~="SplitView.start"] { width: stretch; height: calc(var(--ratio) * 100%); }
[part="SplitView.end"] { height: stretch; overflow: auto; }
[part="SplitView.divider"] { flex: none; width: var(--space-sm); height: stretch; cursor: ew-resize; }
.column [part="SplitView.divider"] { width: stretch; height: var(--space-sm); cursor: ns-resize; }
[part="SplitView.grip"] { width: var(--stroke-regular); height: var(--size-sm); }
.column [part="SplitView.grip"] { width: var(--size-sm); height: var(--stroke-regular); }
#SplitView { --divider-fill: var(--data-divider-fill); --divider-hover-fill: var(--data-row-pressed-fill); --grip-fill: var(--data-grip); }
[part="SplitView.grip"] { box-shadow: var(--data-grip-glow); }
</style>
