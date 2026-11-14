<meta name="component" content="InteractionPrompt">
<meta name="parts" content="InteractionPrompt.frame InteractionPrompt.key InteractionPrompt.verb InteractionPrompt.name InteractionPrompt.reason">
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

<anchor id="InteractionPrompt" entity={entity || null} at={at} offset={"0 " + height + " 0"} occlusion="none" priority="300" max-distance={maxDistance}>
  <div id="Prompt" part="InteractionPrompt.frame" class="hud" class:disabled={disabled} role="status">
    <kbd id="Glyph" part="InteractionPrompt.key" data-action={action}></kbd>
    <div id="Words">
      <span id="Verb" part="InteractionPrompt.verb">{verb}</span>
      <span id="Name" part="InteractionPrompt.name" hidden={!name}>{name}</span>
      <span id="Reason" part="InteractionPrompt.reason" hidden={!(disabled && reason)}>{reason}</span>
    </div>
  </div>
</anchor>

<style>
#InteractionPrompt { align-items: center; }
[part="InteractionPrompt.frame"] { display: flex; gap: var(--space-sm); align-items: center; animation: in var(--motion-duration-fast) var(--motion-easing-out); }
[part="InteractionPrompt.frame"].disabled { opacity: var(--opacity-dim); }
[part="InteractionPrompt.key"] { font: var(--text-sm); }
#Words { gap: 0; }
[part="InteractionPrompt.verb"] { font: var(--text-sm); color: var(--ink); white-space: nowrap; }
[part="InteractionPrompt.name"] { font: var(--text-caption); color: var(--ink-muted); white-space: nowrap; }
[part="InteractionPrompt.reason"] { font: var(--text-caption); color: var(--danger); white-space: nowrap; }
@keyframes in { from { translate: 0 var(--space-xs); opacity: 0; } to { translate: 0 0; opacity: 1; } }
</style>
