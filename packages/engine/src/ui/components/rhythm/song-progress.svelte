<meta name="component" content="RhythmSongProgress">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="parts" content="RhythmSongProgress.frame RhythmSongProgress.line RhythmSongProgress.played RhythmSongProgress.time">
<meta name="prop:value" content="number" data-doc="How far through the song, 0 to 1.">
<meta name="prop:elapsed" content="string" data-doc="The time played as the game words it (2:07); empty hides the time.">
<meta name="prop:length" content="string" data-doc="The song's length as the game words it (3:21), after the time played.">
<meta name="fixture:default" content='{"value":0.63,"elapsed":"2:07","length":"3:21"}'>
<meta name="fixture:start" content='{"value":0,"elapsed":"0:00","length":"3:21"}'>
<meta name="fixture:end" content='{"value":1,"elapsed":"3:21","length":"3:21"}'>
<meta name="fixture:bare" content='{"value":0.4,"elapsed":"","length":""}'>
<meta name="fixture:overflow" content='{"value":0.5,"elapsed":"1:02:44","length":"2:05:28"}'>

<!--
  The song's progress as a thin lit line along the screen's top edge, and
  the time played over the length riding under its played end. The game keeps time; read
  only.
-->
<section id="RhythmSongProgress" part="RhythmSongProgress.frame" class="hud bare" data-layout="stack" data-gap="none" role="progressbar" aria-label={t("rhythm.hud.progress")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={value <= 0 ? 0 : value >= 1 ? 100 : value * 100} aria-valuetext={elapsed != '' ? t("rhythm.progress.time", { elapsed: elapsed, length: length }) : formatPercent(value)} style:--value={value <= 0 ? 0 : value >= 1 ? 1 : value}>
  <div id="Line" part="RhythmSongProgress.line" data-layout="overlay">
    <div id="Played" part="RhythmSongProgress.played" data-anchor="top-start"></div>
  </div>
  {#if elapsed != ''}
  <div class="under" data-layout="row" data-gap="none">
  <div class="clock" data-layout="row" data-gap="hair">
    <span part="RhythmSongProgress.time" class="elapsed">{elapsed}</span>
    {#if length != ''}<span part="RhythmSongProgress.time" class="length">{t("rhythm.progress.length", { length: length })}</span>{/if}
  </div>
  </div>
  {/if}
</section>

<style>
:host { inline-size: stretch; color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; --layout-anchor-inset: 0; }
#Line { inline-size: stretch; block-size: calc(5 * var(--font-step-px)); background: var(--hud-line-well); }
#Played { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--hud-line-fill); box-shadow: var(--hud-line-glow); }
/* The time rides under the played end of the line, held inside the
   line's ends: its start margin is the part played, clamped so the
   label never leaves the screen at either end. */
.under { inline-size: stretch; margin-block-start: var(--space-xs); padding-inline: var(--space-md); }
.clock { flex: none; align-items: baseline; font-size: var(--hud-size-xs); margin-inline-start: clamp(0px, calc(var(--value) * 100% - var(--select-time-lead)), calc(100% - var(--select-time-room))); }
.elapsed { font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.length { font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink-dim); white-space: nowrap; }
</style>
