<meta name="component" content="Celebration">
<meta name="strings" content="./celebration.en.ogt">
<meta name="parts" content="Celebration.ring Celebration.shell Celebration.rays Celebration.halo Celebration.title Celebration.value">
<meta name="prop:kind" content="string" data-doc="Which one: confetti (falling over its box from the top edge), burst (a ring and sparks thrown from its centre), levelUp (a ring of light, rays turning behind the title and the new level, sparks rising) or fireworks (shells bursting across its top).">
<meta name="prop:shots" content="list" data-doc="Its plays, each { id } (the tick it fired, as text): a new id plays it again; a kept one plays once.">
<meta name="prop:title" content="string" data-doc="levelUp: the big word; empty is Level up.">
<meta name="prop:value" content="string" data-doc="levelUp: the new level (12); empty is none.">
<meta name="fixture:default" content='{"kind":"levelUp","shots":[{"id":"1"}],"title":"","value":"12"}'>

<div id="Celebration" inert>
  {#each shots as shot (shot.id)}
    <div class="play" class:confetti={kind == 'confetti'} class:burst={kind == 'burst'} class:level={kind == 'levelUp'} class:fireworks={kind == 'fireworks'}>
      <og-emitter class="sky" effect="confetti 70 burst 110 for 1.4s"></og-emitter>
      <div part="Celebration.ring" class="ring"></div>
      <og-emitter class="sparks" effect="sparks burst 26"></og-emitter>
      <div class="shells">
        <div part="Celebration.shell" class="shell one"></div>
        <div part="Celebration.shell" class="shell two"></div>
        <div part="Celebration.shell" class="shell three"></div>
        <div part="Celebration.shell" class="shell four"></div>
        <div part="Celebration.shell" class="shell five"></div>
      </div>
      <div class="stage" role="status" aria-label={(title || t("component.celebration.level-up")) + " " + value}>
        <svg part="Celebration.rays" class="rays" viewBox="-50 -50 100 100">
          <polygon points="0,-50 4,-9 22,-45 8,-5 39,-31 10,0 48,-11 9,4 45,22 5,8 31,39 0,10 11,48 -4,9 -22,45 -8,5 -39,31 -10,0 -48,11 -9,-4 -45,-22 -5,-8 -31,-39 0,-10 -11,-48 4,-9"></polygon>
        </svg>
        <div part="Celebration.halo" class="halo"></div>
        <span part="Celebration.title" class="title">{title || t("component.celebration.level-up")}</span>
        <span part="Celebration.value" class="value" hidden={!value}>{value}</span>
        <og-emitter class="rise" effect="embers 40 burst 30 for 1.6s life 1.2s"></og-emitter>
      </div>
    </div>
  {/each}
</div>

<style>
#Celebration { position: absolute; top: 0; bottom: 0; inset-inline-start: 0; inset-inline-end: 0; }
.play { position: absolute; top: 0; bottom: 0; inset-inline-start: 0; inset-inline-end: 0; align-items: center; justify-content: center; animation-name: cheer; animation-duration: var(--motion-duration-instant); animation-fill-mode: both; }
.level, .fireworks { animation-name: reward; }
.sky, .ring, .sparks, .stage { display: none; }

.confetti .sky { display: block; position: absolute; top: 0; inset-inline-start: 0; inset-inline-end: 0; height: var(--space-xxs); }

.burst .ring { display: block; width: var(--size-xl); height: var(--size-xl); border-radius: var(--radius-capsule); border: var(--stroke-thick) solid var(--highlight); box-shadow: 0 0 var(--space-sm) var(--highlight); animation-name: ring; animation-duration: calc(var(--motion-duration-spring) * 2); animation-timing-function: var(--motion-easing-out); animation-fill-mode: both; }
.burst .sparks { display: block; position: absolute; width: var(--space-sm); height: var(--space-sm); }

.shells { display: none; }
.fireworks .shells { display: flex; flex-direction: row; position: absolute; top: 0; inset-inline-start: 0; inset-inline-end: 0; height: var(--size-6xl); justify-content: space-between; padding: 0 var(--space-3xl); }
.shell { width: var(--space-sm); height: var(--space-sm); border-radius: var(--radius-capsule); background: var(--gold); opacity: 0; animation-name: shell; animation-duration: calc(var(--motion-duration-slow) * 4); animation-timing-function: var(--motion-easing-out); animation-fill-mode: both; }
.shell.one { margin-top: var(--space-3xl); }
.shell.two { margin-top: var(--space-xl); animation-delay: calc(var(--motion-duration-base) * 4); }
.shell.three { margin-top: calc(var(--space-3xl) * 2); animation-delay: var(--motion-duration-base); }
.shell.four { margin-top: var(--space-xxl); animation-delay: calc(var(--motion-duration-base) * 2); }
.shell.five { margin-top: calc(var(--space-3xl) + var(--space-xl)); animation-delay: calc(var(--motion-duration-base) * 3); }

.level .stage { display: flex; width: var(--measure-lg); height: var(--size-7xl); align-items: center; justify-content: center; animation-name: stage; animation-duration: calc(var(--motion-duration-bounce) * 4); animation-timing-function: linear; animation-fill-mode: both; }
.rays polygon { fill: var(--gold); fill-opacity: var(--opacity-faint); }
.rays { position: absolute; width: var(--measure-lg); height: var(--measure-lg); animation-name: rays; animation-duration: calc(var(--motion-duration-bounce) * 4); animation-timing-function: linear; animation-fill-mode: both; }
.halo { position: absolute; width: var(--size-5xl); height: var(--size-5xl); border-radius: var(--radius-capsule); border: var(--stroke-thick) solid var(--gold); box-shadow: 0 0 var(--space-lg) var(--gold); animation-name: halo; animation-duration: calc(var(--motion-duration-slow) * 2); animation-timing-function: var(--motion-easing-out); animation-fill-mode: both; }
.title { font: var(--text-xl5); text-transform: uppercase; white-space: nowrap; color: var(--gold); text-shadow: 0 0 var(--stroke-regular) var(--scene-shadow), 0 var(--stroke-regular) 0 var(--scene-shadow); animation-name: title; animation-duration: var(--motion-duration-spring); animation-timing-function: var(--motion-easing-bounce); animation-fill-mode: both; }
.value { font: var(--text-xl4); white-space: nowrap; color: var(--gold); text-shadow: 0 0 var(--stroke-regular) var(--scene-shadow), 0 var(--stroke-regular) 0 var(--scene-shadow); animation-name: value; animation-duration: var(--motion-duration-base); animation-timing-function: var(--motion-easing-out); animation-delay: var(--motion-duration-base); animation-fill-mode: both; }
.rise { position: absolute; bottom: var(--space-xxl); width: var(--measure-sm); height: var(--space-xs); }

@keyframes cheer { from { -og-sound: success; } }
@keyframes reward { from { -og-sound: success; } }
@keyframes ring { from { scale: 0.4; opacity: 1; } to { scale: 2.8; opacity: 0; } }
@keyframes shell { 0% { opacity: 1; scale: 1; } 20% { opacity: 0; scale: 4; } 100% { opacity: 0; } }
@keyframes stage { 0% { opacity: 1; } 83% { opacity: 1; } 100% { opacity: 0; } }
@keyframes rays { from { rotate: 0deg; scale: 0.6; opacity: 0; } 17% { scale: 1; opacity: 0.9; } to { rotate: 90deg; scale: 1; opacity: 0.9; } }
@keyframes halo { from { scale: 0.3; opacity: 1; } to { scale: 2.2; opacity: 0; } }
@keyframes title { from { scale: 0.4; } to { scale: 1; } }
@keyframes value { from { translate: 0 var(--space-xl); opacity: 0; } to { translate: 0 0; opacity: 1; } }

@media (prefers-reduced-motion) {
  .rays, .halo { display: none; }
  .title { animation: none; }
}
</style>
