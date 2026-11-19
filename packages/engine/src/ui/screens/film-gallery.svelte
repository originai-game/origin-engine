<meta name="screen" content="film-gallery" data-layer="menu" data-focus="Body/Endings" data-under="dim" data-input="menu" data-pause="true" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./film-gallery.en.ogt">
<meta name="parts" content="film-gallery.frame film-gallery.band film-gallery.title film-gallery.card film-gallery.thumb film-gallery.name film-gallery.shade film-gallery.plate film-gallery.lock film-gallery.mark">
<meta name="prop:endings" content="list" data-doc="The endings, in order (og.film.load's endings): keyed rows of { id, title, thumb (texture: the ending segment's poster), locked (not reached in any playthrough: its title stays hidden) }.">
<meta name="prop:selected" content="string" data-doc="The ending selected now (its key).">
<meta name="action:select" content="Watch the ending again">
<meta name="action:back" content="Back">
<meta name="fixture:default" content="{&quot;selected&quot;:&quot;c2&quot;,&quot;endings&quot;:[{&quot;id&quot;:&quot;c1&quot;,&quot;title&quot;:&quot;Dawn&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:false},{&quot;id&quot;:&quot;c2&quot;,&quot;title&quot;:&quot;The cellar&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:false},{&quot;id&quot;:&quot;c3&quot;,&quot;title&quot;:&quot;Alone&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:true},{&quot;id&quot;:&quot;c4&quot;,&quot;title&quot;:&quot;The fire&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:true}]}">

<div id="Screen" part="film-gallery.frame" class="bare">
  <div id="Veil" part="film-gallery.shade"></div>
  <header id="Header" part="film-gallery.band">
    <button id="Back" variant="quiet" onclick={back}><div class="back"><og-icon class="chevron" icon="chevronLeft"></og-icon><span>{t("screen.common.back")}</span></div></button>
    <h1 id="Title" part="film-gallery.title">{t("screen.film-gallery.title")}</h1>
  </header>
  <div id="Body">
    <div id="Endings" focus-group="wrap">
      {#each endings as c (c.id)}
        <article id="Card" part="film-gallery.card" class="card" class:locked={c.locked} class:selected={c.id == selected} interactive="true" focusable="true" onclick={select} aria-disabled={c.locked}>
          <div id="Plate" part="film-gallery.plate" class="plate">
            {#if c.locked}<div id="Well" part="film-gallery.lock" class="well"><og-icon id="Lock" class="lock" icon="lock"></og-icon></div>
            {:else}<art id="Thumb" part="film-gallery.thumb" class="thumb" kind="keyArt" src={c.thumb}></art>{/if}
          </div>
          <div id="Caption" class="caption">
            {#if !c.locked}<span id="Mark" part="film-gallery.mark" class="mark"></span>{/if}
            <p id="Name" part="film-gallery.name" class="name">{c.locked ? t("screen.film-gallery.locked") : c.title}</p>
          </div>
        </article>
      {/each}
    </div>
  </div>
</div>

<style>
/* Over the dimmed film: a veil; the band (Back, the title); the endings as
   cut stills with the name under the picture, so a long name wraps whole
   and nothing covers the art. The selected one is ringed and lit in the
   accent; one not reached is a hatched dark well under a lock. The grid
   fills whole columns and scrolls. */
:root { position: relative; display: flex; flex-direction: column; row-gap: var(--space-lg); block-size: stretch; overflow: hidden; padding: var(--space-xxl) var(--space-3xl) var(--space-xl); color: var(--front-ink); }
h1, p { margin: 0; }
[part="film-gallery.shade"] { position: absolute; inset: 0; background: var(--system-veil); }
[part="film-gallery.band"], #Body { position: relative; }
[part="film-gallery.band"] { display: flex; align-items: baseline; column-gap: var(--space-lg); padding: 0; border: 0; min-block-size: 0; background: none; }
#Back { --button-ink: var(--front-ink-dim); --button-quiet-ink: var(--front-ink-dim); --button-quiet-hover-ink: var(--front-ink); flex: none; }
.back { display: flex; align-items: center; column-gap: var(--space-xs); }
.back .chevron { inline-size: var(--size-xs); block-size: var(--size-xs); }
.chevron:dir(rtl) { scale: -1 1; }
[part="film-gallery.title"] { flex: 1 1 auto; min-inline-size: 0; font-size: var(--front-title-size); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); font-stretch: var(--front-stretch); letter-spacing: var(--font-tracking-xs); color: var(--front-ink); text-shadow: var(--front-shadow); overflow-wrap: break-word; }
/* The column reaches into the screen's padding by the chosen card's glow,
   so the glow is not cut at its edges. */
#Body { flex: 1; min-block-size: 0; overflow: hidden auto; margin: calc(var(--space-lg) * -1); padding: var(--space-lg); }
#Endings { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(var(--film-card-width), 100%), 1fr)); gap: var(--space-lg); align-items: start; }
[part="film-gallery.card"] { display: flex; flex-direction: column; row-gap: var(--space-sm); min-inline-size: 0; padding: 0; border: 0; border-radius: 0; background: none; box-shadow: none; }
[part="film-gallery.plate"] { position: relative; aspect-ratio: 1.78; overflow: hidden; clip-path: polygon(var(--front-cut) 0, 100% 0, 100% calc(100% - var(--front-cut)), calc(100% - var(--front-cut)) 100%, 0 100%, 0 var(--front-cut)); }
[part="film-gallery.plate"]::after { content: ""; position: absolute; inset: 0; box-shadow: var(--front-card-rim); }
.thumb, .well { position: absolute; inset: 0; inline-size: 100%; block-size: 100%; }
.thumb { object-fit: cover; border-radius: 0; filter: var(--front-art-off); }
.well { display: flex; align-items: center; justify-content: center; border-radius: 0; background: var(--film-locked-fill); box-shadow: none; }
.lock { inline-size: var(--size-sm); block-size: var(--size-sm); color: var(--front-lock-ink); }
.locked [part="film-gallery.plate"]::after { box-shadow: var(--film-locked-rim); }
.selected { filter: var(--front-card-on-glow); }
.selected [part="film-gallery.plate"]::after { box-shadow: var(--front-card-on-rim); }
.selected .thumb { filter: none; }
[part="film-gallery.card"]:hover .thumb, [part="film-gallery.card"]:focus-visible .thumb { filter: none; }
.caption { display: flex; align-items: baseline; column-gap: var(--space-sm); padding-inline: var(--space-xxs); }
[part="film-gallery.name"] { flex: 1; min-inline-size: 0; font-size: var(--film-name-size); line-height: var(--font-leading-snug); font-weight: var(--font-weight-semibold); letter-spacing: 0; text-transform: none; color: var(--front-ink-dim); overflow-wrap: break-word; }
.selected [part="film-gallery.name"] { color: var(--front-ink); }
.locked [part="film-gallery.name"] { color: var(--front-ink-faint); }
/* An ending found carries the gold diamond the story chart draws an ending as. */
[part="film-gallery.mark"] { flex: none; align-self: center; inline-size: calc(8 * var(--font-step-px)); block-size: calc(8 * var(--font-step-px)); rotate: 45deg; background: var(--hud-gold); box-shadow: var(--film-mark-glow); }
@media (width < 600px), (height < 600px) {
  :root { row-gap: var(--space-md); padding: var(--space-lg) calc(env(safe-area-inset-right) + var(--space-lg)) calc(env(safe-area-inset-bottom) + var(--space-md)) calc(env(safe-area-inset-left) + var(--space-lg)); }
  [part="film-gallery.band"] { flex-wrap: wrap; row-gap: var(--space-xs); }
  #Back { margin-inline-start: calc(var(--space-sm) * -1); }
  [part="film-gallery.title"] { flex: 1 0 100%; font-size: var(--front-title-size-compact); }
  #Endings { grid-template-columns: repeat(auto-fill, minmax(min(var(--film-card-width-compact), 100%), 1fr)); gap: var(--space-md); }
  [part="film-gallery.name"] { font-size: var(--film-name-size-compact); }
}
</style>
