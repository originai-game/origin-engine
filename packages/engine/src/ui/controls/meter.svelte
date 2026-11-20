<meta name="control" content="meter">
<meta name="prop:value" content="number" data-doc="The measured value.">
<meta name="prop:min" content="number" data-doc="The range's low end.">
<meta name="prop:max" content="number" data-default="1" data-doc="The range's high end.">
<meta name="prop:region" content="string" data-default="optimum" data-doc="HTML's region of the value: optimum, suboptimum or poor.">

<div id="Track" part="track" class:suboptimum={region == "suboptimum"} class:poor={region == "poor"} style:--fraction={max > min ? (value - min) / (max - min) : 0}>
  <div id="Fill" part="fill" hidden={!(value > min)}></div>
</div>

<style>
#Track { display: block; inline-size: var(--size-4xl); block-size: var(--space-md); }
#Track { overflow: hidden; border-radius: var(--meter-radius); background: var(--control-strong); box-shadow: var(--meter-shadow); }
#Fill { inline-size: calc(clamp(0, var(--fraction), 1) * 100%); block-size: 100%; border-radius: var(--meter-radius); background: var(--success); box-shadow: var(--meter-fill-shadow); }
#Track.suboptimum #Fill { background: var(--warn); }
#Track.poor #Fill { background: var(--danger); }
</style>
