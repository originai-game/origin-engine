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

<div id="Screen" part="save-load.frame" class="bare">
  <header id="Header" part="save-load.band">
    <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
    {#if mode == "save"}
      <h1 id="SaveTitle" part="save-load.title">{t("screen.save-load.saveTitle")}</h1>
    {:else}
      <h1 id="LoadTitle" part="save-load.title">{t("screen.save-load.loadTitle")}</h1>
    {/if}
  </header>
  <div id="Body">
    <div id="Slots">
      <div id="List" focus-group="stop">
        {#each slots as s (s.id)}
          <div id="Slot" part="save-load.row" interactive="true" focusable="true" onclick={select} class:selection-off={s.id != selected} class:selection-on={s.id == selected}>
            <art id="Thumb" part="save-load.thumb" kind="saveThumb" src={s.thumb}></art>
            <div id="Text">
              {#if s.empty}
                <p id="Empty">{t("screen.save-load.empty")}</p>
              {:else}
                <p id="Name" part="save-load.name">{s.title}</p>
                <p id="Place">{s.place}</p>
                <div id="Meta">
                  <p id="Played">{t("screen.save-load.Played", { s_played: formatDuration(s.played) })}</p>
                  <p id="Date">{s.date}</p>
                </div>
              {/if}
            </div>
            {#if s.auto}<badge id="Auto" tone="info">{t("screen.save-load.auto")}</badge>{/if}
          </div>
        {/each}
      </div>
    </div>
    <aside id="Side">
      <div id="Preview">{@render preview?.()}</div>
      <div id="Actions" data-layout="cluster">
        <button id="Delete" variant="danger" onclick={delete} ondetails={delete}>{t("screen.save-load.delete")}</button>
        {#if mode == "save"}
          <button id="New" onclick={newSlot}>{t("screen.save-load.new")}</button>
          <button id="Save" variant="primary" onclick={save}>{t("screen.save-load.save")}</button>
        {:else if slots.length > 0}
          <button id="Load" variant="primary" onclick={load}>{t("screen.save-load.load")}</button>
        {/if}
      </div>
    </aside>
  </div>
</div>

<style>
:root { display: flex; flex-direction: column; gap: var(--space-lg); block-size: stretch; padding: var(--space-xxl); }
h1, p { margin: 0; }
[part="save-load.band"] { display: flex; align-items: center; gap: var(--space-md); }
[part="save-load.title"] { font-size: var(--type-xl2-size); letter-spacing: var(--type-xl2-tracking); }
/* The slots beside the preview; stacked on a portrait phone. */
#Body { flex: 1; min-block-size: 0; display: grid; grid-template-columns: 3fr 2fr; grid-template-rows: 1fr; gap: var(--space-xl); }
#Slots { overflow-y: auto; }
#List { display: flex; flex-direction: column; gap: var(--space-sm); }
[part="save-load.row"] { display: flex; align-items: center; gap: var(--space-md); padding: var(--space-md); }
[part="save-load.row"]:hover, [part="save-load.row"]:focus-visible { scale: 1.01; }
[part="save-load.row"]:active { scale: 0.99; }
/* An unselected slot dims its picture; its words keep their ink. */
.selection-off [part="save-load.thumb"] { opacity: 0.8; }
.selection-on { background: var(--row-selected-fill); }
[part="save-load.thumb"] { inline-size: var(--size-4xl); aspect-ratio: 1.78; }
#Text { flex: 1; min-inline-size: 0; display: flex; flex-direction: column; gap: var(--space-xxs); }
#Empty, [part="save-load.name"] { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
[part="save-load.name"], #Place { line-clamp: 1; text-overflow: ellipsis; }
#Place { color: var(--ink-muted); }
#Meta { display: flex; gap: var(--space-sm); }
#Meta p { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#Side { display: flex; flex-direction: column; gap: var(--space-md); }
#Actions { justify-content: end; gap: var(--space-sm); }
@media (width < 600px), (height < 600px) {
  :root { padding: var(--space-lg); gap: var(--space-md); }
  [part="save-load.title"] { font-size: var(--type-xl-size); letter-spacing: var(--type-xl-tracking); }
  [part="save-load.thumb"] { inline-size: var(--size-xxl); }
}
@media (width < 600px) and (orientation: portrait) {
  #Body { grid-template-columns: 1fr; grid-template-rows: 1fr auto; gap: var(--space-md); }
}
</style>
