<meta name="screen" content="game-over" data-layer="modal" data-focus="Panel/Actions/Retry" data-under="dim" data-input="menu" data-pause="true" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./game-over.en.ogt">
<meta name="parts" content="game-over.frame game-over.title game-over.figure game-over.label game-over.offer">
<meta name="prop:title" content="string" data-doc="The defeat's line.">
<meta name="prop:score" content="number" data-doc="The run's score.">
<meta name="prop:best" content="number" data-doc="The best score.">
<meta name="prop:offer" content="bool" data-doc="A revive is offered.">
<meta name="prop:price" content="string" data-doc="The revive's price, as text (50 gems, Watch an ad).">
<meta name="prop:left" content="number" data-doc="Seconds the offer stays.">
<meta name="prop:checkpoint" content="bool" data-doc="A checkpoint exists.">
<meta name="action:confirm" content="Retry">
<meta name="fixture:default" content="{&quot;best&quot;:24100,&quot;checkpoint&quot;:true,&quot;left&quot;:5,&quot;offer&quot;:true,&quot;price&quot;:&quot;50 gems&quot;,&quot;score&quot;:18420,&quot;title&quot;:&quot;You died&quot;}">

<div id="GameOver" data-layout="cover">
  <section id="Panel" part="game-over.frame" class="bare">
    <h1 id="Title" part="game-over.title">{title}</h1>
    <div id="Rule"><span id="Diamond"></span></div>
    <div id="Score">
      <div id="ScoreCell"><p id="ScoreLabel" part="game-over.label">{t("screen.game-over.score")}</p><p id="Value" part="game-over.figure">{t("screen.game-over.Value", { value: score })}</p></div>
      <p id="Best" part="game-over.label">{t("screen.game-over.Best", { best: best })}</p>
    </div>
    <div id="Stats">{@render stats?.()}</div>
    {#if offer}
      <div id="Offer" part="game-over.offer">
        <p id="Left">{t("screen.game-over.Left", { value: formatCountdown(left) })}</p>
        <p id="Line">{t("screen.game-over.revive")}</p>
        <div id="OfferArt">{@render offerArt?.()}</div>
        <button id="Revive" variant="primary" class="large revive" onclick={revive}>{t("screen.game-over.Revive", { price: price })}</button>
        <button id="Decline" variant="quiet" onclick={decline}>{t("screen.game-over.decline")}</button>
      </div>
    {:else}
      <div id="Actions">
        <button id="Retry" variant="primary" class="large" onclick={retry}>{t("screen.game-over.retry")}</button>
        {#if checkpoint}<button id="Checkpoint" onclick={checkpoint}>{t("screen.game-over.checkpoint")}</button>{/if}
        <button id="Quit" variant="quiet" onclick={quit}>{t("screen.game-over.quit")}</button>
      </div>
    {/if}
  </section>
</div>

<style>
/* The scene dimmed to a red-black vignette; the cover centres the column
   where it fits, and a column taller than the screen starts at the safe
   top and scrolls, so its title is never cut. */
:root { block-size: stretch; padding: var(--space-xxl); background: var(--result-defeat-scrim); }
#GameOver { inline-size: stretch; }
p { margin: 0; }
[part="game-over.frame"] { flex: none; display: flex; flex-direction: column; align-items: center; gap: var(--space-lg); inline-size: 100%; max-inline-size: var(--measure-lg); container-type: inline-size; }
/* The verdict: condensed capitals in the loss's lit red, landing with a thud. */
[part="game-over.title"] { margin: 0; max-inline-size: stretch; font-size: min(var(--result-size-defeat), 24cqi); line-height: 0.95; font-weight: var(--font-weight-bold); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-defeat-tracking); text-transform: uppercase; text-align: center; overflow-wrap: anywhere; color: transparent; background: var(--result-lost-fill); background-clip: var(--hud-text-clip); filter: var(--result-lost-glow); animation: land var(--motion-named-ui-duration) var(--motion-easing-out) forwards; }
@keyframes land {
  from { scale: 1.3; -og-sound: error; }
  to { scale: 1; }
}
#Rule { position: relative; inline-size: 100%; block-size: var(--stroke-thin); background: var(--result-rule); }
#Diamond { position: absolute; inset-inline-start: 50%; inset-block-start: 50%; inline-size: calc(9 * var(--font-step-px)); block-size: calc(9 * var(--font-step-px)); translate: -50% -50%; rotate: 45deg; background: var(--hud-ink); box-shadow: var(--result-defeat-diamond-glow); }
#Score { display: flex; flex-wrap: wrap; justify-content: center; gap: var(--space-xs) var(--space-xl); align-items: flex-end; }
#ScoreCell { display: flex; flex-direction: column; align-items: center; }
[part="game-over.label"] { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); }
#Best { font-size: var(--hud-size-md); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); letter-spacing: 0; text-transform: none; color: var(--hud-ink-dim); }
[part="game-over.figure"] { font-size: var(--hud-size-score-compact); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); }
#Stats, #OfferArt { align-self: stretch; }
/* The offer and the actions: a column of full-width buttons; the offer
   on a cut HUD plate with its time left in a gold ring. */
#Offer, #Actions { display: grid; gap: var(--space-sm); justify-items: stretch; align-self: stretch; }
[part="game-over.offer"] { justify-items: center; text-align: center; padding: var(--space-lg); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(var(--hud-plate-cut) 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%, 0 var(--hud-plate-cut)); }
[part="game-over.offer"] button { justify-self: stretch; }
#Left { justify-self: center; display: block; text-align: center; line-height: calc(88 * var(--font-step-px)); inline-size: calc(88 * var(--font-step-px)); block-size: calc(88 * var(--font-step-px)); border-radius: 50%; font-size: var(--hud-size-lg); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); background: var(--hud-dial-face); box-shadow: var(--result-ring); filter: var(--hud-gold-glow); }
#Line { font-size: var(--hud-size-md); font-weight: var(--font-weight-bold); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--result-defeat-tracking); text-transform: uppercase; color: var(--hud-ink); }
.revive { --button-primary-fill: var(--result-revive-fill); --button-primary-hover-fill: var(--result-revive-fill); --button-primary-pressed-fill: var(--result-revive-fill); --button-primary-ink: var(--result-medal-ink); --button-primary-shadow: var(--result-revive-glow); }
.large, .large::part(base) { min-block-size: var(--control-lg); }
@media (width < 600px), (height < 600px) {
  :root { padding: var(--space-lg); }
}
/* A phone on its side: the line and the score beside the offer or the
   actions, so the whole screen shows without scrolling. */
@media (height < 500px) and (orientation: landscape) {
  [part="game-over.frame"] { display: grid; grid-template-columns: 1fr 1fr; grid-template-rows: 1fr auto auto auto auto 1fr; grid-template-areas: ". side" "title side" "rule side" "score side" "stats side" ". side"; align-items: center; justify-items: center; column-gap: var(--space-xl); row-gap: var(--space-sm); max-inline-size: calc(var(--measure-lg) * 2); container-type: normal; }
  [part="game-over.title"] { font-size: var(--result-size-hero-compact); }
  #Title { grid-area: title; }
  #Rule { grid-area: rule; }
  #Score { grid-area: score; }
  #Stats { grid-area: stats; }
  #Offer, #Actions { grid-area: side; justify-self: stretch; }
  #Left { inline-size: calc(64 * var(--font-step-px)); block-size: calc(64 * var(--font-step-px)); line-height: calc(64 * var(--font-step-px)); }
}
@media (prefers-reduced-motion: reduce) {
  [part="game-over.title"] { animation: none; }
}
</style>
