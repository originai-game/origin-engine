<meta name="component" content="RhythmSongSelect">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="parts" content="RhythmSongSelect.frame RhythmSongSelect.band RhythmSongSelect.title RhythmSongSelect.row RhythmSongSelect.name RhythmSongSelect.badge RhythmSongSelect.jacket RhythmSongSelect.detail RhythmSongSelect.fact RhythmSongSelect.picture RhythmSongSelect.label RhythmSongSelect.headline RhythmSongSelect.caption RhythmSongSelect.figure">
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
  selected song's details. It stands over the game's own art with no panel
  behind it: each song a cut plate of dark glass with its jacket in an
  octagon, the selected one washed in the HUD's accent and stepped in, its
  details a large jacket with a lit edge, the title, the artist and the BPM
  and length on figure plates. A row reports choose with its key; Preview
  (the button, or X / pad Y on a row) reports preview with its key. A
  locked song is a disabled row and reports nothing. The game owns
  unlocking, the selection, the preview audio and the loading; status shows
  its loading and error states, and the error state offers Retry (retry).
  A long title or artist scrolls; the list opens scrolled to the selection.
-->
<section id="RhythmSongSelect" part="RhythmSongSelect.frame" class="hud bare" data-layout="shell" data-gap="loose" aria-label={title != '' ? title : t("rhythm.songs.title")} aria-busy={status == 'loading'} data-status={status}>
  <header part="RhythmSongSelect.band" data-layout="row"><h2 part="RhythmSongSelect.title">{title != '' ? title : t("rhythm.songs.title")}</h2></header>
  {#if status == 'loading'}
    <div class="state" data-layout="cover" data-fill role="status"><div class="state-plate" data-layout="stack" data-gap="tight"><Spinner size={0} name={t("rhythm.songs.loading")} /><p class="state-text" part="RhythmSongSelect.caption">{t("rhythm.songs.loading")}</p></div></div>
  {:else}
    {#if status == 'error'}
      <div class="state" data-layout="cover" data-fill role="alert" data-status="error"><div class="state-plate" data-layout="stack" data-gap="tight"><p class="state-text" part="RhythmSongSelect.caption">{t("rhythm.songs.error")}</p><button id="Retry" class="act go" variant="primary" onclick={retry}><span>{t("rhythm.songs.retry")}</span></button></div></div>
    {:else}
      {#if songs.length}
        <div class="body" data-layout="row" data-gap="wide" data-fill>
          <div class="list" data-layout="scroll">
            <div class="rows" data-layout="stack" data-gap="tight" role="listbox" aria-label={t("rhythm.songs.list")}>
              {#each songs as s (s.id)}
                <button part="RhythmSongSelect.row" class="row" role="option" aria-selected={s.id == selected} aria-label={s.title} aria-description={s.artist} disabled={s.locked} data-locked={s.locked} data-fresh={s.fresh} data-cleared={s.cleared} data-previewing={s.id == previewing} onclick={choose} ondetails={preview}><div class="face" data-layout="row" data-gap="loose">
                  <div class="thumb" data-layout="overlay">{#if s.jacket}<img part="RhythmSongSelect.jacket RhythmSongSelect.picture" class="art" data-anchor="fill" src={s.jacket} alt="">{:else}<div part="RhythmSongSelect.jacket" class="art blank" data-anchor="fill"></div><og-icon class="jacket-icon" data-anchor="center" icon="music"></og-icon>{/if}</div>
                  <div class="text" data-layout="stack" data-gap="none" data-fill><span part="RhythmSongSelect.name" class="name">{s.title}</span><span class="artist" part="RhythmSongSelect.caption">{s.artist}</span>
                  {#if s.id == selected || s.id == previewing || s.fresh || s.cleared || s.locked}
                  <div class="badges" data-layout="cluster" data-gap="hair">
                    {#if s.id == selected}<div part="RhythmSongSelect.badge" class="tag" data-layout="row" data-gap="none" data-tag="selected"><span class="tag-text">{t("rhythm.songs.selected")}</span></div>{/if}
                    {#if s.id == previewing}<div part="RhythmSongSelect.badge" class="tag" data-layout="row" data-gap="none" data-tag="previewing"><div class="eq" data-layout="row" data-gap="none"><div class="bar"></div><div class="bar"></div><div class="bar"></div></div><span class="tag-text">{t("rhythm.songs.previewing")}</span></div>{/if}
                    {#if s.fresh}<div part="RhythmSongSelect.badge" class="tag" data-layout="row" data-gap="none" data-tag="fresh"><span class="tag-text">{t("rhythm.songs.fresh")}</span></div>{/if}
                    {#if s.cleared}<div part="RhythmSongSelect.badge" class="tag" data-layout="row" data-gap="none" data-tag="cleared"><span class="tag-text">{t("rhythm.songs.cleared")}</span></div>{/if}
                    {#if s.locked}<div part="RhythmSongSelect.badge" class="tag" data-layout="row" data-gap="none" data-tag="locked"><og-icon class="tag-icon" icon="lock"></og-icon><span class="tag-text">{t("rhythm.songs.locked")}</span></div>{/if}
                  </div>
                  {/if}</div>
                  <span class="bpm" part="RhythmSongSelect.figure">{t("rhythm.songs.bpm", { bpm: s.bpm })}</span>
                </div></button>
              {/each}
            </div>
          </div>
          {#if selected == ''}<aside part="RhythmSongSelect.detail" class="detail none" data-layout="cover" aria-label={t("rhythm.songs.none")}><p class="state-text" part="RhythmSongSelect.caption">{t("rhythm.songs.none")}</p></aside>{/if}
          {#each songs as s (s.id)}
            {#if s.id == selected}
              <aside part="RhythmSongSelect.detail" class="detail" data-layout="scroll" data-gap="loose" aria-label={s.title} data-locked={s.locked} data-previewing={s.id == previewing}>
                <div class="hero" data-layout="row" data-gap="loose">
                  <div class="cover" data-layout="overlay">{#if s.jacket}<img part="RhythmSongSelect.jacket RhythmSongSelect.picture" class="art" data-anchor="fill" src={s.jacket} alt="">{:else}<div part="RhythmSongSelect.jacket" class="art blank" data-anchor="fill"></div><og-icon class="jacket-icon" data-anchor="center" icon="music"></og-icon>{/if}</div>
                  <div class="info" data-layout="stack" data-gap="tight" data-fill>
                    {#if s.fresh || s.locked}<div class="badges" data-layout="cluster" data-gap="hair">{#if s.fresh}<div part="RhythmSongSelect.badge" class="tag" data-layout="row" data-gap="none" data-tag="fresh"><span class="tag-text">{t("rhythm.songs.fresh")}</span></div>{/if}{#if s.locked}<div part="RhythmSongSelect.badge" class="tag" data-layout="row" data-gap="none" data-tag="locked"><og-icon class="tag-icon" icon="lock"></og-icon><span class="tag-text">{t("rhythm.songs.locked")}</span></div>{/if}</div>{/if}
                    <span class="detail-name" part="RhythmSongSelect.headline">{s.title}</span>
                    <span class="detail-artist" part="RhythmSongSelect.caption">{s.artist}</span>
                    <div class="facts" data-layout="cluster" data-gap="tight">
                      <div part="RhythmSongSelect.fact RhythmSongSelect.label" class="fact" data-layout="stack" data-gap="none"><span class="label">{t("rhythm.songs.bpmLabel")}</span><span class="value" part="RhythmSongSelect.figure">{s.bpm}</span></div>
                      <div part="RhythmSongSelect.fact RhythmSongSelect.label" class="fact" data-layout="stack" data-gap="none"><span class="label">{t("rhythm.songs.lengthLabel")}</span><span class="value" part="RhythmSongSelect.figure">{s.length}</span></div>
                      <button id="Preview" class="act" variant="ghost" aria-label={s.id == previewing ? t("rhythm.songs.previewing") : t("rhythm.songs.preview")} aria-pressed={s.id == previewing} disabled={s.locked} onclick={preview}><kbd class="key" data-action="details"></kbd>{#if s.id == previewing}<div class="eq" data-layout="row" data-gap="none"><div class="bar"></div><div class="bar"></div><div class="bar"></div></div>{/if}<span>{s.id == previewing ? t("rhythm.songs.previewing") : t("rhythm.songs.preview")}</span></button>
                    </div>
                  </div>
                </div>
              </aside>
            {/if}
          {/each}
        </div>
      {:else}
        <div class="state" data-layout="cover" data-fill role="status"><div class="state-plate" data-layout="stack" data-gap="tight"><p class="state-text" part="RhythmSongSelect.caption">{t("rhythm.songs.empty")}</p></div></div>
      {/if}
    {/if}
  {/if}
</section>

<style>
:host { inline-size: stretch; min-inline-size: 0; block-size: stretch; color: var(--hud-ink); --cut: var(--select-row-cut); }
/* The heading stands straight over the art: condensed bold capitals with
   the HUD's stroke under the fill and one soft shadow. */
[part="RhythmSongSelect.band"] { width: stretch; flex: none; min-height: 0; padding: 0; border: 0; background: none; }
[part="RhythmSongSelect.title"] { margin: 0; min-inline-size: 0; font-size: var(--select-size-title); line-height: 1.1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.body { width: stretch; min-height: 0; align-items: stretch; }
.list { overflow-y: auto; flex: 5; min-width: 0; -og-scroll-fade: var(--space-xl); }
.rows { width: stretch; padding-block: var(--space-xs); padding-inline: var(--space-xxs) var(--space-sm); }
/* A song: a cut plate of dark glass fading to the right; the selected one
   steps in, washed in the accent from its leading edge. */
.row { width: stretch; min-width: 0; min-height: var(--size-touch); padding: var(--space-xs) var(--space-md) var(--space-xs) var(--space-xs); border: 0; color: var(--hud-ink); background: var(--select-row-fill); box-shadow: var(--select-row-rim); border-radius: var(--cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; transition: var(--change-motion); }
.row:hover { background: var(--select-row-hover-fill); box-shadow: var(--select-row-hover-rim); }
.row:focus-visible { box-shadow: var(--select-row-focus-rim); }
.row[aria-selected=true] { margin-inline-start: var(--select-row-step); width: auto; align-self: stretch; padding-block: var(--space-sm); background: var(--select-row-on-fill); box-shadow: var(--select-row-on-rim); scroll-initial-target: nearest; scroll-margin-block-start: calc(var(--size-touch) * 1.2); }
.row[aria-selected=true]:focus-visible { box-shadow: var(--select-row-focus-rim); }
.row:disabled { opacity: 0.62; background: var(--select-row-fill); box-shadow: var(--select-row-rim); }
.face { width: stretch; align-items: center; }
/* The jacket: an octagon with a lit edge, the art inset inside it (a
   `fill` anchor sits flush, so the art takes the inset itself). */
.thumb { flex: none; inline-size: calc(60 * var(--font-step-px)); block-size: calc(60 * var(--font-step-px)); clip-path: var(--hud-octagon); background: var(--hud-edge); --layout-anchor-inset: calc(2 * var(--font-step-px)); }
.row[aria-selected=true] .thumb { inline-size: calc(80 * var(--font-step-px)); block-size: calc(80 * var(--font-step-px)); background: var(--hud-ready-edge); }
.art { inset: var(--layout-anchor-inset); clip-path: var(--hud-octagon); background: var(--select-jacket-fill); }
img.art { object-fit: cover; }
.row:disabled .art { filter: var(--hud-unready); opacity: 1; }
.jacket-icon { inline-size: 44%; block-size: 44%; color: var(--hud-accent); }
.text { min-width: 0; align-items: flex-start; gap: calc(2 * var(--font-step-px)); }
.name { max-width: stretch; text-align: start; font-size: var(--select-size-name); line-height: 1.2; font-weight: var(--hud-label-weight); color: var(--hud-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.row[aria-selected=true] .name { font-size: var(--select-size-name-on); font-weight: var(--hud-figure-weight); }
.artist { max-width: stretch; text-align: start; font-size: var(--hud-size-xs); color: var(--hud-ink-dim); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.bpm { flex: none; font-size: var(--hud-size-md); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); white-space: nowrap; }
.badges { margin-block-start: calc(3 * var(--font-step-px)); }
/* Tags: small capitals on solid cut chips, each state in its own ink. */
.tag { align-items: center; gap: calc(4 * var(--font-step-px)); flex: none; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.6; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.tag[data-tag=selected] { color: var(--hud-accent); }
.tag[data-tag=previewing] { color: var(--hud-accent); }
.tag[data-tag=fresh] { color: var(--select-fresh-ink); background: var(--select-fresh-fill); }
.tag[data-tag=cleared] { color: var(--hud-boon); }
.tag[data-tag=locked] { color: var(--hud-ink-dim); }
.tag-icon { inline-size: var(--select-glyph); block-size: var(--select-glyph); }
/* The preview's level bars, beating while the game plays it. */
.eq { flex: none; align-items: flex-end; gap: calc(2 * var(--font-step-px)); block-size: var(--select-eq-height); }
.bar { inline-size: calc(3 * var(--font-step-px)); block-size: 60%; background: var(--hud-accent); animation: var(--pulse-motion); }
.bar:nth-child(2) { block-size: 100%; }
.bar:nth-child(3) { block-size: 40%; }
/* The selected song's details over the art: a large jacket, the title,
   the artist, the BPM and length on plates, and Preview. */
.detail { flex: 4; min-width: 0; min-height: 0; padding: var(--space-xs); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
.detail.none { align-items: center; }
.hero { width: stretch; align-items: flex-start; }
.cover { flex: none; inline-size: min(calc(300 * var(--font-step-px)), 42%); aspect-ratio: 1; clip-path: var(--hud-octagon); background: var(--hud-ready-edge); --layout-anchor-inset: calc(3 * var(--font-step-px)); filter: var(--hud-ready-glow); }
.detail[data-locked=true] .cover { background: var(--hud-edge); filter: var(--hud-unready); }
.info { min-width: 0; align-items: flex-start; }
/* The title keeps one line at any text size and scrolls when long, so
   the facts and Preview under it stay in view. */
.detail-name { max-width: stretch; text-align: start; font-size: var(--select-size-hero); line-height: 1.05; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); color: var(--hud-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.detail-artist { max-width: stretch; text-align: start; font-size: var(--hud-size-sm); color: var(--hud-ink-dim); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.facts { padding-block: var(--space-xs); align-items: stretch; }
.fact { align-items: flex-start; min-inline-size: calc(80 * var(--font-step-px)); padding: var(--space-xs) var(--space-sm); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; -webkit-text-stroke-width: 0; text-shadow: none; }
.label { font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; }
.value { font-size: var(--hud-size-lg); line-height: 1.05; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); white-space: nowrap; }
/* Preview and Retry: a smoked plate lit along its top; pressed while the
   game plays the preview, filled with the accent. */
.act { min-height: var(--size-touch); min-width: calc(var(--size-xxl) * 1.25); gap: var(--space-xs); padding: 0 var(--space-md); border: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); background: var(--select-button-fill); box-shadow: var(--select-button-rim); text-shadow: none; -webkit-text-stroke-width: 0; border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; transition: var(--change-motion); }
.act:hover { background: var(--select-button-hover-fill); }
.act:focus-visible { box-shadow: var(--select-row-focus-rim); }
.act[aria-pressed=true] { color: var(--select-accent-ink); background: var(--select-button-on-fill); box-shadow: var(--select-button-on-rim); }
.act[aria-pressed=true] .bar { background: var(--select-accent-ink); }
.act > span { color: inherit; font-weight: inherit; }
.act:disabled { opacity: 0.5; background: var(--select-button-fill); }
.key { color: var(--hud-key-ink); background: var(--hud-key-fill); box-shadow: var(--hud-key-shadow); border-radius: var(--hud-key-radius); }
.state { align-items: center; }
.state-plate { align-items: center; padding: var(--space-md) var(--space-lg); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); border-radius: var(--hud-plate-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.state-text { margin: 0; text-align: center; font-size: var(--hud-size-sm); color: var(--hud-ink); }
.none .state-text { padding: var(--space-sm) var(--space-md); background: var(--hud-tag-fill); -webkit-text-stroke-width: 0; text-shadow: none; }
@media (pointer: coarse) { .key { display: none; } }
/* A narrow panel (a phone, or a side column on a wide screen) puts the
   selected song's details over the list, the jacket small at the start. */
@container (width < 560px) {
  .body { flex-direction: column; --layout-gap-wide: var(--layout-gap); }
  .detail { order: -1; }
  .list { flex: 1; width: stretch; min-height: 0; }
  .rows { padding-inline-end: var(--space-xxs); }
  .detail { flex: none; width: stretch; overflow-y: visible; padding: 0; }
  .hero { align-items: center; }
  .cover { inline-size: min(calc(92 * var(--font-step-px)), var(--select-cover-compact-max)); }
  .info { flex: 1; min-width: 0; }
  .detail-name { font-size: var(--select-size-hero-compact); }
  .detail-artist { font-size: var(--hud-size-xs); }
  .facts { padding-block: 0; flex-wrap: nowrap; min-width: 0; max-width: stretch; }
  .facts .label { display: none; }
  .facts { align-items: center; }
  .fact { min-block-size: 0; }
  .act { min-width: 0; min-height: calc(var(--size-touch) * 0.8); padding: 0 var(--space-sm); }
  .act > span { white-space: nowrap; }
  .act .eq { display: none; }
  .info .badges { display: none; }
  .fact { flex-direction: row; align-items: baseline; gap: var(--space-xs); min-inline-size: 0; padding: calc(2 * var(--font-step-px)) var(--space-xs); }
  .value { font-size: var(--hud-size-sm); }
  .row[aria-selected=true] { margin-inline-start: 0; }
  .row[aria-selected=true] .thumb, .thumb { inline-size: calc(48 * var(--font-step-px)); block-size: calc(48 * var(--font-step-px)); }
  .name, .row[aria-selected=true] .name { font-size: var(--select-size-name-compact); }
  .bpm { font-size: var(--hud-size-sm); }
}
/* The frame cannot ask its own width, so its title and gap follow the screen. */
@media (width < 600px) {
  :host { --layout-gap-loose: var(--layout-gap-tight); }
  [part="RhythmSongSelect.title"] { font-size: var(--select-size-title-compact); }
}
</style>
