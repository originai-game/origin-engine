<meta name="component" content="RacingBanner">
<meta name="strings" content="./racing.en.ogt">
<meta name="parts" content="RacingBanner.frame RacingBanner.plate RacingBanner.title RacingBanner.badge RacingBanner.caption">
<meta name="prop:kind" content="string" data-default="none" data-doc="What the banner calls: countdown (the count), go, finalLap, finish, call (the game's own words, in text), or none (nothing shown, as for any other word).">
<meta name="prop:figure" content="number" data-doc="The countdown's figure (3, 2, 1); the game steps it.">
<meta name="prop:text" content="string" data-doc="The call's words as the game words them for kind call (Final lap next, Sector best); empty shows nothing.">
<meta name="prop:at" content="number" data-doc="The call's id: a new value replays the entrance (each count, the go, the flag).">
<meta name="prop:place" content="number" data-doc="On the finish banner, the player's place as an ordinal (1st); 0 hides it.">
<meta name="prop:detail" content="string" data-doc="A line under the call as the game words it (New lap record, Lap 3 of 3); empty hides it.">
<meta name="fixture:default" content='{"kind":"call","figure":0,"text":"Final lap next","at":1,"place":0,"detail":""}'>
<meta name="fixture:countdown" content='{"kind":"countdown","figure":3,"text":"","at":2,"place":0,"detail":""}'>
<meta name="fixture:go" content='{"kind":"go","figure":0,"text":"","at":4,"place":0,"detail":""}'>
<meta name="fixture:finalLap" content='{"kind":"finalLap","figure":0,"text":"","at":5,"place":0,"detail":"Lap 3 of 3"}'>
<meta name="fixture:finish" content='{"kind":"finish","figure":0,"text":"","at":6,"place":2,"detail":"New lap record"}'>
<meta name="fixture:none" content='{"kind":"none","figure":0,"text":"","at":0,"place":0,"detail":""}'>
<meta name="fixture:overflow" content='{"kind":"finish","figure":0,"text":"","at":7,"place":23,"detail":"Disqualified for cutting the chicane at the Esses on the final lap"}'>

<!--
  The race's calls at the top centre: a plate of smoked glass with a lit
  top edge in the HUD's accent, the call in widely set capitals (the
  count, GO, the final lap, the finish with the player's place, or the
  game's own words) and a line under it. The game steps the call and its
  id (at); a new id plays the entrance again, which under reduced motion
  shows its end at once. Announced as a status.
-->
<div id="RacingBanner" part="RacingBanner.frame" class="hud bare" data-layout="center" role="status" aria-atomic="true" aria-label={t("racing.banner.name")} data-kind={kind}>
  {#if kind == 'countdown' || kind == 'go' || kind == 'finalLap' || kind == 'finish' || (kind == 'call' && text != '')}
  {#key at}
    <div part="RacingBanner.plate" class="plate" data-layout="stack" data-motion={kind == 'countdown' ? 'stamp' : 'enter'} data-kind={kind}>
      <div class="line" data-layout="row">
        <span part="RacingBanner.title" class="call">{kind == 'countdown' ? formatNumber(figure) : (kind == 'go' ? t("racing.banner.go") : (kind == 'finalLap' ? t("racing.banner.finalLap") : (kind == 'finish' ? t("racing.banner.finish") : text)))}</span>
        {#if kind == 'finish' && place > 0}<span part="RacingBanner.badge" class="place">{formatOrdinal(place)}</span>{/if}
      </div>
      {#if detail != ''}<span class="detail" part="RacingBanner.caption">{detail}</span>{/if}
    </div>
  {/key}
  {/if}
</div>

<style>
/* The frame spans what its page gives it: the centre layout's reading
   measure would cap a call's plate below the room a page gives it. */
:host { inline-size: stretch; max-inline-size: stretch; pointer-events: none; color: var(--hud-ink); }
.plate { align-items: center; max-inline-size: stretch; min-inline-size: 0; row-gap: calc(2 * var(--font-step-px)); padding: calc(6 * var(--font-step-px)) calc(18 * var(--font-step-px)); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-lit-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.line { align-items: baseline; justify-content: center; column-gap: var(--space-sm); max-inline-size: 100%; min-inline-size: 0; }
.call { min-inline-size: 0; font-size: var(--hud-size-md); line-height: 1.1; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-banner-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.plate[data-kind=countdown] .call { font-size: var(--hud-size-lg); letter-spacing: normal; font-variant-numeric: var(--hud-figure-numeric); }
.place { flex: none; font-size: var(--hud-size-md); font-weight: var(--hud-figure-weight); color: var(--hud-accent); white-space: nowrap; }
.detail { max-inline-size: 100%; min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
@media (width < 600px) {
  .plate { padding: calc(4 * var(--font-step-px)) calc(14 * var(--font-step-px)); }
  .call, .place { font-size: var(--hud-size-sm); }
  .detail { letter-spacing: normal; }
}
</style>
