<meta name="component" content="RacingBanner">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingBanner.frame RacingBanner.plate RacingBanner.title RacingBanner.badge RacingBanner.caption">
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
<div id="RacingBanner" part="RacingBanner.frame" class="bare" data-layout="center" role="status" aria-atomic="true" aria-label={t("racing.banner.name")} data-kind={kind}>
  {#if kind == 'countdown' || kind == 'go' || kind == 'finalLap' || kind == 'finish'}
  {#key at}
    <div part="RacingBanner.plate RacingBanner.frame" class="plate hud" data-layout="stack" data-gap="hair" data-motion={kind == 'countdown' ? 'stamp' : 'enter'} data-kind={kind}>
      <span part="RacingBanner.title" class="call">{kind == 'countdown' ? formatNumber(figure) : (kind == 'go' ? t("racing.banner.go") : (kind == 'finalLap' ? t("racing.banner.finalLap") : t("racing.banner.finish")))}</span>
      {#if kind == 'finish' && place > 0}<span part="RacingBanner.badge" class="place">{formatOrdinal(place)}</span>{/if}
      {#if detail != ''}<span class="detail" part="RacingBanner.caption">{detail}</span>{/if}
    </div>
  {/key}
  {/if}
</div>

<style>
:host { width: stretch; pointer-events: none; }
.plate { align-items: center; max-width: stretch; padding: var(--space-sm) var(--space-3xl); }
.call { text-align: center; }
.place { white-space: nowrap; }
.detail { text-align: center; max-width: stretch; }
@media (width < 600px) { .plate { padding: var(--space-xs) var(--space-xl); } }
</style>
