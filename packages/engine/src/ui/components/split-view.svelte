<meta name="component" content="SplitView">
<meta name="strings" content="./split-view.en.ogt">
<meta name="prop:ratio" content="number" data-doc="The start pane's share of the room, 0 to 1; the game keeps it from the divider's drag events.">
<meta name="prop:column" content="bool" data-doc="One pane over the other; false is side by side.">
<meta name="prop:label" content="string" data-doc="What a screen reader calls the divider; empty is Resize.">
<meta name="action:resize" content="Resize">
<meta name="fixture:default" content='{"ratio":0.35,"column":false,"label":""}'>

<div id="SplitView" class:column={column} style:--ratio={ratio}>
  <div id="Start">{@render start?.()}</div>
  <div id="Divider" role="separator" aria-label={label || t("component.split-view.name")} aria-valuenow={ratio * 100} aria-valuemin="0" aria-valuemax="100" onpointerdown={resize} onpointermove={resize}>
    <div id="Grip"></div>
  </div>
  <div id="End">{@render end?.()}</div>
</div>

<style>
#SplitView { display: flex; width: 640px; height: 360px; border-radius: 8px; overflow: hidden; background: var(--panel); border: 1px solid var(--line); }
#SplitView.column { flex-direction: column; }
#Start { width: calc(var(--ratio) * 100%); height: stretch; background: var(--well); }
.column #Start { width: stretch; height: calc(var(--ratio) * 100%); }
#End { width: stretch; height: stretch; }
#Divider { width: 8px; height: stretch; align-items: center; justify-content: center; background: var(--line); cursor: ew-resize; }
.column #Divider { width: stretch; height: 8px; cursor: ns-resize; }
#Divider:hover { background: var(--primary); }
#Grip { width: 2px; height: 28px; border-radius: 1px; background: var(--ink-muted); }
.column #Grip { width: 28px; height: 2px; }
</style>
