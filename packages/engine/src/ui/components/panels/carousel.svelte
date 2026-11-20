<meta name="component" content="Carousel">
<meta name="strings" content="./carousel.en.ogt">
<meta name="prop:pages" content="list" data-doc="The pages: each { id, title, body, image }, image a texture handle (0: none).">
<meta name="prop:index" content="number" data-doc="The current page, from 0 (its dot lit).">
<meta name="prop:label" content="string" data-doc="What a screen reader calls it.">
<meta name="action:prev" content="Previous">
<meta name="action:next" content="Next">
<meta name="fixture:default" content='{"index":1,"label":"Season pass","pages":[{"id":"a","title":"Ember Knight","body":"A blade forged in the first fire.","image":0},{"id":"b","title":"Frost Warden","body":"Holds the northern gate.","image":0},{"id":"c","title":"Storm Caller","body":"Lightning answers her.","image":0}]}'>

<!-- Pages of art on the HUD plate: the current one ringed in the accent,
     its neighbours set back; the place is a row of leaning pips between
     cut arrow keys. -->
<div id="Carousel" part="frame" layout="stack" variant="bare" role="region" aria-label={label || t("component.carousel.name")} style:--index={index} style:--count={pages.length}>
  <div id="Viewport" layout="row" gap="loose">
    {#if !pages.length}<p class="empty body" part="caption" role="status">{t("component.carousel.empty")}</p>{/if}
    {#each pages as page (page.id)}
      <div part="cell" class="page" layout="stack" gap="tight">
        {#if page.image}<img part="picture" class="art" src={page.image}>
        {:else}<div part="well" class="well"></div>{/if}
        {#if page.title}<span part="title" class="title">{page.title}</span>{/if}
        {#if page.body}<span part="caption" class="body">{page.body}</span>{/if}
      </div>
    {/each}
  </div>
  <div id="Controls" layout="row" gap="loose">
    <button id="Prev" part="prev" aria-label={t("component.carousel.prev")} disabled={pages.length < 2} onclick={prev}><og-icon icon="back"></og-icon></button>
    <div id="Dots" layout="row">
      {#each pages as page (page.id)}<div part="dot" class="dot"></div>{/each}
      {#if pages.length}<div class="lit" id="Lit" part="dot" play="change" data-og-overlap="*"></div>{/if}
    </div>
    <button id="Next" part="next" aria-label={t("component.carousel.next")} disabled={pages.length < 2} onclick={next}><og-icon id="NextIcon" icon="back"></og-icon></button>
  </div>
</div>

<style>
#Carousel { inline-size: var(--measure-md); max-inline-size: 100%; align-items: center; }
#Carousel { padding: var(--space-md); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--hud-panel-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; color: var(--hud-ink); }
/* One page fills the window; the next waits beyond its edge, scrolled to. */
#Viewport { inline-size: 100%; overflow-x: auto; scrollbar-width: none; scroll-snap-type: x mandatory; }
.page { flex: none; inline-size: 100%; min-inline-size: 0; padding: 0; scroll-snap-align: center; --cell-radius: var(--hud-plate-cut); --cell-corner-shape: bevel; }
.art { object-fit: cover; }
.art, .well { inline-size: stretch; height: var(--size-3xl); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; --well-fill: var(--loading-placeholder); }
.title { font-size: var(--hud-size-sm); font-weight: var(--data-title-weight); color: var(--hud-ink); overflow-wrap: anywhere; }
.body { color: var(--hud-ink-dim); overflow-wrap: anywhere; }
.empty { margin: 0; padding: var(--space-md); }
#Controls { inline-size: 100%; align-items: center; margin-block-start: var(--space-sm); }
#Prev, #Next { flex: none; width: var(--touch-target); height: var(--touch-target); padding: 0; --button-radius: var(--hud-plate-cut); --button-corner-shape: bevel; }
#NextIcon { rotate: 180deg; }
#Dots { position: relative; flex: 1 1 auto; justify-content: center; --layout-gap: var(--space-xs); }
.dot { width: var(--space-lg); height: var(--space-xxs); transform: skewX(var(--bar-skew)); --dot-fill: var(--hud-pip-off); --dot-radius: 0px; --dot-rim-width: 0px; }
#Lit { --dot-fill: var(--hud-pip-on); --dot-radius: 0px; --dot-rim-width: 0px; position: absolute; top: 0; inset-inline-start: calc(50% - var(--count) * (var(--space-lg) + var(--space-xs)) / 2 + var(--space-xs) / 2 + var(--index) * (var(--space-lg) + var(--space-xs))); width: var(--space-lg); height: var(--space-xxs); transform: skewX(var(--bar-skew)); box-shadow: var(--hud-pip-glow); }
</style>
