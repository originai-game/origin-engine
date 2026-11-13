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

<column id="GameOver">
  <column id="Panel" part="game-over.frame" class="bare">
    <text id="Title" part="game-over.title" heading="true">{$title}</text>
    <row id="Score">
      <text id="Value" part="game-over.figure">{t("screen.game-over.Value", { value: $score })}</text>
      <text id="Best" part="game-over.label">{t("screen.game-over.Best", { best: $best })}</text>
    </row>
    <column id="Stats">{@render stats?.()}</column>
    <column id="Offer" part="game-over.offer" hidden={!($offer)}>
      <text id="Line">{t("screen.game-over.revive")}</text>
      <text id="Left">{t("screen.game-over.Left", { value: formatCountdown($left) })}</text>
      <column id="OfferArt">{@render offerArt?.()}</column>
      <button id="Revive" variant="primary" onclick={revive}>{t("screen.game-over.Revive", { price: $price })}</button>
      <button id="Decline" variant="quiet" onclick={decline}>{t("screen.game-over.decline")}</button>
    </column>
    <column id="Actions" hidden={!(!($offer))}>
      <button id="Retry" variant="primary" onclick={retry}>{t("screen.game-over.retry")}</button>
      <button id="Checkpoint" hidden={!($checkpoint)} onclick={checkpoint}>{t("screen.game-over.checkpoint")}</button>
      <button id="Quit" variant="quiet" onclick={quit}>{t("screen.game-over.quit")}</button>
    </column>
  </column>
</column>

<style>
#Revive, #Retry, #Revive::part(base), #Retry::part(base) { height: var(--control-lg); }
#GameOver { width: stretch; height: stretch; padding: var(--space-xxl); justify-content: center; align-items: center; }
@media (width < 600px), (height < 600px) {
#GameOver { padding: var(--space-lg); }
}
#Panel { width: stretch; max-width: var(--measure-lg); gap: var(--space-lg); align-items: center; }
#Title { font-size: var(--type-xl5-size); text-align: center; color: var(--danger); animation: land var(--motion-named-ui-duration) var(--motion-easing-out) forwards; }
@keyframes land {
  from { scale: 1.3; -og-sound: error; }
  to { scale: 1; }
}
@media (width < 600px), (height < 600px) {
#Title { font-size: var(--type-xl3-size); }
}
#Score { gap: var(--space-lg); }
#Value { font: var(--text-xl); letter-spacing: var(--type-xl-tracking); }
#Stats { width: stretch; }
#Offer { width: stretch; padding: var(--space-lg); gap: var(--space-sm); align-items: center; background: var(--panel-raised); border-radius: var(--radius-card); outline: var(--stroke-hairline) solid var(--line); outline-offset: calc(var(--stroke-hairline) * -1); }
#Line { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#Left { font: var(--text-xl2); letter-spacing: var(--type-xl2-tracking); }
#OfferArt { width: stretch; }
#Revive { width: stretch; }
#Actions { width: stretch; gap: var(--space-sm); }
#Retry { width: stretch; }
#Checkpoint { width: stretch; }
#Quit { width: stretch; }
</style>
