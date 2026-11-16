<meta name="screen" content="film-gallery" data-layer="menu" data-focus="Body/Endings" data-under="dim" data-input="menu" data-pause="true" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./film-gallery.en.ogt">
<meta name="parts" content="film-gallery.frame film-gallery.band film-gallery.title film-gallery.card film-gallery.thumb film-gallery.name">
<meta name="prop:endings" content="list" data-doc="The endings, in order (og.film.load's endings): keyed rows of { id, title, thumb (texture: the ending segment's poster), locked (not reached in any playthrough: its title stays hidden) }.">
<meta name="prop:selected" content="string" data-doc="The ending selected now (its key).">
<meta name="action:select" content="Watch the ending again">
<meta name="action:back" content="Back">
<meta name="fixture:default" content="{&quot;selected&quot;:&quot;c2&quot;,&quot;endings&quot;:[{&quot;id&quot;:&quot;c1&quot;,&quot;title&quot;:&quot;Dawn&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:false},{&quot;id&quot;:&quot;c2&quot;,&quot;title&quot;:&quot;The cellar&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:false},{&quot;id&quot;:&quot;c3&quot;,&quot;title&quot;:&quot;Alone&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:true},{&quot;id&quot;:&quot;c4&quot;,&quot;title&quot;:&quot;The fire&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:true}]}">

<column id="Screen" part="film-gallery.frame" class="bare">
  <row id="Header" part="film-gallery.band">
    <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
    <text id="Title" part="film-gallery.title" heading="true">{t("screen.film-gallery.title")}</text>
  </row>
  <scroll id="Body">
    <div id="Endings" focus-group="wrap">
      {#each endings as c (c.id)}
        <column id="Card" part="film-gallery.card" class="card" class:locked={c.locked} class:selected={c.id == selected} interactive="true" focusable="true" onclick={select} aria-disabled={c.locked}>
          <art id="Thumb" part="film-gallery.thumb" class="thumb" kind="keyArt" src={c.thumb}></art>
          <row id="Caption" class="caption">
            <og-icon id="Lock" class="lock" icon="lock" hidden={!(c.locked)}></og-icon>
            <text id="Name" part="film-gallery.name" class="name">{c.locked ? t("screen.film-gallery.locked") : c.title}</text>
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
#Endings { display: flex; flex-wrap: wrap; width: stretch; gap: var(--space-lg); }
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
#Endings { gap: var(--space-md); }
.card { flex: 1 1 var(--size-3xl); width: auto; min-width: 0; }
}
</style>
