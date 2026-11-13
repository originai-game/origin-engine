<meta name="screen" content="save-load" data-layer="menu" data-focus="Body/Slots/List" data-under="dim" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./save-load.en.ogt">
<meta name="parts" content="save-load.frame save-load.band save-load.title save-load.row save-load.thumb save-load.name">
<meta name="prop:mode" content="string" data-doc="save or load: the title and the main action.">
<meta name="prop:slots" content="list" data-doc="The saves: { title, place, played (seconds), date (text), auto (bool), empty (bool), thumb (texture) }.">
<meta name="prop:selected" content="string" data-doc="The stable collection key selected now.">
<meta name="action:confirm" content="Select">
<meta name="action:back" content="Back">
<meta name="action:details" content="Delete">
<meta name="fixture:default" content="{&quot;mode&quot;:&quot;load&quot;,&quot;selected&quot;:&quot;item-0&quot;,&quot;slots&quot;:[{&quot;auto&quot;:true,&quot;date&quot;:&quot;Today 18:42&quot;,&quot;empty&quot;:false,&quot;id&quot;:&quot;item-0&quot;,&quot;place&quot;:&quot;Old mill&quot;,&quot;played&quot;:15120,&quot;thumb&quot;:0,&quot;title&quot;:&quot;Chapter 3&quot;},{&quot;auto&quot;:false,&quot;date&quot;:&quot;Yesterday&quot;,&quot;empty&quot;:false,&quot;id&quot;:&quot;item-1&quot;,&quot;place&quot;:&quot;Harbour&quot;,&quot;played&quot;:9800,&quot;thumb&quot;:0,&quot;title&quot;:&quot;Chapter 2&quot;},{&quot;auto&quot;:false,&quot;date&quot;:&quot;&quot;,&quot;empty&quot;:true,&quot;id&quot;:&quot;item-2&quot;,&quot;place&quot;:&quot;&quot;,&quot;played&quot;:0,&quot;thumb&quot;:0,&quot;title&quot;:&quot;&quot;}]}">

<column id="Screen" part="save-load.frame" class="bare">
  <row id="Header" part="save-load.band">
    <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
    <text id="SaveTitle" part="save-load.title" heading="true" hidden={!(($mode == "save"))}>{t("screen.save-load.saveTitle")}</text>
    <text id="LoadTitle" part="save-load.title" heading="true" hidden={!(($mode != "save"))}>{t("screen.save-load.loadTitle")}</text>
  </row>
  <grid id="Body">
    <scroll id="Slots">
      <column id="List" focus-group="stop">
        {#each $slots as s (s.id)}
          <row id="Slot" part="save-load.row" interactive="true" focusable="true" onclick={select} class:selection-off={!((s.id == $selected))} class:selection-on={(s.id == $selected)}>
            <art id="Thumb" part="save-load.thumb" kind="saveThumb" src={s.thumb}></art>
            <column id="Text">
              <text id="Empty" hidden={!(s.empty)}>{t("screen.save-load.empty")}</text>
              <text id="Name" part="save-load.name" hidden={!(!(s.empty))}>{s.title}</text>
              <text id="Place" hidden={!(!(s.empty))}>{s.place}</text>
              <row id="Meta" hidden={!(!(s.empty))}>
                <text id="Played">{t("screen.save-load.Played", { s_played: formatDuration(s.played) })}</text>
                <text id="Date">{s.date}</text>
              </row>
            </column>
            <badge id="Auto" tone="info" hidden={!(s.auto)}>{t("screen.save-load.auto")}</badge>
          </row>
        {/each}
      </column>
    </scroll>
    <column id="Side">
      <column id="Preview">{@render preview?.()}</column>
      <row id="Actions">
        <button id="Delete" variant="danger" onclick={delete} ondetails={delete}>{t("screen.save-load.delete")}</button>
        <button id="New" hidden={!(($mode == "save"))} onclick={newSlot}>{t("screen.save-load.new")}</button>
        <button id="Save" variant="primary" hidden={!(($mode == "save"))} onclick={save}>{t("screen.save-load.save")}</button>
        <button id="Load" variant="primary" hidden={!((($mode != "save") && ($slots.length > 0.0)))} onclick={load}>{t("screen.save-load.load")}</button>
      </row>
    </column>
  </grid>
</column>

<style>
#Screen { width: stretch; height: stretch; padding: var(--space-xxl); gap: var(--space-lg); }
@media (width < 600px), (height < 600px) {
#Screen { padding: var(--space-lg); gap: var(--space-md); }
}
#Header { width: stretch; gap: var(--space-md); align-items: center; }
#SaveTitle, #LoadTitle { font-size: var(--type-xl2-size); }
@media (width < 600px), (height < 600px) {
#SaveTitle, #LoadTitle { font-size: var(--type-xl-size); }
}
#Body { width: stretch; height: stretch; gap: var(--space-xl); grid-template-columns: 3fr 2fr; grid-template-rows: 1fr; }
@media (width < 600px) and (orientation: portrait) {
#Body { grid-template-columns: 1fr; grid-template-rows: 1fr auto; gap: var(--space-md); }
}
#Slots { width: stretch; height: stretch; }
#List { width: stretch; gap: var(--space-sm); }
#Slot { width: stretch; padding: var(--space-md); gap: var(--space-md); align-items: center; }
#Slot.selection-off { opacity: 0.8; }
#Slot.selection-on { opacity: 1; background: var(--row-selected-fill); }
#Slot { scale: 1; }
#Slot:hover { scale: 1.01; }
#Slot:active { scale: 0.99; }
#Slot:focus-visible { scale: 1.01; }
#Thumb { width: var(--size-4xl); aspect-ratio: 1.78; }
@media (width < 600px), (height < 600px) {
#Thumb { width: var(--size-xxl); }
}
#Text { width: stretch; gap: var(--space-xxs); }
#Empty { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#Name { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); line-clamp: 1; text-overflow: ellipsis; }
#Place { color: var(--ink-muted); line-clamp: 1; text-overflow: ellipsis; }
#Meta { gap: var(--space-sm); }
#Played { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#Date { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#Side { width: stretch; gap: var(--space-md); }
#Preview { width: stretch; }
#Actions { width: stretch; gap: var(--space-sm); justify-content: end; }
</style>
