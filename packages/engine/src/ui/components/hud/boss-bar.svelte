<meta name="component" content="BossBar">
<meta name="strings" content="./boss-bar.en.ogt">
<meta name="parts" content="BossBar.frame BossBar.figure BossBar.health BossBar.fill BossBar.notch BossBar.phases BossBar.phase BossBar.label BossBar.state BossBar.headline BossBar.card BossBar.caption BossBar.dot">
<meta name="prop:name" content="string" data-default="" data-doc="The boss name; empty uses Boss.">
<meta name="prop:value" content="number" data-default="0" data-doc="Health, clamped to the supplied range for display.">
<meta name="prop:max" content="number" data-default="1" data-doc="Full health; a nonpositive capacity shows no health instead of dividing by zero.">
<meta name="prop:phases" content="list" data-doc="Keyed phases { id, label, complete }; the game owns phase progression.">
<meta name="prop:phase" content="string" data-default="" data-doc="The current phase's stable id; aria-current marks it.">
<meta name="prop:invulnerable" content="bool" data-default="false" data-doc="Health remains visible and a worded invulnerability state is shown.">
<meta name="prop:defeated" content="bool" data-default="false" data-doc="The game has confirmed defeat; this does not follow automatically from zero health.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Waiting for boss data; hides stale health and phases.">
<meta name="fixture:default" content='{"name":"The Glass Warden","value":7200,"max":10000,"phases":[{"id":"shell","label":"Shell","complete":true},{"id":"storm","label":"Storm","complete":false},{"id":"core","label":"Core","complete":false}],"phase":"storm","invulnerable":false,"defeated":false,"loading":false}'>
<meta name="fixture:invulnerable" content='{"name":"The Glass Warden","value":4000,"max":10000,"invulnerable":true}'>
<meta name="fixture:defeated" content='{"name":"The Glass Warden","value":0,"max":10000,"defeated":true}'>
<meta name="fixture:loading" content='{"name":"Unknown presence","loading":true}'>
<meta name="fixture:overflow" content='{"name":"The Warden of the Far Northern Observatory","value":1,"max":10,"phases":[{"id":"last","label":"The final defence of the observatory","complete":false}],"phase":"last"}'>

<section id="BossBar" part="BossBar.frame" class="hud bare" data-layout="stack" data-gap="tight" aria-label={name || t("hud.boss-bar.name")} aria-busy={loading} class:invulnerable={invulnerable && !defeated} class:defeated={defeated} class:critical={!loading && !defeated && max > 0 && value / max <= 0.25}>
  <div class="heading" data-layout="cluster">
    <div part="BossBar.headline">{name || t("hud.boss-bar.name")}</div>
    {#if !loading}<div part="BossBar.figure">{t("hud.boss-bar.value", { value: max <= 0 || value < 0 ? 0 : value > max ? max : value, max: max > 0 ? max : 0 })}</div>{/if}
  </div>
  {#if loading}<div class="wait" data-layout="row" data-gap="tight"><Spinner size={18} name={t("hud.boss-bar.loading")} /><div part="BossBar.state BossBar.caption" role="status">{t("hud.boss-bar.loading")}</div></div>
  {:else}
    <div id="Health" part="BossBar.health" data-layout="overlay" role="meter" aria-label={t("hud.boss-bar.health")} aria-valuemin="0" aria-valuemax={max > 0 ? max : 1} aria-valuenow={max <= 0 || value < 0 ? 0 : value > max ? max : value} style:--value={max <= 0 || value <= 0 ? 0 : value >= max ? 1 : value / max} style:--block={phases.length > 1 ? 1 / phases.length : 0}>
      <div id="Ghost" part="BossBar.fill" data-anchor="top-start" data-ghost="true"></div>
      <div id="Fill" part="BossBar.fill" data-anchor="top-start"></div>
      {#if phases.length > 1}<div id="Gates" part="BossBar.notch" data-anchor="fill"></div>{/if}
    </div>
    {#if defeated || invulnerable || phases.length}<div class="foot" data-layout="cluster">
      {#if phases.length}<ol part="BossBar.phases" data-layout="cluster" data-gap="tight" aria-label={t("hud.boss-bar.phases")}>
        {#each phases as p (p.id)}<li part="BossBar.phase BossBar.card" data-layout="row" data-gap="tight" class:current={p.id == phase} class:complete={p.complete} aria-current={p.id == phase}>
          <div class="mark" part="BossBar.dot" data-tone={p.complete ? 'success' : p.id == phase ? 'info' : null}></div>
          <div part="BossBar.label">{p.label}</div>
          <div part="BossBar.state BossBar.caption" data-tone={p.complete ? 'success' : p.id == phase ? 'info' : null}>{p.complete ? t("hud.boss-bar.complete") : p.id == phase ? t("hud.boss-bar.current") : t("hud.boss-bar.next")}</div>
        </li>{/each}
      </ol>{/if}
      {#if defeated || invulnerable}<div class="outcome" part="BossBar.state BossBar.caption" data-tone={defeated ? 'success' : 'info'} role="status">{defeated ? t("hud.boss-bar.defeated") : t("hud.boss-bar.invulnerable")}</div>{/if}
    </div>{/if}
  {/if}
</section>

<style>
/* A boss's bar is wide but not the screen's width: a measure, which the
   page centres. The name and the figure stand over the scene on the
   style's halo; the bar leans like every HUD bar, its phases are gates cut
   into it (one per phase boundary) and a line of chips under it. */
:host { inline-size: min(100%, var(--measure-xl)); text-shadow: var(--tag-halo); --fill-color: var(--bar-danger); --layout-anchor-inset: 0; }
.heading { align-items: baseline; justify-content: space-between; }
[part="BossBar.headline"] { overflow-wrap: anywhere; }
[part="BossBar.figure"] { white-space: nowrap; --figure-weight: var(--bar-figure-weight); font-stretch: var(--bar-figure-stretch); }
#Health { inline-size: 100%; block-size: calc(14 * var(--font-step-px)); overflow: hidden; transform: skewX(var(--bar-skew)); background: var(--bar-well); box-shadow: var(--bar-rim); }
#Ghost, #Fill { inline-size: calc(var(--value) * 100%); block-size: 100%; --fill-gloss: var(--bar-gloss); }
#Ghost { background: var(--bar-ghost); transition: var(--bar-ghost-motion); }
#Gates { background: var(--bar-notch); mask-image: repeating-linear-gradient(90deg, transparent 0% calc(var(--block) * 100% - 0.4%), #000 calc(var(--block) * 100% - 0.4%) calc(var(--block) * 100%)); }
:host(:dir(rtl)) #Gates { scale: -1 1; }
/* Invulnerable: the bar holds in the info tone; under a quarter the rim
   glows in danger and pulses. */
.invulnerable { --fill-color: var(--info); }
.critical #Health { box-shadow: var(--bar-critical-rim); animation: var(--pulse-motion); }
.foot { align-items: center; }
[part="BossBar.phases"] { margin: 0; padding: 0; list-style: none; }
[part~="BossBar.phase"] { align-items: center; padding: var(--space-xxs) var(--space-sm); }
.mark { flex: none; inline-size: var(--space-sm); block-size: var(--space-sm); }
[part="BossBar.label"] { overflow-wrap: anywhere; }
.outcome { margin-inline-start: auto; }
@media (prefers-reduced-motion: reduce) { .critical #Health { animation: none; } }
</style>
