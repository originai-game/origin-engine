<meta name="component" content="InteractionPrompt">
<meta name="strings" content="./world.en.ogt">
<meta name="parts" content="InteractionPrompt.frame InteractionPrompt.cap InteractionPrompt.key InteractionPrompt.ring InteractionPrompt.plate InteractionPrompt.verb InteractionPrompt.object InteractionPrompt.reason">
<meta name="prop:entity" content="number" data-doc="What it is over, by its bits (0: use at).">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="A world point it sits over without an entity (x y z).">
<meta name="prop:height" content="number" data-doc="World units above the point.">
<meta name="prop:action" content="string" data-doc="The action whose glyph the key cap shows, its binding on the device used last (interact); empty shows key.">
<meta name="prop:key" content="string" data-doc="The key as the game names it (E), shown when action is empty.">
<meta name="prop:pad" content="bool" data-doc="A gamepad's face button: a round cap.">
<meta name="prop:verb" content="string" data-doc="What it does, in condensed capitals (Open, Talk, Revive); it always fits whole.">
<meta name="prop:name" content="string" data-doc="What it is under the verb (Gilded Chest); empty is none.">
<meta name="prop:pressed" content="bool" data-doc="The key is down: the cap sinks 2 px and lights.">
<meta name="prop:hold" content="number" data-default="-1" data-doc="A hold's progress 0 to 1: a white ring round the cap on a dark disc (1 is done: the cap lights); negative draws none. Released, the game sets 0 and the ring drains in 200 ms.">
<meta name="prop:disabled" content="bool" data-doc="It cannot be used now: the cap greys and the reason shows in red with a lock.">
<meta name="prop:reason" content="string" data-doc="Why it cannot (Needs the Warden's key).">
<meta name="prop:maxDistance" content="number" data-default="6" data-doc="Hidden past this many world units (6).">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","height":1.2,"action":"","key":"E","pad":false,"verb":"Open","name":"Gilded Chest","pressed":false,"hold":-1,"disabled":false,"reason":"","maxDistance":6}'>
<meta name="fixture:pressed" content='{"pressed":true}'>
<meta name="fixture:hold" content='{"key":"F","verb":"Hold · Revive","name":"Brannoch","pressed":true,"hold":0.45}'>
<meta name="fixture:done" content='{"key":"F","verb":"Revive","name":"Brannoch","hold":1}'>
<meta name="fixture:disabled" content='{"verb":"Open","name":"Iron Door","disabled":true,"reason":"Needs the Warden’s key"}'>
<meta name="fixture:gamepad" content='{"key":"A","pad":true,"verb":"Talk","name":"Mira Ashgrove"}'>
<meta name="fixture:action" content='{"action":"interact","key":""}'>
<meta name="fixture:overflow" content='{"key":"Left Shoulder","verb":"Pick up and equip","name":"Blade of the Ninefold King’s Last Oath, Reforged in Ember"}'>

<!--
  What the player can do with what is in front of them: a key cap beside
  a glass plate, the verb in condensed capitals and the object under it;
  a hold's ring round the cap. It rises in when in range. The game decides
  when it shows and what the input does; read only.
-->
<anchor id="InteractionPrompt" entity={entity || null} at={at} offset={"0 " + height + " 0"} declutter="shift" avoid="danger" occlusion="none" priority="300" max-distance={maxDistance}>
  <div id="Prompt" part="InteractionPrompt.frame" class="bare" role="status" aria-disabled={disabled} data-pressed={pressed} data-done={hold >= 1} data-disabled={disabled} data-pad={pad}>
    <div part="InteractionPrompt.cap" class="cap">
      {#if hold >= 0}<svg part="InteractionPrompt.ring" class="ring" viewBox="0 0 36 36" role="meter" aria-label={t("world.prompt.hold")} aria-valuemin="0" aria-valuemax="1" aria-valuenow={hold}><circle class="disc" cx="18" cy="18" r="18"></circle><circle class="sweep" cx="18" cy="18" r="16" pathLength="100" style:--hold={hold >= 1 ? 1 : hold}></circle></svg>{/if}
      {#if action != ''}<kbd part="InteractionPrompt.key" class="key" data-action={action}></kbd>{:else}<kbd part="InteractionPrompt.key" class="key">{key}</kbd>{/if}
    </div>
    <div part="InteractionPrompt.plate" class="plate">
      <span part="InteractionPrompt.verb" class="verb">{verb}</span>
      {#if name != ''}<span part="InteractionPrompt.object" class="object">{name}</span>{/if}
      {#if disabled && reason != ''}<div part="InteractionPrompt.reason" class="reason"><og-icon class="lock" icon="lock"></og-icon><span class="why">{reason}</span></div>{/if}
    </div>
  </div>
</anchor>

<style>
#Prompt { display: flex; align-items: center; color: var(--hud-ink); filter: var(--world-speech-shadow); animation: var(--world-prompt-motion); }
/* The cap and its hold ring: art sized by the text scale, so the key's
   glyph stays on it. */
.cap { position: relative; z-index: 1; flex: none; display: flex; align-items: center; justify-content: center; min-inline-size: calc(36 * var(--font-step-px)); block-size: calc(36 * var(--font-step-px)); }
.ring { position: absolute; left: 0; top: 0; inline-size: 100%; block-size: 100%; rotate: -90deg; filter: var(--world-hold-glow); }
.disc { fill: var(--world-hold-disc); }
.sweep { fill: none; stroke: var(--world-hold-ring); stroke-width: 3; stroke-dasharray: 100; stroke-dashoffset: calc(100 - var(--hold) * 100); }
[data-pressed=false] .sweep { transition: var(--world-hold-drain); }
.key { display: inline-block; min-inline-size: calc(26 * var(--font-step-px)); block-size: calc(26 * var(--font-step-px)); padding: 0 calc(5 * var(--font-step-px)); box-sizing: border-box; text-align: center; font-size: var(--hud-size-feed); line-height: calc(26 * var(--font-step-px)); font-weight: var(--hud-figure-weight); white-space: nowrap; color: var(--hud-key-ink); background: var(--hud-key-fill); border-radius: var(--hud-key-radius); box-shadow: var(--hud-key-shadow); }
[data-pressed=true] .key { translate: 0 var(--space-xxs); background: var(--hud-key-lit-fill); box-shadow: var(--hud-key-lit-shadow); }
[data-done=true] .key { background: var(--hud-key-lit-fill); box-shadow: var(--hud-key-lit-shadow); }
[data-pad=true] .key { inline-size: calc(26 * var(--font-step-px)); min-inline-size: 0; padding: 0; color: var(--world-pad-ink); background: var(--world-pad-fill); border-radius: var(--dot-radius); box-shadow: var(--world-pad-rim); }
[data-disabled=true] .key { color: var(--world-key-off-ink); background: var(--world-key-off-fill); }
/* The plate: the HUD's glass with a cut corner, tucked under the cap. A
   text box: it grows with the text scale. */
.plate { display: flex; flex-direction: column; min-inline-size: 0; margin-inline-start: calc(-10 * var(--font-step-px)); padding: calc(4 * var(--font-step-px)) calc(12 * var(--font-step-px)) calc(5 * var(--font-step-px)) calc(16 * var(--font-step-px)); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(0 0, 100% 0, 100% calc(100% - var(--hud-plate-cut)), calc(100% - var(--hud-plate-cut)) 100%, 0 100%); }
.verb { font-size: var(--world-size-name); line-height: 1.15; font-weight: var(--hud-figure-weight); font-stretch: var(--world-caps-stretch); letter-spacing: var(--world-caps-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; }
[data-disabled=true] .verb { color: var(--hud-ink-dim); }
.object { min-inline-size: 0; max-inline-size: calc(200 * var(--font-step-px)); font-size: var(--world-size-title); line-height: 1.2; font-weight: var(--hud-label-weight); letter-spacing: var(--world-title-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.reason { display: flex; align-items: center; column-gap: calc(4 * var(--font-step-px)); min-inline-size: 0; font-size: var(--world-size-title); line-height: 1.2; font-weight: var(--hud-figure-weight); letter-spacing: var(--world-title-tracking); text-transform: var(--hud-label-transform); color: var(--world-hostile-ink); white-space: nowrap; }
.lock { flex: none; inline-size: var(--world-mark-size); block-size: var(--world-mark-size); }
.why { min-inline-size: 0; max-inline-size: calc(200 * var(--font-step-px)); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
@media (prefers-reduced-motion: reduce) {
  #Prompt { animation: none; }
  [data-pressed=true] .key { translate: none; }
  [data-pressed=false] .sweep { transition: none; }
}
</style>
