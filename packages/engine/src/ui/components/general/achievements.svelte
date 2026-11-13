<meta name="component" content="Achievements">
<meta name="strings" content="./general.en.ogt">
<meta name="parts" content="Achievements.frame Achievements.band Achievements.title Achievements.summary Achievements.row Achievements.medal Achievements.name Achievements.state Achievements.tier Achievements.rarity">
<meta name="prop:title" content="string" data-doc="The list's heading; empty takes the group's (Achievements).">
<meta name="prop:summary" content="string" data-doc="The overall count as the game words it (34 / 120); empty hides it with its bar.">
<meta name="prop:done" content="number" data-doc="The overall share unlocked, 0 to 1, for the summary bar.">
<meta name="prop:achievements" content="list" data-doc="The achievements in the game's order: keyed rows of { id, name, about, icon, state, progress, progressText, date, hidden, rarity, tier, claimable, claimed }: state locked, progress or unlocked; progress 0 to 1 and progressText (3 / 10) for one in progress; date when it was unlocked, as the game formats it, empty says Unlocked alone; hidden a secret one, shown as Hidden achievement until unlocked; rarity the share of players who have it (2.4%), empty hides it; tier common, rare, epic or legendary, shown as a word and marked for a pack, empty hides it; claimable a reward waiting; claimed it was taken, which hides Claim.">
<meta name="fixture:default" content='{"title":"","summary":"3 / 5","done":0.6,"achievements":[{"id":"first","name":"First Steps","about":"Finish the tutorial.","icon":"flag","state":"unlocked","progress":1,"progressText":"","date":"12 Nov 2026","hidden":false,"rarity":"92.1%","tier":"common","claimable":true,"claimed":false},{"id":"hunter","name":"Monster Hunter","about":"Defeat 100 monsters.","icon":"sword","state":"progress","progress":0.64,"progressText":"64 / 100","date":"","hidden":false,"rarity":"18.4%","tier":"rare","claimable":false,"claimed":false},{"id":"rich","name":"Tycoon","about":"Hold a million gold at once.","icon":"coin","state":"locked","progress":0,"progressText":"","date":"","hidden":false,"rarity":"1.2%","tier":"legendary","claimable":false,"claimed":false},{"id":"secret","name":"The Truth","about":"Find what lies beneath.","icon":"skull","state":"locked","progress":0,"progressText":"","date":"","hidden":true,"rarity":"0.4%","tier":"epic","claimable":false,"claimed":false},{"id":"pet","name":"Best Friends","about":"Pet the dog.","icon":"heart","state":"unlocked","progress":1,"progressText":"","date":"3 Nov 2026","hidden":false,"rarity":"71.0%","tier":"common","claimable":false,"claimed":true}]}'>
<meta name="fixture:empty" content='{"title":"","summary":"","done":0,"achievements":[]}'>
<meta name="fixture:overflow" content='{"title":"Feats and Accomplishments of the Realm","summary":"1,204 / 9,999","done":0.12,"achievements":[{"id":"a1","name":"An Achievement With an Exceptionally Long Name That Wraps Across Lines","about":"A description that goes on for a good while so it has to wrap onto a second and maybe a third line in narrow layouts.","icon":"trophy","state":"progress","progress":0.33,"progressText":"3,333 / 10,000","date":"","hidden":false,"rarity":"0.01%","tier":"legendary","claimable":false,"claimed":false},{"id":"a2","name":"Collector","about":"Collect every gem.","icon":"gem","state":"unlocked","progress":1,"progressText":"","date":"1 Jan 2026","hidden":false,"rarity":"5.5%","tier":"epic","claimable":true,"claimed":false},{"id":"a3","name":"Secret","about":"","icon":"star","state":"progress","progress":0.5,"progressText":"1 / 2","date":"","hidden":true,"rarity":"","tier":"rare","claimable":false,"claimed":false},{"id":"a4","name":"Sprinter","about":"Run 10 km.","icon":"bolt","state":"progress","progress":0.9,"progressText":"9 / 10 km","date":"","hidden":false,"rarity":"44%","tier":"common","claimable":false,"claimed":false},{"id":"a5","name":"Chef","about":"Cook 50 meals.","icon":"apple","state":"locked","progress":0,"progressText":"","date":"","hidden":false,"rarity":"12%","tier":"common","claimable":false,"claimed":false},{"id":"a6","name":"Angler","about":"Catch 30 fish.","icon":"drop","state":"locked","progress":0,"progressText":"","date":"","hidden":false,"rarity":"9%","tier":"rare","claimable":false,"claimed":false},{"id":"a7","name":"Bard","about":"Play every song.","icon":"music","state":"unlocked","progress":1,"progressText":"","date":"2 Feb 2026","hidden":false,"rarity":"3%","tier":"epic","claimable":false,"claimed":true}]}'>

<!--
  The achievement list: a band with the title and the overall count, then
  one row per achievement with its icon, name, description, its state in
  words (locked, a bar for one in progress, the date for one unlocked), the
  tier in words, the share of players who have it and a Claim button while a
  reward waits and is not yet taken. A
  hidden achievement shows as one, with neither name nor description, until
  it is unlocked. The game owns every number; the list reports claim (an
  achievement's key). It fills its height and scrolls inside it.
-->
<section id="Achievements" part="Achievements.frame" aria-label={title != '' ? title : t("general.achievements.name")}>
  <header part="Achievements.band">
    <h2 part="Achievements.title">{title != '' ? title : t("general.achievements.name")}</h2>
    <div part="Achievements.summary" hidden={summary == ''}>
      <span class="figure">{summary}</span>
      <div class="total"><ProgressBar value={done} buffer={0} label="" valueText={summary} showValue={false} indeterminate={false} tone="neutral" /></div>
    </div>
  </header>
  {#if achievements.length}
    <scroll class="rows-scroll">
      <ul class="rows" aria-label={title != '' ? title : t("general.achievements.name")}>
        {#each achievements as a (a.id)}
          <li part="Achievements.row" data-state={a.state} data-hidden={a.hidden && a.state != 'unlocked'} data-tier={a.tier} data-claimable={a.claimable} data-claimed={a.claimed} aria-label={a.hidden && a.state != 'unlocked' ? t("general.achievements.hidden") : a.name}>
            <div part="Achievements.medal"><og-icon class="medal-icon" icon={a.hidden && a.state != 'unlocked' ? 'lock' : a.icon}></og-icon></div>
            <div class="body">
              <p part="Achievements.name">{a.hidden && a.state != 'unlocked' ? t("general.achievements.hidden") : a.name}</p>
              <p class="about" hidden={!(a.hidden && a.state != 'unlocked') && a.about == ''}>{a.hidden && a.state != 'unlocked' ? t("general.achievements.hiddenAbout") : a.about}</p>
              <div class="meta">
                <div part="Achievements.state" data-state={a.state} hidden={a.state == 'progress'}><og-icon class="state-mark" icon={a.state == 'unlocked' ? 'check' : 'lock'}></og-icon><span>{a.state == 'unlocked' ? (a.date != '' ? t("general.achievements.unlockedOn", { date: a.date }) : t("general.achievements.unlocked")) : t("general.achievements.locked")}</span></div>
                <div class="bar" hidden={a.state != 'progress'}><ProgressBar value={a.progress} buffer={0} label={t("general.achievements.progress")} valueText={a.progressText} showValue={true} indeterminate={false} tone="neutral" /></div>
                <p part="Achievements.tier" data-tier={a.tier} hidden={a.tier == ''}>{a.tier == 'legendary' ? t("general.achievements.tierLegendary") : a.tier == 'epic' ? t("general.achievements.tierEpic") : a.tier == 'rare' ? t("general.achievements.tierRare") : t("general.achievements.tierCommon")}</p>
                <p part="Achievements.rarity" data-tier={a.tier} hidden={a.rarity == ''}>{t("general.achievements.rarity", { percent: a.rarity })}</p>
              </div>
            </div>
            <div class="end" hidden={!a.claimable && !a.claimed}>
              <button id="Claim" variant="primary" hidden={!a.claimable || a.claimed} onclick={claim}>{t("general.achievements.claim")}</button>
              <div class="claimed" hidden={!a.claimed}><og-icon class="state-mark" icon="check"></og-icon><span>{t("general.achievements.claimed")}</span></div>
            </div>
          </li>
        {/each}
      </ul>
    </scroll>
  {:else}
    <div class="empty" role="status"><og-icon class="empty-mark" icon="trophy"></og-icon><p class="empty-text">{t("general.achievements.empty")}</p></div>
  {/if}
</section>

<style>
[part="Achievements.frame"][hidden], [part="Achievements.frame"] [hidden] { display: none; }
[part="Achievements.frame"] { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-sm); container-type: inline-size; }
[part="Achievements.band"] { display: flex; width: stretch; align-items: center; gap: var(--space-md); flex: none; flex-wrap: wrap; }
[part="Achievements.title"] { margin: 0; flex: 1; min-width: 0; }
[part="Achievements.summary"] { display: flex; align-items: center; gap: var(--space-sm); width: calc(var(--size-xxl) * 3); max-width: stretch; }
.total { width: stretch; }
.figure { font-variant-numeric: tabular-nums; white-space: nowrap; }
.rows-scroll { width: stretch; flex: 1; min-height: 0; }
.rows { display: flex; flex-direction: column; width: stretch; margin: 0; padding: 0; list-style: none; gap: var(--space-xs); }
[part="Achievements.row"] { display: flex; width: stretch; align-items: center; gap: var(--space-md); padding: var(--space-sm) var(--space-md); }
[part="Achievements.medal"] { display: flex; flex: none; width: var(--size-xl); height: var(--size-xl); align-items: center; justify-content: center; }
.medal-icon { width: var(--size-md); height: var(--size-md); }
.body { display: flex; flex-direction: column; width: stretch; min-width: 0; gap: var(--space-xxs); }
[part="Achievements.name"] { text-align: start; overflow-wrap: anywhere; }
.about { margin: 0; text-align: start; }
.meta { display: flex; width: stretch; align-items: center; flex-wrap: wrap; gap: var(--space-xxs) var(--space-md); }
[part="Achievements.state"], .claimed { display: flex; align-items: center; gap: var(--space-xxs); white-space: nowrap; }
.state-mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
.bar { width: calc(var(--size-xxl) * 3); max-width: stretch; }
[part="Achievements.tier"] { margin: 0; white-space: nowrap; }
[part="Achievements.rarity"] { white-space: nowrap; font-variant-numeric: tabular-nums; }
.end { display: flex; flex: none; align-items: center; }
#Claim { min-height: var(--size-touch); }
.empty { display: flex; flex-direction: column; width: stretch; flex: 1; gap: var(--space-sm); align-items: center; justify-content: center; }
.empty-mark { width: var(--size-lg); height: var(--size-lg); }
.empty-text { margin: 0; }
@container (width < 480px) {
  [part="Achievements.row"] { flex-wrap: wrap; gap: var(--space-sm); padding: var(--space-xs) var(--space-sm); }
  [part="Achievements.medal"] { width: var(--size-lg); height: var(--size-lg); }
  .body { flex: 1; }
  .end { width: stretch; justify-content: flex-end; }
}
</style>
