<meta name="component" content="RadialMeter">
<meta name="strings" content="./radial-meter.en.ogt">
<meta name="prop:value" content="number" data-doc="The value in wheels: 1 is one full wheel, up to 3.">
<meta name="prop:wheels" content="number" data-doc="Wheels (an extended stamina's second and third ring), 1 to 3.">
<meta name="prop:exhausted" content="bool" data-doc="Ran out: red until full again.">
<meta name="prop:hidden" content="bool" data-doc="Faded out (the game hides it once it has been full a while).">
<meta name="prop:name" content="string" data-doc="What a screen reader calls it; empty is Stamina.">
<meta name="fixture:default" content='{"value":1.4,"wheels":2,"exhausted":false,"hidden":false,"name":""}'>

<div id="RadialMeter" role="meter" aria-label={name || t("component.radial-meter.stamina")} aria-valuemin="0" aria-valuemax={wheels} aria-valuenow={value} class:exhausted={exhausted} class:gone={hidden}>
  <svg id="Wheels" viewBox="0 0 56 56">
    <circle class="track" cx="28" cy="28" r="17" fill="none" stroke-width="7"></circle>
    <circle class="fill" cx="28" cy="28" r="17" fill="none" stroke-width="7" pathLength="1" stroke-dasharray={(value >= 1 ? 1 : value) + " 1"} transform="rotate(-90 28 28)"></circle>
    <circle class="track outer" cx="28" cy="28" r="23" fill="none" stroke-width="3" opacity={wheels >= 2 ? 1 : 0}></circle>
    <circle class="fill" cx="28" cy="28" r="23" fill="none" stroke-width="3" pathLength="1" stroke-dasharray={(value - 1 >= 1 ? 1 : (value - 1 <= 0 ? 0 : value - 1)) + " 1"} transform="rotate(-90 28 28)"></circle>
    <circle class="track outer" cx="28" cy="28" r="27" fill="none" stroke-width="2" opacity={wheels >= 3 ? 1 : 0}></circle>
    <circle class="fill" cx="28" cy="28" r="27" fill="none" stroke-width="2" pathLength="1" stroke-dasharray={(value - 2 >= 1 ? 1 : (value - 2 <= 0 ? 0 : value - 2)) + " 1"} transform="rotate(-90 28 28)"></circle>
  </svg>
</div>

<style>
#RadialMeter { width: 56px; height: 56px; --meter: var(--success); transition: opacity 400ms ease-out; }
#RadialMeter.exhausted { --meter: var(--danger); }
#RadialMeter.gone { opacity: 0; }
#Wheels { width: 56px; height: 56px; }
.track { stroke: #00000088; }
.fill { stroke: var(--meter); }
</style>
