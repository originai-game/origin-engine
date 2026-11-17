<meta name="component" content="InteractionPrompt">
<meta name="parts" content="InteractionPrompt.frame InteractionPrompt.key InteractionPrompt.verb InteractionPrompt.name InteractionPrompt.reason InteractionPrompt.label InteractionPrompt.headline InteractionPrompt.caption">
<meta name="prop:entity" content="number" data-doc="What it is over, by its bits (0: use at).">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="A world point it sits over without an entity (x y z).">
<meta name="prop:height" content="number" data-doc="World units above the point.">
<meta name="prop:action" content="string" data-doc="The action whose glyph it shows, its binding on the device used last (interact).">
<meta name="prop:verb" content="string" data-doc="What it does (Open, Talk, Pick up).">
<meta name="prop:name" content="string" data-doc="What it is (Chest); empty is none.">
<meta name="prop:disabled" content="bool" data-doc="It cannot be used now: dimmed, with the reason.">
<meta name="prop:reason" content="string" data-doc="Why it cannot (Locked).">
<meta name="prop:maxDistance" content="number" data-default="6" data-doc="Hidden past this many world units (6).">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","height":1.2,"action":"interact","verb":"Open","name":"Chest","disabled":false,"reason":""}'>

<anchor id="InteractionPrompt" entity={entity || null} at={at} offset={"0 " + height + " 0"} declutter="shift" occlusion="none" priority="300" max-distance={maxDistance}>
  <div id="Prompt" part="InteractionPrompt.frame" class="hud" data-layout="row" data-motion="enter" class:disabled={disabled} aria-disabled={disabled} role="status">
    <kbd id="Glyph" part="InteractionPrompt.key InteractionPrompt.label" data-action={action}></kbd>
    <div id="Words" data-layout="stack" data-gap="hair">
      <span id="Verb" part="InteractionPrompt.verb InteractionPrompt.headline">{verb}</span>
      {#if name}<span id="Name" part="InteractionPrompt.name InteractionPrompt.caption">{name}</span>{/if}
      {#if disabled && reason}<span id="Reason" part="InteractionPrompt.reason InteractionPrompt.label InteractionPrompt.caption" data-tone="danger">{reason}</span>{/if}
    </div>
  </div>
</anchor>

<style>
:host { text-shadow: var(--tag-halo); }
[part~="InteractionPrompt.verb"], [part~="InteractionPrompt.name"], [part~="InteractionPrompt.reason"] { white-space: nowrap; }
</style>
