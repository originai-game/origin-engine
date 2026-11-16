<meta name="screen" content="film-chapters" data-layer="menu" data-focus="Body/Chapters" data-under="dim" data-input="menu" data-pause="true" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./film-chapters.en.ogt">
<meta name="parts" content="film-chapters.frame film-chapters.band film-chapters.title film-chapters.card film-chapters.thumb film-chapters.name">
<meta name="prop:chapters" content="list" data-doc="The chapters, in order (og.film.load's chapters): keyed rows of { id, title, thumb (texture: the chapter's save's poster), locked (not reached yet: no save to play from) }.">
<meta name="prop:selected" content="string" data-doc="The chapter selected now (its key).">
<meta name="action:select" content="Play the chapter">
<meta name="action:back" content="Back">
<meta name="fixture:default" content="{&quot;selected&quot;:&quot;c2&quot;,&quot;chapters&quot;:[{&quot;id&quot;:&quot;c1&quot;,&quot;title&quot;:&quot;1 · The kitchen&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:false},{&quot;id&quot;:&quot;c2&quot;,&quot;title&quot;:&quot;2 · The hall&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:false},{&quot;id&quot;:&quot;c3&quot;,&quot;title&quot;:&quot;3 · The stairs&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:true},{&quot;id&quot;:&quot;c4&quot;,&quot;title&quot;:&quot;4 · Dawn&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:true}]}">

<column id="Screen" part="film-chapters.frame" class="bare">
  <row id="Header" part="film-chapters.band">
    <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
    <text id="Title" part="film-chapters.title" heading="true">{t("screen.film-chapters.title")}</text>
  </row>
  <scroll id="Body">
    <div id="Chapters" focus-group="wrap">
      {#each chapters as c (c.id)}
        <column id="Card" part="film-chapters.card" class="card" class:locked={c.locked} class:selected={c.id == selected} interactive="true" focusable="true" onclick={select} aria-disabled={c.locked}>
          <art id="Thumb" part="film-chapters.thumb" class="thumb" kind="saveThumb" src={c.thumb}></art>
          <row id="Caption" class="caption">
            <og-icon id="Lock" class="lock" icon="lock" hidden={!(c.locked)}></og-icon>
            <text id="Name" part="film-chapters.name" class="name">{c.locked ? t("screen.film-chapters.locked") : c.title}</text>
          </row>
        </column>
      {/each}
    </div>
  </scroll>
</column>

<style>
#Screen { width: stretch; height: stretch; padding: var(--space-xxl); gap: var(--space-lg); min-height: 0; }
#Header { width: stretch; gap: var(--space-md); align-items: center; }
#Title { font-size: var(--type-xl2-size); }
#Body { width: stretch; flex: 1; min-height: 0; }
#Chapters { display: flex; flex-wrap: wrap; width: stretch; gap: var(--space-lg); }
.card { flex: none; width: var(--size-5xl); gap: var(--space-xs); padding: var(--space-xs); border-radius: var(--radius-card); background: var(--panel-raised); }
.card.selected { background: var(--row-selected-fill); outline: var(--state-focus-ring-width) solid var(--focus); outline-offset: var(--state-focus-ring-offset); }
.card.locked { opacity: var(--opacity-dim); }
.thumb { width: stretch; aspect-ratio: 1.78; border-radius: var(--radius-field); }
.caption { width: stretch; gap: var(--space-xs); align-items: center; }
.lock { width: var(--size-xs); height: var(--size-xs); }
.name { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); line-clamp: 1; text-overflow: ellipsis; }
@media (width < 600px), (height < 600px) {
#Screen { padding: var(--space-lg) calc(env(safe-area-inset-right) + var(--space-md)) calc(env(safe-area-inset-bottom) + var(--space-lg)) calc(env(safe-area-inset-left) + var(--space-md)); gap: var(--space-md); }
#Title { font-size: var(--type-xl-size); }
#Chapters { gap: var(--space-md); }
.card { width: stretch; }
}
</style>
