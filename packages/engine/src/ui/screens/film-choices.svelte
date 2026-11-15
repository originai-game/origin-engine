<meta name="screen" content="film-choices" data-layer="hud" data-input="all">
<meta name="strings" content="./film-choices.en.ogt">
<meta name="parts" content="film-choices.frame film-choices.timer film-choices.choice">
<meta name="prop:open" content="bool" data-doc="The film's choice window is open or locked (og.film.view's window is set): the choices show.">
<meta name="prop:choices" content="list" data-doc="The node's choices, a keyed collection of { id, text, enabled, chosen }: id is ChoiceView.index as text (choose reports it as the row's key), chosen marks WindowView.pending.">
<meta name="prop:left" content="number" data-doc="The window's time left, 0 to 1 (WindowView.ticksLeft / ticksTotal).">
<meta name="prop:locked" content="bool" data-doc="Past film.lock (WindowView.locked): the choices stay, no longer taken.">
<meta name="action:choose" content="Choose">
<meta name="fixture:default" content="{&quot;open&quot;:true,&quot;left&quot;:0.6,&quot;locked&quot;:false,&quot;choices&quot;:[{&quot;id&quot;:&quot;0&quot;,&quot;text&quot;:&quot;Take the knife&quot;,&quot;enabled&quot;:true,&quot;chosen&quot;:false},{&quot;id&quot;:&quot;1&quot;,&quot;text&quot;:&quot;Open the door&quot;,&quot;enabled&quot;:true,&quot;chosen&quot;:true},{&quot;id&quot;:&quot;2&quot;,&quot;text&quot;:&quot;Wait in the dark&quot;,&quot;enabled&quot;:false,&quot;chosen&quot;:false}]}">

<column id="FilmChoices" class="bare" hidden={!($open)}>
  <column id="Panel" part="film-choices.frame" role="group" aria-label={t("screen.film-choices.label")}>
    <progress id="Timer" part="film-choices.timer" bind:value={$left}></progress>
    <div id="Choices">
      {#each $choices as c (c.id)}
        <button id="Choice" part="film-choices.choice" class="choice" class:chosen={c.chosen} variant={c.chosen ? "primary" : "default"} aria-current={c.chosen} disabled={!(c.enabled) || $locked} onclick={choose}>{c.text}</button>
      {/each}
    </div>
  </column>
</column>

<style>
#FilmChoices { width: stretch; height: stretch; justify-content: end; align-items: center; padding: var(--space-xxl) calc(env(safe-area-inset-right) + var(--space-xl)) calc(env(safe-area-inset-bottom) + var(--space-xxl)) calc(env(safe-area-inset-left) + var(--space-xl)); }
#Panel { width: stretch; max-width: var(--measure-xxl); gap: var(--space-sm); padding: var(--space-md); background: var(--hud); border-radius: var(--radius-card); }
#Timer { width: stretch; }
#Choices { display: flex; flex-wrap: wrap; width: stretch; gap: var(--space-sm); }
.choice { flex: 1 1 0; min-width: var(--measure-xs); min-height: var(--control-lg); }
.chosen { outline: var(--stroke-regular) solid var(--accent); outline-offset: var(--stroke-regular); }
@media (width < 600px), (height < 600px) {
#FilmChoices { padding: var(--space-lg) calc(env(safe-area-inset-right) + var(--space-md)) calc(env(safe-area-inset-bottom) + var(--space-lg)) calc(env(safe-area-inset-left) + var(--space-md)); }
#Choices { flex-direction: column; }
.choice { flex: none; width: stretch; min-width: 0; }
}
</style>
