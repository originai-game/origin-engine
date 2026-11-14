<meta name="component" content="IdlePrestige">
<meta name="strings" content="./idle.en.ogt">
<meta name="parts" content="IdlePrestige.frame IdlePrestige.band IdlePrestige.title IdlePrestige.figure IdlePrestige.row IdlePrestige.name IdlePrestige.cell IdlePrestige.requirement IdlePrestige.badge IdlePrestige.confirm">
<meta name="prop:title" content="string" data-doc="The panel's heading; empty takes the group's (Prestige).">
<meta name="prop:currency" content="string" data-doc="The prestige currency's name (Souls).">
<meta name="prop:icon" content="string" data-doc="The prestige currency's icon.">
<meta name="prop:held" content="string" data-doc="How much of it the player holds now, as the game formats it.">
<meta name="prop:gain" content="string" data-doc="How much a reset now would give, as the game formats it.">
<meta name="prop:rows" content="list" data-doc="What changes, keyed rows of { id, icon, label, now, after }: now the value before the reset and after the value it would be after, both as the game words them.">
<meta name="prop:requirement" content="string" data-doc="What a reset needs, as the game words it (Reach 1M gold); empty hides it.">
<meta name="prop:progress" content="number" data-default="-1" data-doc="Toward the requirement, 0 to 1, for its bar; negative hides the bar.">
<meta name="prop:progressText" content="string" data-doc="That progress in words (640K / 1M); empty shows the percent.">
<meta name="prop:met" content="bool" data-doc="The requirement is met: Prestige is enabled and the requirement reads Ready (else Not yet).">
<meta name="prop:confirming" content="bool" data-doc="The game is asking the player to confirm (set after request): Reset and Cancel stand in for Prestige.">
<meta name="action:back" content="Cancel">
<meta name="fixture:default" content='{"title":"","currency":"Souls","icon":"skull","held":"12","gain":"+48","rows":[{"id":"income","icon":"coin","label":"Income bonus","now":"×1.12","after":"×1.60"},{"id":"speed","icon":"bolt","label":"Cycle speed","now":"+5%","after":"+24%"}],"requirement":"Reach 1M gold","progress":1,"progressText":"1.2M / 1M","met":true,"confirming":false}'>
<meta name="fixture:disabled" content='{"title":"","currency":"Souls","icon":"skull","held":"0","gain":"+0","rows":[{"id":"income","icon":"coin","label":"Income bonus","now":"×1.00","after":"×1.00"}],"requirement":"Reach 1M gold","progress":0.64,"progressText":"640K / 1M","met":false,"confirming":false}'>
<meta name="fixture:confirming" content='{"title":"","currency":"Souls","icon":"skull","held":"12","gain":"+48","rows":[{"id":"income","icon":"coin","label":"Income bonus","now":"×1.12","after":"×1.60"},{"id":"speed","icon":"bolt","label":"Cycle speed","now":"+5%","after":"+24%"}],"requirement":"Reach 1M gold","progress":1,"progressText":"1.2M / 1M","met":true,"confirming":true}'>
<meta name="fixture:empty" content='{"title":"","currency":"Souls","icon":"skull","held":"0","gain":"+1","rows":[],"requirement":"","progress":-1,"progressText":"","met":true,"confirming":false}'>
<meta name="fixture:overflow" content='{"title":"Ascension Beyond the Veil of the Seventh Celestial Sphere","currency":"Fragments of Forgotten Eternity","icon":"star","held":"1.234 quattuordecillion","gain":"+98.76 quindecillion","rows":[{"id":"r1","icon":"coin","label":"Multiplier applied to every single generator in every realm","now":"×1,234,567","after":"×98,765,432,100"},{"id":"r2","icon":"bolt","label":"Speed","now":"+5%","after":"+24%"},{"id":"r3","icon":"gem","label":"Gem chance","now":"1%","after":"3%"},{"id":"r4","icon":"heart","label":"Offline cap","now":"8h","after":"12h"},{"id":"r5","icon":"crown","label":"Starting gold","now":"0","after":"1M"}],"requirement":"Reach one undecillion gold in a single run without prestiging","progress":0.42,"progressText":"420 decillion / 1 undecillion","met":false,"confirming":false}'>

<!--
  The prestige panel of an idle game: what the player holds and what a
  reset would give, each changed bonus now and after the reset, and the
  requirement with its bar and Ready or Not yet in words. Prestige reports
  request; the game then sets confirming and the panel asks inline, Reset
  (confirm) and Cancel (cancel, or Escape and pad B while asking). Cancel
  comes first so no key resets by accident. The game decides whether a
  reset is allowed and what it gives.
-->
<section id="IdlePrestige" part="IdlePrestige.frame" aria-label={title != '' ? title : t("idle.prestige.name")} data-met={met} data-confirming={confirming}>
  <header part="IdlePrestige.band"><h2 part="IdlePrestige.title">{title != '' ? title : t("idle.prestige.name")}</h2></header>
  <scroll class="body">
    <div class="body-inner">
      <div class="totals">
        <og-icon class="currency-icon" icon={icon}></og-icon>
        <div class="total">
          <span class="caption">{t("idle.prestige.now")}</span>
          <span part="IdlePrestige.figure" data-tag="held">{held}</span>
        </div>
        <og-icon class="arrow" icon="play"></og-icon>
        <div class="total">
          <span class="caption">{t("idle.prestige.gain")}</span>
          <span part="IdlePrestige.figure" data-tag="gain">{gain}</span>
        </div>
        <span class="currency">{currency}</span>
      </div>
      <ul class="rows" aria-label={t("idle.prestige.bonus")} hidden={!rows.length}>
        {#each rows as r (r.id)}
          <li part="IdlePrestige.row" aria-label={r.label}>
            <og-icon class="row-icon" icon={r.icon}></og-icon>
            <span part="IdlePrestige.name" class="name">{r.label}</span>
            <span part="IdlePrestige.cell" data-tag="now">{r.now}</span>
            <og-icon class="row-arrow" icon="play"></og-icon>
            <span part="IdlePrestige.cell" data-tag="after">{r.after}</span>
          </li>
        {/each}
      </ul>
      <div part="IdlePrestige.requirement" data-met={met} hidden={requirement == ''}>
        <div class="req-line">
          <og-icon class="req-mark" icon={met ? 'check' : 'lock'}></og-icon>
          <span class="req-text">{requirement}</span>
          <span part="IdlePrestige.badge" data-met={met}>{met ? t("idle.prestige.met") : t("idle.prestige.notMet")}</span>
        </div>
        <div class="bar" role="group" aria-label={t("idle.prestige.progress")} hidden={progress < 0}><ProgressBar value={progress} buffer={0} label="" valueText={progressText} showValue={true} indeterminate={false} tone={met ? 'success' : 'neutral'} /></div>
      </div>
    </div>
  </scroll>
  {#if confirming}
    <div part="IdlePrestige.confirm" role="alertdialog" aria-label={t("idle.prestige.ask", { gain: gain, currency: currency })} onback={cancel}>
      <p class="ask">{t("idle.prestige.ask", { gain: gain, currency: currency })}</p>
      <div class="actions">
        <button id="Cancel" variant="ghost" onclick={cancel}><span>{t("idle.prestige.cancel")}</span></button>
        <button id="Confirm" variant="danger" onclick={confirm}><span>{t("idle.prestige.confirm")}</span></button>
      </div>
    </div>
  {:else}
    <div class="actions">
      <button id="Request" variant="primary" disabled={!met} onclick={request}><og-icon class="button-icon" icon={icon}></og-icon><span>{t("idle.prestige.request")}</span></button>
    </div>
  {/if}
</section>

<style>
[part="IdlePrestige.frame"][hidden], [part="IdlePrestige.frame"] [hidden] { display: none; }
[part="IdlePrestige.frame"] { display: flex; flex-direction: column; width: stretch; max-height: 100%; min-height: 0; gap: var(--space-md); container-type: inline-size; }
[part="IdlePrestige.band"] { display: flex; width: stretch; align-items: center; flex: none; }
[part="IdlePrestige.title"] { margin: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.body { width: stretch; flex: 0 1 auto; min-height: 0; }
.body-inner { display: flex; flex-direction: column; width: stretch; gap: var(--space-md); }
.totals { display: flex; width: stretch; align-items: center; flex-wrap: wrap; gap: var(--space-xs) var(--space-md); }
.currency-icon { width: var(--size-lg); height: var(--size-lg); flex: none; }
.total { display: flex; flex-direction: column; min-width: 0; }
.caption { white-space: nowrap; }
[part="IdlePrestige.figure"] { font-variant-numeric: tabular-nums; overflow-wrap: anywhere; }
.arrow, .row-arrow { width: var(--size-xs); height: var(--size-xs); flex: none; }
.arrow:dir(rtl), .row-arrow:dir(rtl) { scale: -1 1; }
.currency { min-width: 0; overflow-wrap: anywhere; }
.rows { display: flex; flex-direction: column; width: stretch; margin: 0; padding: 0; list-style: none; gap: var(--space-xxs); }
[part="IdlePrestige.row"] { display: flex; width: stretch; align-items: center; gap: var(--space-sm); padding: var(--space-xxs) var(--space-sm); min-height: var(--size-md); }
.row-icon { width: var(--size-sm); height: var(--size-sm); flex: none; }
[part="IdlePrestige.name"] { flex: 1; min-width: 0; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
[part="IdlePrestige.cell"] { flex: none; white-space: nowrap; font-variant-numeric: tabular-nums; }
[part="IdlePrestige.requirement"] { display: flex; flex-direction: column; width: stretch; gap: var(--space-xs); padding: var(--space-xs) var(--space-sm); }
.req-line { display: flex; width: stretch; align-items: center; gap: var(--space-xs); }
.req-mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
.req-text { flex: 1; min-width: 0; text-align: start; overflow-wrap: anywhere; }
[part="IdlePrestige.badge"] { flex: none; white-space: nowrap; padding: 0 var(--space-xs); }
.bar { width: stretch; }
[part="IdlePrestige.confirm"] { display: flex; flex-direction: column; width: stretch; flex: none; gap: var(--space-sm); padding: var(--space-sm); }
.ask { margin: 0; overflow-wrap: anywhere; }
.actions { display: flex; width: stretch; flex: none; justify-content: flex-end; flex-wrap: wrap; gap: var(--space-sm); }
.actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); gap: var(--space-xs); }
.button-icon { width: var(--size-xs); height: var(--size-xs); flex: none; }
@media (pointer: coarse) { .actions button { flex: 1; } }
/* A narrow panel (a phone, or a side column on a wide screen) wraps each
   row's figures under its name; the actions share the width. */
@container (width < 560px) {
  [part="IdlePrestige.row"] { flex-wrap: wrap; gap: 0 var(--space-xs); padding: var(--space-xxs) var(--space-xs); }
  [part="IdlePrestige.name"] { flex-basis: 60%; }
  .actions button { flex: 1; min-width: 0; }
}
</style>
