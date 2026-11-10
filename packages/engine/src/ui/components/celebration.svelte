<meta name="component" content="Celebration">
<meta name="strings" content="./celebration.en.ogt">
<meta name="prop:kind" content="string" data-doc="Which one: confetti (falling over its box from the top edge), burst (a ring and sparks thrown from its centre), levelUp (a ring of light, rays turning behind the title and the new level, sparks rising) or fireworks (shells bursting across its top).">
<meta name="prop:shots" content="list" data-doc="Its plays, each { id } (the tick it fired, as text): a new id plays it again; a kept one plays once.">
<meta name="prop:title" content="string" data-doc="levelUp: the big word; empty is Level up.">
<meta name="prop:value" content="string" data-doc="levelUp: the new level (12); empty is none.">
<meta name="fixture:default" content='{"kind":"levelUp","shots":[{"id":"1"}],"title":"","value":"12"}'>

<div id="Celebration" inert>
  {#each shots as shot (shot.id)}
    <div class="play" class:confetti={kind == 'confetti'} class:burst={kind == 'burst'} class:level={kind == 'levelUp'} class:fireworks={kind == 'fireworks'}>
      <og-emitter class="sky" effect="confetti 70 burst 110 for 1.4s"></og-emitter>
      <div class="ring"></div>
      <og-emitter class="sparks" effect="sparks burst 26 diamond 7px"></og-emitter>
      <div class="shells">
        <div class="shell one"></div>
        <div class="shell two"></div>
        <div class="shell three"></div>
        <div class="shell four"></div>
        <div class="shell five"></div>
      </div>
      <div class="stage" role="status" aria-label={(title || t("component.celebration.level-up")) + " " + value}>
        <svg class="rays" viewBox="-50 -50 100 100">
          <defs><radialGradient id="Glow"><stop offset="0" stop-color="#ffd166" stop-opacity="0.55"></stop><stop offset="0.6" stop-color="#ffd166" stop-opacity="0.16"></stop><stop offset="1" stop-color="#ffd166" stop-opacity="0"></stop></radialGradient></defs>
          <polygon points="0,-50 4,-9 22,-45 8,-5 39,-31 10,0 48,-11 9,4 45,22 5,8 31,39 0,10 11,48 -4,9 -22,45 -8,5 -39,31 -10,0 -48,11 -9,-4 -45,-22 -5,-8 -31,-39 0,-10 -11,-48 4,-9" fill="url(#Glow)"></polygon>
        </svg>
        <div class="halo"></div>
        <span class="title">{title || t("component.celebration.level-up")}</span>
        <span class="value" hidden={!value}>{value}</span>
        <og-emitter class="rise" effect="embers 40 burst 30 for 1.6s diamond 6px life 1.2s 80px/s 220px/s #ffd166 #fff6cc"></og-emitter>
      </div>
    </div>
  {/each}
</div>

<style>
#Celebration { position: absolute; top: 0; bottom: 0; inset-inline-start: 0; inset-inline-end: 0; }
.play { position: absolute; top: 0; bottom: 0; inset-inline-start: 0; inset-inline-end: 0; align-items: center; justify-content: center; animation: cheer 10ms both; }
.level, .fireworks { animation-name: reward; }
.sky, .ring, .sparks, .stage { display: none; }

.confetti .sky { display: block; position: absolute; top: 0; inset-inline-start: 0; inset-inline-end: 0; height: 2px; }

.burst .ring { display: block; width: 60px; height: 60px; border-radius: 30px; border: 4px solid var(--highlight); box-shadow: 0 0 10px var(--highlight); animation: ring 1s ease-out both; }
.burst .sparks { display: block; position: absolute; width: 8px; height: 8px; }

.shells { display: none; }
.fireworks .shells { display: flex; flex-direction: row; position: absolute; top: 0; inset-inline-start: 0; inset-inline-end: 0; height: 240px; justify-content: space-between; padding: 0 64px; }
.shell { width: 10px; height: 10px; border-radius: 5px; background: #ffffff; opacity: 0; animation: shell 1.5s ease-out both; }
.shell.one { margin-top: 64px; }
.shell.two { margin-top: 24px; animation-delay: 967ms; }
.shell.three { margin-top: 112px; animation-delay: 233ms; }
.shell.four { margin-top: 40px; animation-delay: 500ms; }
.shell.five { margin-top: 88px; animation-delay: 733ms; }

.level .stage { display: flex; width: 420px; height: 300px; align-items: center; justify-content: center; animation: stage 2.6s linear both; }
.rays { position: absolute; width: 420px; height: 420px; animation: rays 2.6s linear both; }
.halo { position: absolute; width: 200px; height: 200px; border-radius: 100px; border: 6px solid var(--gold); box-shadow: 0 0 18px var(--gold); animation: halo 800ms ease-out both; }
.title { font: var(--text-xl5); text-transform: uppercase; white-space: nowrap; color: var(--gold); text-shadow: 0 0 2px #000000, 0 2px 0 #00000099; animation: title 450ms cubic-bezier(0.34, 1.8, 0.64, 1) both; }
.value { font: var(--text-xl4); white-space: nowrap; color: var(--gold); text-shadow: 0 0 2px #000000, 0 2px 0 #00000099; animation: value 220ms ease-out 300ms both; }
.rise { position: absolute; bottom: 40px; width: 300px; height: 4px; }

@keyframes cheer { from { -og-sound: success; } }
@keyframes reward { from { -og-sound: success; } }
@keyframes ring { from { scale: 0.4; opacity: 1; } to { scale: 2.8; opacity: 0; } }
@keyframes shell { 0% { opacity: 1; scale: 1; -og-burst: 21 170px #ffd166 1500ms; } 20% { opacity: 0; scale: 4; } 100% { opacity: 0; } }
@keyframes stage { 0% { opacity: 1; } 83% { opacity: 1; } 100% { opacity: 0; } }
@keyframes rays { from { rotate: 0deg; scale: 0.6; opacity: 0; } 17% { scale: 1; opacity: 0.9; } to { rotate: 90deg; scale: 1; opacity: 0.9; } }
@keyframes halo { from { scale: 0.3; opacity: 1; } to { scale: 2.2; opacity: 0; } }
@keyframes title { from { scale: 0.4; } to { scale: 1; } }
@keyframes value { from { translate: 0 20px; opacity: 0; } to { translate: 0 0; opacity: 1; } }

@media (prefers-reduced-motion) {
  .rays, .halo { display: none; }
  .title { animation: none; }
}
</style>
