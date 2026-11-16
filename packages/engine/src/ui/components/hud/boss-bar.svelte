<meta name="component" content="BossBar">
<meta name="strings" content="./boss-bar.en.ogt">
<meta name="parts" content="BossBar.frame BossBar.heading BossBar.figure BossBar.health BossBar.phases BossBar.phase BossBar.label BossBar.state">
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

<section id="BossBar" part="BossBar.frame" class="hud" aria-label={name || t("hud.boss-bar.name")} aria-busy={loading}>
  <div class="heading">
    <div part="BossBar.heading">{name || t("hud.boss-bar.name")}</div>
    {#if !loading}<div part="BossBar.figure">{t("hud.boss-bar.value", { value: max <= 0 || value < 0 ? 0 : value > max ? max : value, max: max > 0 ? max : 0 })}</div>{/if}
  </div>
  {#if loading}<div part="BossBar.state" role="status">{t("hud.boss-bar.loading")}</div>
  {:else}
    <meter id="Health" part="BossBar.health" min="0" max={max > 0 ? max : 1} value={max <= 0 || value < 0 ? 0 : value > max ? max : value} aria-label={t("hud.boss-bar.health")}></meter>
    {#if defeated || invulnerable}<div part="BossBar.state" role="status">{defeated ? t("hud.boss-bar.defeated") : t("hud.boss-bar.invulnerable")}</div>{/if}
    {#if phases.length}<ol part="BossBar.phases" aria-label={t("hud.boss-bar.phases")}>
      {#each phases as p (p.id)}<li part="BossBar.phase" class:current={p.id == phase} class:complete={p.complete} aria-current={p.id == phase}>
        <div part="BossBar.label">{p.label}</div>
        <div part="BossBar.state">{p.complete ? t("hud.boss-bar.complete") : p.id == phase ? t("hud.boss-bar.current") : t("hud.boss-bar.next")}</div>
      </li>{/each}
    </ol>{/if}
  {/if}
</section>

<style>
:host { display: flex; flex-direction: column; gap: var(--space-sm); inline-size: 100%; padding: var(--space-md); }
.heading { display: flex; flex-wrap: wrap; align-items: baseline; justify-content: space-between; gap: var(--space-xs) var(--space-md); }
[part="BossBar.heading"] { font: var(--text-lg); overflow-wrap: anywhere; }
[part="BossBar.figure"] { font: var(--text-caption); white-space: nowrap; }
[part="BossBar.health"] { inline-size: 100%; block-size: var(--space-md); }
[part="BossBar.phases"] { display: flex; flex-wrap: wrap; gap: var(--space-sm); margin: 0; padding: 0; list-style: none; }
[part="BossBar.phase"] { display: flex; flex-direction: column; gap: var(--space-xxs); padding: var(--space-xs) var(--space-sm); border: var(--stroke-hairline) solid var(--line); border-radius: var(--radius-control); }
.current { border-color: var(--primary); background: var(--control-strong); }
.complete { color: var(--ink-muted); }
[part="BossBar.label"] { overflow-wrap: anywhere; }
[part="BossBar.state"] { font: var(--text-caption); color: var(--ink-muted); }
</style>
