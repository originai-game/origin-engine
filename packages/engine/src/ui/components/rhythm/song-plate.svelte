<meta name="component" content="RhythmSongPlate">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="prop:title" content="string" data-doc="The song's title.">
<meta name="prop:artist" content="string" data-doc="The artist as the game words it; empty hides it.">
<meta name="prop:bpm" content="string" data-doc="The tempo as the game words it (174, 90–180); empty hides it.">
<meta name="prop:jacket" content="number" data-doc="The cover art texture (og.load.texture); 0 shows the music glyph in its place.">
<meta name="prop:tier" content="string" data-doc="The chart's family (easy, normal, hard, expert, master, special): the colour of its chip, on data-tier, never alone.">
<meta name="prop:chart" content="string" data-doc="The chart's name as the game words it (Expert); empty hides the chip.">
<meta name="prop:rating" content="number" data-doc="The chart's level, beside its name; 0 hides it.">
<meta name="prop:modifiers" content="list" data-doc="The modifiers in play, keyed rows of { id, label } in the game's order, each on a tag (×1.0, Mirror).">
<meta name="fixture:default" content='{"title":"Neon Tidal","artist":"Halcyon Array","bpm":"174","jacket":0,"tier":"expert","chart":"Expert","rating":27,"modifiers":[{"id":"speed","label":"×1.0"},{"id":"mirror","label":"Mirror"}]}'>
<meta name="fixture:plain" content='{"title":"Skyline Drive","artist":"","bpm":"","jacket":0,"tier":"normal","chart":"","rating":0,"modifiers":[]}'>
<meta name="fixture:master" content='{"title":"Glasswing","artist":"Teleport Choir","bpm":"200","jacket":0,"tier":"master","chart":"Master","rating":31,"modifiers":[{"id":"hidden","label":"Hidden"}]}'>
<meta name="fixture:special" content='{"title":"Final Ascent","artist":"Orchid Unit","bpm":"90–240","jacket":0,"tier":"special","chart":"Special","rating":99,"modifiers":[]}'>
<meta name="fixture:overflow" content='{"title":"The Unbearably Long Extended Twelve-Minute Midnight Orchestral Remix of a Song","artist":"The Philharmonic Synthesizer Ensemble of the Northern Coast featuring Everyone","bpm":"60–240","jacket":0,"tier":"special","chart":"Remaster Ultimate Challenge","rating":16.5,"modifiers":[{"id":"speed","label":"×10.5"},{"id":"mirror","label":"Mirror"},{"id":"hidden","label":"Hidden"},{"id":"sudden","label":"Sudden death"}]}'>

<!--
  The song in play, at the screen's top corner: the jacket in an octagon
  with a lit edge, the title, the artist and tempo, the chart's chip in its
  tier's colour with its level, and the modifiers in play. Read only.
-->
<section id="RhythmSongPlate" part="frame" variant="hud" layout="row" gap="wide" role="group" aria-label={t("rhythm.plate.name", { title: title })}>
  <div id="Jacket" part="jacket" layout="overlay">
    {#if jacket}<img id="Cover" part="picture" anchor="fill" src={jacket} alt="">{:else}<div id="Blank" anchor="fill" layout="center"><og-icon id="Glyph" icon="music"></og-icon></div>{/if}
  </div>
  <div class="text" layout="stack" gap="none" fill>
    <span part="song" class="title">{title}</span>
    {#if artist != '' || bpm != ''}<span part="byline" class="byline">{artist != '' && bpm != '' ? t("rhythm.plate.byline", { artist: artist, bpm: bpm }) : (artist != '' ? artist : t("rhythm.songs.bpm", { bpm: bpm }))}</span>{/if}
    {#if chart != '' || modifiers.length}
    <div class="tags" layout="wrap" gap="tight">
      {#if chart != ''}<span part="chart" class="chart" class:tier-easy={(tier) == 'easy'} class:tier-expert={(tier) == 'expert'} class:tier-hard={(tier) == 'hard'} class:tier-master={(tier) == 'master'} class:tier-special={(tier) == 'special'}>{rating > 0 ? t("rhythm.plate.chart", { chart: chart, rating: rating }) : chart}</span>{/if}
      {#each modifiers as m (m.id)}<span part="modifier" class="tag">{m.label}</span>{/each}
    </div>
    {/if}
  </div>
</section>

<style>
#RhythmSongPlate { align-items: center; max-inline-size: min(100%, var(--measure-lg)); color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; --layout-anchor-inset: var(--space-xxs); }
/* The jacket: an octagon with a lit edge, the art inset inside it. */
#Jacket { flex: none; inline-size: calc(84 * var(--font-step-px)); block-size: calc(84 * var(--font-step-px)); clip-path: var(--hud-octagon); background: var(--hud-edge); }
#Cover, #Blank { clip-path: var(--hud-octagon); background: var(--hud-plate-fill); }
#Cover { object-fit: cover; }
#Glyph { inline-size: 44%; block-size: 44%; color: var(--hud-accent); }
.text { min-inline-size: 0; gap: var(--space-xxs); }
.title { font-size: var(--hud-size-md); line-height: 1.15; font-weight: var(--hud-figure-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.byline { font-size: var(--hud-size-sm); font-weight: var(--hud-label-weight); color: var(--hud-ink-dim); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
/* Chips: small figures on solid plates; the chart's in its tier. */
.tags { margin-block-start: var(--space-xxs); }
.chart, .tag { max-inline-size: stretch; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; text-overflow: ellipsis; overflow: hidden; -webkit-text-stroke-width: 0; text-shadow: none; border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.tag { background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); }
.chart { color: var(--chart-ink); background: var(--chart-normal); }
.chart.tier-easy { background: var(--chart-easy); }
.chart.tier-hard { background: var(--chart-hard); }
.chart.tier-expert { background: var(--chart-expert); }
.chart.tier-master { background: var(--chart-master); }
.chart.tier-special { background: var(--chart-special); }
@media (width < 600px) {
  #Jacket { inline-size: calc(56 * var(--font-step-px)); block-size: calc(56 * var(--font-step-px)); }
  .title { font-size: var(--hud-size-sm); }
  .byline { font-size: var(--hud-size-xs); }
}
.cover { justify-content: center; align-items: center; }
.cover { justify-content: center; align-items: center; }
</style>
