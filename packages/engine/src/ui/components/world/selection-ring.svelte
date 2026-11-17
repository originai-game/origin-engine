<meta name="component" content="SelectionRing">
<meta name="parts" content="SelectionRing.ring">
<meta name="prop:entity" content="number" data-doc="The unit it rings, by its bits (0: use at).">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="Its centre on the ground without an entity (x y z: a unit's feet).">
<meta name="prop:radius" content="number" data-default="1" data-doc="Its radius in world units.">
<meta name="prop:reaction" content="string" data-doc="Its colour: friendly (empty, green), hostile (red) or neutral (yellow).">
<meta name="prop:hover" content="bool" data-doc="Hovered, not selected: thin and dim.">
<meta name="prop:segmented" content="bool" data-doc="Broken into arcs turning slowly (a target ring).">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","radius":1,"reaction":"","hover":false,"segmented":false}'>

<ground id="SelectionRing" entity={entity || null} at={at} size={(radius * 2.4) + " " + (radius * 2.4)}>
  <svg id="Ring" part="SelectionRing.ring" data-motion={segmented ? 'spin' : 'pop'} viewBox="0 0 120 120" class:hostile={reaction == 'hostile'} class:neutral={reaction == 'neutral'} class:hover={hover} class:segmented={segmented}>
    <circle id="Glow" cx="60" cy="60" r="50" fill="none" stroke-width="10"></circle>
    <circle id="Line" cx="60" cy="60" r="50" fill="none" stroke-width="4"></circle>
  </svg>
</ground>

<style>
:host { width: var(--size-3xl); height: var(--size-3xl); }
#Ring { width: var(--size-3xl); height: var(--size-3xl); --fill-color: var(--tone-success); }
#Ring.hostile { --fill-color: var(--tone-danger); }
#Ring.neutral { --fill-color: var(--tone-warn); }
#Glow { stroke: var(--fill-color); opacity: var(--opacity-faint); }
#Line { stroke: var(--fill-color); }
#Ring.hover #Line { stroke-width: 2; opacity: var(--opacity-muted); }
#Ring.hover #Glow { opacity: 0; }
#Ring.segmented #Line { stroke-dasharray: 26 13.36; }
</style>
