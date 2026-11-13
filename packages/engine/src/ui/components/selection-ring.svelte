<meta name="component" content="SelectionRing">
<meta name="parts" content="SelectionRing.ring">
<meta name="prop:entity" content="number" data-doc="The unit it rings, by its bits (0: use at).">
<meta name="prop:at" content="string" data-doc="Its centre on the ground without an entity (x y z: a unit's feet).">
<meta name="prop:radius" content="number" data-doc="Its radius in world units.">
<meta name="prop:reaction" content="string" data-doc="Its colour: friendly (empty, green), hostile (red) or neutral (yellow).">
<meta name="prop:hover" content="bool" data-doc="Hovered, not selected: thin and dim.">
<meta name="prop:segmented" content="bool" data-doc="Broken into arcs turning slowly (a target ring).">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","radius":1,"reaction":"","hover":false,"segmented":false}'>

<ground id="SelectionRing" entity={entity || null} at={at} size={(radius * 2.4) + " " + (radius * 2.4)}>
  <svg id="Ring" part="SelectionRing.ring" viewBox="0 0 120 120" class:hostile={reaction == 'hostile'} class:neutral={reaction == 'neutral'} class:hover={hover} class:segmented={segmented}>
    <circle id="Glow" cx="60" cy="60" r="50" fill="none" stroke-width="10"></circle>
    <circle id="Line" cx="60" cy="60" r="50" fill="none" stroke-width="4"></circle>
  </svg>
</ground>

<style>
#SelectionRing { width: var(--size-3xl); height: var(--size-3xl); }
#Ring { width: var(--size-3xl); height: var(--size-3xl); --tone: var(--success); animation-name: land; animation-duration: var(--motion-duration-base); animation-timing-function: var(--motion-easing-out); }
#Ring.hostile { --tone: var(--danger); }
#Ring.neutral { --tone: var(--warn); }
#Glow { stroke: var(--tone); opacity: var(--opacity-faint); }
#Line { stroke: var(--tone); }
#Ring.hover #Line { stroke-width: 2; opacity: var(--opacity-muted); }
#Ring.hover #Glow { opacity: 0; }
#Ring.segmented #Line { stroke-dasharray: 26 13.36; }
#Ring.segmented { animation-name: turn; animation-duration: calc(var(--motion-duration-slow) * 15); animation-timing-function: linear; animation-iteration-count: infinite; }
@keyframes land { from { scale: 1.3; opacity: 0; } to { scale: 1; opacity: 1; } }
@keyframes turn { from { rotate: 0deg; } to { rotate: 360deg; } }
</style>
