<meta name="component" content="RpgLockOn">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgLockOn.frame RpgLockOn.ring">
<meta name="prop:target" content="string" data-doc="The locked target's name for a reader; empty hides the reticle (nothing is locked).">
<meta name="fixture:default" content='{"target":"Ashen Colossus"}'>
<meta name="fixture:empty" content='{"target":""}'>

<!--
  The lock-on reticle the page centres on the locked target: four lit arcs
  round it with a glow in the HUD's accent. The game decides what is
  locked and where it stands on screen; read only.
-->
<section id="RpgLockOn" part="RpgLockOn.frame" class="hud bare" data-layout="overlay" role="status" aria-label={target != '' ? t("rpg.lock.target", { name: target }) : t("rpg.lock.none")} data-locked={target != ''}>
  {#if target != ''}<div id="Ring" part="RpgLockOn.ring" data-anchor="fill" data-motion="pop"></div>{/if}
</section>

<style>
:host { --side: calc(var(--size-xxl) * 0.9); inline-size: var(--side); block-size: var(--side); flex: none; pointer-events: none; --layout-anchor-inset: 0; }
/* The ring: a thin band of the arcs, cut from a disc by a mask. */
#Ring { border-radius: var(--hud-radar-radius); background: var(--hud-lock-ring); mask-image: radial-gradient(closest-side, transparent 93%, #000 94%); filter: var(--hud-lock-glow); }
@media (width < 600px) { :host { --side: calc(var(--size-xxl) * 0.6); } }
</style>
