<meta name="component" content="PlacementGhost">
<meta name="prop:at" content="string" data-doc="Its footprint's centre on the ground (x y z).">
<meta name="prop:width" content="number" data-doc="Its width along x before rotation, world units.">
<meta name="prop:depth" content="number" data-doc="Its depth along z, world units.">
<meta name="prop:rotation" content="number" data-doc="Its turn about y in degrees, clockwise from above.">
<meta name="prop:cells" content="number" data-doc="Grid cells across its width, drawn on the footprint (0: none).">
<meta name="prop:valid" content="bool" data-doc="It can be placed here (success); false is danger with hatching.">
<meta name="prop:label" content="string" data-doc="Its cost or why not (Blocked); empty is none.">
<meta name="prop:range" content="number" data-doc="A range ring round it as a multiple of its width (a tower's reach); 0 is none.">
<meta name="fixture:default" content='{"at":"0 0 0","width":3,"depth":3,"rotation":0,"cells":3,"valid":false,"label":"Blocked","range":0}'>

<ground id="PlacementGhost" at={at} size={(width * (range > 1 ? range : 1)) + " " + (depth * (range > 1 ? range : 1))} rotation={"0 " + rotation + " 0"}>
  <div id="Room" class:invalid={!valid}>
    <svg id="Reach" viewBox="0 0 200 200" hidden={!(range > 1)}><circle cx="100" cy="100" r="98" fill="none" stroke-width="2" stroke-dasharray="8 6"></circle></svg>
    <div id="Footprint" style:--cells={cells} style:--k={range > 1 ? 1 / range : 1}>
      <div id="Grid" hidden={!(cells > 0)} style:--lines={"repeating-linear-gradient(0deg, #ffffff99 0px, #ffffff99 1px, #00000000 1px, #00000000 " + (200 / (cells || 1)) + "px), repeating-linear-gradient(90deg, #ffffff99 0px, #ffffff99 1px, #00000000 1px, #00000000 " + (200 / (cells || 1)) + "px)"}></div>
      <span id="Label" hidden={!label}>{label}</span>
    </div>
  </div>
</ground>

<style>
#PlacementGhost { width: 200px; height: 200px; }
#Room { width: 200px; height: 200px; align-items: center; justify-content: center; --tone: #3ddc84ff; --wash: #3ddc8433; }
#Room.invalid { --tone: #e5484dff; --wash: #e5484d40; }
#Reach { position: absolute; top: 0; inset-inline-start: 0; width: 200px; height: 200px; stroke: var(--tone); }
#Footprint { width: calc(var(--k) * 200px); height: calc(var(--k) * 200px); border: 3px solid var(--tone); background: var(--wash); align-items: center; justify-content: center; }
.invalid #Footprint { background: repeating-linear-gradient(45deg, #e5484d59 0px, #e5484d59 8px, #00000000 8px, #00000000 16px), #e5484d26; }
#Grid { position: absolute; top: 0; inset-inline-start: 0; width: stretch; height: stretch; opacity: 0.5; background: var(--lines); }
#Label { font: var(--text-lg); padding: 2px 8px; border-radius: 4px; background: #000000b3; color: var(--tone); white-space: nowrap; }
</style>
