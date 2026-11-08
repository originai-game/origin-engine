<meta name="component" content="Waypoint">
<meta name="prop:entity" content="number" data-doc="What it marks, by its bits (0: use at).">
<meta name="prop:at" content="string" data-doc="A world point it marks without an entity (x y z).">
<meta name="prop:height" content="number" data-doc="World units above the point.">
<meta name="prop:kind" content="string" data-doc="What it is: objective (empty), ping, danger, ally or loot.">
<meta name="prop:icon" content="string" data-doc="Its icon; empty is the kind's mark.">
<meta name="prop:label" content="string" data-doc="A word under it (Defend, Enemy spotted); empty is none.">
<meta name="prop:distance" content="number" data-doc="How far it is, shown under it; negative is none.">
<meta name="prop:units" content="string" data-doc="The distance's unit (m).">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","height":1.5,"kind":"objective","icon":"","label":"Defend","distance":142,"units":"m"}'>

<anchor id="Waypoint" entity={entity || null} at={at} offset={"0 " + height + " 0"} clamp="edge" declutter="shift" priority="500">
  <div id="Mark" class:ping={kind == 'ping'} class:danger={kind == 'danger'} class:ally={kind == 'ally'} class:loot={kind == 'loot'}>
    <div id="Pin">
      <svg id="Diamond" viewBox="0 0 28 28" hidden={!!icon}><path d="M14 2l12 12-12 12L2 14z"></path></svg>
      <icon id="Icon" icon={icon} hidden={!icon}></icon>
    </div>
    <span id="Label" hidden={!label}>{label}</span>
    <span id="Distance" hidden={distance < 0}>{formatNumber(distance) + units}</span>
  </div>
</anchor>

<style>
#Waypoint { align-items: center; }
#Mark { align-items: center; gap: 2px; --tone: var(--gold); }
#Mark.ping { --tone: var(--info); }
#Mark.danger { --tone: var(--danger); }
#Mark.ally { --tone: var(--success); }
#Mark.loot { --tone: var(--xp); }
#Pin { width: 28px; height: 28px; align-items: center; justify-content: center; }
.ping #Pin { animation: pulse 1200ms ease-out infinite; }
#Diamond { width: 28px; height: 28px; fill: var(--tone); stroke: #000000b3; stroke-width: 1.5; }
#Icon { width: 24px; height: 24px; color: var(--tone); }
#Label { font: var(--text-caption); color: var(--tone); white-space: nowrap; text-shadow: 0 1px 2px #000000ff; }
#Distance { font: var(--text-caption); color: var(--ink); white-space: nowrap; text-shadow: 0 1px 2px #000000ff; }
@keyframes pulse { 0% { scale: 1.3; } 100% { scale: 1; } }
</style>
