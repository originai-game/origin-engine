<meta name="component" content="IdleOffline">
<meta name="strings" content="./idle.en.ogt">
<meta name="parts" content="IdleOffline.frame IdleOffline.band IdleOffline.kicker IdleOffline.title IdleOffline.away IdleOffline.figure IdleOffline.label IdleOffline.haul IdleOffline.socket IdleOffline.amount IdleOffline.name">
<meta name="prop:title" content="string" data-doc="The panel's heading; empty takes the group's (Welcome back).">
<meta name="prop:away" content="string" data-doc="How long the player was away, as the game words it (it wins; empty shows seconds as a duration: 1h 2m).">
<meta name="prop:seconds" content="number" data-doc="How long the player was away, in seconds, when away is empty.">
<meta name="prop:earned" content="list" data-doc="What was gathered while away, keyed rows of { id, icon, name, amount }: amount as the game formats it (+12.4K).">
<meta name="prop:canDouble" content="bool" data-doc="The game offers to double the haul (an ad, a gem); false hides Double.">
<meta name="prop:doubleText" content="string" data-doc="What doubling costs or asks, as the game words it (Watch an ad); empty is none.">
<meta name="fixture:default" content='{"title":"","away":"","seconds":13530,"earned":[{"id":"gold","icon":"coin","name":"Gold","amount":"+1.24M"},{"id":"wood","icon":"tree","name":"Wood","amount":"+5,000"},{"id":"gems","icon":"gem","name":"Gems","amount":"+3"}],"canDouble":true,"doubleText":"Watch an ad"}'>
<meta name="fixture:disabled" content='{"title":"","away":"8h (capped)","seconds":0,"earned":[{"id":"gold","icon":"coin","name":"Gold","amount":"+880K"}],"canDouble":false,"doubleText":""}'>
<meta name="fixture:empty" content='{"title":"","away":"","seconds":42,"earned":[],"canDouble":false,"doubleText":""}'>
<meta name="fixture:overflow" content='{"title":"While You Slumbered, the Clockwork Empire Kept Turning","away":"3 days, 14 hours, 52 minutes and 9 seconds (capped at 72 hours)","seconds":0,"earned":[{"id":"gold","icon":"coin","name":"Gold Sovereigns of the Eastern Treasury","amount":"+123.45 quattuordecillion"},{"id":"wood","icon":"tree","name":"Wood","amount":"+5,000"},{"id":"gems","icon":"gem","name":"Gems","amount":"+99,999"},{"id":"stone","icon":"shield","name":"Stone","amount":"+48K"},{"id":"mana","icon":"drop","name":"Mana","amount":"+250"},{"id":"souls","icon":"skull","name":"Souls","amount":"+7"},{"id":"stars","icon":"star","name":"Stardust","amount":"+1.5B"}],"canDouble":true,"doubleText":"Spend 50 gems to double everything gathered"}'>

<!--
  The welcome-back panel of an idle game: how long the player was away,
  large (the game's words, or its seconds as a duration), then what was
  gathered as sockets glowing gold from their foot, each with its art, the
  amount and its name. Collect reports collect; Double, the game's gold
  offer and shown only when it sets canDouble, reports double with what it
  asks under its word. The game computes the haul and applies it. Many
  currencies wrap and the haul scrolls.
-->
<section id="IdleOffline" part="IdleOffline.frame" data-layout="stack" data-gap="loose" aria-label={title != '' ? title : t("idle.offline.name")}>
  <div class="lit"></div>
  <header part="IdleOffline.band" data-layout="stack" data-gap="none">
    <span part="IdleOffline.kicker" class="kicker">{t("idle.offline.kicker")}</span>
    <h2 part="IdleOffline.title">{title != '' ? title : t("idle.offline.name")}</h2>
  </header>
  <div part="IdleOffline.away" class="away" data-layout="cluster" data-gap="tight" role="status" aria-label={t("idle.offline.away", { time: away != '' ? away : formatDuration(seconds) })}>
    <span part="IdleOffline.figure" class="figure">{away != '' ? away : formatDuration(seconds)}</span>
    <span part="IdleOffline.label" class="label">{t("idle.offline.awayLabel")}</span>
  </div>
  {#if earned.length}
    <div class="body" data-layout="scroll" data-gap="none">
      <ul class="haul" data-layout="cluster" data-gap="tight" aria-label={t("idle.offline.earned")}>
        {#each earned as e (e.id)}
          <li part="IdleOffline.haul" data-layout="stack" data-gap="none" aria-label={t("idle.offline.row", { name: e.name, amount: e.amount })}>
            <div part="IdleOffline.socket" class="socket" data-layout="cover"><og-icon class="icon" icon={e.icon}></og-icon></div>
            <span part="IdleOffline.amount" class="amount">{e.amount}</span>
            <span part="IdleOffline.name" class="name">{e.name}</span>
          </li>
        {/each}
      </ul>
    </div>
  {:else}
    <p class="empty-text" part="IdleOffline.label" role="status">{t("idle.offline.empty")}</p>
  {/if}
  <footer class="actions" data-layout="cluster">
    {#if canDouble}<button id="Double" variant="secondary" onclick={double}><div class="double" data-layout="stack" data-gap="none"><span>{t("idle.offline.double")}</span>{#if doubleText != ''}<span class="double-text">{doubleText}</span>{/if}</div></button>{/if}
    <button id="Collect" data-tone="reward" class="go" variant="primary" onclick={collect}><span>{t("idle.offline.collect")}</span></button>
  </footer>
</section>

<style>
:host { inline-size: stretch; min-inline-size: 0; max-inline-size: var(--modal-width); max-block-size: 100%; min-block-size: 0; }
[part="IdleOffline.frame"] { position: relative; border-radius: var(--inv-frame-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.lit { position: absolute; inset-inline: var(--inv-frame-cut) 30%; top: 0; block-size: var(--stroke-hairline); background: var(--inv-lit-edge); pointer-events: none; }
.kicker { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; font-size: var(--inv-kicker-size); font-weight: var(--hud-label-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--hud-label-ink); }
[part="IdleOffline.band"] { width: stretch; flex: none; align-items: flex-start; }
[part="IdleOffline.title"] { margin: 0; text-align: start; overflow-wrap: anywhere; }
/* The time away, large, its word beside it. */
.away { flex: none; align-items: baseline; }
.figure { min-width: 0; overflow-wrap: anywhere; font-size: var(--play-figure-lg-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--ink-bright); }
.label { font-size: var(--hud-tag-size); font-weight: var(--hud-label-weight); color: var(--hud-label-ink); }
.body { overflow-y: auto; flex: 0 1 auto; min-height: 0; }
.haul { width: stretch; margin: 0; padding: 0; list-style: none; }
/* A haul socket: glowing gold from its foot, cut at two corners. */
[part="IdleOffline.haul"] { flex: 1 1 0; align-items: center; min-inline-size: calc(var(--size-xl) * 1.7); max-inline-size: calc(var(--size-xl) * 3); padding: var(--space-xs); background: var(--play-haul-fill); box-shadow: var(--inv-row-rim); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.socket { inline-size: var(--size-lg); block-size: var(--size-lg); }
.icon { width: 100%; height: 100%; filter: var(--inv-slot-art-shadow); }
.amount { max-inline-size: 100%; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; font-size: var(--play-figure-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--ink-bright); }
.name { max-inline-size: 100%; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; font-size: var(--hud-tag-size); font-weight: var(--hud-label-weight); color: var(--hud-label-ink); }
.empty-text { margin: 0; padding: var(--space-lg); text-align: center; }
.actions { width: stretch; flex: none; justify-content: flex-end; }
.actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); max-width: stretch; gap: var(--space-xs); }
/* Double: the game's gold offer. */
#Double { --button-fill: var(--play-double-fill); --button-hover-fill: var(--play-double-fill); --button-pressed-fill: var(--play-double-fill); --button-ink: var(--play-double-ink); }
.double { align-items: center; min-width: 0; max-width: stretch; }
.double-text { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; max-width: stretch; font-size: var(--hud-tag-size); }
@media (pointer: coarse) { .actions button { flex: 1; } }
@container (width < 560px) {
  .actions button { flex: 1; min-width: 0; }
  [part="IdleOffline.haul"] { min-inline-size: calc(var(--size-xl) * 1.4); }
}
</style>
