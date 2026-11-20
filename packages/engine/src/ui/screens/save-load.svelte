<meta name="screen" content="save-load" data-layer="menu" data-focus="Body/Slots/List" data-under="dim" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./save-load.en.ogt">
<meta name="prop:mode" content="string" data-doc="save or load: the title and the main action.">
<meta name="prop:slots" content="list" data-doc="The saves: { title, place, played (seconds), date (text), auto (bool), empty (bool), thumb (texture) }.">
<meta name="prop:selected" content="string" data-doc="The stable collection key selected now.">
<meta name="action:confirm" content="Select">
<meta name="action:back" content="Back">
<meta name="action:details" content="Delete">
<meta name="fixture:default" content="{&quot;mode&quot;:&quot;load&quot;,&quot;selected&quot;:&quot;item-0&quot;,&quot;slots&quot;:[{&quot;auto&quot;:true,&quot;date&quot;:&quot;Today 18:42&quot;,&quot;empty&quot;:false,&quot;id&quot;:&quot;item-0&quot;,&quot;place&quot;:&quot;Old mill&quot;,&quot;played&quot;:15120,&quot;thumb&quot;:0,&quot;title&quot;:&quot;Chapter 3&quot;},{&quot;auto&quot;:false,&quot;date&quot;:&quot;Yesterday&quot;,&quot;empty&quot;:false,&quot;id&quot;:&quot;item-1&quot;,&quot;place&quot;:&quot;Harbour&quot;,&quot;played&quot;:9800,&quot;thumb&quot;:0,&quot;title&quot;:&quot;Chapter 2&quot;},{&quot;auto&quot;:false,&quot;date&quot;:&quot;&quot;,&quot;empty&quot;:true,&quot;id&quot;:&quot;item-2&quot;,&quot;place&quot;:&quot;&quot;,&quot;played&quot;:0,&quot;thumb&quot;:0,&quot;title&quot;:&quot;&quot;}]}">

<div id="Screen" part="frame" variant="bare">
  <div class="shade" id="Veil" part="shade"></div>
  <header class="band" id="Header" part="band">
    <button id="Back" variant="quiet" onclick={back}><div class="back"><og-icon class="chevron" icon="chevronLeft"></og-icon><span>{t("screen.common.back")}</span></div></button>
    {#if mode == "save"}
      <h1 class="title" id="SaveTitle" part="title">{t("screen.save-load.saveTitle")}</h1>
    {:else}
      <h1 class="title" id="LoadTitle" part="title">{t("screen.save-load.loadTitle")}</h1>
    {/if}
    <hr id="Rule">
  </header>
  <div id="Body">
    <div id="Slots">
      <div id="List" focus-group="stop">
        {#each slots as s (s.id)}
          <div class="row" id="Slot" part="row" interactive="true" focusable="true" onclick={select} class:selection-off={s.id != selected} class:selection-on={s.id == selected} class:empty-slot={s.empty}>
            <art class="thumb" id="Thumb" part="thumb" kind="saveThumb" src={s.thumb}></art>
            <div id="Text">
              {#if s.empty}
                <p id="Empty">{t("screen.save-load.empty")}</p>
              {:else}
                <p class="name" id="Name" part="name">{s.title}</p>
                <p id="Place">{s.place}</p>
                <div id="Meta">
                  <p id="Played">{t("screen.save-load.Played", { s_played: formatDuration(s.played) })}</p>
                  <p id="Date">{s.date}</p>
                </div>
              {/if}
            </div>
            {#if s.auto}<span class="tag" id="Auto" part="tag">{t("screen.save-load.auto")}</span>{/if}
          </div>
        {/each}
      </div>
    </div>
    <aside class="detail" id="Side" part="detail" class:blank={selected == ""}>
      {#each slots as s (s.id)}
        {#if s.id == selected}
          <art class="shot" id="Shot" part="shot" kind="saveThumb" src={s.thumb}></art>
          {#if s.empty}
            <p id="ShotEmpty">{t("screen.save-load.empty")}</p>
          {:else}
            <p id="ShotName">{s.title}</p>
            <div id="ShotMeta">
              <p id="ShotPlace">{s.place}</p>
              <p id="ShotPlayed">{t("screen.save-load.Played", { s_played: formatDuration(s.played) })}</p>
              <p id="ShotDate">{s.date}</p>
            </div>
          {/if}
        {/if}
      {/each}
      <div class="preview" id="Preview" part="preview">{@render preview?.()}</div>
      <div class="actions" id="Actions" part="actions">
        <button id="Delete" variant="danger" class="plate" onclick={delete} ondetails={delete}>{t("screen.save-load.delete")}</button>
        {#if mode == "save"}
          <button id="New" class="plate" onclick={newSlot}>{t("screen.save-load.new")}</button>
          <button id="Save" variant="primary" class="plate go" onclick={save}>{t("screen.save-load.save")}</button>
        {:else if slots.length > 0}
          <button id="Load" variant="primary" class="plate go" onclick={load}>{t("screen.save-load.load")}</button>
        {/if}
      </div>
    </aside>
  </div>
</div>

<style>
/* Over the dimmed game: a veil; the band (Back, the title, a rule); the
   slots as cut glass strips (thumb, chapter, place, time and date, an Auto
   tag), the chosen one ringed and lit; the game's preview and the actions
   beside them on a cut plate: the chosen slot's shot, chapter, place, time
   and date, the game's preview under them and the actions at the plate's
   foot; on a phone held upright only the actions, at the screen's foot. */
#Screen { position: relative; display: flex; flex-direction: column; row-gap: var(--space-xl); block-size: stretch; overflow: hidden; padding: var(--space-xxl) var(--space-3xl) var(--space-xl); color: var(--front-ink); }
.shade { position: absolute; inset: 0; background: var(--system-veil); }
.band, #Body { position: relative; }
h1, p { margin: 0; }
.band { display: flex; align-items: center; column-gap: var(--space-lg); }
#Back { --button-ink: var(--front-ink-dim); --button-quiet-ink: var(--front-ink-dim); --button-quiet-hover-ink: var(--front-ink); }
.back { display: flex; align-items: center; column-gap: var(--space-xs); }
.back .chevron { inline-size: var(--size-xs); block-size: var(--size-xs); }
.chevron:dir(rtl) { scale: -1 1; }
.title { flex: 0 1 auto; min-inline-size: 0; font-size: var(--front-title-size); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); font-stretch: var(--front-stretch); letter-spacing: var(--font-tracking-xs); text-shadow: var(--front-shadow); }
#Rule { flex: 1 1 0; min-inline-size: 0; block-size: var(--stroke-hairline); margin: 0; border: 0; background: var(--front-divider); }
#Body { flex: 1; min-block-size: 0; display: grid; grid-template-columns: minmax(0, 3fr) minmax(0, 2fr); grid-template-rows: minmax(0, 1fr); column-gap: var(--space-3xl); }
#Slots { min-block-size: 0; overflow: hidden auto; margin: calc(var(--space-xs) * -1); padding: var(--space-xs); }
#List { display: flex; flex-direction: column; row-gap: var(--space-sm); }
.row { position: relative; display: flex; align-items: center; column-gap: var(--space-lg); padding: var(--space-sm); border: 0; background: var(--system-row-fill); box-shadow: var(--system-row-rim); border-radius: var(--front-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.row:hover, .row:focus-visible { background: var(--system-row-on-fill); }
.selection-on { background: var(--system-row-on-fill); box-shadow: var(--system-slot-on-rim); }
.thumb { flex: none; inline-size: calc(150 * var(--font-step-px)); aspect-ratio: 1.78; block-size: auto; object-fit: cover; }
.selection-off .thumb { filter: var(--front-art-off); }
#Text { flex: 1; min-inline-size: 0; display: flex; flex-direction: column; row-gap: var(--space-xxs); padding-inline-end: var(--space-3xl); }
.name { font-size: calc(20 * var(--font-step-px)); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); }
#Empty { font-size: calc(16 * var(--font-step-px)); color: var(--front-ink-faint); text-align: center; }
.name, #Place { line-clamp: 1; text-overflow: ellipsis; }
#Place { font-size: calc(14 * var(--font-step-px)); color: var(--front-ink-dim); }
#Meta { display: flex; flex-wrap: wrap; column-gap: var(--space-md); }
#Meta p { font-size: calc(13 * var(--font-step-px)); font-variant-numeric: tabular-nums; color: var(--front-ink-faint); }
.row.empty-slot { justify-content: center; min-block-size: calc(104 * var(--font-step-px)); background: none; box-shadow: var(--system-slot-empty-rim); }
.row.empty-slot .thumb { display: none; }
.row.empty-slot #Text { padding-inline-end: 0; }
.tag { position: absolute; inset-block-start: var(--space-sm); inset-inline-end: var(--space-md); padding: var(--space-xxs) var(--space-sm); font-size: var(--hud-tag-size); font-weight: var(--font-weight-bold); letter-spacing: var(--hud-label-tracking); text-transform: uppercase; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); }
.detail { min-block-size: 0; display: flex; flex-direction: column; row-gap: var(--space-sm); padding: var(--space-lg); overflow: hidden auto; background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--front-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
/* No slot chosen: no plate, only the actions at the foot. */
.detail.blank { background: none; box-shadow: none; }
.shot { flex: none; inline-size: 100%; aspect-ratio: 1.78; block-size: auto; object-fit: cover; margin-block-end: var(--space-xs); box-shadow: var(--system-slot-empty-rim); }
#ShotName { font-size: calc(24 * var(--font-step-px)); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); line-clamp: 2; }
#ShotEmpty { font-size: calc(18 * var(--font-step-px)); color: var(--front-ink-faint); }
#ShotMeta { display: flex; flex-wrap: wrap; column-gap: var(--space-md); row-gap: var(--space-xxs); }
#ShotMeta p { font-size: calc(14 * var(--font-step-px)); font-variant-numeric: tabular-nums; color: var(--front-ink-dim); }
#ShotPlace { flex: 1 1 100%; min-inline-size: 0; color: var(--front-ink); line-clamp: 2; }
.preview { flex: 0 1 auto; min-block-size: 0; }
/* The actions share the plate's foot; when the plate is too narrow for
   them all, the main one takes its own row under the others. */
.actions { display: flex; flex-wrap: wrap; justify-content: end; gap: var(--space-sm); margin-block-start: auto; padding-block-start: var(--space-md); }
button.plate { --button-radius: 0; --button-focus-radius: 0; --button-border-width: 0; --button-font-size: calc(16 * var(--font-step-px)); --button-tracking: var(--font-tracking-md); --button-transform: uppercase; flex: 1 1 calc(120 * var(--font-step-px)); min-inline-size: 0; }
button.plate, button.plate { block-size: var(--control-lg); }
button.plate { border-width: 0; border-radius: var(--front-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
#New { --button-fill: var(--hud-plate-fill); --button-hover-fill: var(--hud-plate-fill); --button-base-shadow: var(--system-plate-edge); --button-hover-shadow: var(--system-plate-lit-edge); --button-rim: transparent; --button-hover-rim: transparent; --button-ink: var(--front-ink); --button-hover-ink: var(--front-ink); }
#Screen:size(compact) { row-gap: var(--space-md); padding: var(--space-lg) var(--space-lg) var(--space-md); }
.title:size(compact) { font-size: var(--front-title-size-compact); }
.thumb:size(compact) { inline-size: calc(96 * var(--font-step-px)); }
.name:size(compact) { font-size: calc(17 * var(--font-step-px)); }
#Text:size(compact) { padding-inline-end: 0; }
.tag:size(compact) { inset-inline: var(--space-sm) auto; }
#Meta p:size(compact) { white-space: nowrap; }
/* A phone held upright: the title on its own row under Back; the slots
   scroll; the actions share the foot. */
.band:size(narrow) { flex-wrap: wrap; row-gap: var(--space-xs); }
#Back:size(narrow) { margin-inline-start: calc(var(--space-sm) * -1); }
.title:size(narrow) { flex: 1 0 100%; }
#Rule:size(narrow) { display: none; }
#Body:size(narrow) { grid-template-columns: minmax(0, 1fr); grid-template-rows: minmax(0, 1fr) auto; row-gap: var(--space-md); }
.detail:size(narrow) { padding: 0; overflow: visible; background: none; box-shadow: none; clip-path: none; }
.shot:size(narrow), #ShotName:size(narrow), #ShotEmpty:size(narrow), #ShotMeta:size(narrow), .preview:size(narrow) { display: none; }
.actions:size(narrow) { padding-block-start: 0; }
button.plate:size(narrow) { flex-basis: calc(100 * var(--font-step-px)); }
</style>
