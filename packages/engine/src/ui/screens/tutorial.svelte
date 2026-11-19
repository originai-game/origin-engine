<meta name="screen" content="tutorial" data-layer="popover" data-focus="" data-under="none" data-input="all" data-pause="false" data-capture="false" data-remember="true" data-back="skip">
<meta name="strings" content="./tutorial.en.ogt">
<meta name="parts" content="tutorial.frame tutorial.title tutorial.text tutorial.step tutorial.track tutorial.fill">
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

<div id="Tutorial">
  <div id="Highlight">{@render highlight?.()}</div>
  {#if target}
    <og-spotlight id="Spot" class="card" part="tutorial.frame" open target={target}>{@render card()}</og-spotlight>
  {:else}
    <section id="Card" class="card" part="tutorial.frame">{@render card()}</section>
  {/if}
</div>

{#snippet card()}
  <header id="Head">
    <p id="Step" part="tutorial.step">{t("screen.tutorial.Step", { step: step, steps: steps })}</p>
    <div id="Track" part="tutorial.track"><div id="Fill" part="tutorial.fill" style:--at={step / steps}></div></div>
  </header>
  {#if texture != 0}<art id="Art" kind="tutorial" src={texture}></art>{/if}
  <h2 id="Title" part="tutorial.title">{title}</h2>
  <p id="Text" part="tutorial.text">{text}</p>
  <div id="Hint">{@render hint?.()}</div>
  <footer id="Foot">
    <button id="Skip" variant="quiet" onclick={skip}>{t("screen.tutorial.skip")}</button>
    {#if !waits}<button id="Next" variant="primary" class="plate go" onclick={next} onconfirm={next}>{t("screen.tutorial.next")}</button>{/if}
  </footer>
{/snippet}

<style>
:root { position: relative; block-size: stretch; }
#Highlight { position: absolute; inset: 0; }
h2, p { margin: 0; }
/* The card: a dense smoked plate lit in the accent, at the spotlight's
   target, else at the game's x and y. It grows with its words: the step's
   text is never clamped. */
.card { --frame-fill: var(--dialog-fill); --frame-radius: 0; --frame-rim-width: 0; --frame-shadow: var(--hud-plate-lit-rim), var(--dialog-lift); --frame-pad: var(--space-lg) var(--space-xl); gap: var(--space-sm); inline-size: 100%; max-inline-size: calc(400 * var(--font-step-px)); color: var(--front-ink); transition: opacity var(--motion-enter), scale var(--motion-enter); }
@starting-style { .card { opacity: 0; scale: 0.6; } }
#Card { display: flex; flex-direction: column; position: absolute; inset-inline-start: bind(x); top: bind(y); }
/* The step count as a kicker, the lesson's progress a thin lit line beside it. */
#Head { display: flex; align-items: center; column-gap: var(--space-md); }
[part="tutorial.step"] { flex: none; font-size: var(--hud-label-size); font-weight: var(--font-weight-bold); letter-spacing: var(--hud-label-tracking); text-transform: uppercase; font-variant-numeric: tabular-nums; color: var(--front-accent); }
[part="tutorial.step"]:lang(zh, ja, ko) { font-size: calc(14 * var(--font-step-px)); letter-spacing: 0; }
[part="tutorial.track"] { flex: 1 1 auto; min-inline-size: var(--space-xl); block-size: calc(4 * var(--font-step-px)); background: var(--hud-pip-off); }
[part="tutorial.fill"] { inline-size: calc(var(--at) * 100%); block-size: 100%; background: var(--hud-pip-on); box-shadow: var(--hud-pip-glow); }
[part="tutorial.title"] { font-size: calc(24 * var(--font-step-px)); line-height: var(--font-leading-tight); font-weight: var(--font-weight-bold); font-stretch: var(--front-stretch); letter-spacing: var(--font-tracking-xs); color: var(--front-ink); }
[part="tutorial.text"] { font-size: var(--dialog-body-size); line-height: var(--font-leading-normal); color: var(--front-ink-dim); }
/* Skip at the line's start, Next at its end; they wrap where the card is
   too narrow for both. */
#Foot { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: var(--space-sm); margin-block-start: var(--space-xs); }
#Skip { --button-ink: var(--front-ink-dim); --button-quiet-ink: var(--front-ink-dim); --button-quiet-hover-ink: var(--front-ink); margin-inline-start: calc(var(--space-md) * -1); }
#Skip, #Skip::part(base), #Next, #Next::part(base) { min-block-size: var(--touch-target); }
#Next { margin-inline-start: auto; min-inline-size: calc(120 * var(--font-step-px)); }
button.plate { --button-radius: 0; --button-focus-radius: 0; --button-border-width: 0; --button-font-size: var(--dialog-label-size); --button-tracking: var(--font-tracking-lg); --button-transform: uppercase; }
button.plate::part(base) { border-width: 0; clip-path: polygon(var(--dialog-cut) 0, 100% 0, 100% calc(100% - var(--dialog-cut)), calc(100% - var(--dialog-cut)) 100%, 0 100%, 0 var(--dialog-cut)); }
/* Chinese, Japanese and Korean have no capitals to track: the label is set a step larger instead. */
button.plate:lang(zh, ja, ko) { --button-font-size: var(--dialog-label-cjk-size); --button-tracking: 0; }
button.go { --button-primary-fill: var(--front-play-fill); --button-primary-hover-fill: var(--front-play-hover-fill); --button-primary-pressed-fill: var(--front-play-pressed-fill); --button-primary-ink: var(--front-play-ink); --button-primary-hover-ink: var(--front-play-ink); --button-primary-pressed-ink: var(--front-play-ink); --button-primary-shadow: none; --button-primary-rim: transparent; --button-primary-hover-rim: transparent; }
@media (width < 600px) and (orientation: portrait) {
  #Card { inset-inline: var(--space-md); inline-size: auto; max-inline-size: none; }
}
</style>
