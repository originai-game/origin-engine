<meta name="screen" content="save-load" data-layer="menu" data-focus="Body/Slots/List" data-under="dim" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./save-load.en.ogt">
<meta name="parts" content="save-load.frame save-load.band save-load.title save-load.row save-load.thumb save-load.name save-load.shade save-load.tag save-load.preview save-load.actions save-load.detail save-load.shot">
<meta name="prop:mode" content="string" data-doc="save or load: the title and the main action.">
<meta name="prop:slots" content="list" data-doc="The saves: { title, place, played (seconds), date (text), auto (bool), empty (bool), thumb (texture) }.">
<meta name="prop:selected" content="string" data-doc="The stable collection key selected now.">
<meta name="action:confirm" content="Select">
<meta name="action:back" content="Back">
<meta name="action:details" content="Delete">
<meta name="fixture:default" content="{&quot;mode&quot;:&quot;load&quot;,&quot;selected&quot;:&quot;item-0&quot;,&quot;slots&quot;:[{&quot;auto&quot;:true,&quot;date&quot;:&quot;Today 18:42&quot;,&quot;empty&quot;:false,&quot;id&quot;:&quot;item-0&quot;,&quot;place&quot;:&quot;Old mill&quot;,&quot;played&quot;:15120,&quot;thumb&quot;:0,&quot;title&quot;:&quot;Chapter 3&quot;},{&quot;auto&quot;:false,&quot;date&quot;:&quot;Yesterday&quot;,&quot;empty&quot;:false,&quot;id&quot;:&quot;item-1&quot;,&quot;place&quot;:&quot;Harbour&quot;,&quot;played&quot;:9800,&quot;thumb&quot;:0,&quot;title&quot;:&quot;Chapter 2&quot;},{&quot;auto&quot;:false,&quot;date&quot;:&quot;&quot;,&quot;empty&quot;:true,&quot;id&quot;:&quot;item-2&quot;,&quot;place&quot;:&quot;&quot;,&quot;played&quot;:0,&quot;thumb&quot;:0,&quot;title&quot;:&quot;&quot;}]}">

<div id="Screen" part="save-load.frame" class="bare">
  <div id="Veil" part="save-load.shade"></div>
  <header id="Header" part="save-load.band">
    <button id="Back" variant="quiet" onclick={back}><div class="back"><og-icon class="chevron" icon="chevronLeft"></og-icon><span>{t("screen.common.back")}</span></div></button>
    {#if mode == "save"}
      <h1 id="SaveTitle" part="save-load.title">{t("screen.save-load.saveTitle")}</h1>
    {:else}
      <h1 id="LoadTitle" part="save-load.title">{t("screen.save-load.loadTitle")}</h1>
    {/if}
    <hr id="Rule">
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
            {#if s.auto}<span id="Auto" part="save-load.tag">{t("screen.save-load.auto")}</span>{/if}
          </div>
        {/each}
      </div>
    </div>
    <aside id="Side" part="save-load.detail">
      {#each slots as s (s.id)}
        {#if s.id == selected}
          <art id="Shot" part="save-load.shot" kind="saveThumb" src={s.thumb}></art>
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
      <div id="Preview" part="save-load.preview">{@render preview?.()}</div>
      <div id="Actions" part="save-load.actions">
        <button id="Delete" variant="danger" class="plate" onclick={delete} ondetails={delete}>{t("screen.save-load.delete")}</button>
        {#if mode == "save"}
          <button id="New" class="plate" onclick={newSlot}>{t("screen.save-load.new")}</button>
          <button id="Save" variant="primary" class="plate lit" onclick={save}>{t("screen.save-load.save")}</button>
        {:else if slots.length > 0}
          <button id="Load" variant="primary" class="plate lit" onclick={load}>{t("screen.save-load.load")}</button>
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
:root { position: relative; display: flex; flex-direction: column; row-gap: var(--space-xl); block-size: stretch; overflow: hidden; padding: var(--space-xxl) var(--space-3xl) var(--space-xl); color: var(--front-ink); }
[part="save-load.shade"] { position: absolute; inset: 0; background: var(--system-veil); }
[part="save-load.band"], #Body { position: relative; }
h1, p { margin: 0; }
[part="save-load.band"] { display: flex; align-items: center; column-gap: var(--space-lg); }
#Back { --button-ink: var(--front-ink-dim); --button-quiet-ink: var(--front-ink-dim); --button-quiet-hover-ink: var(--front-ink); }
.back { display: flex; align-items: center; column-gap: var(--space-xs); }
.back .chevron { inline-size: var(--size-xs); block-size: var(--size-xs); }
.chevron:dir(rtl) { scale: -1 1; }
[part="save-load.title"] { flex: 0 1 auto; min-inline-size: 0; font-size: var(--front-title-size); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); font-stretch: var(--front-stretch); letter-spacing: var(--font-tracking-xs); text-shadow: var(--front-shadow); }
#Rule { flex: 1 1 0; min-inline-size: 0; block-size: var(--stroke-hairline); margin: 0; border: 0; background: var(--front-divider); }
#Body { flex: 1; min-block-size: 0; display: grid; grid-template-columns: minmax(0, 3fr) minmax(0, 2fr); grid-template-rows: minmax(0, 1fr); column-gap: var(--space-3xl); }
#Slots { min-block-size: 0; overflow: hidden auto; margin: calc(var(--space-xs) * -1); padding: var(--space-xs); }
#List { display: flex; flex-direction: column; row-gap: var(--space-sm); }
[part="save-load.row"] { position: relative; display: flex; align-items: center; column-gap: var(--space-lg); padding: var(--space-sm); border: 0; border-radius: 0; background: var(--system-row-fill); box-shadow: var(--system-row-rim); clip-path: polygon(var(--front-cut) 0, 100% 0, 100% calc(100% - var(--front-cut)), calc(100% - var(--front-cut)) 100%, 0 100%, 0 var(--front-cut)); }
[part="save-load.row"]:hover, [part="save-load.row"]:focus-visible { background: var(--system-row-on-fill); }
.selection-on { background: var(--system-row-on-fill); box-shadow: var(--system-slot-on-rim); }
[part="save-load.thumb"] { flex: none; inline-size: calc(150 * var(--font-step-px)); aspect-ratio: 1.78; block-size: auto; object-fit: cover; }
.selection-off [part="save-load.thumb"] { filter: var(--front-art-off); }
#Text { flex: 1; min-inline-size: 0; display: flex; flex-direction: column; row-gap: var(--space-xxs); padding-inline-end: var(--space-3xl); }
[part="save-load.name"] { font-size: calc(20 * var(--font-step-px)); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); }
#Empty { font-size: calc(16 * var(--font-step-px)); color: var(--front-ink-faint); text-align: center; }
[part="save-load.name"], #Place { line-clamp: 1; text-overflow: ellipsis; }
#Place { font-size: calc(14 * var(--font-step-px)); color: var(--front-ink-dim); }
#Meta { display: flex; flex-wrap: wrap; column-gap: var(--space-md); }
#Meta p { font-size: calc(13 * var(--font-step-px)); font-variant-numeric: tabular-nums; color: var(--front-ink-faint); }
[part="save-load.row"]:has(#Empty) { justify-content: center; min-block-size: calc(104 * var(--font-step-px)); background: none; box-shadow: var(--system-slot-empty-rim); }
[part="save-load.row"]:has(#Empty) [part="save-load.thumb"] { display: none; }
[part="save-load.row"]:has(#Empty) #Text { padding-inline-end: 0; }
[part="save-load.tag"] { position: absolute; inset-block-start: var(--space-sm); inset-inline-end: var(--space-md); padding: var(--space-xxs) var(--space-sm); font-size: var(--hud-tag-size); font-weight: var(--font-weight-bold); letter-spacing: var(--hud-label-tracking); text-transform: uppercase; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); }
[part="save-load.detail"] { min-block-size: 0; display: flex; flex-direction: column; row-gap: var(--space-sm); padding: var(--space-lg); overflow: hidden auto; background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(var(--front-cut) 0, 100% 0, 100% calc(100% - var(--front-cut)), calc(100% - var(--front-cut)) 100%, 0 100%, 0 var(--front-cut)); }
/* No slot chosen: no plate, only the actions at the foot. */
[part="save-load.detail"]:not(:has([part="save-load.shot"])) { background: none; box-shadow: none; }
[part="save-load.shot"] { flex: none; inline-size: 100%; aspect-ratio: 1.78; block-size: auto; object-fit: cover; margin-block-end: var(--space-xs); box-shadow: var(--system-slot-empty-rim); }
#ShotName { font-size: calc(24 * var(--font-step-px)); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); line-clamp: 2; }
#ShotEmpty { font-size: calc(18 * var(--font-step-px)); color: var(--front-ink-faint); }
#ShotMeta { display: flex; flex-wrap: wrap; column-gap: var(--space-md); row-gap: var(--space-xxs); }
#ShotMeta p { font-size: calc(14 * var(--font-step-px)); font-variant-numeric: tabular-nums; color: var(--front-ink-dim); }
#ShotPlace { flex: 1 1 100%; min-inline-size: 0; color: var(--front-ink); line-clamp: 2; }
[part="save-load.preview"] { flex: 0 1 auto; min-block-size: 0; }
/* The actions share the plate's foot; when the plate is too narrow for
   them all, the main one takes its own row under the others. */
[part="save-load.actions"] { display: flex; flex-wrap: wrap; justify-content: end; gap: var(--space-sm); margin-block-start: auto; padding-block-start: var(--space-md); }
button.plate { --button-radius: 0; --button-focus-radius: 0; --button-border-width: 0; --button-font-size: calc(16 * var(--font-step-px)); --button-tracking: var(--font-tracking-md); --button-transform: uppercase; flex: 1 1 calc(120 * var(--font-step-px)); min-inline-size: 0; }
button.plate, button.plate::part(base) { block-size: var(--control-lg); }
button.plate::part(base) { border-width: 0; clip-path: polygon(var(--front-cut) 0, 100% 0, 100% calc(100% - var(--front-cut)), calc(100% - var(--front-cut)) 100%, 0 100%, 0 var(--front-cut)); }
button.lit { --button-primary-fill: var(--front-play-fill); --button-primary-hover-fill: var(--front-play-hover-fill); --button-primary-pressed-fill: var(--front-play-pressed-fill); --button-primary-ink: var(--front-play-ink); --button-primary-hover-ink: var(--front-play-ink); --button-primary-pressed-ink: var(--front-play-ink); --button-primary-shadow: none; --button-primary-rim: transparent; --button-primary-hover-rim: transparent; }
#New { --button-fill: var(--hud-plate-fill); --button-hover-fill: var(--hud-plate-fill); --button-base-shadow: var(--system-plate-edge); --button-hover-shadow: var(--system-plate-lit-edge); --button-rim: transparent; --button-hover-rim: transparent; --button-ink: var(--front-ink); --button-hover-ink: var(--front-ink); }
@media (width < 600px), (height < 600px) {
  :root { row-gap: var(--space-md); padding: var(--space-lg) var(--space-lg) var(--space-md); }
  [part="save-load.title"] { font-size: var(--front-title-size-compact); }
  [part="save-load.thumb"] { inline-size: calc(96 * var(--font-step-px)); }
  [part="save-load.name"] { font-size: calc(17 * var(--font-step-px)); }
  #Text { padding-inline-end: 0; }
  /* The tag moves onto the thumb's corner, leaving the chapter its row. */
  [part="save-load.tag"] { inset-inline: var(--space-sm) auto; }
  #Meta p { white-space: nowrap; }
}
/* A phone held upright: the title on its own row under Back; the slots
   scroll; the actions share the foot. */
@media (width < 600px) and (orientation: portrait) {
  [part="save-load.band"] { flex-wrap: wrap; row-gap: var(--space-xs); }
  #Back { margin-inline-start: calc(var(--space-sm) * -1); }
  [part="save-load.title"] { flex: 1 0 100%; }
  #Rule { display: none; }
  #Body { grid-template-columns: minmax(0, 1fr); grid-template-rows: minmax(0, 1fr) auto; row-gap: var(--space-md); }
  [part="save-load.detail"] { padding: 0; overflow: visible; background: none; box-shadow: none; clip-path: none; }
  [part="save-load.shot"], #ShotName, #ShotEmpty, #ShotMeta, [part="save-load.preview"] { display: none; }
  [part="save-load.actions"] { padding-block-start: 0; }
  /* The plates share the foot; at a large text size the primary one
     takes its own row rather than clip a label. */
  button.plate { flex-basis: calc(100 * var(--font-step-px)); }
}
</style>
