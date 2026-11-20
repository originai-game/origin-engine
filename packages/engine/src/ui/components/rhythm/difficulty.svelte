<meta name="component" content="RhythmDifficulty">
<meta name="strings" content="./rhythm.en.ogt">
<meta name="prop:title" content="string" data-doc="The heading (the song's title); empty takes the kit's (Difficulty).">
<meta name="prop:levels" content="list" data-doc="The song's charts in the game's order, keyed rows of { id, tier, label, rating, grade, score, locked }: tier the chart's family (easy, normal, hard, expert, master, special; on data-tier for the pack's colour, never alone); label its name as the game words it (Hard); rating its level number; grade the player's best grade on it (S, AA; empty unplayed); score the best score as the game formats it (981,200; empty Not played); locked it cannot be played yet (disabled).">
<meta name="prop:selected" content="string" data-doc="The chart the game holds as chosen (selected, named Selected); empty is none.">
<meta name="fixture:default" content='{"title":"","selected":"hard","levels":[{"id":"easy","tier":"easy","label":"Easy","rating":3,"grade":"S","score":"995,120","locked":false},{"id":"normal","tier":"normal","label":"Normal","rating":6,"grade":"A","score":"912,400","locked":false},{"id":"hard","tier":"hard","label":"Hard","rating":9,"grade":"","score":"","locked":false},{"id":"expert","tier":"expert","label":"Expert","rating":12,"grade":"","score":"","locked":true}]}'>
<meta name="fixture:selected" content='{"title":"Skyline Drive","selected":"easy","levels":[{"id":"easy","tier":"easy","label":"Easy","rating":3,"grade":"S","score":"995,120","locked":false},{"id":"normal","tier":"normal","label":"Normal","rating":6,"grade":"A","score":"912,400","locked":false}]}'>
<meta name="fixture:disabled" content='{"title":"","selected":"","levels":[{"id":"expert","tier":"expert","label":"Expert","rating":12,"grade":"","score":"","locked":true},{"id":"master","tier":"master","label":"Master","rating":14,"grade":"","score":"","locked":true}]}'>
<meta name="fixture:empty" content='{"title":"","selected":"","levels":[]}'>
<meta name="fixture:overflow" content='{"title":"The Unbearably Long Extended Twelve-Minute Midnight Orchestral Remix of a Song","selected":"special","levels":[{"id":"beginner","tier":"easy","label":"Beginner","rating":1,"grade":"SSS+","score":"1,000,000","locked":false},{"id":"easy","tier":"easy","label":"Easy","rating":4,"grade":"SS","score":"998,870","locked":false},{"id":"normal","tier":"normal","label":"Normal","rating":7,"grade":"S","score":"975,010","locked":false},{"id":"hard","tier":"hard","label":"Hard","rating":10,"grade":"AA","score":"931,445","locked":false},{"id":"expert","tier":"expert","label":"Expert","rating":13,"grade":"B","score":"801,992","locked":false},{"id":"master","tier":"master","label":"Master","rating":15,"grade":"","score":"","locked":false},{"id":"special","tier":"special","label":"Remaster Ultimate Challenge","rating":16.5,"grade":"","score":"","locked":false},{"id":"world","tier":"special","label":"Final Ascent","rating":99,"grade":"","score":"","locked":true}]}'>

<!--
  Choosing a chart: one leaning tile per difficulty with its tier's colour
  along its top, the tier's name, its level as a large figure, the
  player's best grade on a tag beside the best score, and Locked or
  Selected in words. The chosen tile fills with its tier and is ringed in
  it. A tile reports choose with its key; a locked chart is disabled and
  reports nothing. The game owns the charts, unlocking, grades and scores.
-->
<section id="RhythmDifficulty" part="frame" variant="hud" layout="stack" gap="tight" aria-label={title != '' ? title : t("rhythm.difficulty.title")}>
  <header class="band" part="band" layout="row"><h2 class="title" part="title">{title != '' ? title : t("rhythm.difficulty.title")}</h2></header>
  {#if levels.length}
    <div class="list" layout="scroll" gap="none">
      <div gap="base" class="tiles grid" role="listbox" aria-label={t("rhythm.difficulty.list")}>
        {#each levels as l (l.id)}
          <button part="row" class="tile" role="option" aria-selected={l.id == selected} aria-label={t("rhythm.difficulty.ratingName", { label: l.label, rating: l.rating })} disabled={l.locked} data-locked={l.locked} onclick={choose} state:selected={l.id == selected} class:tier-easy={(l.tier) == 'easy'} class:tier-expert={(l.tier) == 'expert'} class:tier-hard={(l.tier) == 'hard'} class:tier-master={(l.tier) == 'master'} class:tier-special={(l.tier) == 'special'} class:tier={l.tier}><i class="strip"></i><div class="face" layout="stack" gap="none">
            <div class="head" layout="row" gap="tight"><span part="name" class="name">{l.label}</span>
              {#if l.locked}<div part="badge" class="tag" layout="row" gap="none" class:tag-locked={true}><og-icon class="tag-icon" icon="lock"></og-icon><span class="tag-text">{t("rhythm.difficulty.locked")}</span></div>{/if}
              {#if l.id == selected}<div part="badge" class="tag" layout="row" gap="none"><span class="tag-text">{t("rhythm.difficulty.selected")}</span></div>{/if}</div>
            <span part="rating" class="rating"><span class="lv">{t("rhythm.difficulty.rating", { rating: l.rating })}</span></span>
            <div class="best" layout="row" gap="tight">
              {#if l.grade}<div part="badge" class="tag grade" layout="row" gap="none"><span class="tag-text">{l.grade}</span></div>{/if}
              <span class="score" part="caption">{l.score ? l.score : t("rhythm.difficulty.unplayed")}</span>
            </div>
          </div></button>
        {/each}
      </div>
    </div>
  {:else}
    <p class="empty-text" part="caption" role="status">{t("rhythm.difficulty.empty")}</p>
  {/if}
</section>

<style>
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(var(--layout-tile), 100%), 1fr)); }
#RhythmDifficulty { inline-size: stretch; min-inline-size: 0; color: var(--hud-ink); --lean: var(--select-tile-lean); }
.band { width: stretch; flex: none; min-height: 0; padding: 0; border: 0; background: none; }
.title { margin: 0; min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.list { overflow-y: auto; min-height: 0; flex: 0 1 auto; }
/* Tiles share the width, as many to a row as fit their measure. */
.tiles { width: stretch; padding: var(--space-sm) var(--space-xxs) var(--space-xxs); grid-template-columns: repeat(auto-fill, minmax(calc(150 * var(--font-step-px)), 1fr)); gap: var(--space-sm); }
/* A tile: a leaning plate, its tier's colour along its top. A tier the
   game names nothing else wears normal's. */
.tile { position: relative; width: stretch; min-width: 0; min-height: calc(var(--size-touch) * 2); align-items: stretch; justify-content: flex-start; padding: calc(var(--space-sm) + var(--select-tile-strip)) calc(var(--space-sm) + var(--lean)) var(--space-sm); border: 0; border-radius: 0; color: var(--hud-ink); background: var(--select-tile-fill); box-shadow: none; clip-path: polygon(var(--lean) 0, 100% 0, calc(100% - var(--lean)) 100%, 0 100%); transition: var(--change-motion); }
.strip { position: absolute; inset-inline: 0; inset-block-start: 0; block-size: var(--select-tile-strip); background: var(--select-normal-ink); }
.tile .name { color: var(--select-normal-ink); }
.tile:hover { background: var(--select-tile-hover-fill); }
.tile:state(selected) { background: var(--select-tile-normal-fill); box-shadow: var(--select-tile-normal-rim); translate: 0 calc(-6 * var(--font-step-px)); }
.tile.tier-easy .strip { background: var(--select-easy-ink); }
.tile.tier-easy .name { color: var(--select-easy-ink); }
.tile.tier-easy:state(selected) { background: var(--select-tile-easy-fill); box-shadow: var(--select-tile-easy-rim); }
.tile.tier-hard .strip { background: var(--select-hard-ink); }
.tile.tier-hard .name { color: var(--select-hard-ink); }
.tile.tier-hard:state(selected) { background: var(--select-tile-hard-fill); box-shadow: var(--select-tile-hard-rim); }
.tile.tier-expert .strip { background: var(--select-expert-ink); }
.tile.tier-expert .name { color: var(--select-expert-ink); }
.tile.tier-expert:state(selected) { background: var(--select-tile-expert-fill); box-shadow: var(--select-tile-expert-rim); }
.tile.tier-master .strip { background: var(--select-master-ink); }
.tile.tier-master .name { color: var(--select-master-ink); }
.tile.tier-master:state(selected) { background: var(--select-tile-master-fill); box-shadow: var(--select-tile-master-rim); }
.tile.tier-special .strip { background: var(--select-special-ink); }
.tile.tier-special .name { color: var(--select-special-ink); }
.tile.tier-special:state(selected) { background: var(--select-tile-special-fill); box-shadow: var(--select-tile-special-rim); }
.tile:focus-visible, .tile:state(selected):focus-visible { box-shadow: var(--select-tile-focus-rim); }
.tile:disabled { opacity: 0.55; background: var(--select-tile-fill); }
.face { width: stretch; block-size: stretch; align-items: flex-start; gap: calc(2 * var(--font-step-px)); }
.name { max-width: stretch; text-align: start; font-size: var(--hud-label-size); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.tile:state(selected) .name, .tile:state(selected).tier .name { color: var(--hud-ink); }
.rating { align-self: flex-start; text-align: start; white-space: nowrap; }
.lv { font-size: var(--select-size-level); line-height: 1; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink); }
.tile:disabled .lv { color: var(--hud-ink-dim); }
/* The tier name keeps its whole word; Locked and Selected wrap under it
   when the tile is narrow. */
.head { max-width: stretch; min-width: 0; align-items: center; flex-wrap: wrap; row-gap: calc(2 * var(--font-step-px)); }
.head .name { flex: none; max-width: stretch; }
.best { max-width: stretch; min-width: 0; align-items: center; }
.score { min-width: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-figure-weight); font-variant-numeric: var(--hud-figure-numeric); color: var(--hud-ink-dim); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.tag { align-items: center; gap: calc(4 * var(--font-step-px)); flex: none; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); line-height: 1.6; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; color: var(--hud-ink); background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.tag.grade { color: var(--hud-gold); }
.tag.tag-locked { color: var(--hud-ink-dim); }
.tag-icon { inline-size: var(--select-glyph); block-size: var(--select-glyph); }
.empty-text { margin: 0; padding: var(--space-sm) var(--space-md); align-self: flex-start; font-size: var(--hud-size-sm); color: var(--hud-ink); background: var(--hud-tag-fill); }
/* A phone sets the charts two to a row: the tier's name and its state
   across the top, the level under them and the best beside it, wrapping
   under the level when larger text leaves no room. */
#RhythmDifficulty:size(compact) { --lean: calc(var(--select-tile-lean) / 2); }
.tiles:size(compact) { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--space-xs); padding-block-start: var(--space-xxs); }
.tile:size(compact) { min-height: var(--size-touch); padding: calc(var(--space-xxs) + var(--select-tile-strip)) calc(var(--space-xs) + var(--lean)) var(--space-xxs); }
.face:size(compact) { flex-direction: row; flex-wrap: wrap; align-items: center; column-gap: var(--space-xs); }
.rating:size(compact) { align-self: center; }
.head:size(compact) { flex: 1 0 100%; }
.best:size(compact) { flex: none; }
.lv:size(compact) { font-size: var(--select-size-level-compact); }
.tile:state(selected):size(compact) { translate: none; }
</style>
