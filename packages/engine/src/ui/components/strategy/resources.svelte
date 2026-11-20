<meta name="component" content="StrategyResources">
<meta name="strings" content="./strategy.en.ogt">
<meta name="prop:resources" content="list" data-doc="The stockpiles in the game's order, keyed rows of { id, name, icon, glyph, amount, cap, rate, trend, note, warn }: name for a reader (Gold); icon its picture (og.load.texture), 0 draws glyph (an icon of the atlas) instead; amount the store as the game words it (1,250); cap a ceiling shown after it (/ 50, the population's), empty none; rate the income as the game words it (+12/s), under the amount, inked by trend (gain, loss or none: data-trend); note a word in place of the rate (Near cap), empty none; warn marks a store at its limit in gold (data-warn).">
<meta name="prop:tier" content="string" data-doc="The age or tier as the game words it (Castle Age), on a lit tag after the stockpiles; empty hides it.">
<meta name="fixture:default" content='{"tier":"","resources":[{"id":"gold","name":"Gold","icon":0,"glyph":"coin","amount":"1,250","cap":"","rate":"+12/s","trend":"gain","note":"","warn":false},{"id":"wood","name":"Wood","icon":0,"glyph":"leaf","amount":"840","cap":"","rate":"+6/s","trend":"gain","note":"","warn":false},{"id":"food","name":"Food","icon":0,"glyph":"apple","amount":"615","cap":"","rate":"−2/s","trend":"loss","note":"","warn":false},{"id":"crystal","name":"Crystal","icon":0,"glyph":"gem","amount":"90","cap":"","rate":"+0/s","trend":"none","note":"","warn":false},{"id":"pop","name":"Population","icon":0,"glyph":"person","amount":"47","cap":"50","rate":"","trend":"none","note":"Near cap","warn":true}]}'>
<meta name="fixture:tier" content='{"tier":"Castle Age","resources":[{"id":"food","name":"Food","icon":0,"glyph":"apple","amount":"2,410","cap":"","rate":"+31/s","trend":"gain","note":"","warn":false},{"id":"wood","name":"Wood","icon":0,"glyph":"leaf","amount":"1,180","cap":"","rate":"+18/s","trend":"gain","note":"","warn":false},{"id":"gold","name":"Gold","icon":0,"glyph":"coin","amount":"960","cap":"","rate":"+9/s","trend":"gain","note":"","warn":false},{"id":"stone","name":"Stone","icon":0,"glyph":"shield","amount":"350","cap":"","rate":"+2/s","trend":"gain","note":"","warn":false},{"id":"pop","name":"Population","icon":0,"glyph":"person","amount":"112","cap":"200","rate":"","trend":"none","note":"","warn":false}]}'>
<meta name="fixture:capped" content='{"tier":"","resources":[{"id":"gold","name":"Gold","icon":0,"glyph":"coin","amount":"0","cap":"","rate":"−8/s","trend":"loss","note":"","warn":true},{"id":"pop","name":"Population","icon":0,"glyph":"person","amount":"50","cap":"50","rate":"","trend":"none","note":"Housed out","warn":true}]}'>
<meta name="fixture:empty" content='{"tier":"","resources":[]}'>
<meta name="fixture:overflow" content='{"tier":"Imperial Age of the Unconquered Sun","resources":[{"id":"gold","name":"Gold","icon":0,"glyph":"coin","amount":"1,284,000,000","cap":"","rate":"+125,000/s","trend":"gain","note":"","warn":false},{"id":"wood","name":"Wood","icon":0,"glyph":"leaf","amount":"98,765,432","cap":"","rate":"−4,500/s","trend":"loss","note":"","warn":false},{"id":"food","name":"Food","icon":0,"glyph":"apple","amount":"12,000,000","cap":"","rate":"+0/s","trend":"none","note":"","warn":false},{"id":"crystal","name":"Crystal","icon":0,"glyph":"gem","amount":"4,096","cap":"","rate":"+64/s","trend":"gain","note":"","warn":false},{"id":"stone","name":"Stone","icon":0,"glyph":"shield","amount":"55,555","cap":"","rate":"+1/s","trend":"gain","note":"","warn":false},{"id":"favour","name":"Favour","icon":0,"glyph":"star","amount":"999","cap":"","rate":"","trend":"none","note":"Storage full","warn":true},{"id":"pop","name":"Population","icon":0,"glyph":"person","amount":"1,000","cap":"1,000","rate":"","trend":"none","note":"Housed out","warn":true}]}'>

<!--
  The resource bar at the top left: each stockpile's picture in a cut tile,
  its amount large (a ceiling after it, dim, as the population's), and its
  income under it, green while it grows and red while it drains, or the
  game's word (Near cap) in gold at a limit; the age or tier on a lit tag.
  The game owns every figure, rate and limit; read only.
-->
<section id="StrategyResources" part="frame" variant="hud" role="group" aria-label={t("strategy.resources.name")}>
  {#each resources as r (r.id)}
    <div part="resource" class="resource" role="group" aria-label={t("strategy.resources.value", { name: r.name, amount: r.amount })} class:warn={r.warn}>
      {#if r.icon}<img part="picture" class="picture" src={r.icon} alt="">{:else}<og-icon part="picture" class="glyph" icon={r.glyph}></og-icon>{/if}
      <div class="figures">
        <div class="line"><span part="amount" class="amount">{r.amount}</span>{#if r.cap}<span part="cap" class="cap">{t("strategy.resources.cap", { cap: r.cap })}</span>{/if}</div>
        {#if r.note}<span part="rate" class="rate" trend="note" class:trend-note={true}>{r.note}</span>{:else if r.rate}<span part="rate" class="rate" trend={r.trend} class:trend-gain={(r.trend) == 'gain'} class:trend-loss={(r.trend) == 'loss'} class:trend-note={(r.trend) == 'note'}>{r.rate}</span>{/if}
      </div>
    </div>
  {/each}
  {#if tier != ''}<span part="tier" class="tier">{tier}</span>{/if}
  {#if !resources.length && tier == ''}<span class="empty" role="status">{t("strategy.resources.empty")}</span>{/if}
</section>

<style>
/* Smoked glass with a lit top edge and two corners cut; the stockpiles in
   a row that wraps to a second when its room is short. */
#StrategyResources { display: flex; flex-wrap: wrap; align-items: center; column-gap: calc(22 * var(--font-step-px)); row-gap: calc(6 * var(--font-step-px)); inline-size: fit-content; max-inline-size: stretch; min-inline-size: 0; padding: calc(6 * var(--font-step-px)) calc(16 * var(--font-step-px)); color: var(--hud-ink); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.resource { display: flex; align-items: center; column-gap: calc(7 * var(--font-step-px)); flex: 0 1 auto; min-inline-size: 0; }
/* The pictures are art: they keep their size at any text scale. */
.picture, .glyph { flex: none; inline-size: var(--size-sm); block-size: var(--size-sm); }
.picture { object-fit: cover; border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.glyph { color: var(--hud-ink); filter: var(--hud-glyph-shadow); }
.figures { display: flex; flex-direction: column; min-inline-size: 0; }
.line { display: flex; align-items: baseline; column-gap: calc(3 * var(--font-step-px)); min-inline-size: 0; }
.amount { min-inline-size: 0; font-size: var(--hud-size-readout); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; text-shadow: var(--hud-shadow); }
.cap { flex: none; font-size: var(--hud-size-xs); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; color: var(--hud-ink-dim); }
.rate { min-inline-size: 0; font-size: var(--hud-tag-size); line-height: 1.2; font-weight: var(--hud-label-weight); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; color: var(--hud-ink-dim); }
.rate.trend-gain { color: var(--hud-boon); }
.rate.trend-loss { color: var(--hud-harm); }
.rate.trend-note { color: var(--hud-gold); }
.warn .amount { color: var(--hud-gold); }
.tier { flex: 0 1 auto; min-inline-size: 0; padding: calc(2 * var(--font-step-px)) calc(8 * var(--font-step-px)); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; color: var(--hud-accent-ink); background: var(--hud-accent); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.empty { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; }
/* On a phone the bar takes the width: the stockpiles spread across one
   row, or two at a large text scale, smaller. */
@media (width < 600px) {
  #StrategyResources { inline-size: stretch; justify-content: space-between; column-gap: calc(8 * var(--font-step-px)); row-gap: calc(4 * var(--font-step-px)); padding: calc(4 * var(--font-step-px)) calc(10 * var(--font-step-px)); }
  .resource { column-gap: calc(4 * var(--font-step-px)); }
  .picture, .glyph { inline-size: var(--size-xs); block-size: var(--size-xs); }
  .amount { font-size: var(--hud-size-readout-compact); }
  .cap { font-size: var(--hud-size-label-compact); }
  .rate { font-size: var(--hud-size-label-compact); }
}
</style>
