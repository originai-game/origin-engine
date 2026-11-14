<meta name="component" content="Waypoint">
<meta name="parts" content="Waypoint.pin Waypoint.label Waypoint.figure">
<meta name="prop:entity" content="number" data-doc="What it marks, by its bits (0: use at).">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="A world point it marks without an entity (x y z).">
<meta name="prop:height" content="number" data-doc="World units above the point.">
<meta name="prop:kind" content="string" data-doc="What it is: objective (empty), ping, danger, ally or loot.">
<meta name="prop:icon" content="string" data-doc="Its icon; empty is the kind's mark.">
<meta name="prop:label" content="string" data-doc="A word under it (Defend, Enemy spotted); empty is none.">
<meta name="prop:distance" content="number" data-default="-1" data-doc="How far it is, shown under it; negative is none.">
<meta name="prop:units" content="string" data-doc="The distance's unit (m).">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","height":1.5,"kind":"objective","icon":"","label":"Defend","distance":142,"units":"m"}'>

<anchor id="Waypoint" entity={entity || null} at={at} offset={"0 " + height + " 0"} clamp="edge" declutter="shift" priority="500">
  <div id="Mark" class:ping={kind == 'ping'} class:danger={kind == 'danger'} class:ally={kind == 'ally'} class:loot={kind == 'loot'}>
    <div id="Pin" part="Waypoint.pin">
      <svg id="Diamond" viewBox="0 0 28 28" hidden={!!icon}><path d="M14 2l12 12-12 12L2 14z"></path></svg>
      <og-icon id="Icon" icon={icon} hidden={!icon}></og-icon>
    </div>
    <span id="Label" part="Waypoint.label" hidden={!label}>{label}</span>
    <span id="Distance" part="Waypoint.figure" hidden={distance < 0}>{formatNumber(distance) + units}</span>
  </div>
</anchor>

<style>
#Waypoint { align-items: center; }
#Mark { align-items: center; gap: var(--space-xxs); --tone: var(--gold); }
#Mark.ping { --tone: var(--info); }
#Mark.danger { --tone: var(--danger); }
#Mark.ally { --tone: var(--success); }
#Mark.loot { --tone: var(--xp); }
[part="Waypoint.pin"] { width: var(--size-sm); height: var(--size-sm); align-items: center; justify-content: center; }
.ping [part="Waypoint.pin"] { animation-name: pulse; animation-duration: calc(var(--motion-duration-slow) * 3); animation-timing-function: var(--motion-easing-out); animation-iteration-count: infinite; }
#Diamond { width: var(--size-sm); height: var(--size-sm); fill: var(--tone); stroke: var(--scene-shadow); stroke-width: 1.5; }
#Icon { width: var(--size-sm); height: var(--size-sm); color: var(--tone); }
/* In the world, over any scene: the kind's tone and the scene's ink, through the role tokens. */
[part="Waypoint.label"] { font: var(--text-caption); font-family: var(--label-font); white-space: nowrap; text-shadow: 0 var(--stroke-hairline) var(--stroke-regular) var(--scene-shadow); --ink-label: var(--tone); }
[part="Waypoint.figure"] { font: var(--text-caption); font-family: var(--figure-font); white-space: nowrap; text-shadow: 0 var(--stroke-hairline) var(--stroke-regular) var(--scene-shadow); --figure-ink: var(--scene-ink); }
@keyframes pulse { 0% { scale: 1.3; } 100% { scale: 1; } }
</style>
