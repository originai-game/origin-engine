<meta name="component" content="Carousel">
<meta name="strings" content="./carousel.en.ogt">
<meta name="parts" content="Carousel.frame Carousel.cell Carousel.art Carousel.well Carousel.title Carousel.body Carousel.prev Carousel.next Carousel.dot Carousel.lit">
<meta name="prop:pages" content="list" data-doc="The pages: each { id, title, body, image }, image a texture handle (0: none).">
<meta name="prop:index" content="number" data-doc="The current page, from 0 (its dot lit).">
<meta name="prop:label" content="string" data-doc="What a screen reader calls it.">
<meta name="action:prev" content="Previous">
<meta name="action:next" content="Next">
<meta name="fixture:default" content='{"index":1,"label":"Season pass","pages":[{"id":"a","title":"Ember Knight","body":"A blade forged in the first fire.","image":0},{"id":"b","title":"Frost Warden","body":"Holds the northern gate.","image":0},{"id":"c","title":"Storm Caller","body":"Lightning answers her.","image":0}]}'>

<div id="Carousel" part="Carousel.frame" class="bare" role="region" aria-label={label || t("component.carousel.name")} style:--index={index}>
  <div id="Viewport">
    {#each pages as page (page.id)}
      <div part="Carousel.cell" class="page">
        <img part="Carousel.art" class="art" src={page.image} hidden={!page.image}>
        <div part="Carousel.well" class="well" hidden={!!page.image}></div>
        <span part="Carousel.title" class="title">{page.title}</span>
        <span part="Carousel.body" class="body">{page.body}</span>
      </div>
    {/each}
  </div>
  <div id="Controls">
    <button id="Prev" part="Carousel.prev" aria-label={t("component.carousel.prev")} onclick={prev}><og-icon icon="back"></og-icon></button>
    <div id="Dots">
      {#each pages as page (page.id)}<div part="Carousel.dot" class="dot"></div>{/each}
      <div id="Lit" part="Carousel.lit"></div>
    </div>
    <button id="Next" part="Carousel.next" aria-label={t("component.carousel.next")} onclick={next}><og-icon id="NextIcon" icon="back"></og-icon></button>
  </div>
</div>

<style>
#Carousel { width: var(--measure-md); gap: var(--space-sm); align-items: center; }
#Viewport { display: flex; width: var(--measure-md); gap: var(--space-md); overflow-x: scroll; scroll-snap-type: x mandatory; }
.page { width: var(--measure-sm); padding: var(--space-md); gap: var(--space-xs); scroll-snap-align: center; }
.art, .well { width: stretch; height: var(--size-3xl); }
.art { border-radius: var(--well-radius); }
.body { font: var(--text-sm); color: var(--ink-muted); }
#Controls { display: flex; gap: var(--space-md); align-items: center; }
#Prev, #Next { width: var(--control-lg); height: var(--control-lg); padding: 0; border-radius: var(--radius-capsule); }
#NextIcon { rotate: 180deg; }
#Dots { display: flex; gap: var(--space-sm); }
.dot { width: var(--space-sm); height: var(--space-sm); border-radius: var(--radius-capsule); background: var(--ink-faint); }
#Lit { position: absolute; top: 0; inset-inline-start: calc(var(--index) * var(--space-sm) * 2); width: var(--space-sm); height: var(--space-sm); border-radius: var(--radius-capsule); background: var(--primary); transition-property: translate; transition-duration: var(--motion-duration-fast); transition-timing-function: var(--motion-easing-out); }
</style>
