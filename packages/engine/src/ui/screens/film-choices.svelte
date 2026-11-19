<meta name="screen" content="film-choices" data-layer="hud" data-input="all">
<meta name="strings" content="./film-choices.en.ogt">
<meta name="parts" content="film-choices.frame film-choices.timer film-choices.choice film-choices.hotspot film-choices.qte film-choices.shade">
<meta name="prop:open" content="bool" data-doc="The film's choice window is open or locked (og.film.view's window is set): the choices show.">
<meta name="prop:choices" content="list" data-doc="The node's choices, a keyed collection of { id, text, enabled, chosen, progress }: id is ChoiceView.index as text (choose reports it as the row's key), chosen marks WindowView.pending, progress a quick-time event's QteView.progress (-1 for a choice without one: no bar).">
<meta name="prop:hotspots" content="list" data-doc="FilmView.hotspots as a keyed collection of { id, x, y, width, height, label }: id is the choice's index as text (pick reports it), the rect in fractions of the picture, which the screen takes to fill it; label is the choice's text, for a screen reader.">
<meta name="prop:left" content="number" data-doc="The window's time left, 0 to 1 (WindowView.ticksLeft / ticksTotal).">
<meta name="prop:locked" content="bool" data-doc="Past film.lock (WindowView.locked): the choices stay, no longer taken.">
<meta name="action:choose" content="Choose">
<meta name="action:pick" content="Choose on the picture">
<meta name="fixture:default" content="{&quot;open&quot;:true,&quot;left&quot;:0.6,&quot;locked&quot;:false,&quot;hotspots&quot;:[{&quot;id&quot;:&quot;0&quot;,&quot;x&quot;:0.58,&quot;y&quot;:0.3,&quot;width&quot;:0.14,&quot;height&quot;:0.22,&quot;label&quot;:&quot;Take the knife&quot;}],&quot;choices&quot;:[{&quot;id&quot;:&quot;0&quot;,&quot;text&quot;:&quot;Take the knife&quot;,&quot;enabled&quot;:true,&quot;chosen&quot;:false,&quot;progress&quot;:-1},{&quot;id&quot;:&quot;1&quot;,&quot;text&quot;:&quot;Open the door&quot;,&quot;enabled&quot;:true,&quot;chosen&quot;:true,&quot;progress&quot;:-1},{&quot;id&quot;:&quot;2&quot;,&quot;text&quot;:&quot;Hold to dodge&quot;,&quot;enabled&quot;:true,&quot;chosen&quot;:false,&quot;progress&quot;:0.45},{&quot;id&quot;:&quot;3&quot;,&quot;text&quot;:&quot;Wait in the dark&quot;,&quot;enabled&quot;:false,&quot;chosen&quot;:false,&quot;progress&quot;:-1}]}">

<div id="FilmChoices" class="bare" hidden={!open}>
  <div id="Shade" part="film-choices.shade"></div>
  <div id="Hotspots">
    {#each hotspots as h (h.id)}
      <button id="Hotspot" part="film-choices.hotspot" class="hotspot" variant="quiet" aria-label={h.label} disabled={locked} style:--x={h.x} style:--y={h.y} style:--w={h.width} style:--h={h.height} onclick={pick}><span class="corner a"></span><span class="corner b"></span><span class="corner c"></span><span class="corner d"></span></button>
    {/each}
  </div>
  <div id="Dock">
    <section id="Panel" part="film-choices.frame" class="bare" role="group" aria-label={t("screen.film-choices.label")}>
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
/* The film plays on: only the foot is shaded, behind the choices. A thin
   lit line over them drains with the time left. A choice is a cut smoked
   plate whose words wrap whole; the chosen one is ringed and washed in the
   accent, one that cannot be taken is flat and dim, a quick-time one
   carries a gold bar. A hotspot on the picture is four lit brackets round
   a faint wash. */
:root { position: relative; block-size: stretch; color: var(--front-ink); }
[part="film-choices.shade"] { position: absolute; inset: 0; background: var(--film-shade); }
/* Hotspots over the picture, at the fractions the film gives them. */
#Hotspots { position: absolute; inset: 0; }
.hotspot { position: absolute; inset-inline-start: calc(var(--x) * 100%); top: calc(var(--y) * 100%); width: calc(var(--w) * 100%); height: calc(var(--h) * 100%); min-width: var(--control-lg); min-height: var(--control-lg); padding: 0; border: 0; border-radius: 0; background: none; box-shadow: none; filter: var(--film-spot-glow); }
.hotspot::part(base) { padding: 0; border: 0; border-radius: 0; background: var(--film-spot-wash); box-shadow: none; }
.hotspot:disabled { filter: none; opacity: var(--opacity-dim); }
.corner { position: absolute; inline-size: var(--film-spot-size); block-size: var(--film-spot-size); border: 0 solid var(--film-spot-edge); }
.corner.a { inset-block-start: 0; inset-inline-start: 0; border-block-start-width: var(--stroke-thick); border-inline-start-width: var(--stroke-thick); }
.corner.b { inset-block-start: 0; inset-inline-start: calc(100% - var(--film-spot-size)); border-block-start-width: var(--stroke-thick); border-inline-end-width: var(--stroke-thick); }
.corner.c { inset-block-start: calc(100% - var(--film-spot-size)); inset-inline-start: 0; border-block-end-width: var(--stroke-thick); border-inline-start-width: var(--stroke-thick); }
.corner.d { inset-block-start: calc(100% - var(--film-spot-size)); inset-inline-start: calc(100% - var(--film-spot-size)); border-block-end-width: var(--stroke-thick); border-inline-end-width: var(--stroke-thick); }
/* The choices docked at the foot, clear of the safe area. */
#Dock { position: relative; display: flex; flex-direction: column; justify-content: end; align-items: center; block-size: 100%; padding: var(--space-xxl) calc(env(safe-area-inset-right) + var(--space-3xl)) calc(env(safe-area-inset-bottom) + var(--space-xl)) calc(env(safe-area-inset-left) + var(--space-3xl)); }
[part="film-choices.frame"] { display: flex; flex-direction: column; gap: var(--space-sm); inline-size: 100%; max-inline-size: calc(960 * var(--font-step-px)); }
#Timer { inline-size: 100%; block-size: calc(4 * var(--font-step-px)); min-block-size: 0; --meter-track-fill: var(--hud-line-well); --progress-fill: var(--hud-line-fill); }
#Timer::part(track), #Timer::part(fill) { border-radius: 0; }
#Timer::part(fill) { box-shadow: var(--hud-line-glow); }
#Choices { display: flex; flex-wrap: wrap; gap: var(--space-sm); }
button.choice { --button-radius: 0; --button-focus-radius: 0; --button-font-size: var(--film-choice-size); --button-fill: var(--hud-plate-fill); --button-hover-fill: var(--hud-plate-fill); --button-pressed-fill: var(--hud-plate-fill); --button-base-shadow: var(--hud-plate-rim); --button-hover-shadow: var(--hud-plate-lit-rim); --button-pressed-shadow: var(--hud-plate-lit-rim); --button-rim: transparent; --button-hover-rim: transparent; --button-pressed-rim: transparent; --button-ink: var(--front-ink); --button-hover-ink: var(--front-ink); --button-pressed-ink: var(--front-ink); --button-disabled-fill: var(--film-choice-off-fill); --button-disabled-shadow: var(--film-choice-off-rim); --button-disabled-rim: transparent; --button-disabled-ink: var(--film-choice-off-ink); --button-primary-fill: var(--film-choice-on-fill); --button-primary-hover-fill: var(--film-choice-on-fill); --button-primary-pressed-fill: var(--film-choice-on-fill); --button-primary-shadow: var(--front-card-on-rim); --button-primary-rim: transparent; --button-primary-hover-rim: transparent; --button-primary-ink: var(--front-ink); --button-primary-hover-ink: var(--front-ink); --button-primary-pressed-ink: var(--front-ink); --button-primary-disabled-fill: var(--film-choice-on-fill); --button-primary-disabled-ink: var(--front-ink); flex: 1 1 calc(170 * var(--font-step-px)); min-inline-size: 0; block-size: auto; min-block-size: calc(56 * var(--font-step-px)); white-space: normal; outline: 0; }
button.choice::part(base) { padding: var(--space-sm) var(--space-md); border-width: 0; clip-path: polygon(var(--dialog-cut) 0, 100% 0, 100% calc(100% - var(--dialog-cut)), calc(100% - var(--dialog-cut)) 100%, 0 100%, 0 var(--dialog-cut)); }
button.choice.chosen::part(base) { box-shadow: var(--front-card-on-rim); }
.body { display: flex; flex-direction: column; align-items: center; gap: var(--space-xs); inline-size: 100%; }
.body p { margin: 0; font-size: var(--film-choice-size); line-height: var(--font-leading-snug); font-weight: var(--font-weight-semibold); text-align: center; overflow-wrap: break-word; }
.qte { inline-size: 100%; block-size: calc(6 * var(--font-step-px)); min-block-size: 0; --meter-track-fill: var(--hud-line-well); --progress-fill: var(--film-qte-fill); }
.qte::part(track), .qte::part(fill) { border-radius: 0; }
.qte::part(fill) { box-shadow: var(--film-qte-glow); }
@media (width < 600px), (height < 600px) {
  [part="film-choices.shade"] { background: var(--film-shade-compact); }
  #Dock { padding: var(--space-lg) calc(env(safe-area-inset-right) + var(--space-lg)) calc(env(safe-area-inset-bottom) + var(--space-lg)) calc(env(safe-area-inset-left) + var(--space-lg)); }
  #Choices { flex-direction: column; flex-wrap: nowrap; }
  button.choice { flex: none; }
}
</style>
