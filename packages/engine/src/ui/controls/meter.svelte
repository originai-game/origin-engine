<meta name="control" content="meter">
<meta name="parts" content="track fill">
<meta name="prop:value" content="number" data-doc="The measured value.">
<meta name="prop:min" content="number" data-doc="The range's low end.">
<meta name="prop:max" content="number" data-default="1" data-doc="The range's high end.">
<meta name="prop:region" content="string" data-default="optimum" data-doc="HTML's region of the value: optimum, suboptimum or poor.">

<div id="Track" part="track" style:--fraction={max > min ? (value - min) / (max - min) : 0}>
  <div id="Fill" part="fill" hidden={!(value > min)}></div>
</div>

<style>
:host { display: block; inline-size: var(--size-4xl); block-size: var(--space-md); }
[part="track"] { inline-size: 100%; block-size: 100%; overflow: hidden; border-radius: var(--meter-radius); background: var(--control-strong); box-shadow: var(--meter-shadow); }
[part="fill"] { inline-size: calc(clamp(0, var(--fraction), 1) * 100%); block-size: 100%; border-radius: var(--meter-radius); background: var(--success); box-shadow: var(--meter-fill-shadow); }
:host([region="suboptimum"]) [part="fill"] { background: var(--warn); }
:host([region="poor"]) [part="fill"] { background: var(--danger); }
</style>
