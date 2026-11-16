<meta name="screen" content="film-choices" data-layer="hud" data-input="all">
<meta name="strings" content="./film-choices.en.ogt">
<meta name="parts" content="film-choices.frame film-choices.timer film-choices.choice film-choices.hotspot film-choices.qte">
<meta name="prop:open" content="bool" data-doc="The film's choice window is open or locked (og.film.view's window is set): the choices show.">
<meta name="prop:choices" content="list" data-doc="The node's choices, a keyed collection of { id, text, enabled, chosen, progress }: id is ChoiceView.index as text (choose reports it as the row's key), chosen marks WindowView.pending, progress a quick-time event's QteView.progress (-1 for a choice without one: no bar).">
<meta name="prop:hotspots" content="list" data-doc="FilmView.hotspots as a keyed collection of { id, x, y, width, height, label }: id is the choice's index as text (pick reports it), the rect in fractions of the picture, which the screen takes to fill it; label is the choice's text, for a screen reader.">
<meta name="prop:left" content="number" data-doc="The window's time left, 0 to 1 (WindowView.ticksLeft / ticksTotal).">
<meta name="prop:locked" content="bool" data-doc="Past film.lock (WindowView.locked): the choices stay, no longer taken.">
<meta name="action:choose" content="Choose">
<meta name="action:pick" content="Choose on the picture">
<meta name="fixture:default" content="{&quot;open&quot;:true,&quot;left&quot;:0.6,&quot;locked&quot;:false,&quot;hotspots&quot;:[{&quot;id&quot;:&quot;0&quot;,&quot;x&quot;:0.58,&quot;y&quot;:0.3,&quot;width&quot;:0.14,&quot;height&quot;:0.22,&quot;label&quot;:&quot;Take the knife&quot;}],&quot;choices&quot;:[{&quot;id&quot;:&quot;0&quot;,&quot;text&quot;:&quot;Take the knife&quot;,&quot;enabled&quot;:true,&quot;chosen&quot;:false,&quot;progress&quot;:-1},{&quot;id&quot;:&quot;1&quot;,&quot;text&quot;:&quot;Open the door&quot;,&quot;enabled&quot;:true,&quot;chosen&quot;:true,&quot;progress&quot;:-1},{&quot;id&quot;:&quot;2&quot;,&quot;text&quot;:&quot;Hold to dodge&quot;,&quot;enabled&quot;:true,&quot;chosen&quot;:false,&quot;progress&quot;:0.45},{&quot;id&quot;:&quot;3&quot;,&quot;text&quot;:&quot;Wait in the dark&quot;,&quot;enabled&quot;:false,&quot;chosen&quot;:false,&quot;progress&quot;:-1}]}">

<div id="FilmChoices" class="bare" hidden={!open}>
  <div id="Hotspots">
    {#each hotspots as h (h.id)}
      <button id="Hotspot" part="film-choices.hotspot" class="hotspot" variant="quiet" aria-label={h.label} disabled={locked} style:--x={h.x} style:--y={h.y} style:--w={h.width} style:--h={h.height} onclick={pick}></button>
    {/each}
  </div>
  <div id="Dock">
    <section id="Panel" part="film-choices.frame" role="group" aria-label={t("screen.film-choices.label")}>
      <progress id="Timer" part="film-choices.timer" bind:value={left}></progress>
      <div id="Choices">
        {#each choices as c (c.id)}
          <button id="Choice" part="film-choices.choice" class="choice" class:chosen={c.chosen} variant={c.chosen ? "primary" : "default"} aria-current={c.chosen} disabled={!c.enabled || locked} onclick={choose}>
            <div id="Body" class="body">
              <p id="Text">{c.text}</p>
              {#if c.progress >= 0}<progress id="Qte" part="film-choices.qte" class="qte" value={c.progress} aria-label={t("screen.film-choices.qte")}></progress>{/if}
            </div>
          </button>
        {/each}
      </div>
    </section>
  </div>
</div>

<style>
:root { position: relative; block-size: stretch; }
/* Hotspots over the picture, at the fractions the film gives them. */
#Hotspots { position: absolute; inset: 0; }
.hotspot { position: absolute; inset-inline-start: calc(var(--x) * 100%); top: calc(var(--y) * 100%); width: calc(var(--w) * 100%); height: calc(var(--h) * 100%); min-width: var(--control-lg); min-height: var(--control-lg); padding: 0; border-radius: var(--radius-card); border: var(--stroke-thick) solid var(--accent); background: color-mix(in srgb, var(--accent) 18%, transparent); box-shadow: 0 0 var(--space-md) var(--accent); }
/* The choices docked at the foot, clear of the safe area. */
#Dock { position: relative; display: flex; flex-direction: column; justify-content: end; align-items: center; block-size: 100%; padding: var(--space-xxl) calc(env(safe-area-inset-right) + var(--space-xl)) calc(env(safe-area-inset-bottom) + var(--space-xxl)) calc(env(safe-area-inset-left) + var(--space-xl)); }
[part="film-choices.frame"] { display: flex; flex-direction: column; gap: var(--space-sm); inline-size: 100%; max-inline-size: var(--measure-xxl); padding: var(--space-md); background: var(--hud); border-radius: var(--radius-card); }
#Choices { display: flex; flex-wrap: wrap; gap: var(--space-sm); }
.choice { flex: 1 1 0; min-inline-size: var(--measure-xs); min-block-size: var(--control-lg); }
.body { display: flex; flex-direction: column; align-items: center; gap: var(--space-xxs); inline-size: 100%; }
.body p { margin: 0; }
.qte { inline-size: 100%; }
.chosen { outline: var(--stroke-regular) solid var(--accent); outline-offset: var(--stroke-regular); }
@media (width < 600px), (height < 600px) {
  #Dock { padding: var(--space-lg) calc(env(safe-area-inset-right) + var(--space-md)) calc(env(safe-area-inset-bottom) + var(--space-lg)) calc(env(safe-area-inset-left) + var(--space-md)); }
  #Choices { flex-direction: column; }
  .choice { flex: none; min-inline-size: 0; }
}
</style>
