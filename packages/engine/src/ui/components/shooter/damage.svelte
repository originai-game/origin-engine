<meta name="component" content="ShooterDamage">
<meta name="strings" content="./shooter.en.ogt">
<meta name="parts" content="ShooterDamage.frame ShooterDamage.hit ShooterDamage.call ShooterDamage.amount">
<meta name="prop:hits" content="list" data-doc="The player's latest hits, newest first, keyed rows of { id, amount, kind }: amount the damage dealt; kind normal, headshot, shield (it struck a shield) or kill (on data-kind: a headshot and a kill in gold under their word, a shield hit in the shield's blue). The newest stands largest beside the reticle and the older ones step down and away (a phone shows the newest alone); the game trims the list (three or so) and a new row pops in.">
<meta name="fixture:default" content='{"hits":[{"id":"d2","amount":86,"kind":"headshot"},{"id":"d1","amount":31,"kind":"normal"}]}'>
<meta name="fixture:shield" content='{"hits":[{"id":"s1","amount":24,"kind":"shield"}]}'>
<meta name="fixture:kill" content='{"hits":[{"id":"k3","amount":112,"kind":"kill"},{"id":"k2","amount":45,"kind":"normal"},{"id":"k1","amount":45,"kind":"normal"}]}'>
<meta name="fixture:empty" content='{"hits":[]}'>
<meta name="fixture:overflow" content='{"hits":[{"id":"o1","amount":1234567,"kind":"headshot"},{"id":"o2","amount":99999,"kind":"shield"}]}'>

<!--
  Damage the player deals, beside the reticle: the newest hit's figure
  largest with its word (Headshot) over it, the older ones stepping down
  and away from the aim. Read only.
-->
<section id="ShooterDamage" part="ShooterDamage.frame" class="hud bare" data-layout="stack" data-gap="none" role="log" aria-label={t("shooter.dealt.name")}>
  {#each hits as h (h.id)}
  <div part="ShooterDamage.hit" class="hit" data-layout="stack" data-gap="none" data-kind={h.kind} data-motion="pop" out:fade={{ duration: 200 }}>
    {#if h.kind == 'headshot'}<span part="ShooterDamage.call" class="call">{t("shooter.dealt.headshot")}</span>{/if}
    {#if h.kind == 'kill'}<span part="ShooterDamage.call" class="call">{t("shooter.dealt.kill")}</span>{/if}
    <span part="ShooterDamage.amount" class="amount">{formatNumber(h.amount)}</span>
  </div>
  {/each}
</section>

<style>
:host { align-items: flex-start; max-inline-size: stretch; pointer-events: none; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.hit { align-items: flex-start; }
.call { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-gold); white-space: nowrap; }
.amount { font-size: var(--hud-size-damage); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
[data-kind=headshot] .amount, [data-kind=kill] .amount { color: var(--hud-gold); }
[data-kind=shield] .amount { color: var(--hud-ally); }
/* Older hits step down and away from the aim, a size smaller and dimmed. */
.hit + .hit { margin-inline-start: calc(36 * var(--font-step-px)); opacity: 0.8; }
.hit + .hit .call { display: none; }
.hit + .hit .amount { font-size: var(--hud-size-md); }
.hit + .hit + .hit { margin-inline-start: calc(64 * var(--font-step-px)); opacity: 0.6; }
@media (width < 600px), (height < 500px) {
  .amount { font-size: var(--hud-size-damage-compact); }
  .call { letter-spacing: normal; }
  .hit + .hit { display: none; }
}
</style>
