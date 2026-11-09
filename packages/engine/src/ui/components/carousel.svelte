<meta name="component" content="Carousel">
<meta name="strings" content="./carousel.en.ogt">
<meta name="prop:pages" content="list" data-doc="The pages: each { id, title, body, image }, image a texture handle (0: none).">
<meta name="prop:index" content="number" data-doc="The current page, from 0 (its dot lit).">
<meta name="prop:label" content="string" data-doc="What a screen reader calls it.">
<meta name="action:prev" content="Previous">
<meta name="action:next" content="Next">
<meta name="fixture:default" content='{"index":1,"label":"Season pass","pages":[{"id":"a","title":"Ember Knight","body":"A blade forged in the first fire.","image":0},{"id":"b","title":"Frost Warden","body":"Holds the northern gate.","image":0},{"id":"c","title":"Storm Caller","body":"Lightning answers her.","image":0}]}'>

<div id="Carousel" role="region" aria-label={label || t("component.carousel.name")} style:--index={index}>
  <div id="Viewport">
    {#each pages as page (page.id)}
      <div class="page">
        <img class="art" src={page.image} hidden={!page.image}>
        <div class="well" hidden={!!page.image}></div>
        <span class="title">{page.title}</span>
        <span class="body">{page.body}</span>
      </div>
    {/each}
  </div>
  <div id="Controls">
    <button id="Prev" aria-label={t("component.carousel.prev")} onclick={prev}><icon icon="back"></icon></button>
    <div id="Dots">
      {#each pages as page (page.id)}<div class="dot"></div>{/each}
      <div id="Lit"></div>
    </div>
    <button id="Next" aria-label={t("component.carousel.next")} onclick={next}><icon id="NextIcon" icon="back"></icon></button>
  </div>
</div>

<style>
#Carousel { width: 360px; gap: 10px; align-items: center; }
#Viewport { display: flex; width: 360px; gap: 12px; overflow-x: scroll; scroll-snap-type: x mandatory; }
.page { width: 320px; padding: 12px; gap: 6px; border-radius: 10px; background: var(--panel-raised); outline: 1px solid var(--line); outline-offset: -1px; scroll-snap-align: center; }
.art, .well { width: stretch; height: 140px; border-radius: 6px; }
.well { background: linear-gradient(160deg, #4c8dff66, #0a0d14ff); }
.title { font: var(--text-lg); color: var(--ink); }
.body { font: var(--text-sm); color: var(--ink-muted); }
#Controls { display: flex; gap: 12px; align-items: center; }
#Prev, #Next { width: 36px; height: 36px; padding: 0; border-radius: 18px; }
#NextIcon { rotate: 180deg; }
#Dots { display: flex; gap: 8px; }
.dot { width: 8px; height: 8px; border-radius: 4px; background: var(--ink-faint); }
#Lit { position: absolute; top: 0; inset-inline-start: calc(var(--index) * 16px); width: 8px; height: 8px; border-radius: 4px; background: var(--primary); transition: translate 200ms ease-out; }
</style>
