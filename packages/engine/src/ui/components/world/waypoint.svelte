<meta name="component" content="Waypoint">
<meta name="parts" content="Waypoint.pin Waypoint.label Waypoint.figure Waypoint.tag">
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
  <div id="Mark" data-layout="stack" data-gap="hair" class:ping={kind == 'ping'} class:danger={kind == 'danger'} class:ally={kind == 'ally'} class:loot={kind == 'loot'}>
    <div id="Pin" part="Waypoint.pin" data-layout="center" data-motion={kind == 'ping' ? 'ping' : null}>
      {#if icon}<og-icon id="Icon" icon={icon}></og-icon>
      {:else}<svg id="Diamond" viewBox="0 0 28 28"><path d="M14 2l12 12-12 12L2 14z"></path></svg>{/if}
    </div>
    {#if label}<div part="Waypoint.tag"><span id="Label" class="label" part="Waypoint.label">{label}</span></div>{/if}
    {#if distance >= 0}<div part="Waypoint.tag"><span id="Distance" class="distance" part="Waypoint.figure">{formatNumber(distance) + units}</span></div>{/if}
  </div>
</anchor>

<style>
:host { text-shadow: var(--tag-halo); }
#Mark { align-items: center; --tone: var(--gold); }
#Mark.ping { --tone: var(--info); }
#Mark.danger { --tone: var(--danger); }
#Mark.ally { --tone: var(--success); }
#Mark.loot { --tone: var(--xp); }
[part="Waypoint.pin"] { width: var(--size-sm); height: var(--size-sm); --dot-fill: var(--tone); }
#Diamond { width: var(--size-sm); height: var(--size-sm); fill: var(--dot-fill); stroke: var(--dot-rim); stroke-width: var(--dot-rim-width); }
#Icon { width: var(--size-sm); height: var(--size-sm); color: var(--dot-fill); }
/* In the world, over any scene: the kind's tone and the scene's ink, through the role tokens; the tag tokens outline them, and give them a ground where a pack sets one. */
[part="Waypoint.label"] { white-space: nowrap; --ink-label: var(--tone); }
[part="Waypoint.figure"] { white-space: nowrap; --figure-size: var(--font-step-sm); --figure-ink: var(--scene-ink); }
</style>
