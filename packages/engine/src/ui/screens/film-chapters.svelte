<meta name="screen" content="film-chapters" data-layer="menu" data-focus="Body/Chapters" data-under="dim" data-input="menu" data-pause="true" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./film-chapters.en.ogt">
<meta name="parts" content="film-chapters.frame film-chapters.band film-chapters.title film-chapters.card film-chapters.thumb film-chapters.name">
<meta name="prop:chapters" content="list" data-doc="The chapters, in order (og.film.load's chapters): keyed rows of { id, title, thumb (texture: the chapter's save's poster), locked (not reached yet: no save to play from) }.">
<meta name="prop:selected" content="string" data-doc="The chapter selected now (its key).">
<meta name="action:select" content="Play the chapter">
<meta name="action:back" content="Back">
<meta name="fixture:default" content="{&quot;selected&quot;:&quot;c2&quot;,&quot;chapters&quot;:[{&quot;id&quot;:&quot;c1&quot;,&quot;title&quot;:&quot;1 · The kitchen&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:false},{&quot;id&quot;:&quot;c2&quot;,&quot;title&quot;:&quot;2 · The hall&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:false},{&quot;id&quot;:&quot;c3&quot;,&quot;title&quot;:&quot;3 · The stairs&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:true},{&quot;id&quot;:&quot;c4&quot;,&quot;title&quot;:&quot;4 · Dawn&quot;,&quot;thumb&quot;:0,&quot;locked&quot;:true}]}">

<div id="Screen" part="film-chapters.frame" class="bare">
  <header id="Header" part="film-chapters.band">
    <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
    <h1 id="Title" part="film-chapters.title">{t("screen.film-chapters.title")}</h1>
  </header>
  <div id="Body">
    <div id="Chapters" focus-group="wrap">
      {#each chapters as c (c.id)}
        <article id="Card" part="film-chapters.card" class="card" class:locked={c.locked} class:selected={c.id == selected} interactive="true" focusable="true" onclick={select} aria-disabled={c.locked}>
          <art id="Thumb" part="film-chapters.thumb" class="thumb" kind="saveThumb" src={c.thumb}></art>
          <div id="Caption" class="caption">
            {#if c.locked}<og-icon id="Lock" class="lock" icon="lock"></og-icon>{/if}
            <p id="Name" part="film-chapters.name" class="name">{c.locked ? t("screen.film-chapters.locked") : c.title}</p>
          </div>
        </article>
      {/each}
    </div>
  </div>
</div>

<style>
:root { display: flex; flex-direction: column; gap: var(--space-lg); block-size: stretch; padding: var(--space-xxl); }
[part="film-chapters.band"] { display: flex; align-items: center; gap: var(--space-md); }
[part="film-chapters.title"] { margin: 0; font-size: var(--type-xl2-size); letter-spacing: var(--type-xl2-tracking); }
#Body { flex: 1; min-block-size: 0; overflow-y: auto; }
#Chapters { display: flex; flex-wrap: wrap; gap: var(--space-lg); }
.card { display: flex; flex-direction: column; gap: var(--space-xs); inline-size: var(--size-5xl); padding: var(--space-xs); border-radius: var(--radius-card); background: var(--panel-raised); }
.card.selected { background: var(--row-selected-fill); outline: var(--state-focus-ring-width) solid var(--focus); outline-offset: var(--state-focus-ring-offset); }
.card.locked { opacity: var(--opacity-dim); }
.thumb { inline-size: 100%; aspect-ratio: 1.78; border-radius: var(--radius-field); }
.caption { display: flex; align-items: center; gap: var(--space-xs); }
.lock { inline-size: var(--size-xs); block-size: var(--size-xs); }
.name { flex: 1; min-inline-size: 0; margin: 0; font: var(--text-lg); letter-spacing: var(--type-lg-tracking); line-clamp: 1; text-overflow: ellipsis; }
@media (width < 600px), (height < 600px) {
  :root { padding: var(--space-lg) calc(env(safe-area-inset-right) + var(--space-md)) calc(env(safe-area-inset-bottom) + var(--space-lg)) calc(env(safe-area-inset-left) + var(--space-md)); gap: var(--space-md); }
  [part="film-chapters.title"] { font-size: var(--type-xl-size); letter-spacing: var(--type-xl-tracking); }
  #Chapters { gap: var(--space-md); }
  .card { flex: 1 1 var(--size-3xl); inline-size: auto; min-inline-size: 0; }
}
</style>
