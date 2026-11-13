<meta name="component" content="DailyQuests">
<meta name="strings" content="./general.en.ogt">
<meta name="parts" content="DailyQuests.frame DailyQuests.band DailyQuests.title DailyQuests.refresh DailyQuests.row DailyQuests.medal DailyQuests.name DailyQuests.reward">
<meta name="prop:title" content="string" data-doc="The list's heading; empty takes the group's (Daily quests).">
<meta name="prop:refresh" content="string" data-doc="When the quests refresh, as the game words it (Resets in 4h 12m); empty hides it. The game counts it down.">
<meta name="prop:quests" content="list" data-doc="Today's quests in the game's order: keyed rows of { id, name, icon, progress, progressText, reward, rewardIcon, state }: progress 0 to 1 and progressText (3 / 5); reward the prize in words (50) with its icon; state active, done (its reward waiting) or claimed.">
<meta name="prop:loading" content="bool" data-doc="Today's quests are on their way: a spinner stands in.">
<meta name="fixture:default" content='{"title":"","refresh":"Resets in 4h 12m","quests":[{"id":"win","name":"Win 3 matches","icon":"trophy","progress":0.67,"progressText":"2 / 3","reward":"50","rewardIcon":"gem","state":"active"},{"id":"gather","name":"Gather 20 herbs","icon":"leaf","progress":1,"progressText":"20 / 20","reward":"500","rewardIcon":"coin","state":"done"},{"id":"login","name":"Log in","icon":"check","progress":1,"progressText":"1 / 1","reward":"100","rewardIcon":"coin","state":"claimed"}],"loading":false}'>
<meta name="fixture:empty" content='{"title":"","refresh":"Resets in 23h 59m","quests":[],"loading":false}'>
<meta name="fixture:loading" content='{"title":"","refresh":"","quests":[],"loading":true}'>
<meta name="fixture:overflow" content='{"title":"Daily Challenges of the Adventurers Guild","refresh":"Resets in 0h 01m, hurry before the next rotation","quests":[{"id":"q1","name":"Defeat twenty-five elite guardians in the Sunken Cathedral without healing","icon":"sword","progress":0.2,"progressText":"5 / 25","reward":"1,000,000 gold and a chest","rewardIcon":"coin","state":"active"},{"id":"q2","name":"Craft","icon":"gear","progress":0,"progressText":"0 / 1","reward":"5","rewardIcon":"gem","state":"active"},{"id":"q3","name":"Fish","icon":"drop","progress":1,"progressText":"10 / 10","reward":"20","rewardIcon":"gem","state":"done"},{"id":"q4","name":"Explore","icon":"map","progress":0.5,"progressText":"1 / 2","reward":"200","rewardIcon":"coin","state":"active"},{"id":"q5","name":"Trade","icon":"bag","progress":1,"progressText":"3 / 3","reward":"30","rewardIcon":"gem","state":"claimed"}],"loading":false}'>

<!--
  Today's quests: a band with the title and when they refresh (the game
  counts it down), then one row per quest with its icon, name, a progress
  bar, its reward and, once done, a Claim button. The game owns progress,
  rewards and the clock; the list reports claim (a quest's key).
-->
<section id="DailyQuests" part="DailyQuests.frame" aria-label={title != '' ? title : t("general.quests.name")} aria-busy={loading}>
  <header part="DailyQuests.band">
    <h2 part="DailyQuests.title">{title != '' ? title : t("general.quests.name")}</h2>
    <div part="DailyQuests.refresh" role="timer" hidden={refresh == ''}><og-icon class="mark" icon="clock"></og-icon><span class="refresh-text">{refresh}</span></div>
  </header>
  {#if loading}
    <div class="empty" role="status"><Spinner size={0} name={t("general.quests.loading")} /><p class="empty-text">{t("general.quests.loading")}</p></div>
  {:else if quests.length}
    <scroll class="rows-scroll">
      <ul class="rows" aria-label={title != '' ? title : t("general.quests.name")}>
        {#each quests as q (q.id)}
          <li part="DailyQuests.row" data-state={q.state} aria-label={q.name}>
            <div part="DailyQuests.medal"><og-icon class="medal-icon" icon={q.icon}></og-icon></div>
            <div class="body">
              <p part="DailyQuests.name">{q.name}</p>
              <ProgressBar value={q.progress} buffer={0} label="" valueText={q.progressText} showValue={true} indeterminate={false} tone={q.state == 'active' ? 'neutral' : 'success'} />
            </div>
            <div class="end">
              <div part="DailyQuests.reward" aria-label={t("general.quests.reward", { reward: q.reward })}><og-icon class="mark" icon={q.rewardIcon}></og-icon><span class="figure">{q.reward}</span></div>
              <button id="Claim" variant="primary" hidden={q.state != 'done'} onclick={claim}>{t("general.quests.claim")}</button>
              <div class="claimed" hidden={q.state != 'claimed'}><og-icon class="mark" icon="check"></og-icon><span>{t("general.quests.claimed")}</span></div>
            </div>
          </li>
        {/each}
      </ul>
    </scroll>
  {:else}
    <div class="empty" role="status"><og-icon class="empty-mark" icon="flag"></og-icon><p class="empty-text">{t("general.quests.empty")}</p></div>
  {/if}
</section>

<style>
[part="DailyQuests.frame"][hidden], [part="DailyQuests.frame"] [hidden] { display: none; }
[part="DailyQuests.frame"] { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-sm); container-type: inline-size; }
[part="DailyQuests.band"] { display: flex; width: stretch; align-items: center; gap: var(--space-md); flex: none; flex-wrap: wrap; }
[part="DailyQuests.title"] { margin: 0; flex: 1; min-width: 0; }
[part="DailyQuests.refresh"] { display: flex; align-items: center; gap: var(--space-xxs); max-width: stretch; }
.refresh-text { text-align: end; }
.mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
.figure { font-variant-numeric: tabular-nums; }
.rows-scroll { width: stretch; flex: 1; min-height: 0; }
.rows { display: flex; flex-direction: column; width: stretch; margin: 0; padding: 0; list-style: none; gap: var(--space-xs); }
[part="DailyQuests.row"] { display: flex; width: stretch; align-items: center; gap: var(--space-md); padding: var(--space-sm) var(--space-md); }
[part="DailyQuests.medal"] { display: flex; flex: none; width: var(--size-lg); height: var(--size-lg); align-items: center; justify-content: center; }
.medal-icon { width: var(--size-sm); height: var(--size-sm); }
.body { display: flex; flex-direction: column; width: stretch; min-width: 0; gap: var(--space-xs); }
[part="DailyQuests.name"] { text-align: start; overflow-wrap: anywhere; }
.end { display: flex; flex: none; flex-direction: column; align-items: flex-end; gap: var(--space-xxs); max-width: 40%; }
[part="DailyQuests.reward"] { display: flex; align-items: center; gap: var(--space-xxs); max-width: stretch; }
[part="DailyQuests.reward"] .figure { text-align: end; }
#Claim { min-height: var(--size-touch); }
.claimed { display: flex; align-items: center; gap: var(--space-xxs); white-space: nowrap; }
.empty { display: flex; flex-direction: column; width: stretch; flex: 1; gap: var(--space-sm); align-items: center; justify-content: center; }
.empty-mark { width: var(--size-lg); height: var(--size-lg); }
.empty-text { margin: 0; }
@container (width < 420px) {
  [part="DailyQuests.row"] { gap: var(--space-sm); padding: var(--space-xs) var(--space-sm); }
  [part="DailyQuests.medal"] { display: none; }
}
</style>
