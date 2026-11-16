<meta name="screen" content="film-gallery" data-layer="menu" data-focus="Body/Endings" data-under="dim" data-input="menu" data-pause="true" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./film-gallery.en.ogt">
<meta name="parts" content="film-gallery.frame film-gallery.band film-gallery.title film-gallery.card film-gallery.thumb film-gallery.name">
<meta name="prop:endings" content="list" data-doc="The endings, in order (og.film.load's endings): keyed rows of { id, title, thumb (texture: the ending segment's poster), locked (not reached in any playthrough: its title stays hidden) }.">
<meta name="prop:selected" content="string" data-doc="The ending selected now (its key).">
<meta name="action:select" content="Watch the ending again">
<meta name="action:back" content="Back">
<meta name="fixture:default" content="{&quot;selected&quot;:&quot;c2&quot;,&quot;endings&quot;:[{&quot;id&quot;:&quot;c1&quot;,&quot;title&quot;:&quot;Dawn&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:false},{&quot;id&quot;:&quot;c2&quot;,&quot;title&quot;:&quot;The cellar&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:false},{&quot;id&quot;:&quot;c3&quot;,&quot;title&quot;:&quot;Alone&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:true},{&quot;id&quot;:&quot;c4&quot;,&quot;title&quot;:&quot;The fire&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:true}]}">

<div id="Screen" part="film-gallery.frame" class="bare">
  <header id="Header" part="film-gallery.band">
    <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
    <h1 id="Title" part="film-gallery.title">{t("screen.film-gallery.title")}</h1>
  </header>
  <div id="Body">
    <div id="Endings" focus-group="wrap">
      {#each endings as c (c.id)}
        <article id="Card" part="film-gallery.card" class="card" class:locked={c.locked} class:selected={c.id == selected} interactive="true" focusable="true" onclick={select} aria-disabled={c.locked}>
          <art id="Thumb" part="film-gallery.thumb" class="thumb" kind="keyArt" src={c.thumb}></art>
          <div id="Caption" class="caption">
            {#if c.locked}<og-icon id="Lock" class="lock" icon="lock"></og-icon>{/if}
            <p id="Name" part="film-gallery.name" class="name">{c.locked ? t("screen.film-gallery.locked") : c.title}</p>
          </div>
        </article>
      {/each}
    </div>
  </div>
</div>

<style>
:root { display: flex; flex-direction: column; gap: var(--space-lg); block-size: stretch; padding: var(--space-xxl); }
[part="film-gallery.band"] { display: flex; align-items: center; gap: var(--space-md); }
[part="film-gallery.title"] { margin: 0; font-size: var(--type-xl2-size); letter-spacing: var(--type-xl2-tracking); }
#Body { flex: 1; min-block-size: 0; overflow-y: auto; }
#Endings { display: flex; flex-wrap: wrap; gap: var(--space-lg); }
.card { display: flex; flex-direction: column; gap: var(--space-xs); inline-size: var(--size-5xl); padding: var(--space-xs); border-radius: var(--radius-card); background: var(--panel-raised); }
.card.selected { background: var(--row-selected-fill); outline: var(--state-focus-ring-width) solid var(--focus); outline-offset: var(--state-focus-ring-offset); }
.card.locked { opacity: var(--opacity-dim); }
.thumb { inline-size: 100%; aspect-ratio: 1.78; border-radius: var(--radius-field); }
.caption { display: flex; align-items: center; gap: var(--space-xs); }
.lock { inline-size: var(--size-xs); block-size: var(--size-xs); }
.name { flex: 1; min-inline-size: 0; margin: 0; font: var(--text-lg); letter-spacing: var(--type-lg-tracking); line-clamp: 1; text-overflow: ellipsis; }
@media (width < 600px), (height < 600px) {
  :root { padding: var(--space-lg) calc(env(safe-area-inset-right) + var(--space-md)) calc(env(safe-area-inset-bottom) + var(--space-lg)) calc(env(safe-area-inset-left) + var(--space-md)); gap: var(--space-md); }
  [part="film-gallery.title"] { font-size: var(--type-xl-size); letter-spacing: var(--type-xl-tracking); }
  #Endings { gap: var(--space-md); }
  .card { flex: 1 1 var(--size-3xl); inline-size: auto; min-inline-size: 0; }
}
</style>
