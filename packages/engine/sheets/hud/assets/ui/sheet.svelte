<meta name="screen" content="HudSheet" data-layer="hud">
<meta name="prop:page" content="string">
<meta name="prop:actions" content="list">
<meta name="prop:slots" content="list">
<meta name="prop:pois" content="list">
<meta name="prop:markers" content="list">
<meta name="prop:items" content="list">
<meta name="prop:overlays" content="list">

<div id="Page">
  <header><span class="eyebrow">EXPEDITION 07</span><h1>Field instruments</h1><p>Resources, recovery and time at a glance</p></header>
  <div class="instruments">
    {#if page == 'meters'}<section class="vitals">
      <h2>Vitals</h2>
      <StatBar value={72} max={100} shield={60} shieldMax={100} heal={12} segment={25} label="HP" showValue={true} />
      <StatBar value={28} max={100} kind="health" label="HP" showValue={true} />
      <StatBar value={12} max={100} kind="health" label="HP" showValue={true} />
      <StatBar value={64} max={100} kind="mana" label="MP" showValue={true} valueInside={true} />
      <StatBar value={100} max={100} kind="stamina" label="SP" showValue={true} />
      <StatBar value={0} max={100} kind="xp" label="XP" showValue={true} />
    </section>
    <section>
      <h2>Reserves</h2>
      <div class="row"><OrbMeter value={340} max={500} label="Life" showValue={true} /><OrbMeter value={96} max={200} kind="mana" label="Mana" showValue={true} /></div>
      <div class="row"><RadialMeter value={1.4} wheels={2} /><RadialMeter value={0.15} exhausted={true} /></div>
    </section>{/if}
    {#if page == 'objectives'}<section>
      <h2>Objectives</h2>
      <div class="row"><ProgressRing value={0.62} /><ProgressRing value={1} tone="success" /><ProgressRing spinning={true} /></div>
      <div class="row"><HudTimer seconds={84} label="ROUND 2" countdown={true} warnAt={10} /><HudTimer seconds={8} total={90} countdown={true} warnAt={10} /></div>
      <HudTimer seconds={2} total={90} countdown={true} warnAt={10} label="EXTRACTION" />
    </section>{/if}
    {#if page == 'actions'}<section>
      <h2>Abilities</h2>
      <AbilityBar slots={slots} />
      <div class="row"><AbilitySlot icon="bolt" name="Arc" hotkey="Q" cooldown={6} cooldownMax={10} /><AbilitySlot icon="shield" name="Ward" hotkey="W" charges={2} maxCharges={3} /><AbilitySlot icon="fire" name="Burst" hotkey="E" unaffordable={true} cost="80" /></div>
      <div class="row"><Cooldown icon="potion" label="Recover" remaining={8} total={20} /><Cooldown icon="bolt" label="Dash" remaining={0} total={6} /><AbilitySlot icon="star" name="Nova" ultimate={0.7} locked={true} /></div>
    </section>{/if}
    {#if page == 'buttons'}<section>
      <h2>Mission actions</h2>
      <HeroButton label="Deploy" key="E" />
      <HeroButton label="Waiting for squad" disabled={true} />
      <HoldButton label="Abandon mission" holding="Keep holding…" hint="Hold to confirm" />
      <HoldButton label="Secure checkpoint" variant="primary" hint="Hold to secure" />
      <HoldButton label="Abandon the current expedition" holding="Keep holding to confirm" hint="Hold to abandon the expedition" icon="close" />
      <HoldButton label="Delete save" hint="Unavailable during combat" disabled={true} />
    </section>{/if}
    {#if page == 'navigation'}<section class="wide">
      <h2>Heading and objectives</h2>
      <Compass heading={0} pois={pois} units="m" />
    </section>
    <section>
      <h2>Local map</h2>
      <Minimap markers={markers} label="Ashen Vale" />
    </section>
    <section>
      <h2>Player-up map</h2>
      <MinimapView centerX={0.5} centerY={0.5} markers={markers} heading={30} rotate={true} zone="Ashen Vale" />
    </section>{/if}
    {#if page == 'touch'}<section class="touch">
      <h2>Field controls</h2>
      <TouchControls move="move" actions={actions} always={true} />
      <TouchControls move="move" look="look" actions={actions} always={true} disabled={true} />
    </section>{/if}
    {#if page == 'tactics'}<section>
      <h2>Quick commands</h2>
      <RadialMenu items={items} span={0.1667} chosen="heal" />
    </section>
    <section>
      <h2>Map layers</h2>
      <OverlayBar overlays={overlays} selected={"heat"} />
      <OverlayBar overlays={overlays} selected={""} />
    </section>{/if}
  </div>
</div>

<style>
#Page { display: flex; flex-direction: column; gap: var(--space-xl); padding: calc(env(safe-area-inset-top) + var(--space-lg)) var(--space-lg) calc(env(safe-area-inset-bottom) + var(--space-lg)); block-size: stretch; overflow-y: auto; background: var(--backdrop); }
header { display: flex; flex-direction: column; gap: var(--space-xs); }
.eyebrow { font: var(--text-caption); color: var(--ink-muted); letter-spacing: var(--type-caption-tracking); }
h1 { margin: 0; font: var(--text-xl2); }
header p { margin: 0; color: var(--ink-muted); }
h2 { margin: 0; font: var(--text-sm); color: var(--ink-muted); }
.instruments { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--space-lg); }
section { display: flex; flex-direction: column; gap: var(--space-lg); padding: var(--space-lg); border-radius: var(--radius-panel); background: var(--panel); border: var(--stroke-hairline) solid var(--line); }
.row { display: flex; align-items: center; flex-wrap: wrap; gap: var(--space-lg); }
.touch { grid-column: 1 / -1; }
.wide { grid-column: 1 / -1; }
#Page:size(compact) { gap: var(--space-lg); }
.instruments:size(compact) { grid-template-columns: minmax(0, 1fr); }
</style>
