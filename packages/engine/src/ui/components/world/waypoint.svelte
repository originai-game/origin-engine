<meta name="component" content="Waypoint">
<meta name="strings" content="./world.en.ogt">
<meta name="prop:entity" content="number" data-doc="What it marks, by its bits (0: use at).">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="A world point it marks without an entity (x y z).">
<meta name="prop:height" content="number" data-doc="World units above the point.">
<meta name="prop:kind" content="string" data-doc="What it is: objective (empty, gold), ping (cyan, a ripple), danger (red), ally (green) or loot (violet).">
<meta name="prop:label" content="string" data-doc="Its name in the kind's ink (The Ninefold Gate); empty is none. It drops when far.">
<meta name="prop:distance" content="string" data-doc="How far it is as the game words it (412), on a dark tag; empty is none.">
<meta name="prop:units" content="string" data-doc="The distance's unit, smaller after it (m).">
<meta name="prop:range" content="string" data-doc="near (a soft halo), mid (empty) or far (the label drops, the distance stays), as the game judges it.">
<meta name="prop:edge" content="string" data-doc="The side of the screen it is off, as the game reads it (left or right): a chevron toward it before the pin; empty is on screen. The anchor holds it inside the screen's edge.">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","height":1.5,"kind":"","label":"The Ninefold Gate","distance":"412","units":"m","range":"","edge":""}'>
<meta name="fixture:near" content='{"label":"Ninefold Gate","distance":"12","range":"near"}'>
<meta name="fixture:far" content='{"distance":"1,240","range":"far"}'>
<meta name="fixture:ping" content='{"kind":"ping","label":"Look here","distance":"34"}'>
<meta name="fixture:danger" content='{"kind":"danger","label":"Enemy","distance":"58"}'>
<meta name="fixture:ally" content='{"kind":"ally","label":"Sylvenne","distance":"22"}'>
<meta name="fixture:loot" content='{"kind":"loot","label":"Cache","distance":"140"}'>
<meta name="fixture:offscreen" content='{"kind":"ally","label":"Brannoch","distance":"86","edge":"left"}'>
<meta name="fixture:offscreenRight" content='{"kind":"danger","label":"Enemy","distance":"58","edge":"right"}'>
<meta name="fixture:overflow" content='{"label":"Light every lamp of the northern reach before nightfall","distance":"12,400","units":"km"}'>

<!--
  A place to go: a lit diamond with a dark core in the kind's colour, its
  name under it and the distance on a dark tag. Off screen, a chevron
  points the way. Read only; the game owns the distance and what counts
  as near and far.
-->
<anchor id="Waypoint" entity={entity || null} at={at} offset={"0 " + height + " 0"} clamp="edge" arrow-size="0" avoid="player" declutter="shift" priority="500">
  <div id="Mark" part="frame" variant="bare" aria-label={label != '' ? label : distance} class:kind-ally={(kind == '' ? 'objective' : kind) == 'ally'} class:kind-danger={(kind == '' ? 'objective' : kind) == 'danger'} class:kind-loot={(kind == '' ? 'objective' : kind) == 'loot'} class:kind-ping={(kind == '' ? 'objective' : kind) == 'ping'} class:range-far={(range == '' ? 'mid' : range) == 'far'} class:range-near={(range == '' ? 'mid' : range) == 'near'} class:way-left={(edge == '' ? 'none' : edge) == 'left'} class:way-right={(edge == '' ? 'none' : edge) == 'right'}>
    {#if edge != ''}<span part="chevron" class="chevron"></span>{/if}
    <span part="pin" class="pin"><span class="core"></span></span>
    {#if (label != '' && range != 'far') || distance != ''}
      <div class="words">
        {#if label != '' && range != 'far'}<span part="name" class="label">{label}</span>{/if}
        {#if distance != ''}<span part="distance" class="distance">{distance}<span class="units">{units}</span></span>{/if}
      </div>
    {/if}
  </div>
</anchor>

<style>
#Mark { display: flex; flex-direction: column; align-items: center; row-gap: calc(3 * var(--font-step-px)); color: var(--hud-gold); }
#Mark.kind-ping { color: var(--hud-rare); }
#Mark.kind-danger { color: var(--world-hostile-ink); }
#Mark.kind-ally { color: var(--world-friendly-ink); }
#Mark.kind-loot { color: var(--hud-epic); }
#Mark.range-far { opacity: var(--world-far-opacity); }
/* Off screen: the chevron, the pin and the words in a row. */
#Mark.way-left, #Mark.way-right { flex-direction: row; column-gap: calc(5 * var(--font-step-px)); }
#Mark.way-right { flex-direction: row-reverse; }
.words { display: flex; flex-direction: column; align-items: center; row-gap: calc(2 * var(--font-step-px)); min-inline-size: 0; }
.way-left .words { align-items: flex-start; }
.way-right .words { align-items: flex-end; }
/* The pin: a lit diamond with a dark core; art, it keeps its size. */
.pin { position: relative; flex: none; inline-size: var(--world-pin-size); block-size: var(--world-pin-size); margin: var(--space-xs); rotate: 45deg; background: currentColor; box-shadow: var(--world-pin-rim); }
.range-near .pin { box-shadow: var(--world-pin-near-rim); }
.kind-ping .pin { animation: var(--world-ping-motion); }
.core { position: absolute; inset: 32%; background: var(--world-pin-core); }
.chevron { flex: none; inline-size: var(--world-chevron-width); block-size: var(--world-chevron-height); background: currentColor; filter: var(--world-pin-glow); clip-path: polygon(100% 0, 0 50%, 100% 100%, 55% 50%); }
.way-right .chevron { scale: -1 1; }
/* The name: the kind's ink in tracked capitals, over the scene. */
.label { min-inline-size: 0; max-inline-size: calc(200 * var(--font-step-px)); font-size: var(--world-size-caps); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--world-caps-stretch); letter-spacing: var(--world-title-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
/* The distance on a dark tag; a short figure, always whole. */
.distance { flex: none; padding: calc(1 * var(--font-step-px)) calc(6 * var(--font-step-px)); font-size: var(--world-size-distance); line-height: 1.2; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); white-space: nowrap; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.units { margin-inline-start: var(--stroke-hairline); font-size: var(--world-size-units); color: var(--hud-ink-dim); }
</style>
