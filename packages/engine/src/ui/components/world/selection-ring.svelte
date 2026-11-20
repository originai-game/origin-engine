<meta name="component" content="SelectionRing">
<meta name="prop:entity" content="number" data-doc="The unit it rings, by its bits (0: use at).">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="Its centre on the ground without an entity (x y z: a unit's feet).">
<meta name="prop:radius" content="number" data-default="1" data-doc="Its radius in world units.">
<meta name="prop:side" content="string" data-doc="Whose it is: own (empty, green), ally (blue), enemy (red) or neutral (yellow).">
<meta name="prop:leader" content="bool" data-doc="The group's leader: four ticks round the ring.">
<meta name="prop:target" content="bool" data-doc="The target: segmented, turning once every 8 s (still under reduced motion).">
<meta name="prop:hover" content="bool" data-doc="Hovered, not selected: thin and dim, without the glow.">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","radius":1,"side":"","leader":false,"target":false,"hover":false}'>
<meta name="fixture:leader" content='{"leader":true}'>
<meta name="fixture:ally" content='{"side":"ally"}'>
<meta name="fixture:target" content='{"side":"enemy","target":true,"radius":1.2}'>
<meta name="fixture:neutral" content='{"side":"neutral"}'>
<meta name="fixture:hover" content='{"side":"enemy","hover":true}'>

<!--
  A ring on the ground round a selected unit, in its side's colour, with a
  soft glow; ticks for the group's leader, segments turning for a target.
  It pops in. Read only.
-->
<ground id="SelectionRing" entity={entity || null} at={at} size={(radius * 2.4) + " " + (radius * 2.4)}>
  <svg id="Ring" part="ring" viewBox="0 0 120 120" class:side-ally={(side == '' ? 'own' : side) == 'ally'} class:side-enemy={(side == '' ? 'own' : side) == 'enemy'} class:side-neutral={(side == '' ? 'own' : side) == 'neutral'} class:hover={hover} class:target={target}>
    <circle part="glow" class="glow" cx="60" cy="60" r="50"></circle>
    <circle part="line" class="line" cx="60" cy="60" r="50"></circle>
    {#if leader && !hover && !target}
      <line part="tick" class="tick" x1="91" y1="29" x2="100" y2="20"></line>
      <line part="tick" class="tick" x1="91" y1="91" x2="100" y2="100"></line>
      <line part="tick" class="tick" x1="29" y1="91" x2="20" y2="100"></line>
      <line part="tick" class="tick" x1="29" y1="29" x2="20" y2="20"></line>
    {/if}
  </svg>
</ground>

<style>
#SelectionRing { inline-size: var(--size-3xl); block-size: var(--size-3xl); }
#Ring { inline-size: var(--size-3xl); block-size: var(--size-3xl); color: var(--world-friendly-ink); animation: var(--world-ring-motion); }
#Ring.side-ally { color: var(--world-ally-ink); }
#Ring.side-enemy { color: var(--world-hostile-ink); }
#Ring.side-neutral { color: var(--world-neutral-ink); }
.glow { fill: none; stroke: currentColor; stroke-width: 9; opacity: var(--world-ring-glow); }
.line { fill: none; stroke: currentColor; stroke-width: 2.6; }
.tick { stroke: currentColor; stroke-width: 3.4; }
.hover .glow { display: none; }
.hover .line { stroke-width: 1.4; opacity: var(--opacity-muted); }
/* A target: segments turning slowly. */
.target .line { stroke-dasharray: 22 9; }
#Ring.target { animation: var(--world-target-ring-motion); }
@media (prefers-reduced-motion: reduce) { #Ring, #Ring.target { animation: none; } }
</style>
