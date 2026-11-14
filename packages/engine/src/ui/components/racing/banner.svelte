<meta name="component" content="RacingBanner">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingBanner.frame RacingBanner.plate RacingBanner.title RacingBanner.badge">
<meta name="prop:kind" content="string" data-default="none" data-doc="What the banner calls: countdown (the count), go, finalLap, finish, or none (nothing shown, as for any other word).">
<meta name="prop:figure" content="number" data-doc="The countdown's figure (3, 2, 1); the game steps it.">
<meta name="prop:at" content="number" data-doc="The call's id: a new value replays the entrance (each count, the go, the flag).">
<meta name="prop:place" content="number" data-doc="On the finish banner, the player's place as an ordinal (1st); 0 hides it.">
<meta name="prop:detail" content="string" data-doc="A line under the call as the game words it (New lap record, Lap 3 of 3); empty hides it.">
<meta name="fixture:default" content='{"kind":"countdown","figure":3,"at":1,"place":0,"detail":""}'>
<meta name="fixture:go" content='{"kind":"go","figure":0,"at":4,"place":0,"detail":""}'>
<meta name="fixture:finalLap" content='{"kind":"finalLap","figure":0,"at":5,"place":0,"detail":"Lap 3 of 3"}'>
<meta name="fixture:finish" content='{"kind":"finish","figure":0,"at":6,"place":2,"detail":"New lap record"}'>
<meta name="fixture:none" content='{"kind":"none","figure":0,"at":0,"place":0,"detail":""}'>
<meta name="fixture:overflow" content='{"kind":"finish","figure":0,"at":7,"place":23,"detail":"Disqualified for cutting the chicane at the Esses on the final lap"}'>

<!--
  The race's calls across the screen: the start countdown (3, 2, 1), GO,
  the final lap and the finish with the player's place. The game steps the
  call and its id (at); a new id plays the entrance again, which under
  reduced motion shows its end at once. Announced as a status.
-->
<div id="RacingBanner" part="RacingBanner.frame" class="bare" role="status" aria-atomic="true" aria-label={t("racing.banner.name")} data-kind={kind}>
  {#key at}
    <div part="RacingBanner.plate" class="plate" hidden={!(kind == 'countdown' || kind == 'go' || kind == 'finalLap' || kind == 'finish')} data-kind={kind}>
      <span part="RacingBanner.title" class="call">{kind == 'countdown' ? formatNumber(figure) : (kind == 'go' ? t("racing.banner.go") : (kind == 'finalLap' ? t("racing.banner.finalLap") : t("racing.banner.finish")))}</span>
      <span part="RacingBanner.badge" class="place" hidden={!(kind == 'finish' && place > 0)}>{formatOrdinal(place)}</span>
      <span class="detail" hidden={detail == ''}>{detail}</span>
    </div>
  {/key}
</div>

<style>
[part="RacingBanner.frame"][hidden], [part="RacingBanner.frame"] [hidden] { display: none; }
[part="RacingBanner.frame"] { display: flex; width: stretch; align-items: center; justify-content: center; pointer-events: none; }
.plate { display: flex; flex-direction: column; align-items: center; gap: var(--space-xxs); max-width: stretch; padding: var(--space-sm) var(--space-3xl); animation-name: racing-call; animation-duration: var(--motion-duration-slow); animation-timing-function: var(--motion-easing-out); animation-fill-mode: both; }
.plate[data-kind=countdown] { animation-name: racing-count; animation-duration: var(--motion-duration-bounce); animation-timing-function: var(--motion-easing-snappy); }
.call { white-space: nowrap; line-height: 1; font-variant-numeric: tabular-nums; }
.place { white-space: nowrap; }
.detail { text-align: center; max-width: stretch; }
@keyframes racing-count { from { opacity: 0; scale: 1.6; } to { opacity: 1; scale: 1; } }
@keyframes racing-call { from { opacity: 0; translate: 0 -40%; } to { opacity: 1; translate: 0 0; } }
/* Reduced motion: no zoom or slide; the call is simply there. */
@media (prefers-reduced-motion: reduce) { .plate, .plate[data-kind=countdown] { animation-name: none; } }
@media (width < 600px) { .plate { padding: var(--space-xs) var(--space-xl); } }
</style>
