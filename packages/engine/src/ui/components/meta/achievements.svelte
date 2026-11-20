<meta name="component" content="Achievements">
<meta name="strings" content="./general.en.ogt">
<meta name="parts" content="Achievements.frame Achievements.band Achievements.title Achievements.summary Achievements.figure Achievements.row Achievements.medal Achievements.name Achievements.caption Achievements.state Achievements.label Achievements.tier Achievements.rarity Achievements.claimed">
<meta name="prop:title" content="string" data-doc="The list's heading; empty takes the group's (Achievements).">
<meta name="prop:summary" content="string" data-doc="The overall count as the game words it (34 / 120); empty hides it with its bar.">
<meta name="prop:done" content="number" data-doc="The overall share unlocked, 0 to 1, for the summary bar.">
<meta name="prop:achievements" content="list" data-doc="The achievements in the game's order: keyed rows of { id, name, about, icon, state, progress, progressText, date, hidden, rarity, tier, claimable, claimed }: state locked, progress or unlocked; progress 0 to 1 and progressText (3 / 10) for one in progress; date when it was unlocked, as the game formats it, empty says Unlocked alone; hidden a secret one, shown as Hidden achievement until unlocked; rarity the share of players who have it (2.4%), empty hides it; tier common, rare, epic or legendary, shown as a word and as the medal's colour, empty hides the word; claimable a reward waiting; claimed it was taken, which hides Claim.">
<meta name="fixture:default" content='{"title":"","summary":"3 / 5","done":0.6,"achievements":[{"id":"first","name":"First Steps","about":"Finish the tutorial.","icon":"flag","state":"unlocked","progress":1,"progressText":"","date":"12 Nov 2026","hidden":false,"rarity":"92.1%","tier":"common","claimable":true,"claimed":false},{"id":"hunter","name":"Monster Hunter","about":"Defeat 100 monsters.","icon":"sword","state":"progress","progress":0.64,"progressText":"64 / 100","date":"","hidden":false,"rarity":"18.4%","tier":"rare","claimable":false,"claimed":false},{"id":"rich","name":"Tycoon","about":"Hold a million gold at once.","icon":"coin","state":"locked","progress":0,"progressText":"","date":"","hidden":false,"rarity":"1.2%","tier":"legendary","claimable":false,"claimed":false},{"id":"secret","name":"The Truth","about":"Find what lies beneath.","icon":"skull","state":"locked","progress":0,"progressText":"","date":"","hidden":true,"rarity":"0.4%","tier":"epic","claimable":false,"claimed":false},{"id":"pet","name":"Best Friends","about":"Pet the dog.","icon":"heart","state":"unlocked","progress":1,"progressText":"","date":"3 Nov 2026","hidden":false,"rarity":"71.0%","tier":"common","claimable":false,"claimed":true}]}'>
<meta name="fixture:empty" content='{"title":"","summary":"","done":0,"achievements":[]}'>
<meta name="fixture:overflow" content='{"title":"Feats and Accomplishments of the Realm","summary":"1,204 / 9,999","done":0.12,"achievements":[{"id":"a1","name":"An Achievement With an Exceptionally Long Name That Wraps Across Lines","about":"A description that goes on for a good while so it has to wrap onto a second and maybe a third line in narrow layouts.","icon":"trophy","state":"progress","progress":0.33,"progressText":"3,333 / 10,000","date":"","hidden":false,"rarity":"0.01%","tier":"legendary","claimable":false,"claimed":false},{"id":"a2","name":"Collector","about":"Collect every gem.","icon":"gem","state":"unlocked","progress":1,"progressText":"","date":"1 Jan 2026","hidden":false,"rarity":"5.5%","tier":"epic","claimable":true,"claimed":false},{"id":"a3","name":"Secret","about":"","icon":"star","state":"progress","progress":0.5,"progressText":"1 / 2","date":"","hidden":true,"rarity":"","tier":"rare","claimable":false,"claimed":false},{"id":"a4","name":"Sprinter","about":"Run 10 km.","icon":"bolt","state":"progress","progress":0.9,"progressText":"9 / 10 km","date":"","hidden":false,"rarity":"44%","tier":"common","claimable":false,"claimed":false},{"id":"a5","name":"Chef","about":"Cook 50 meals.","icon":"apple","state":"locked","progress":0,"progressText":"","date":"","hidden":false,"rarity":"12%","tier":"common","claimable":false,"claimed":false},{"id":"a6","name":"Angler","about":"Catch 30 fish.","icon":"drop","state":"locked","progress":0,"progressText":"","date":"","hidden":false,"rarity":"9%","tier":"rare","claimable":false,"claimed":false},{"id":"a7","name":"Bard","about":"Play every song.","icon":"music","state":"unlocked","progress":1,"progressText":"","date":"2 Feb 2026","hidden":false,"rarity":"3%","tier":"epic","claimable":false,"claimed":true}]}'>

<!--
  The achievement list: a band with the title, then the overall count as a
  large figure over a leaning gold bar; then one entry per achievement, a
  card with two corners cut: its medal (a hexagon rimmed in its tier's
  colour), its name, description, its state in words (locked, a bar for one
  in progress, the date for one unlocked) and its tier on a chip; at the end
  the share of players who have it, large, and a gold Claim while a reward
  waits (Claimed once taken). A hidden achievement shows as one, hatched,
  with neither name nor description, until it is unlocked. On a narrow list
  the rarity takes a line under the words and Claim takes a line of its own. The
  game owns every number; the list reports claim (an achievement's key). It
  fills its height and scrolls inside it.
-->
<section id="Achievements" part="Achievements.frame" data-layout="shell" aria-label={title != '' ? title : t("general.achievements.name")}>
  <header part="Achievements.band" data-layout="cluster" data-gap="loose">
    <h2 part="Achievements.title" data-fill>{title != '' ? title : t("general.achievements.name")}</h2>
  </header>
  {#if summary}<div part="Achievements.summary" class="summary" data-layout="row" data-gap="loose">
    <span part="Achievements.figure" class="total">{summary}</span>
    <div class="total-bar" data-fill><ProgressBar value={done} buffer={0} label="" valueText={summary} showValue={false} indeterminate={false} tone="" /></div>
  </div>{/if}
  {#if achievements.length}
    <div class="rows-scroll" data-layout="scroll" data-fill>
      <ul class="rows" data-layout="stack" data-gap="tight" aria-label={title != '' ? title : t("general.achievements.name")}>
        {#each achievements as a (a.id)}
          <li part="Achievements.row" class="entry" data-layout="row" data-gap="loose" data-state={a.state} data-hidden={a.hidden && a.state != 'unlocked'} data-tier={a.tier} data-ready={a.claimable && !a.claimed} data-claimed={a.claimed} aria-label={a.hidden && a.state != 'unlocked' ? t("general.achievements.hidden") : a.name}>
            <div part="Achievements.medal" class="medal" data-layout="overlay" data-tier={a.tier} data-state={a.state}><div class="medal-face" data-anchor="fill" data-layout="cover"><og-icon class="medal-icon" icon={a.hidden && a.state != 'unlocked' ? 'lock' : a.icon} data-principal></og-icon></div></div>
            <div class="body" data-layout="stack" data-gap="hair" data-fill>
              <p part="Achievements.name">{a.hidden && a.state != 'unlocked' ? t("general.achievements.hidden") : a.name}</p>
              {#if (a.hidden && a.state != 'unlocked') || a.about}<p part="Achievements.caption" class="about">{a.hidden && a.state != 'unlocked' ? t("general.achievements.hiddenAbout") : a.about}</p>{/if}
              <div class="meta" data-layout="cluster" data-gap="tight">
                {#if a.state == 'progress'}
                  <div class="bar"><ProgressBar value={a.progress} buffer={0} label="" valueText={a.progressText} showValue={false} indeterminate={false} tone="" /></div>
                  <span class="count" aria-label={t("general.achievements.progress")}>{a.progressText}</span>
                {:else}
                  <div part="Achievements.state Achievements.label" class="state" data-layout="row" data-gap="hair" data-state={a.state}><og-icon class="state-mark" icon={a.state == 'unlocked' ? 'check' : 'lock'}></og-icon><span>{a.state == 'unlocked' ? (a.date ? t("general.achievements.unlockedOn", { date: a.date }) : t("general.achievements.unlocked")) : t("general.achievements.locked")}</span></div>
                {/if}
                {#if a.tier}<p part="Achievements.tier" class="tier" data-tier={a.tier}>{a.tier == 'legendary' ? t("general.achievements.tierLegendary") : a.tier == 'epic' ? t("general.achievements.tierEpic") : a.tier == 'rare' ? t("general.achievements.tierRare") : t("general.achievements.tierCommon")}</p>{/if}
              </div>
            </div>
            {#if a.rarity}<div part="Achievements.rarity" class="rarity" data-tier={a.tier} role="group" aria-label={t("general.achievements.rarity", { percent: a.rarity })}><span class="share">{a.rarity}</span><span class="of">{t("general.achievements.players")}</span></div>{/if}
            {#if a.claimed}<div part="Achievements.claimed" class="stamp" data-layout="row" data-gap="hair" data-motion="stamp"><og-icon class="state-mark" icon="check"></og-icon><span>{t("general.achievements.claimed")}</span></div>
            {:else if a.claimable}<button id="Claim" data-tone="reward" class="claim go" variant="primary" data-motion="pop" onclick={claim}>{t("general.achievements.claim")}</button>{/if}
          </li>
        {/each}
      </ul>
    </div>
  {:else}
    <div class="empty" data-layout="cover" data-fill role="status"><og-icon class="empty-mark" icon="trophy"></og-icon><p class="empty-text">{t("general.achievements.empty")}</p></div>
  {/if}
</section>

<style>
[part="Achievements.band"] { width: stretch; flex: none; }
[part="Achievements.title"] { margin: 0; min-inline-size: 0; }
/* The overall count: a large figure over a leaning gold bar. */
.summary { width: stretch; flex: none; padding: var(--space-sm) var(--space-lg) var(--space-xs); --primary: var(--quest-gold); }
.total { flex: none; white-space: nowrap; --figure-size: var(--quest-size-total); --figure-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); line-height: var(--font-leading-flat); }
.total-bar { min-width: 0; }
.rows { width: stretch; margin: 0; padding: 0 0 var(--space-xs); list-style: none; }
/* An entry: a card with two corners cut; gold from its edge while a reward waits; hatched while hidden. */
.entry { width: stretch; min-width: 0; padding: var(--space-sm) var(--space-md) var(--space-sm) var(--space-sm); background: var(--quest-entry-fill); box-shadow: var(--quest-entry-rim); border-radius: var(--quest-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.entry:hover { background: var(--quest-entry-fill); }
.entry[data-ready=true] { background: var(--quest-ready-fill); box-shadow: var(--quest-ready-rim); }
.entry[data-hidden=true] { background: var(--quest-locked-fill); }
.entry[data-claimed=true] .medal, .entry[data-claimed=true] .body, .entry[data-claimed=true] .rarity { opacity: var(--quest-taken-opacity); }
/* The medal: a hexagon in its tier's colour (the rarity colours), grey while locked. */
.medal { flex: none; width: var(--size-xl); height: calc(var(--size-xl) * 1.1); color: var(--quest-ink-quiet); background: var(--quest-medal-rim); filter: var(--quest-medal-glow); clip-path: polygon(50% 0, 93.3% 25%, 93.3% 75%, 50% 100%, 6.7% 75%, 6.7% 25%); --layout-anchor-inset: var(--stroke-thick); }
/* A common medal, and one with no tier, takes the muted ink: the common rarity colour is a line grey, too dark for a rim and its icon. */
.medal[data-tier=rare] { color: var(--rare); }
.medal[data-tier=epic] { color: var(--epic); }
.medal[data-tier=legendary] { color: var(--legendary); }
.medal[data-state=locked] { color: var(--quest-ink-faint); background: var(--quest-medal-locked-rim); filter: none; }
.medal-face { background: var(--quest-medal-fill); clip-path: polygon(50% 0, 93.3% 25%, 93.3% 75%, 50% 100%, 6.7% 75%, 6.7% 25%); }
.medal-icon { width: var(--size-sm); height: var(--size-sm); }
.body { min-width: 0; }
[part="Achievements.name"] { margin: 0; text-align: start; overflow-wrap: anywhere; font: var(--headline-text); font-weight: var(--quest-weight); color: var(--ink-bright); }
.entry[data-state=locked] [part="Achievements.name"] { color: var(--quest-ink-quiet); }
.about { margin: 0; text-align: start; overflow-wrap: anywhere; }
.meta { width: stretch; align-items: center; }
.bar { inline-size: calc(var(--size-xxl) * 3); max-inline-size: stretch; min-inline-size: 0; flex: 0 1 auto; }
.count { flex: none; white-space: nowrap; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--ink-bright); }
/* The state wraps between its words rather than run under the rarity. */
.state { min-inline-size: 0; }
.state[data-state=unlocked] { color: var(--tone-success); }
.state-mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
/* The tier on a cut chip in its colour. */
.tier { margin: 0; flex: none; white-space: nowrap; padding: 0 var(--space-xs); font: var(--caption-text); font-weight: var(--quest-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--quest-ink-quiet); background: var(--quest-chip-fill); box-shadow: var(--quest-chip-rim); border-radius: var(--badge-radius); }
.tier[data-tier=rare] { color: var(--rare); }
.tier[data-tier=epic] { color: var(--epic); }
.tier[data-tier=legendary] { color: var(--legendary); }
/* The share of players who have it, large at the end of the entry. */
.rarity { flex: none; margin: 0; min-inline-size: calc(var(--size-xl) * 1.5); display: flex; flex-direction: column; align-items: flex-end; text-align: end; }
.share { white-space: nowrap; font-size: var(--quest-size-figure); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--ink-bright); }
.rarity[data-tier=legendary] .share { color: var(--quest-gold); }
.of { white-space: nowrap; font: var(--caption-text); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--quest-ink-faint); }
.claim { flex: none; min-height: var(--size-touch);  }
.stamp { flex: none; white-space: nowrap; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--tone-success); }
.empty { width: stretch; }
.empty-mark { width: var(--size-lg); height: var(--size-lg); }
.empty-text { margin: 0; }
/* A narrow list: the rarity takes a line under the words, Claim and Claimed take a line of their own. */
@container (width < 480px) {
  .summary { padding: var(--space-xs) var(--space-md); }
  .total { --figure-size: var(--quest-size-total-compact); }
  .entry { flex-wrap: wrap; align-items: flex-start; row-gap: var(--space-xs); padding: var(--space-sm); }
  .medal { width: var(--size-lg); height: calc(var(--size-lg) * 1.1); }
  .rarity { width: stretch; flex-direction: row; align-items: baseline; column-gap: var(--space-xs); padding-inline-start: calc(var(--size-lg) + var(--space-md)); }
  .claim { width: stretch; }
  .stamp { width: stretch; justify-content: flex-end; }
}
</style>
