<meta name="component" content="InteractionPrompt">
<meta name="parts" content="InteractionPrompt.frame InteractionPrompt.key InteractionPrompt.verb InteractionPrompt.name InteractionPrompt.reason InteractionPrompt.label">
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
  <div id="Prompt" part="InteractionPrompt.frame" class="hud" class:disabled={disabled} role="status">
    <kbd id="Glyph" part="InteractionPrompt.key InteractionPrompt.label" data-action={action}></kbd>
    <div id="Words">
      <span id="Verb" part="InteractionPrompt.verb">{verb}</span>
      {#if name}<span id="Name" part="InteractionPrompt.name">{name}</span>{/if}
      {#if disabled && reason}<span id="Reason" part="InteractionPrompt.reason InteractionPrompt.label">{reason}</span>{/if}
    </div>
  </div>
</anchor>

<style>
:host { text-shadow: var(--tag-halo); }
[part="InteractionPrompt.frame"] { display: flex; gap: var(--space-sm); align-items: center; animation: in var(--motion-duration-fast) var(--motion-easing-out); }
[part="InteractionPrompt.frame"].disabled { opacity: var(--opacity-dim); }
[part="InteractionPrompt.key InteractionPrompt.label"] { font: var(--text-sm); }
#Words { display: flex; flex-direction: column; gap: var(--space-xxs); }
[part="InteractionPrompt.verb"] { font: var(--text-sm); color: var(--ink); white-space: nowrap; }
[part="InteractionPrompt.name"] { font: var(--text-caption); color: var(--ink-muted); white-space: nowrap; }
[part="InteractionPrompt.reason InteractionPrompt.label"] { font: var(--text-caption); color: var(--danger); white-space: nowrap; }
@keyframes in { from { translate: 0 var(--space-xs); opacity: 0; } to { translate: 0 0; opacity: 1; } }
</style>
