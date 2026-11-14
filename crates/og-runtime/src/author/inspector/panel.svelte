<meta name="component" content="inspector">
<meta name="parts" content="Inspector.frame Inspector.title">
<meta name="prop:key" content="string" data-doc="The toggle key as the toolbar says it (F1, Ctrl+F1).">
<meta name="prop:keyTip" content="string" data-doc="What the toggle key button's tooltip says.">
<meta name="prop:dirty" content="number" data-doc="Fields edited since the last save.">
<meta name="prop:paused" content="bool" data-doc="The game's clock is paused.">
<meta name="prop:picking" content="bool" data-doc="The next press in the game picks what is under it.">
<meta name="prop:canUndo" content="bool">
<meta name="prop:canRedo" content="bool">
<meta name="prop:tab" content="string" data-default="entities" data-doc="The view shown: entities, resources or ui.">
<meta name="prop:filter" content="string" data-doc="The filter's text (written back).">
<meta name="prop:hint" content="string" data-doc="The filter's placeholder for the view shown.">
<meta name="prop:tree" content="list" data-doc="The hierarchy's rows, keyed by the entity's bits: { id, label, depth, fold (open, closed or leaf), selected, named, children }.">
<meta name="prop:note" content="string" data-doc="A line under the hierarchy or the detail when it shows nothing (empty: none).">
<meta name="prop:head" content="object" data-doc="The selected entity: { id, raw } (an empty id: none).">
<meta name="prop:prefab" content="object" data-doc="Its prefab: { what, none } (an empty what: none).">
<meta name="prop:overrides" content="list" data-doc="The prefab instance's overrides: { id (the field), text }.">
<meta name="prop:sections" content="list" data-doc="The components or resources shown: { id (C: or R: and the name), name, open, fields }, each field { id (its set target), name, mark, tone, doc, editor, parts, min, max, step, value, on, options, choice, text, swatch, flash }.">
<meta name="prop:bounds" content="bool" data-doc="The UI view outlines every element of the game's UI.">
<meta name="prop:lines" content="list" data-doc="The UI view's lines about the element picked: { id, text, kind (head, line or muted) }.">
<meta name="prop:boxes" content="list" data-doc="The boxes drawn over the game for the UI view, in CSS px: { id, kind (bound, margin, border, padding or content), x, y, w, h, t, r, b, l (a ring's widths) }.">
<meta name="prop:mode" content="string" data-default="translate" data-doc="The gizmo's mode: translate, rotate or scale.">
<meta name="prop:space" content="string" data-default="World" data-doc="The gizmo's axes as the toggle says them: World or Local.">
<meta name="prop:clock" content="string" data-doc="The footer's tick.">
<meta name="prop:status" content="string" data-doc="The footer's last message.">

<div id="Screen">
  <div id="Inspector" part="Inspector.frame">
    <div id="Bar" class="line">
      <p id="Title" part="Inspector.title">Inspector</p>
      <button id="Key" class="tool" variant="quiet" title={keyTip}>{key}</button>
      <span class="gap"></span>
      <button id="Save" class="tool" variant={dirty > 0 ? "primary" : "secondary"} disabled={dirty == 0} title="Write the edited fields into the scenes' text (save_scene)">{dirty > 0 ? "Save (" + dirty + ")" : "Save"}</button>
      <button id="Close" class="tool" variant="quiet">Close</button>
    </div>
    <div id="Tools" class="line">
      <button id="Clock" class="tool">{paused ? "Play" : "Pause"}</button>
      <button id="Step" class="tool" disabled={!paused} title="Advance one tick while paused">Step</button>
      <button id="Pick" class="tool" variant={picking ? "primary" : "secondary"} title="Select what is under the next click in the game">{picking ? "Picking…" : "Pick"}</button>
      <span class="gap"></span>
      <button id="Undo" class="tool" disabled={!canUndo} title="Undo the last edit (Ctrl+Z)">Undo</button>
      <button id="Redo" class="tool" disabled={!canRedo} title="Redo (Ctrl+Shift+Z)">Redo</button>
    </div>
    <tabs id="View" items="Entities Resources UI" values="entities resources ui" current={tab} fill="true" aria-label="Inspector view"></tabs>
    <input id="Filter" placeholder={hint} value={filter}>
    {#if tab == "entities"}
      <list id="Tree" aria-label="Entities">
        {#each tree as r (r.id)}
          <div id="Entry" class="entry" style:--depth={r.depth}>
            {#if r.fold != "leaf"}
              <button id="Fold" class="fold" variant="quiet" aria-label={r.label}>
                <svg class="chevron" viewBox="0 0 14 14">{#if r.fold == "open"}<path d="M3.4 5.3 L7 9 L10.6 5.3"></path>{:else}<path d="M5.3 3.4 L9 7 L5.3 10.6"></path>{/if}</svg>
              </button>
            {:else}
              <span class="fold"></span>
            {/if}
            <button id="Label" class="name" class:unnamed={!r.named} variant={r.selected ? "primary" : "quiet"} aria-current={r.selected}>{r.label}</button>
            <span class="count" hidden={r.children == 0}>{r.children}</span>
          </div>
        {/each}
      </list>
      <hr id="Split">
    {/if}
    {#if tab == "ui"}
      <label id="Bounds" class="check"><input type="checkbox" checked={bounds}>Layout bounds</label>
    {/if}
    <div id="Detail">
      <p id="Note" class="muted" hidden={note == ""}>{note}</p>
      {#if tab == "entities"}
        <div id="Head" class="line" hidden={head.id == ""}>
          <p id="Id" class="grow">{head.id}</p>
          <p id="Raw" class="faint">{head.raw}</p>
        </div>
        <div id="Prefab" class="line" hidden={prefab.what == ""}>
          <p id="Of" class="grow prefab">{prefab.what}</p>
          <button id="ApplyAll" class="tool" disabled={prefab.none} title="Write every override into the prefab's file: every instance takes it">Apply all</button>
          <button id="RevertAll" class="tool" variant="quiet" disabled={prefab.none} title="Take every override back to the prefab's value">Revert all</button>
        </div>
        {#each overrides as o (o.id)}
          <div id="Override" class="line indent">
            <p id="Target" class="grow muted">{o.text}</p>
            <button id="Apply" class="tool" variant="quiet" title="Write this value into the prefab">Apply</button>
            <button id="Revert" class="tool" variant="quiet" title="Back to the prefab's value">Revert</button>
          </div>
        {/each}
      {/if}
      {#each sections as s (s.id)}
        <div id="Group" class="group">
          <button id="Section" class="section" variant="quiet">
            <svg class="chevron" viewBox="0 0 14 14">{#if s.open}<path d="M3.4 5.3 L7 9 L10.6 5.3"></path>{:else}<path d="M5.3 3.4 L9 7 L5.3 10.6"></path>{/if}</svg>
            <span class="grow">{s.name}</span>
          </button>
          {#each s.fields as f (f.id)}
            <div id="Field" class="field">
              <div id="Row" class="line">
                <span id="Mark" class="mark" class:dirty={f.mark}></span>
                <span id="FieldName" class="label" class:flash={f.tone == "flash"} class:override={f.tone == "override"} title={f.doc}>{f.name}</span>
                {#if f.editor == "slider"}
                  <input type="range" id="Slide" class="grow" aria-label={f.name} min={f.min} max={f.max} step={f.step} value={f.value} show-value="false">
                {/if}
                {#if f.editor == "check"}
                  <label class="check grow"><input type="checkbox" id="Check" checked={f.on}>{f.name}</label>
                {/if}
                {#if f.editor == "pick"}
                  <select id="Choice" class="grow" aria-label={f.name} items={f.options} values={f.options} current={f.choice}></select>
                {/if}
                {#if f.editor == "link"}
                  <button id="Link" class="tool grow" variant="quiet" title="Select this entity">{f.text}</button>
                {/if}
                {#if f.editor == "value"}
                  <p id="Value" class="grow faint">{f.text}</p>
                {/if}
                {#if f.swatch != ""}
                  <span id="Swatch" class="swatch" style:--swatch={f.swatch}></span>
                {/if}
                {#each f.parts as p (p.id)}
                  <input id="Part" class:narrow={f.editor == "slider"} placeholder={p.hint} value={p.text}>
                {/each}
              </div>
              <p id="Changed" class="changed" hidden={!f.flash}>changed by the game: a system writes it every tick</p>
            </div>
          {/each}
        </div>
      {/each}
      {#each lines as l (l.id)}
        <p id="Line" class="ui-line" class:head={l.kind == "head"} class:muted={l.kind == "muted"}>{l.text}</p>
      {/each}
    </div>
    {#if tab == "entities"}
      <div id="Gizmo" class="line">
        <button id="Move" class="tool" variant={mode == "translate" ? "primary" : "secondary"} title="Move along an axis or a plane (W); Ctrl snaps to 0.5">Move</button>
        <button id="Rotate" class="tool" variant={mode == "rotate" ? "primary" : "secondary"} title="Turn about an axis or the view (E); Ctrl snaps to 15°">Rotate</button>
        <button id="Scale" class="tool" variant={mode == "scale" ? "primary" : "secondary"} title="Scale along the entity's own axes, or all from the centre (R); Ctrl snaps to 0.1">Scale</button>
        <span class="gap"></span>
        <button id="Space" class="tool" variant="quiet" title="Move and rotate along the world's axes or the entity's own">{space}</button>
      </div>
    {/if}
    <div id="Status" class="line">
      <p id="Tick" class="faint">{clock}</p>
      <p id="Message" class="grow muted">{status}</p>
    </div>
  </div>
  <div id="Boxes">
    {#each boxes as b (b.id)}
      <div id="Box" class="box" class:bound={b.kind == "bound"} class:margin={b.kind == "margin"} class:border={b.kind == "border"} class:padding={b.kind == "padding"} class:content={b.kind == "content"} style:left={b.x + "px"} style:top={b.y + "px"} style:width={b.w + "px"} style:height={b.h + "px"} style:border-top-width={b.t + "px"} style:border-right-width={b.r + "px"} style:border-bottom-width={b.b + "px"} style:border-left-width={b.l + "px"}></div>
    {/each}
  </div>
</div>

<style>
#Screen { width: stretch; height: stretch; display: flex; flex-direction: row; justify-content: end; padding: var(--space-md); }
#Inspector { width: var(--measure-md); height: stretch; display: flex; flex-direction: column; gap: var(--space-sm); padding: var(--space-md); }
#Title { font: var(--text-base); color: var(--ink); white-space: nowrap; }
.line { display: flex; flex-direction: row; align-items: center; gap: var(--space-xs); width: stretch; }
.gap, .grow { flex-grow: 1; width: stretch; }
.tool { height: var(--control-sm); min-height: var(--control-sm); font: var(--text-sm); }
p, .label, .count { margin: 0; font: var(--text-sm); white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.muted { color: var(--ink-muted); }
.faint { color: var(--ink-faint); font: var(--text-caption); }
.prefab { color: var(--info); font: var(--text-caption); }
.indent { padding-inline-start: var(--space-md); }
#Filter { width: stretch; }
#Tree { width: stretch; height: var(--size-6xl); }
.entry { display: flex; flex-direction: row; align-items: center; gap: var(--space-xxs); width: stretch; padding-inline-start: calc(var(--depth) * var(--space-md)); }
.fold { width: var(--size-sm); height: var(--size-sm); min-height: var(--size-sm); padding: 0; flex-shrink: 0; }
.name { flex-grow: 1; width: stretch; height: var(--control-sm); min-height: var(--control-sm); text-align: start; font: var(--text-sm); }
.name.unnamed { color: var(--ink-muted); }
.count { color: var(--ink-faint); font: var(--text-caption); }
.chevron { width: var(--size-xs); height: var(--size-xs); fill: none; stroke: var(--ink-muted); stroke-width: 1.75; stroke-linecap: round; stroke-linejoin: round; }
#Split { width: stretch; }
#Detail { width: stretch; height: stretch; flex-grow: 1; display: flex; flex-direction: column; gap: var(--space-xs); overflow-y: auto; padding-inline-end: var(--space-md); }
.group { display: flex; flex-direction: column; gap: var(--space-xs); width: stretch; }
.section { width: stretch; height: var(--control-sm); min-height: var(--control-sm); justify-content: start; text-align: start; gap: var(--space-xs); background: var(--well); font: var(--text-sm); }
.field { display: flex; flex-direction: column; width: stretch; }
.mark { width: var(--space-sm); height: var(--space-sm); flex-shrink: 0; border-radius: var(--radius-capsule); }
.mark.dirty { background: var(--ink); }
.label { width: var(--size-xxl); flex-shrink: 0; color: var(--ink-muted); font: var(--text-caption); }
.label.override { color: var(--info); }
.label.flash { color: var(--warn); }
#Part { width: stretch; flex-grow: 1; min-width: 0; height: var(--control-sm); font: var(--text-sm); }
#Part.narrow { width: var(--size-xl); flex-grow: 0; }
.swatch { width: var(--size-sm); height: var(--size-sm); flex-shrink: 0; border-radius: var(--radius-control); background: var(--swatch); }
.check { font: var(--text-sm); }
.changed { color: var(--warn); font: var(--text-caption); white-space: normal; padding-inline-start: calc(var(--space-sm) + var(--size-xxl) + var(--space-sm)); }
.ui-line { font: var(--text-caption); color: var(--ink); white-space: normal; }
.ui-line.head { font: var(--text-sm); }
.ui-line.muted { color: var(--ink-muted); }
#Link { text-align: start; }
#Boxes { position: absolute; inset: 0; pointer-events: none; }
.box { position: absolute; box-sizing: border-box; border-style: solid; border-color: transparent; }
.box.bound { border-color: var(--danger); opacity: 0.9; }
.box.margin { border-color: var(--warn); opacity: 0.45; }
.box.border { border-color: var(--gold); opacity: 0.55; }
.box.padding { border-color: var(--success); opacity: 0.6; }
.box.content { background: var(--info); opacity: 0.45; }
</style>
