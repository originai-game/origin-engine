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
    <div id="Score">
      <p id="Value" part="game-over.figure">{t("screen.game-over.Value", { value: score })}</p>
      <p id="Best" part="game-over.label">{t("screen.game-over.Best", { best: best })}</p>
    </div>
    <div id="Stats">{@render stats?.()}</div>
    {#if offer}
      <div id="Offer" part="game-over.offer">
        <p id="Line">{t("screen.game-over.revive")}</p>
        <p id="Left">{t("screen.game-over.Left", { value: formatCountdown(left) })}</p>
        <div id="OfferArt">{@render offerArt?.()}</div>
        <button id="Revive" variant="primary" class="large" onclick={revive}>{t("screen.game-over.Revive", { price: price })}</button>
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
/* The cover centres the panel where it fits; a panel taller than the
   screen starts at the safe top and scrolls, so its title is never cut. */
:root { block-size: stretch; padding: var(--space-xxl); }
#GameOver { inline-size: stretch; }
p { margin: 0; }
[part="game-over.frame"] { flex: none; display: flex; flex-direction: column; align-items: center; gap: var(--space-lg); inline-size: 100%; max-inline-size: var(--measure-lg); }
[part="game-over.title"] { margin: 0; font-size: var(--type-xl5-size); text-align: center; color: var(--danger); animation: land var(--motion-named-ui-duration) var(--motion-easing-out) forwards; }
@keyframes land {
  from { scale: 1.3; -og-sound: error; }
  to { scale: 1; }
}
#Score { display: flex; gap: var(--space-lg); align-items: baseline; }
[part="game-over.figure"] { font: var(--text-xl); letter-spacing: var(--type-xl-tracking); }
#Stats, #OfferArt { align-self: stretch; }
/* The offer and the actions: a column of full-width buttons. */
#Offer, #Actions { display: grid; gap: var(--space-sm); justify-items: stretch; align-self: stretch; }
[part="game-over.offer"] { text-align: center; padding: var(--space-lg); background: var(--panel-raised); border-radius: var(--radius-card); outline: var(--stroke-hairline) solid var(--line); outline-offset: calc(var(--stroke-hairline) * -1); }
[part="game-over.offer"] button { justify-self: stretch; }
#Line { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#Left { font: var(--text-xl2); letter-spacing: var(--type-xl2-tracking); }
.large, .large::part(base) { block-size: var(--control-lg); }
@media (width < 600px), (height < 600px) {
  :root { padding: var(--space-lg); }
  [part="game-over.title"] { font-size: var(--type-xl3-size); }
}
/* A phone on its side: the line and the score beside the offer or the
   actions, so the whole screen shows without scrolling. */
@media (height < 500px) and (orientation: landscape) {
  [part="game-over.frame"] { display: grid; grid-template-columns: 1fr 1fr; grid-template-rows: 1fr auto auto auto 1fr; grid-template-areas: ". side" "title side" "score side" "stats side" ". side"; align-items: center; justify-items: center; column-gap: var(--space-xl); row-gap: var(--space-sm); max-inline-size: calc(var(--measure-lg) * 2); }
  #Title { grid-area: title; }
  #Score { grid-area: score; }
  #Stats { grid-area: stats; }
  #Offer, #Actions { grid-area: side; justify-self: stretch; }
}
</style>
