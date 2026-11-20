<meta name="component" content="IdlePrestige">
<meta name="strings" content="./idle.en.ogt">
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
  The prestige panel of an idle game: the currency in a socket glowing in
  the epic purple, what the player holds and, in that purple, what a reset
  would give; each changed bonus now and after the reset; and the
  requirement with its leaning bar, green once met, and Ready or Not yet in
  words. Prestige reports request; the game then sets confirming and the
  panel asks inline, Reset (confirm) and Cancel (cancel, or Escape and pad
  B while asking). Cancel comes first so no key resets by accident. The
  game decides whether a reset is allowed and what it gives.
-->
<section class="frame" id="IdlePrestige" part="frame" layout="stack" fill gap="loose" aria-label={title != '' ? title : t("idle.prestige.name")} data-confirming={confirming} class:met={met}>
  <div class="lit"></div>
  <header class="band" part="band" layout="stack" gap="none">
    <span part="kicker" class="kicker">{t("idle.prestige.kicker")}</span>
    <h2 class="title" part="title">{title != '' ? title : t("idle.prestige.name")}</h2>
  </header>
  <div class="body" layout="scroll" gap="none">
    <div class="body-inner" layout="stack" gap="loose">
      <div class="totals" layout="row" gap="loose">
        <div part="socket" class="socket cover" layout="stack" fill><og-icon class="currency-icon" icon={icon}></og-icon></div>
        <div class="total" layout="stack" gap="none" fill>
          <span class="label" part="label">{t("idle.prestige.held", { currency: currency })}</span>
          <span part="figure" class="figure">{held}</span>
        </div>
        <div class="total gain" layout="stack" gap="none">
          <span class="label" part="label">{t("idle.prestige.gain")}</span>
          <span part="figure" class="figure" class:tag-gain={true}>{gain}</span>
        </div>
      </div>
      {#if rows.length}
      <ul class="rows" layout="stack" gap="hair" aria-label={t("idle.prestige.bonus")}>
        {#each rows as r (r.id)}
          <li class="row" part="row" layout="row" gap="tight" aria-label={t("idle.prestige.rowName", { label: r.label, now: r.now, after: r.after })}>
            <og-icon class="row-icon" icon={r.icon}></og-icon>
            <span part="name" class="name" fill>{r.label}</span>
            <div class="readouts" layout="row" gap="tight">
              <span part="readout" class="readout">{r.now}</span>
              <og-icon class="row-arrow" icon="play"></og-icon>
              <span part="readout" class="readout" class:tag-after={true}>{r.after}</span>
            </div>
          </li>
        {/each}
      </ul>
      {/if}
      {#if requirement != ''}
      <div class="requirement" part="requirement" layout="stack" gap="tight" class:met={met}>
        <div class="req-line" layout="row" gap="tight">
          <span class="req-text" fill>{requirement}</span>
          <div part="tag" class="tag" layout="row" gap="hair" class:met={met}><og-icon class="req-mark" icon={met ? 'check' : 'lock'}></og-icon><span>{met ? t("idle.prestige.met") : t("idle.prestige.notMet")}</span></div>
        </div>
        {#if progress >= 0}
        <div class="bar-line" layout="row" gap="tight">
          <div part="track" class="track" fill role="progressbar" aria-label={t("idle.prestige.progress")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={progress * 100} aria-valuetext={progressText != '' ? progressText : formatPercent(progress)}><div part="fill" class="fill" style:--value={progress > 1 ? 1 : progress} class:met={met}></div></div>
          <span class="progress-text">{progressText != '' ? progressText : formatPercent(progress)}</span>
        </div>
        {/if}
      </div>
      {/if}
    </div>
  </div>
  {#if confirming}
    <div class="confirm" part="confirm" layout="stack" gap="loose" role="alertdialog" aria-label={t("idle.prestige.ask", { gain: gain, currency: currency })} onback={cancel}>
      <p class="ask">{t("idle.prestige.ask", { gain: gain, currency: currency })}</p>
      <div class="actions" layout="wrap">
        <button id="Cancel" variant="ghost" onclick={cancel}><span>{t("idle.prestige.cancel")}</span></button>
        <button id="Confirm" variant="danger" onclick={confirm}><span>{t("idle.prestige.confirm")}</span></button>
      </div>
    </div>
  {:else}
    <div class="actions" layout="wrap">
      <button id="Request" class="go" variant="primary" disabled={!met} onclick={request}><og-icon class="button-icon" icon={icon}></og-icon><span>{t("idle.prestige.request")}</span></button>
    </div>
  {/if}
</section>

<style>
.cover { justify-content: center; align-items: center; }
#IdlePrestige { inline-size: stretch; block-size: auto; min-inline-size: 0; max-block-size: 100%; min-block-size: 0; }
.frame { position: relative; border-radius: var(--inv-frame-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.lit { position: absolute; inset-inline: var(--inv-frame-cut) 30%; top: 0; block-size: var(--stroke-hairline); background: var(--inv-lit-edge); pointer-events: none; }
.kicker { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; font-size: var(--inv-kicker-size); font-weight: var(--hud-label-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--hud-label-ink); }
.band { width: stretch; flex: none; align-items: flex-start; }
.title { margin: 0; max-inline-size: 100%; overflow-wrap: anywhere; }
.body { overflow-y: auto; flex: 0 1 auto; min-height: 0; }
.body-inner { width: stretch; }
/* What the player holds and what a reset gives, the gain in epic purple. */
.totals { width: stretch; align-items: center; }
.socket { flex: none; inline-size: var(--size-xl); block-size: var(--size-xl); border-radius: var(--play-socket-radius); background: var(--play-epic-glow); box-shadow: var(--play-socket-rim); }
.currency-icon { width: 70%; height: 70%; }
.total { min-width: 0; }
.gain { align-items: flex-end; max-width: 50%; }
.label { overflow-wrap: anywhere; max-inline-size: 100%; font-size: var(--hud-tag-size); font-weight: var(--hud-label-weight); color: var(--hud-label-ink); }
.figure { max-inline-size: 100%; overflow-wrap: anywhere; font-size: var(--play-figure-lg-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--ink-bright); }
.figure.tag-gain { text-align: end; color: var(--play-epic-ink); }
.rows { width: stretch; margin: 0; padding: 0; list-style: none; }
.row { width: stretch; align-items: center; padding: var(--space-xxs) var(--space-sm); min-height: var(--size-md); background: var(--inv-row-fill); box-shadow: var(--inv-row-rim); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.row-icon { width: var(--size-sm); height: var(--size-sm); flex: none; }
.name { flex: 1; min-width: 0; text-align: start; overflow-wrap: anywhere; font-weight: var(--inv-name-weight); color: var(--ink-bright); }
.readouts { flex: none; justify-content: flex-end; align-items: center; }
.readout { white-space: nowrap; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-label-ink); }
.readout.tag-after { font-size: var(--play-figure-size); color: var(--play-epic-ink); }
.row-arrow { width: var(--size-xs); height: var(--size-xs); flex: none; color: var(--hud-label-ink); }
.row-arrow:dir(rtl) { scale: -1 1; }
/* The requirement in a well: its words, Ready or Not yet on a tag, its bar. */
.requirement { padding: var(--space-sm); background: var(--play-well-fill); box-shadow: var(--play-well-rim); }
.req-line, .bar-line { width: stretch; align-items: center; }
.req-text { flex: 1; min-width: 0; text-align: start; overflow-wrap: anywhere; color: var(--ink-bright); }
.tag { flex: none; align-items: center; padding: 0 var(--space-xs); white-space: nowrap; font-size: var(--hud-tag-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); background: var(--hud-tag-fill); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.tag.met { color: var(--play-auto-ink); background: var(--play-auto-fill); }
.req-mark { width: calc(var(--size-xs) * 0.75); height: calc(var(--size-xs) * 0.75); flex: none; }
.track { position: relative; min-width: var(--size-lg); block-size: calc(10 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
.fill { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--play-cycle-fill); }
.fill.met { background: var(--play-ready-fill); }
.progress-text { flex: none; max-width: 50%; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-label-ink); }
.confirm { flex: none; padding: var(--space-md); background: var(--play-well-fill); box-shadow: var(--play-low-rim); }
.ask { margin: 0; overflow-wrap: anywhere; color: var(--ink-bright); }
.actions { width: stretch; flex: none; justify-content: flex-end; }
.actions button { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); gap: var(--space-xs); }
/* Ascend: washed in the epic purple. */
.button-icon { width: var(--size-xs); height: var(--size-xs); flex: none; }
@media (pointer: coarse) { .actions button { flex: 1; } }
/* A narrow panel (a phone, or a side column on a wide screen) wraps each
   row's figures under its name; the actions share the width. */
@container (width < 560px) {
  .row { flex-wrap: wrap; padding: var(--space-xxs) var(--space-xs); }
  .readouts { flex: 1 0 100%; }
  .actions button { flex: 1; min-width: 0; }
}
</style>
