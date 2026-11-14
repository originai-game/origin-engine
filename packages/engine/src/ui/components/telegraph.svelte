<meta name="component" content="Telegraph">
<meta name="parts" content="Telegraph.area">
<meta name="prop:entity" content="number" data-doc="Its origin's entity, by its bits (0: use at).">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="Its origin on the ground without an entity (x y z).">
<meta name="prop:shape" content="string" data-doc="Its area: disc (empty), ring, cone or line.">
<meta name="prop:radius" content="number" data-default="1" data-doc="A disc's or ring's radius, a cone's or line's length, in world units.">
<meta name="prop:inner" content="number" data-doc="A ring's inner radius as a fraction of radius (0 to 1).">
<meta name="prop:angle" content="number" data-default="90" data-doc="A cone's width in degrees.">
<meta name="prop:width" content="number" data-default="0.2" data-doc="A line's width as a fraction of its length (0 to 1).">
<meta name="prop:direction" content="number" data-doc="Which way a cone or line faces, degrees clockwise from +z seen from above.">
<meta name="prop:progress" content="number" data-doc="The wind-up 0 to 1: the fill grows to the edge as the hit nears.">
<meta name="prop:friendly" content="bool" data-doc="An ally's (info); false is an enemy's (danger).">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","shape":"cone","radius":6,"inner":0.5,"angle":70,"width":0.2,"direction":30,"progress":0.55,"friendly":false}'>

<ground id="Telegraph" entity={entity || null} at={at} size={(radius * 2) + " " + (radius * 2)} rotation={"0 " + direction + " 0"}>
  <svg id="Area" part="Telegraph.area" viewBox="0 0 200 200" class:friendly={friendly} style:--p={progress}>
    <g hidden={!(shape == '' || shape == 'disc')}>
      <circle class="edge" cx="100" cy="100" r="98"></circle>
      <circle class="fill" cx="100" cy="100" r={98 * progress}></circle>
    </g>
    <g hidden={shape != 'ring'}>
      <circle class="band" cx="100" cy="100" r={49 * (1 + inner)} stroke-width={98 * (1 - inner)}></circle>
      <circle class="line" cx="100" cy="100" r="98"></circle>
      <circle class="line" cx="100" cy="100" r={98 * inner}></circle>
      <circle class="growth" cx="100" cy="100" r={98 * inner + 98 * (1 - inner) * progress}></circle>
    </g>
    <g hidden={shape != 'cone'}>
      <circle class="sector" cx="100" cy="100" r="49" stroke-width="98" pathLength="360" stroke-dasharray={angle + " 360"} transform={"rotate(" + (-90 - angle / 2) + " 100 100)"}></circle>
      <circle class="sector fill" cx="100" cy="100" r={49 * progress} stroke-width={98 * progress} pathLength="360" stroke-dasharray={angle + " 360"} transform={"rotate(" + (-90 - angle / 2) + " 100 100)"}></circle>
    </g>
    <g hidden={shape != 'line'}>
      <rect class="edge" x={100 - 100 * width / 2} y="2" width={200 * width / 2} height="98"></rect>
      <rect class="fill" x={100 - 100 * width / 2} y={100 - 98 * progress} width={200 * width / 2} height={98 * progress}></rect>
    </g>
  </svg>
</ground>

<style>
#Telegraph { width: var(--measure-xs); height: var(--measure-xs); }
[part="Telegraph.area"] { width: var(--measure-xs); height: var(--measure-xs); --tone: var(--danger); --wash: color-mix(in srgb, var(--danger) 25%, transparent); }
[part="Telegraph.area"].friendly { --tone: var(--info); --wash: color-mix(in srgb, var(--info) 25%, transparent); }
.edge { fill: var(--wash); stroke: var(--tone); stroke-width: 3; }
.fill { fill: var(--wash); }
.band { fill: none; stroke: var(--wash); }
.line { fill: none; stroke: var(--tone); stroke-width: 3; }
.growth { fill: none; stroke: var(--tone); stroke-width: 2; opacity: 0.7; }
.sector { fill: none; stroke: var(--wash); }
.sector.fill { stroke: var(--wash); }
</style>
