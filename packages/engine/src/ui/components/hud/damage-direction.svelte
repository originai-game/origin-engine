<meta name="component" content="DamageDirection">
<meta name="strings" content="./damage-direction.en.ogt">
<meta name="prop:hits" content="list" data-doc="Recent hits, a keyed collection of { id, angle, strength, kind }: angle in radians from straight ahead, clockwise, relative to the view; strength 0 to 1 (the arc's opacity and length); kind normal, shield (the shield took it) or critical. The game removes a hit when its mark should go.">
<meta name="parts" content="DamageDirection.arc DamageDirection.fill">
<meta name="fixture:default" content='{"hits":[{"id":"h1","angle":0.9,"strength":0.9,"kind":"normal"},{"id":"h2","angle":-2.4,"strength":0.5,"kind":"shield"}]}'>
<meta name="fixture:empty" content='{"hits":[]}'>
<meta name="fixture:crowded" content='{"hits":[{"id":"a","angle":0,"strength":1,"kind":"critical"},{"id":"b","angle":1.57,"strength":0.7,"kind":"normal"},{"id":"c","angle":3.14,"strength":0.4,"kind":"normal"},{"id":"d","angle":-1.57,"strength":0.6,"kind":"shield"},{"id":"e","angle":-0.6,"strength":0.3,"kind":"normal"}]}'>

<div id="DamageDirection" data-layout="overlay" role="image" aria-label={t("shooter.damage.name")}>
  {#each hits as h (h.id)}
    <div class="hit" data-anchor="fill" style:--angle={h.angle} style:--strength={h.strength} transition:fade={{ duration: 300 }}>
      <svg class="ring" viewBox="0 0 100 100"><path class="arc" part="DamageDirection.arc DamageDirection.fill" data-tone={h.kind == 'shield' ? 'info' : 'danger'} class:shield={h.kind == 'shield'} class:critical={h.kind == 'critical'} d="M 4 50 A 46 46 0 0 1 96 50" fill="none" pathLength="1" stroke-dasharray={(0.18 + h.strength * 0.14) + " 1"} stroke-dashoffset={-(0.41 - h.strength * 0.07)}></path></svg>
    </div>
  {/each}
</div>

<style>
/* Each hit is a stretch of a ring around the centre, turned to where the
   damage came from: the upper half circle's middle, its length and weight
   by the hit's strength. */
:host { width: 100%; height: 100%; min-width: calc(var(--size-xxl) * 2); min-height: calc(var(--size-xxl) * 2); pointer-events: none; --layout-anchor-inset: 0; }
.hit { rotate: calc(var(--angle) * 1rad); opacity: calc(0.5 + var(--strength) * 0.5); }
.ring { width: 100%; height: 100%; }
.arc { stroke-width: var(--zone-rim-width); stroke-linecap: round; }
.arc.critical { stroke-width: calc(var(--zone-rim-width) * 1.5); }
</style>
