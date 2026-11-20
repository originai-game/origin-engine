<meta name="component" content="OrbMeter">
<meta name="strings" content="./orb-meter.en.ogt">
<meta name="parts" content="OrbMeter.orb OrbMeter.bezel OrbMeter.socket OrbMeter.glass OrbMeter.ornament OrbMeter.fill OrbMeter.value OrbMeter.label OrbMeter.figure OrbMeter.tag">
<meta name="prop:value" content="number" data-doc="The value.">
<meta name="prop:max" content="number" data-default="1" data-doc="Its maximum.">
<meta name="prop:kind" content="string" data-doc="Its liquid's colour: health (the style's --health) or mana (--mana).">
<meta name="prop:showValue" content="bool" data-doc="The value in the middle.">
<meta name="prop:label" content="string" data-doc="A caption under it; empty is none.">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is the label, else Health.">
<meta name="fixture:default" content='{"value":340,"max":500,"kind":"health","showValue":true,"label":"Life","name":""}'>

<!-- A glass vessel in a bezel, over the scene. It is built in layers, and a
     pack sets each through its --orb-meter-* tokens: the bezel (its metal,
     width, edge, shape) and an ornament a pack may lay over it; the socket
     the glass sits in; the glass (a dark well in its liquid's hue, the
     sphere's shade, the gleam); the liquid under two waves. The glass's
     view box is its own square, 82 across: the liquid's surface runs from
     y 91 (empty) to y 9 (full). -->
<div id="OrbMeter" data-layout="stack" data-gap="hair" role="meter" aria-label={name || label || t("component.orb-meter.health")} aria-valuemin="0" aria-valuemax={max} aria-valuenow={value} class:mana={kind == 'mana'}>
  <div id="Orb" part="OrbMeter.orb" data-layout="overlay">
    <div id="Bezel" part="OrbMeter.bezel" data-anchor="fill"></div>
    <div id="Socket" part="OrbMeter.socket"></div>
    <div id="Glass" part="OrbMeter.glass">
      <svg id="Vessel" viewBox="9 9 82 82">
        <defs>
          <radialGradient id="OrbMeterShade" cx="0.5" cy="0.42" r="0.6"><stop offset="0.55" stop-color="#000" stop-opacity="0"></stop><stop offset="1" stop-color="#000" stop-opacity="0.62"></stop></radialGradient>
          <radialGradient id="OrbMeterGleam" cx="0.5" cy="0.5" r="0.5"><stop offset="0" stop-color="#fff" stop-opacity="0.2"></stop><stop offset="1" stop-color="#fff" stop-opacity="0"></stop></radialGradient>
          <linearGradient id="OrbMeterDepth" x1="0" y1="0" x2="0" y2="1"><stop offset="0.35" stop-color="#000" stop-opacity="0"></stop><stop offset="1" stop-color="#000" stop-opacity="0.4"></stop></linearGradient>
        </defs>
        <rect class="well" x="9" y="9" width="82" height="82"></rect>
        <g transform={"translate(0 " + (value >= max ? 9 : value <= 0 ? 96 : 91 - 82 * value / max) + ")"}>
          <path class="swell" d="M-36 -1.5q6 2.4 12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0V100H-36z"><animateTransform attributeName="transform" type="translate" from="-24 0" to="0 0" dur="5s" repeatCount="indefinite"></animateTransform></path>
          <path class="liquid" part="OrbMeter.fill" d="M-48 0q6 -2.4 12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0V100H-48z"><animateTransform attributeName="transform" type="translate" from="0 0" to="-24 0" dur="3.5s" repeatCount="indefinite"></animateTransform></path>
          <circle class="lume" cx="50" cy="34" r="30"></circle>
          <path class="surface" d="M-48 0q6 -2.4 12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0t12 0"><animateTransform attributeName="transform" type="translate" from="0 0" to="-24 0" dur="3.5s" repeatCount="indefinite"></animateTransform></path>
        </g>
        <rect class="depth" x="9" y="9" width="82" height="82"></rect>
        <rect class="shade" x="9" y="9" width="82" height="82"></rect>
        <path class="glint" d="M19 43A33 33 0 0 1 45 17.5"></path>
        <ellipse class="spark" cx="60" cy="16.5" rx="4" ry="1.5" transform="rotate(14 60 16.5)"></ellipse>
        <path class="bounce" d="M27 78A36 36 0 0 0 80 70"></path>
      </svg>
    </div>
    <div id="Lip"></div>
    <div id="Ornament" part="OrbMeter.ornament"></div>
    {#if showValue}<span id="Value" class="value" part="OrbMeter.value OrbMeter.figure" data-anchor="center">{formatNumber(value)}</span>{/if}
  </div>
  {#if label}<span id="Label" class="label" part="OrbMeter.label OrbMeter.tag">{label}</span>{/if}
</div>

<style>
:host { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
:host { align-items: center; --meter: var(--orb-meter-health); --fill-color: var(--meter); --layout-anchor-inset: 0; }
.mana { --meter: var(--orb-meter-mana); --fill-color: var(--meter); }
/* The glass's depth takes its liquid's hue, and the figure the ink that
   reads on it, unless a pack names its own (--orb-meter-well,
   --orb-meter-figure-ink). */
:host { --orb-well: var(--orb-meter-well, color-mix(in oklab, var(--meter) 24%, var(--color-shadow))); --orb-inset: calc(var(--orb-meter-rim-width) + var(--orb-meter-socket-width)); }
[part~="OrbMeter.orb"] { width: var(--orb-meter-size); height: var(--orb-meter-size); }
#Bezel { background: var(--orb-meter-rim-fill); border-radius: var(--orb-meter-radius); corner-shape: var(--orb-meter-corner-shape); box-shadow: var(--orb-meter-rim-shadow); }
#Socket { position: absolute; inset: var(--orb-meter-rim-width); background: var(--orb-meter-socket-fill); border-radius: var(--orb-meter-radius); corner-shape: var(--orb-meter-corner-shape); }
#Glass { position: absolute; inset: var(--orb-inset); overflow: hidden; border-radius: var(--orb-meter-radius); corner-shape: var(--orb-meter-corner-shape); clip-path: border-box; }
/* The glass's lip: a ring of light inside its edge, whatever its shape. */
#Lip { position: absolute; inset: var(--orb-inset); border-radius: var(--orb-meter-radius); corner-shape: var(--orb-meter-corner-shape); box-shadow: var(--orb-meter-glass-shadow); pointer-events: none; }
#Vessel { display: block; inline-size: 100%; block-size: 100%; }
#Ornament { position: absolute; inset: calc(-1 * var(--orb-meter-ornament-outset)); background: var(--orb-meter-ornament); pointer-events: none; }
.well { fill: var(--orb-well); }
.liquid { fill: var(--meter); stroke: none; }
.swell { fill: color-mix(in oklab, var(--meter) 62%, var(--color-shadow)); opacity: var(--orb-meter-swell); }
.surface { fill: none; stroke: color-mix(in oklab, var(--meter) 45%, var(--orb-meter-light)); stroke-width: var(--orb-meter-surface-width); }
.depth { fill: url(#OrbMeterDepth); opacity: var(--orb-meter-depth); }
.shade { fill: url(#OrbMeterShade); opacity: var(--orb-meter-shade); }
.lume { fill: url(#OrbMeterGleam); opacity: var(--orb-meter-gleam); }
.glint { fill: none; stroke: var(--orb-meter-light); stroke-width: 3.2; stroke-linecap: round; opacity: calc(0.42 * var(--orb-meter-gleam)); }
.spark { fill: var(--orb-meter-light); opacity: calc(0.5 * var(--orb-meter-gleam)); }
.bounce { fill: none; stroke: var(--orb-meter-light); stroke-width: 1.6; stroke-linecap: round; opacity: calc(0.22 * var(--orb-meter-gleam)); }
[part~="OrbMeter.value"] { --figure-size: var(--orb-meter-figure-size); --figure-ink: var(--orb-meter-figure-ink, contrast-color(var(--orb-well))); --figure-shadow: var(--orb-meter-figure-shadow); }
</style>