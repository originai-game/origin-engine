<meta name="control" content="progress">
<meta name="parts" content="track fill">
<meta name="prop:value" content="number" data-doc="Done, 0 to 1.">

<div id="Track" part="track" style:--fraction={value}>
  <div id="Fill" part="fill" hidden={!(value > 0)}></div>
</div>

<style>
:host { display: block; inline-size: stretch; block-size: var(--space-sm); }
[part="track"] { inline-size: 100%; block-size: 100%; overflow: hidden; border-radius: var(--meter-radius); background: var(--control-strong); box-shadow: var(--meter-shadow); }
[part="fill"] { inline-size: calc(clamp(0, var(--fraction), 1) * 100%); min-inline-size: min(100%, var(--space-sm)); block-size: 100%; border-radius: var(--meter-radius); background: var(--primary); box-shadow: var(--meter-fill-shadow); transition: inline-size var(--motion-duration-fast) var(--motion-easing-out); }
</style>
