<meta name="component" content="objective-tracker">
<meta name="parts" content="objective-tracker.frame objective-tracker.title objective-tracker.row objective-tracker.well objective-tracker.fill">
<meta name="prop:title" content="string" data-doc="The quest or chapter.">
<meta name="prop:objectives" content="list" data-doc="A keyed collection of { id, label, progress (0–1), done, steps }: steps, when above 1, cuts the bar into that many cells (2 / 5 is five); 0 or absent is one bar.">
<meta name="fixture:default" content="{&quot;title&quot;:&quot;The Lighthouse&quot;,&quot;objectives&quot;:[{&quot;id&quot;:&quot;a&quot;,&quot;label&quot;:&quot;Find the keeper&quot;,&quot;progress&quot;:1,&quot;done&quot;:true,&quot;steps&quot;:0},{&quot;id&quot;:&quot;b&quot;,&quot;label&quot;:&quot;Light the lamps 2/5&quot;,&quot;progress&quot;:0.4,&quot;done&quot;:false,&quot;steps&quot;:5}]}">

<!--
  The tracked objectives over the scene, with no plate: the quest in gold
  small capitals, then each objective with a gold diamond (filled once
  done, its words struck and dimmed) over a thin leaning bar in a dark
  well, cut into cells when the game counts its steps. The first not done
  is lit; on a phone only it is shown. Read only.
-->
<section id="Tracker" part="objective-tracker.frame" class="bare" role="status" aria-label={title}>
  <h3 id="Title" part="objective-tracker.title">{title}</h3>
  <ul id="Objectives">
    {#each objectives as o, i (o.id)}
      <li class="objective" part="objective-tracker.row" data-done={o.done} data-current={!o.done && (i == 0 || objectives[i - 1]['done'])} style:--p={o.progress} style:--steps={o.steps > 1 ? o.steps : 1}>
        <div class="line"><div class="mark"></div><p class="label">{o.label}</p></div>
        {#if !o.done}<div class="bar" part="objective-tracker.well"><div class="fill" part="objective-tracker.fill"></div>{#if o.steps > 1}<div class="cells"></div>{/if}</div>{/if}
      </li>
    {/each}
  </ul>
</section>

<style>
/* HUD type over the scene: one stroke under the fill and one soft shadow. */
:host { color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
/* Inside the HUD's safe edge, as the notifications are. */
#Tracker { display: flex; flex-direction: column; gap: calc(6 * var(--font-step-px)); inline-size: calc(300 * var(--font-step-px)); max-inline-size: stretch; margin: calc(env(-og-hud-inset-top) + var(--space-lg)) calc(env(-og-hud-inset-right) + var(--space-lg)) 0 calc(env(-og-hud-inset-left) + var(--space-lg)); }
#Title { margin: 0; min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-gold); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
#Objectives { display: flex; flex-direction: column; list-style: none; padding: 0; margin: 0; gap: calc(6 * var(--font-step-px)); }
.objective { display: flex; flex-direction: column; gap: calc(4 * var(--font-step-px)); min-inline-size: 0; opacity: 0.72; }
.objective[data-current=true] { opacity: 1; }
.objective[data-done=true] { opacity: 0.55; }
/* The diamond sits on the first line's baseline, however many lines the words take. */
.line { display: flex; align-items: baseline; gap: calc(8 * var(--font-step-px)); min-inline-size: 0; }
/* A diamond: hollow gold for a step to do, filled once done, glowing on the one in play. */
.mark { flex: none; inline-size: calc(8 * var(--font-step-px)); block-size: calc(8 * var(--font-step-px)); margin-inline: calc(2 * var(--font-step-px)); rotate: 45deg; box-shadow: var(--hud-step-rim); }
.objective[data-done=true] .mark { background: var(--hud-gold); }
.objective[data-current=true] .mark { background: var(--hud-gold); box-shadow: var(--quest-step-lit); }
.label { margin: 0; min-inline-size: 0; font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); overflow-wrap: anywhere; }
.objective[data-done=true] .label { text-decoration-line: var(--hud-done-line); }
/* The bar: leaning, a dark well with one lit rim, a gold fill lit along its top; cells cut by the well's colour. */
.bar { position: relative; block-size: calc(5 * var(--font-step-px)); margin-inline-start: calc(20 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); -webkit-text-stroke-width: 0; }
.fill { block-size: 100%; inline-size: calc(var(--p) * 100%); --fill-color: var(--hud-gold); --fill-gloss: var(--bar-gloss); }
.cells { position: absolute; inset: 0; background: repeating-linear-gradient(90deg, transparent 0 calc(100% / var(--steps) - var(--stroke-thick)), var(--bar-notch) calc(100% / var(--steps) - var(--stroke-thick)) calc(100% / var(--steps))); }
@media (width < 600px) {
  #Tracker { inline-size: stretch; }
  /* Only the first objective in play. */
  .objective:not([data-current=true]), .objective[data-current=true] ~ .objective { display: none; }
  /* Every objective done: the last one, struck, under its quest. */
  #Objectives:not(:has(.objective[data-current=true])) .objective:last-child { display: flex; }
}
</style>
