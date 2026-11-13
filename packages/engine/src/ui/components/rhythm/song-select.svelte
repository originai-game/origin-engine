<meta name="component" content="RhythmSongSelect">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="parts" content="RhythmSongSelect.frame RhythmSongSelect.band RhythmSongSelect.title RhythmSongSelect.row RhythmSongSelect.name RhythmSongSelect.badge RhythmSongSelect.jacket RhythmSongSelect.detail RhythmSongSelect.cell">
<meta name="prop:title" content="string" data-doc="The page's heading (a pack or folder name); empty takes the kit's (Songs).">
<meta name="prop:songs" content="list" data-doc="The songs in the game's order, keyed rows of { id, title, artist, bpm, length, jacket, locked, fresh, cleared }: bpm and length as the game words them (128, 90–180; 2:14); jacket the cover art texture (og.load.texture, 0 none); locked the song cannot be played yet (its row is disabled); fresh newly added (New); cleared the player has cleared it (Cleared).">
<meta name="prop:selected" content="string" data-doc="The song in focus as the game holds it (its row selected, its details shown); empty is none.">
<meta name="prop:previewing" content="string" data-doc="The song whose preview the game is playing (Previewing); empty is none.">
<meta name="prop:status" content="string" data-doc="The list's state: empty when the songs are there, loading while the game fetches them, error when it could not.">
<meta name="action:details" content="Preview">
<meta name="fixture:default" content='{"title":"","selected":"skyline","previewing":"","status":"","songs":[{"id":"skyline","title":"Skyline Drive","artist":"Neon Harbor","bpm":"128","length":"2:14","jacket":0,"locked":false,"fresh":false,"cleared":true},{"id":"ember","title":"Ember Waltz","artist":"Lumen & Vale","bpm":"96","length":"3:02","jacket":0,"locked":false,"fresh":true,"cleared":false},{"id":"static","title":"Static Bloom","artist":"Kiri","bpm":"174","length":"1:58","jacket":0,"locked":false,"fresh":false,"cleared":false},{"id":"vault","title":"Vault of Echoes","artist":"Orchid Unit","bpm":"90–180","length":"2:47","jacket":0,"locked":true,"fresh":false,"cleared":false}]}'>
<meta name="fixture:selected" content='{"title":"","selected":"ember","previewing":"ember","status":"","songs":[{"id":"skyline","title":"Skyline Drive","artist":"Neon Harbor","bpm":"128","length":"2:14","jacket":0,"locked":false,"fresh":false,"cleared":true},{"id":"ember","title":"Ember Waltz","artist":"Lumen & Vale","bpm":"96","length":"3:02","jacket":0,"locked":false,"fresh":true,"cleared":false},{"id":"static","title":"Static Bloom","artist":"Kiri","bpm":"174","length":"1:58","jacket":0,"locked":false,"fresh":false,"cleared":false}]}'>
<meta name="fixture:disabled" content='{"title":"","selected":"vault","previewing":"","status":"","songs":[{"id":"skyline","title":"Skyline Drive","artist":"Neon Harbor","bpm":"128","length":"2:14","jacket":0,"locked":false,"fresh":false,"cleared":true},{"id":"vault","title":"Vault of Echoes","artist":"Orchid Unit","bpm":"90–180","length":"2:47","jacket":0,"locked":true,"fresh":false,"cleared":false}]}'>
<meta name="fixture:empty" content='{"title":"","selected":"","previewing":"","status":"","songs":[]}'>
<meta name="fixture:loading" content='{"title":"","selected":"","previewing":"","status":"loading","songs":[]}'>
<meta name="fixture:error" content='{"title":"","selected":"","previewing":"","status":"error","songs":[]}'>
<meta name="fixture:overflow" content='{"title":"Extended Festival Collection · Volume Twelve · Encore Edition","selected":"s7","previewing":"s2","status":"","songs":[{"id":"s1","title":"Skyline Drive","artist":"Neon Harbor","bpm":"128","length":"2:14","jacket":0,"locked":false,"fresh":false,"cleared":true},{"id":"s2","title":"Ember Waltz","artist":"Lumen & Vale","bpm":"96","length":"3:02","jacket":0,"locked":false,"fresh":true,"cleared":false},{"id":"s3","title":"Static Bloom","artist":"Kiri","bpm":"174","length":"1:58","jacket":0,"locked":false,"fresh":false,"cleared":true},{"id":"s4","title":"Vault of Echoes","artist":"Orchid Unit","bpm":"90–180","length":"2:47","jacket":0,"locked":true,"fresh":false,"cleared":false},{"id":"s5","title":"Paper Lanterns","artist":"Mira Okonkwo-Lindqvist","bpm":"112","length":"2:31","jacket":0,"locked":false,"fresh":false,"cleared":false},{"id":"s6","title":"Glasswing","artist":"Teleport Choir","bpm":"200","length":"2:05","jacket":0,"locked":false,"fresh":true,"cleared":false},{"id":"s7","title":"The Unbearably Long Extended Twelve-Minute Midnight Orchestral Remix of a Song","artist":"The Philharmonic Synthesizer Ensemble of the Northern Coast featuring Everyone","bpm":"60–240","length":"12:48","jacket":0,"locked":false,"fresh":true,"cleared":true},{"id":"s8","title":"Rust Belt","artist":"Hollow Pines","bpm":"140","length":"2:22","jacket":0,"locked":true,"fresh":false,"cleared":false},{"id":"s9","title":"Moth Light","artist":"Sela","bpm":"88","length":"3:40","jacket":0,"locked":false,"fresh":false,"cleared":true},{"id":"s10","title":"Overclock","artist":"Byte Parade","bpm":"185","length":"1:49","jacket":0,"locked":false,"fresh":false,"cleared":false},{"id":"s11","title":"Tidewater","artist":"Juniper","bpm":"104","length":"2:58","jacket":0,"locked":false,"fresh":false,"cleared":false},{"id":"s12","title":"Last Train","artist":"Neon Harbor","bpm":"150","length":"2:36","jacket":0,"locked":true,"fresh":false,"cleared":false}]}'>

<!--
  Choosing a song: the game's list (jacket, title, artist, BPM, with New,
  Cleared, Locked, Selected and Previewing named in words) beside the
  selected song's details. A row reports choose with its key; Preview (the
  button, or X / pad Y on a row) reports preview with its key. A locked
  song is a disabled row and reports nothing. The game owns unlocking,
  the selection, the preview audio and the loading; status shows its
  loading and error states, and the error state offers Retry (retry).
-->
<section id="RhythmSongSelect" part="RhythmSongSelect.frame" aria-label={title != '' ? title : t("rhythm.songs.title")} aria-busy={status == 'loading'} data-status={status}>
  <header part="RhythmSongSelect.band"><h2 part="RhythmSongSelect.title">{title != '' ? title : t("rhythm.songs.title")}</h2></header>
  {#if status == 'loading'}
    <div class="state" role="status"><Spinner size={0} name={t("rhythm.songs.loading")} /><p class="state-text">{t("rhythm.songs.loading")}</p></div>
  {:else}
    {#if status == 'error'}
      <div class="state" role="alert" data-status="error"><p class="state-text">{t("rhythm.songs.error")}</p><button id="Retry" variant="primary" onclick={retry}><span>{t("rhythm.songs.retry")}</span></button></div>
    {:else}
      {#if songs.length}
        <div class="body">
          <scroll class="list">
            <div class="rows" role="listbox" aria-label={t("rhythm.songs.list")}>
              {#each songs as s (s.id)}
                <button part="RhythmSongSelect.row" role="option" aria-selected={s.id == selected} aria-label={s.title} aria-description={s.artist} disabled={s.locked} data-locked={s.locked} data-fresh={s.fresh} data-cleared={s.cleared} data-previewing={s.id == previewing} onclick={choose} ondetails={preview}><div class="face">
                  <img part="RhythmSongSelect.jacket" class="thumb" src={s.jacket} alt="" hidden={!s.jacket}><div part="RhythmSongSelect.jacket" class="thumb well" hidden={!!s.jacket}></div>
                  <div class="text"><span part="RhythmSongSelect.name" class="name">{s.title}</span><span class="artist">{s.artist}</span>
                  <div class="badges">
                    <span part="RhythmSongSelect.badge" data-tag="selected" hidden={s.id != selected}>{t("rhythm.songs.selected")}</span>
                    <span part="RhythmSongSelect.badge" data-tag="previewing" hidden={s.id != previewing}>{t("rhythm.songs.previewing")}</span>
                    <span part="RhythmSongSelect.badge" data-tag="fresh" hidden={!s.fresh}>{t("rhythm.songs.fresh")}</span>
                    <span part="RhythmSongSelect.badge" data-tag="cleared" hidden={!s.cleared}>{t("rhythm.songs.cleared")}</span>
                    <span part="RhythmSongSelect.badge" data-tag="locked" hidden={!s.locked}>{t("rhythm.songs.locked")}</span>
                  </div></div>
                  <span class="bpm">{t("rhythm.songs.bpm", { bpm: s.bpm })}</span>
                </div></button>
              {/each}
            </div>
          </scroll>
          <aside part="RhythmSongSelect.detail" class="detail" aria-label={t("rhythm.songs.none")} hidden={selected != ''}><p class="state-text">{t("rhythm.songs.none")}</p></aside>
          {#each songs as s (s.id)}
            {#if s.id == selected}
              <aside part="RhythmSongSelect.detail" class="detail" aria-label={s.title} data-locked={s.locked} data-previewing={s.id == previewing}>
                <img part="RhythmSongSelect.jacket" class="cover" src={s.jacket} alt="" hidden={!s.jacket}><div part="RhythmSongSelect.jacket" class="cover well" hidden={!!s.jacket}></div>
                <div class="info">
                  <span class="detail-name">{s.title}</span>
                  <span class="detail-artist">{s.artist}</span>
                  <div class="facts">
                    <div part="RhythmSongSelect.cell" class="fact"><span class="label">{t("rhythm.songs.bpmLabel")}</span><span class="value">{s.bpm}</span></div>
                    <div part="RhythmSongSelect.cell" class="fact"><span class="label">{t("rhythm.songs.lengthLabel")}</span><span class="value">{s.length}</span></div>
                  </div>
                  <button id="Preview" variant="ghost" aria-pressed={s.id == previewing} disabled={s.locked} onclick={preview}><kbd class="key" data-action="details"></kbd><span>{s.id == previewing ? t("rhythm.songs.previewing") : t("rhythm.songs.preview")}</span></button>
                </div>
              </aside>
            {/if}
          {/each}
        </div>
      {:else}
        <div class="state" role="status"><p class="state-text">{t("rhythm.songs.empty")}</p></div>
      {/if}
    {/if}
  {/if}
</section>

<style>
[part="RhythmSongSelect.frame"][hidden], [part="RhythmSongSelect.frame"] [hidden] { display: none; }
[part="RhythmSongSelect.frame"] { display: flex; flex-direction: column; width: stretch; min-width: 0; height: stretch; gap: var(--space-md); container-type: inline-size; }
[part="RhythmSongSelect.band"] { display: flex; width: stretch; align-items: center; flex: none; }
[part="RhythmSongSelect.title"] { margin: 0; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.body { display: flex; width: stretch; flex: 1; min-height: 0; gap: var(--space-lg); }
.list { flex: 3; min-width: 0; height: stretch; }
.rows { display: flex; flex-direction: column; width: stretch; gap: var(--space-xs); padding: var(--space-xxs); }
[part="RhythmSongSelect.row"] { width: stretch; min-height: var(--size-touch); padding: var(--space-xs) var(--space-sm); }
.face { display: flex; width: stretch; align-items: center; gap: var(--space-sm); }
.thumb { flex: none; width: var(--size-lg); height: var(--size-lg); }
img.thumb, img.cover { object-fit: cover; }
.text { display: flex; flex-direction: column; flex: 1; min-width: 0; align-items: flex-start; }
.name, .artist { max-width: stretch; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.bpm { flex: none; white-space: nowrap; font-variant-numeric: tabular-nums; }
.badges { display: flex; flex-wrap: wrap; gap: var(--space-xxs); }
[part="RhythmSongSelect.badge"] { white-space: nowrap; padding: 0 var(--space-xs); }
.detail { display: flex; flex-direction: column; flex: 2; min-width: 0; min-height: 0; overflow-y: auto; gap: var(--space-sm); padding: var(--space-sm); }
.cover { flex: none; width: stretch; max-width: calc(var(--size-xxl) * 1.8); aspect-ratio: 1; align-self: center; }
.info { display: flex; flex-direction: column; width: stretch; gap: var(--space-xxs); align-items: flex-start; }
.detail-name { max-width: stretch; text-align: start; }
.detail-artist { max-width: stretch; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.facts { display: flex; gap: var(--space-md); padding-block: var(--space-xs); }
[part="RhythmSongSelect.cell"] { display: flex; flex-direction: column; align-items: flex-start; }
.value { white-space: nowrap; font-variant-numeric: tabular-nums; }
#Preview, #Retry { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); gap: var(--space-xs); }
.state { display: flex; flex-direction: column; flex: 1; align-items: center; justify-content: center; gap: var(--space-md); }
.state-text { margin: 0; text-align: center; }
@media (pointer: coarse) { .key { display: none; } }
/* A narrow panel (a phone, or a side column on a wide screen) puts the
   selected song's details over the list, the cover small at the start. */
@container (width < 560px) {
  .body { flex-direction: column-reverse; gap: var(--space-sm); }
  .list { flex: 1; width: stretch; min-height: 0; }
  .detail { flex: none; width: stretch; flex-direction: row; align-items: center; overflow-y: visible; padding: var(--space-xs); }
  .cover { flex: none; width: calc(var(--size-xxl) * 1.1); }
  .info { flex: 1; min-width: 0; }
  .detail-name { white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
  .bpm { display: none; }
}
/* The frame cannot ask its own width, so its gap follows the screen. */
@media (width < 600px) {
  [part="RhythmSongSelect.frame"] { gap: var(--space-sm); }
}
</style>
