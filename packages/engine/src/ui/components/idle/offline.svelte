<meta name="component" content="IdleOffline">
<meta name="strings" content="./idle.en.ogt">
<meta name="parts" content="IdleOffline.frame IdleOffline.band IdleOffline.title IdleOffline.summary IdleOffline.row IdleOffline.name IdleOffline.figure IdleOffline.label IdleOffline.caption">
<meta name="prop:title" content="string" data-doc="The panel's heading; empty takes the group's (Welcome back).">
<meta name="prop:away" content="string" data-doc="How long the player was away, as the game words it (it wins; empty shows seconds as a duration: 1h 2m).">
<meta name="prop:seconds" content="number" data-doc="How long the player was away, in seconds, when away is empty.">
<meta name="prop:earned" content="list" data-doc="What was gathered while away, keyed rows of { id, icon, name, amount }: amount as the game formats it (+12.4K).">
<meta name="prop:canDouble" content="bool" data-doc="The game offers to double the haul (an ad, a gem); false hides Double.">
<meta name="prop:doubleText" content="string" data-doc="What doubling costs or asks, as the game words it (Watch an ad); empty is none.">
<meta name="fixture:default" content='{"title":"","away":"","seconds":13530,"earned":[{"id":"gold","icon":"coin","name":"Gold","amount":"+1.24M"},{"id":"wood","icon":"leaf","name":"Wood","amount":"+5,000"},{"id":"gems","icon":"gem","name":"Gems","amount":"+3"}],"canDouble":true,"doubleText":"Watch an ad"}'>
<meta name="fixture:disabled" content='{"title":"","away":"8h (capped)","seconds":0,"earned":[{"id":"gold","icon":"coin","name":"Gold","amount":"+880K"}],"canDouble":false,"doubleText":""}'>
<meta name="fixture:empty" content='{"title":"","away":"","seconds":42,"earned":[],"canDouble":false,"doubleText":""}'>
<meta name="fixture:overflow" content='{"title":"While You Slumbered, the Clockwork Empire Kept Turning","away":"3 days, 14 hours, 52 minutes and 9 seconds (capped at 72 hours)","seconds":0,"earned":[{"id":"gold","icon":"coin","name":"Gold Sovereigns of the Eastern Treasury","amount":"+123.45 quattuordecillion"},{"id":"wood","icon":"leaf","name":"Wood","amount":"+5,000"},{"id":"gems","icon":"gem","name":"Gems","amount":"+99,999"},{"id":"stone","icon":"shield","name":"Stone","amount":"+48K"},{"id":"mana","icon":"drop","name":"Mana","amount":"+250"},{"id":"souls","icon":"skull","name":"Souls","amount":"+7"},{"id":"stars","icon":"star","name":"Stardust","amount":"+1.5B"}],"canDouble":true,"doubleText":"Spend 50 gems to double everything gathered"}'>

<!--
  The welcome-back panel of an idle game: how long the player was away
  (the game's words, or its seconds as a duration) and what was gathered,
  one row per currency. Collect reports collect; Double, offered only when
  the game sets canDouble, reports double. The game computes the haul and
  applies it.
-->
<section id="IdleOffline" part="IdleOffline.frame" data-layout="stack" data-gap="wide" aria-label={title != '' ? title : t("idle.offline.name")}>
  <header part="IdleOffline.band" data-layout="row"><h2 part="IdleOffline.title">{title != '' ? title : t("idle.offline.name")}</h2></header>
  <p part="IdleOffline.summary IdleOffline.label IdleOffline.caption">{t("idle.offline.away", { time: away != '' ? away : formatDuration(seconds) })}</p>
  {#if earned.length}
    <div class="body" data-layout="scroll" data-gap="none">
      <ul class="rows" data-layout="stack" data-gap="hair" aria-label={t("idle.offline.earned")}>
        {#each earned as e (e.id)}
          <li part="IdleOffline.row" data-layout="cluster" aria-label={e.name}>
            <og-icon class="icon" icon={e.icon}></og-icon>
            <span part="IdleOffline.name" class="name" data-fill>{e.name}</span>
            <span part="IdleOffline.figure" class="figure">{e.amount}</span>
          </li>
        {/each}
      </ul>
    </div>
  {:else}
    <p class="empty-text" part="IdleOffline.caption" role="status">{t("idle.offline.empty")}</p>
  {/if}
  <footer class="actions" data-layout="cluster">
    {#if canDouble}<button id="Double" variant="secondary" onclick={double}><div class="double" data-layout="stack" data-gap="none"><span>{t("idle.offline.double")}</span>{#if doubleText != ''}<span class="double-text" part="IdleOffline.caption">{doubleText}</span>{/if}</div></button>{/if}
    <button id="Collect" variant="primary" onclick={collect}><span>{t("idle.offline.collect")}</span></button>
  </footer>
</section>

<style>
:host { inline-size: stretch; min-inline-size: 0; max-inline-size: var(--modal-width); max-block-size: 100%; min-block-size: 0; }
[part="IdleOffline.band"] { width: stretch; justify-content: center; flex: none; }
[part="IdleOffline.title"] { margin: 0; text-align: center; overflow-wrap: anywhere; }
[part~="IdleOffline.summary"] { margin: 0; text-align: center; flex: none; overflow-wrap: anywhere; }
.body { overflow-y: auto; flex: 0 1 auto; min-height: 0; }
.rows { width: stretch; margin: 0; padding: 0; list-style: none; }
[part="IdleOffline.row"] { padding: var(--space-sm); min-height: var(--size-touch); }
.icon { width: var(--size-sm); height: var(--size-sm); flex: none; }
[part="IdleOffline.name"] { flex: 1 1 0; min-width: calc(var(--size-xxl) * 1.5); text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
[part="IdleOffline.figure"] { --figure-size: var(--type-lg-size); --figure-weight: var(--font-weight-semibold); flex: 0 1 auto; min-width: 0; max-width: stretch; margin-inline-start: auto; text-align: end; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.empty-text { margin: 0; padding: var(--space-lg); text-align: center; }
.actions { width: stretch; flex: none; justify-content: flex-end; }
.actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); max-width: stretch; gap: var(--space-xs); }
.double { align-items: center; min-width: 0; max-width: stretch; }
.double-text { --caption-ink: currentColor; --caption-text: var(--text-xs); white-space: nowrap; text-overflow: ellipsis; overflow: hidden; max-width: stretch; }
@media (pointer: coarse) { .actions button { flex: 1; } }
@media (width < 600px) { .actions button { flex: 1; min-width: 0; } }
</style>
