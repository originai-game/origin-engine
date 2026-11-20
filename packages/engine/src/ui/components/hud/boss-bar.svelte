<meta name="component" content="BossBar">
<meta name="strings" content="./boss-bar.en.ogt">
<meta name="parts" content="BossBar.frame BossBar.kind BossBar.boss BossBar.epithet BossBar.health BossBar.ghost BossBar.threshold BossBar.stagger BossBar.weakness BossBar.share BossBar.state">
<meta name="prop:name" content="string" data-default="" data-doc="The boss name; empty uses Boss.">
<meta name="prop:epithet" content="string" data-default="" data-doc="The boss's title as the game words it (Warden of the Ninefold Gate), beside the name; empty hides it.">
<meta name="prop:value" content="number" data-default="0" data-doc="Health, clamped to the supplied range for display; its share of max stands in figures at the bar's end.">
<meta name="prop:max" content="number" data-default="1" data-doc="Full health; a nonpositive capacity shows no health instead of dividing by zero.">
<meta name="prop:ghost" content="number" data-default="0" data-doc="Health before the latest loss: a warm ghost behind the fill that drains after it (at or under value: none).">
<meta name="prop:phases" content="list" data-doc="Keyed phases { id, label, complete } in the fight's order; the game owns phase progression. A diamond on the bar marks each boundary between phases (the first phase is the bar's right end), dimmed once health has passed it.">
<meta name="prop:phase" content="string" data-default="" data-doc="The current phase's stable id; read out with the health.">
<meta name="prop:stagger" content="number" data-default="-1" data-doc="The stagger gauge's fill, 0 to 1 (the boss breaks at 1); negative hides the gauge.">
<meta name="prop:staggerCells" content="number" data-default="10" data-doc="Cells the stagger gauge is cut into (10).">
<meta name="prop:weaknesses" content="list" data-doc="The elements the boss is weak to, keyed rows of { id, name, icon, glyph }: icon a texture (0: the glyph, an icon name, in its place); name for a reader.">
<meta name="prop:invulnerable" content="bool" data-default="false" data-doc="Health remains visible, greyed, and a worded invulnerability state is shown.">
<meta name="prop:defeated" content="bool" data-default="false" data-doc="The game has confirmed defeat; this does not follow automatically from zero health.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Waiting for boss data; hides stale health and phases.">
<meta name="fixture:default" content='{"name":"Ashen Colossus","epithet":"Warden of the Ninefold Gate","value":5800,"max":10000,"ghost":7100,"phases":[{"id":"shell","label":"Shell","complete":true},{"id":"storm","label":"Storm","complete":false},{"id":"core","label":"Core","complete":false}],"phase":"storm","stagger":0.7,"staggerCells":10,"weaknesses":[{"id":"fire","name":"Fire","icon":0,"glyph":"fire"},{"id":"frost","name":"Frost","icon":0,"glyph":"drop"}],"invulnerable":false,"defeated":false,"loading":false}'>
<meta name="fixture:critical" content='{"name":"Ashen Colossus","epithet":"","value":900,"max":10000,"ghost":1600,"phases":[],"phase":"","stagger":0,"staggerCells":10,"weaknesses":[]}'>
<meta name="fixture:invulnerable" content='{"name":"The Glass Warden","value":4000,"max":10000,"invulnerable":true}'>
<meta name="fixture:defeated" content='{"name":"The Glass Warden","value":0,"max":10000,"defeated":true}'>
<meta name="fixture:loading" content='{"name":"Unknown presence","loading":true}'>
<meta name="fixture:overflow" content='{"name":"The Warden of the Far Northern Observatory and the Drowned Abbey","epithet":"Keeper of the Seventh Bell beneath the Mere, Last of the Tidebound Choir","value":1,"max":10,"ghost":9,"phases":[{"id":"a","label":"A","complete":true},{"id":"b","label":"B","complete":true},{"id":"c","label":"C","complete":true},{"id":"last","label":"The final defence of the observatory","complete":false}],"phase":"last","stagger":1,"staggerCells":40,"weaknesses":[{"id":"a","name":"A","icon":0,"glyph":"fire"},{"id":"b","name":"B","icon":0,"glyph":"drop"},{"id":"c","name":"C","icon":0,"glyph":"bolt"},{"id":"d","name":"D","icon":0,"glyph":"star"},{"id":"e","name":"E","icon":0,"glyph":"skull"}]}'>

<!--
  A boss's bar across the top of the screen: a Boss tag, the name and its
  epithet; a long leaning health bar with a warm ghost behind the fill and
  a diamond at each phase boundary; under it the stagger gauge cut into
  cells, the elements the boss is weak to and its health's share. The game
  owns every value; read only.
-->
<section id="BossBar" part="BossBar.frame" class="hud bare" data-layout="stack" data-gap="none" aria-label={name || t("hud.boss-bar.name")} aria-busy={loading} class:invulnerable={invulnerable && !defeated} class:defeated={defeated} class:critical={!loading && !defeated && max > 0 && value / max <= 0.25}>
  <div id="Heading" data-layout="row" data-gap="tight">
    <div part="BossBar.kind" class="kind" data-layout="row" data-gap="tight"><div class="gem"></div><span>{t("hud.boss-bar.tag")}</span></div>
    <span part="BossBar.boss" class="boss">{name || t("hud.boss-bar.name")}</span>
    {#if epithet != ''}<span part="BossBar.epithet" class="epithet">{epithet}</span>{/if}
  </div>
  {#if loading}<div class="wait" data-layout="row" data-gap="tight"><Spinner size={18} name={t("hud.boss-bar.loading")} /><span part="BossBar.state" class="state" role="status">{t("hud.boss-bar.loading")}</span></div>
  {:else}
    <div id="Track" data-layout="overlay">
      <div id="Health" part="BossBar.health" data-anchor="fill" data-layout="overlay" role="meter" aria-label={t("hud.boss-bar.health")} aria-valuemin="0" aria-valuemax={max > 0 ? max : 1} aria-valuenow={max <= 0 || value < 0 ? 0 : value > max ? max : value} aria-valuetext={t("hud.boss-bar.value", { value: max <= 0 || value < 0 ? 0 : value > max ? max : value, max: max > 0 ? max : 0 })} style:--value={max <= 0 || value <= 0 ? 0 : value >= max ? 1 : value / max} style:--ghost={max <= 0 || ghost <= value ? 0 : ghost >= max ? 1 : ghost / max}>
        <div id="Ghost" part="BossBar.ghost" data-anchor="top-start"></div>
        <div id="Fill" data-anchor="top-start"></div>
      </div>
      {#each phases as p, i (p.id)}{#if i > 0}<div part="BossBar.threshold" class="threshold" data-passed={max > 0 && value / max <= (phases.length - i) / phases.length} style:--at={(phases.length - i) / phases.length}></div>{/if}{/each}
    </div>
    <div id="Foot" data-layout="row">
      {#if stagger >= 0}<div id="Stagger" data-layout="row" data-gap="tight">
        <span class="label">{t("hud.boss-bar.stagger")}</span>
        <div part="BossBar.stagger" class="gauge" data-layout="overlay" role="meter" aria-label={t("hud.boss-bar.stagger")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={stagger >= 1 ? 100 : stagger * 100} style:--value={stagger >= 1 ? 1 : stagger} style:--cells={staggerCells > 0 ? staggerCells : 1}>
          <div class="lit" data-anchor="top-start"></div>
          <div class="cuts" data-anchor="fill"></div>
        </div>
      </div>{/if}
      <div id="End" data-layout="row" data-gap="tight">
        {#if defeated || invulnerable}<span part="BossBar.state" class="state" role="status">{defeated ? t("hud.boss-bar.defeated") : t("hud.boss-bar.invulnerable")}</span>{/if}
        {#each weaknesses as w (w.id)}<div part="BossBar.weakness" class="weakness" data-layout="overlay" role="image" aria-label={t("hud.boss-bar.weak", { name: w.name })}>{#if w.icon}<img class="art" data-anchor="fill" src={w.icon} alt="">{:else}<div class="blank" data-anchor="fill" data-layout="center">{#if w.glyph}<og-icon class="glyph" icon={w.glyph}></og-icon>{/if}</div>{/if}</div>{/each}
        <span part="BossBar.share" class="share">{formatPercent(max <= 0 || value <= 0 ? 0 : value >= max ? 1 : value / max)}</span>
      </div>
    </div>
  {/if}
</section>

<style>
:host { inline-size: min(100%, calc(600 * var(--font-step-px))); align-items: center; row-gap: calc(6 * var(--font-step-px)); color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; --layout-anchor-inset: 0; }
/* The heading: the tag, the name and the epithet on one line; the name
   takes the room first, the epithet scrolls. */
#Heading { align-items: baseline; justify-content: center; max-inline-size: 100%; min-inline-size: 0; column-gap: calc(10 * var(--font-step-px)); }
.kind { flex: none; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-harm); white-space: nowrap; }
.kind > .gem { flex: none; inline-size: calc(6 * var(--font-step-px)); block-size: calc(6 * var(--font-step-px)); transform: rotate(45deg); background: var(--hud-harm); }
.kind { align-items: center; column-gap: calc(5 * var(--font-step-px)); }
.boss { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-size-boss); line-height: 1.1; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.epithet { flex: 0 100 auto; min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
/* The bar leans like every HUD bar; the phase diamonds stand on it. */
#Track { position: relative; inline-size: 100%; block-size: calc(14 * var(--font-step-px)); }
#Health { overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
#Ghost { inline-size: calc(var(--ghost) * 100%); block-size: 100%; background: var(--bar-ghost); transition: var(--bar-ghost-motion); }
#Fill { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--hud-boss-fill); box-shadow: var(--hud-boss-glow); }
.threshold { position: absolute; --d: calc(9 * var(--font-step-px)); inline-size: var(--d); block-size: var(--d); left: calc(var(--at) * 100% - var(--d) / 2); top: calc(50% - var(--d) / 2); transform: rotate(45deg); background: var(--hud-tick); box-shadow: var(--hud-marker-rim); }
.threshold[data-passed=true] { opacity: 0.5; }
#Foot { align-items: center; justify-content: space-between; inline-size: 100%; min-inline-size: 0; column-gap: var(--space-md); }
#Stagger { flex: 0 1 auto; align-items: center; min-inline-size: 0; }
.label { flex: none; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; }
.gauge { flex: 0 1 auto; inline-size: calc(170 * var(--font-step-px)); min-inline-size: calc(40 * var(--font-step-px)); block-size: calc(8 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
.lit { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--hud-stagger-fill); box-shadow: var(--hud-stagger-glow); }
.cuts { background: var(--bar-notch); mask-image: repeating-linear-gradient(90deg, transparent 0 calc(100% / var(--cells) - 3 * var(--font-step-px)), #000 calc(100% / var(--cells) - 3 * var(--font-step-px)) calc(100% / var(--cells))); }
#End { flex: none; align-items: center; margin-inline-start: auto; }
.weakness { flex: none; inline-size: calc(22 * var(--font-step-px)); block-size: calc(22 * var(--font-step-px)); background: var(--hud-harm); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; --layout-anchor-inset: calc(2 * var(--font-step-px)); filter: var(--hud-lift); }
.art, .blank { border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.art { object-fit: cover; }
.blank { background: var(--hud-plate-fill); }
.glyph { inline-size: 64%; block-size: 64%; color: var(--hud-ink); }
.share { flex: none; font-size: var(--hud-size-sm); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.state { flex: none; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; }
.defeated .state { color: var(--hud-boon); }
/* Invulnerable: the fill greys; under a quarter the rim glows in danger
   and pulses. */
.invulnerable #Fill { filter: var(--hud-unready); }
.critical #Health { box-shadow: var(--bar-critical-rim); animation: var(--pulse-motion); }
@media (width < 600px), (height < 500px) {
  :host { row-gap: calc(4 * var(--font-step-px)); }
  .boss { font-size: var(--hud-size-boss-compact); }
  .epithet { display: none; }
  #Track { block-size: calc(9 * var(--font-step-px)); }
  .threshold { --d: calc(7 * var(--font-step-px)); }
  .gauge { block-size: calc(5 * var(--font-step-px)); }
  .weakness { inline-size: calc(16 * var(--font-step-px)); block-size: calc(16 * var(--font-step-px)); }
  .share { font-size: var(--hud-size-xs); }
  .label, .kind { letter-spacing: normal; }
}
@media (prefers-reduced-motion: reduce) { .critical #Health { animation: none; } }
</style>
