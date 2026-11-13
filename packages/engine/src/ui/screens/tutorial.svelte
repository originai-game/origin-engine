<meta name="screen" content="tutorial" data-layer="popover" data-focus="" data-under="none" data-input="all" data-pause="false" data-capture="false" data-remember="true" data-back="skip">
<meta name="strings" content="./tutorial.en.ogt">
<meta name="parts" content="tutorial.frame tutorial.title tutorial.text tutorial.step">
<meta name="prop:title" content="string" data-doc="The step's title.">
<meta name="prop:text" content="string" data-doc="What to do.">
<meta name="prop:step" content="number" data-doc="The step, from 1.">
<meta name="prop:steps" content="number" data-doc="Steps in the lesson.">
<meta name="prop:waits" content="bool" data-doc="The step waits for the player to act (Next hidden).">
<meta name="prop:x" content="number" data-doc="The card's start edge, CSS px from the screen's start (the right, right to left).">
<meta name="prop:y" content="number" data-doc="The card's top, CSS px.">
<meta name="prop:texture" content="number" data-doc="The step's art (0: none).">
<meta name="prop:target" content="string" data-default="" data-doc="The id of the element the step points at, in any document under this one: the rest of the screen dims and takes presses, the element stays lit and takes its own, and the card stands against it in place of x and y. Empty: nothing is dimmed.">
<meta name="action:confirm" content="Next">
<meta name="action:back" content="Skip">
<meta name="fixture:default" content="{&quot;step&quot;:2,&quot;steps&quot;:5,&quot;target&quot;:&quot;&quot;,&quot;text&quot;:&quot;Press the dodge button just before an attack lands to slip past it.&quot;,&quot;texture&quot;:0,&quot;title&quot;:&quot;Dodge&quot;,&quot;waits&quot;:false,&quot;x&quot;:64,&quot;y&quot;:120}">

<stack id="Tutorial">
  <column id="Highlight">{@render highlight?.()}</column>
  {#if $target}
    <og-spotlight id="Spot" class="card" part="tutorial.frame" open target={$target}>{@render card()}</og-spotlight>
  {:else}
    <column id="Card" class="card" part="tutorial.frame">{@render card()}</column>
  {/if}
</stack>

{#snippet card()}
  <art id="Art" kind="tutorial" src={$texture} hidden={!(($texture != 0.0))}></art>
  <text id="Title" part="tutorial.title" heading="true">{$title}</text>
  <text id="Text" part="tutorial.text">{$text}</text>
  <column id="Hint">{@render hint?.()}</column>
  <row id="Foot">
    <text id="Step" part="tutorial.step">{t("screen.tutorial.Step", { step: $step, steps: $steps })}</text>
    <spacer id="Node9"></spacer>
    <button id="Skip" variant="quiet" onclick={skip}>{t("screen.tutorial.skip")}</button>
    <button id="Next" variant="primary" hidden={!(!($waits))} onclick={next} onconfirm={next}>{t("screen.tutorial.next")}</button>
  </row>
{/snippet}

<style>
button, button::part(base) { height: var(--control-sm); }
button::part(base) { padding: 0 var(--space-md); }
@media (width < 600px), (height < 600px) {
button, button::part(base) { height: var(--control-md); }
button::part(base) { padding: 0 var(--space-lg); }
}
#Tutorial { width: stretch; height: stretch; }
#Highlight { width: stretch; height: stretch; position: absolute; inset-inline-start: 0; top: 0; inset-inline-end: 0; bottom: 0; }
.card { width: stretch; max-width: var(--measure-md); gap: var(--space-sm); opacity: 1; scale: 1; transition: opacity var(--motion-enter), scale var(--motion-enter); }
@starting-style { .card { opacity: 0; scale: 0.6; } }
#Card { position: absolute; inset-inline-start: bind(x); top: bind(y); }
@media (width < 600px) and (orientation: portrait) {
#Card { inset-inline-start: var(--space-md); inset-inline-end: var(--space-md); top: bind(y); }
}
#Art { width: stretch; }
#Title { font-size: var(--type-lg-size); }
#Text { font: var(--text-reading); letter-spacing: var(--type-reading-tracking); line-clamp: 5; }
#Hint { width: stretch; }
#Foot { width: stretch; gap: var(--space-sm); align-items: center; }
#Step { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
</style>
