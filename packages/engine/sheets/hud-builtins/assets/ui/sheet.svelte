<meta name="screen" content="HudBuiltinsSheet" data-layer="menu" data-input="menu">
<meta name="prop:page" content="string">
<meta name="prop:lives" content="list">
<meta name="prop:full" content="list">
<meta name="prop:empty" content="list">
<meta name="prop:depleted" content="list">
<meta name="prop:phases" content="list">
<meta name="prop:pickups" content="list">
<meta name="prop:longPickup" content="list">

<div id="Page">
  <h1>Expedition · {page == 'resources' || page == 'resource-states' ? 'Resources' : 'Field report'}</h1>
  <div class="columns">
    {#if page == 'resources'}
      <section><h2>Reserves</h2><Pips label="Lives" items={lives} /><Pips label="Action points" icon="bolt" items={full} /><Pips label="Ammunition" icon="bolt" items={depleted} /></section>
      <BossBar name="The Glass Warden" value={7200} max={10000} phases={phases} phase="storm" />
      <Combo count={17} multiplier={2.5} remaining={0.65} state="active" />
      <Combo count={0} state="idle" />
    {:else if page == 'resource-states'}
      <section><h2>Reserves</h2><Pips label="Lives" items={empty} /><Pips label="Action points" items={full} icon="bolt" disabled={true} /><Pips label="Lives" loading={true} /></section>
      <BossBar name="The Warden of the Far Northern Observatory" value={4000} max={10000} invulnerable={true} />
      <BossBar name="The Glass Warden" value={0} max={10000} defeated={true} />
      <Combo count={1234567} multiplier={12.5} remaining={0} state="broken" label="Uninterrupted expedition chain" />
    {:else if page == 'activity'}
      <PickupFeed label="Recovered supplies" items={pickups} />
      <PickupFeed label="Recovered relic" items={longPickup} />
      <PerfReadout fps={60} frameMs={16.7} ping={42} loss={0} status="online" />
      <PerfReadout fps={24} frameMs={41.7} ping={240} loss={0.12} status="degraded" />
    {:else if page == 'activity-states'}
      <PickupFeed items={empty} /><PickupFeed loading={true} />
      <PerfReadout status="connecting" />
      <PerfReadout fps={60} frameMs={16.7} status="offline" />
      <PerfReadout />
      <PerfReadout fps={0} frameMs={0} ping={0} loss={0} status="online" />
    {/if}
  </div>
</div>
<style>
:host { display: flex; flex-direction: column; inline-size: stretch; padding: var(--space-md); gap: var(--space-md); background: var(--backdrop); }
h1 { margin: 0; font: var(--text-lg); }
h2 { margin: 0; font: var(--text-body); }
.columns { display: grid; grid-template-columns: repeat(2, 1fr); align-items: start; gap: var(--space-md); }
section { display: flex; flex-direction: column; gap: var(--space-sm); padding: var(--space-md); }
@media (width < 600px) { .columns { grid-template-columns: 1fr; } }
</style>
