<meta name="screen" content="game-over" data-layer="modal" data-focus="Panel/Actions/Retry" data-under="dim" data-input="menu" data-pause="true" data-capture="true" data-remember="true" data-back="none">
<meta name="strings" content="./game-over.en.ogt">
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
  <column id="Panel">
    <text id="Title" heading="true">{$title}</text>
    <row id="Score">
      <text id="Value">{t("screen.game-over.Value", { value: $score })}</text>
      <text id="Best">{t("screen.game-over.Best", { best: $best })}</text>
    </row>
    <column id="Stats">{@render stats?.()}</column>
    <column id="Offer" hidden={!($offer)}>
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
#Revive, #Retry, #Revive::part(base), #Retry::part(base) { height: calc(18rem / 7); }
@media (pointer: coarse) {
#Revive, #Retry, #Revive::part(base), #Retry::part(base) { height: calc(24rem / 7); }
}
@media (-og-input: pad) {
#Revive, #Retry, #Revive::part(base), #Retry::part(base) { height: calc(26rem / 7); }
}
#GameOver { width: stretch; height: stretch; padding: 32px; justify-content: center; align-items: center; }
@media (width < 600px), (height < 600px) {
#GameOver { padding: 16px; }
}
#Panel { width: stretch; max-width: 440px; gap: 16px; align-items: center; }
#Title { font: var(--text-xl5); letter-spacing: var(--type-xl5-tracking); text-align: center; color: var(--danger); animation: land 300ms cubic-bezier(0.23,1,0.32,1) forwards; }
@keyframes land {
  from { scale: 1.3; -og-sound: error; }
  to { scale: 1; }
}
@media (width < 600px), (height < 600px) {
#Title { font: var(--text-xl3); letter-spacing: var(--type-xl3-tracking); }
}
#Score { gap: 16px; }
#Value { font: var(--text-xl); letter-spacing: var(--type-xl-tracking); }
#Best { color: var(--ink-muted); }
#Stats { width: stretch; }
#Offer { width: stretch; padding: 16px; gap: 8px; align-items: center; background: var(--panel-raised); border-radius: var(--radius-card); outline: 1px solid var(--line); outline-offset: -1px; }
#Line { font: var(--text-lg); letter-spacing: var(--type-lg-tracking); }
#Left { font: var(--text-xl2); letter-spacing: var(--type-xl2-tracking); }
#OfferArt { width: stretch; }
#Revive { width: stretch; }
#Actions { width: stretch; gap: 8px; }
#Retry { width: stretch; }
#Checkpoint { width: stretch; }
#Quit { width: stretch; }
</style>
