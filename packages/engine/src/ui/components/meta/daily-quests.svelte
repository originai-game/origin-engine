<meta name="component" content="DailyQuests">
<meta name="strings" content="./general.en.ogt">
<meta name="parts" content="DailyQuests.frame DailyQuests.band DailyQuests.title DailyQuests.refresh DailyQuests.row DailyQuests.medal DailyQuests.name DailyQuests.reward DailyQuests.label DailyQuests.figure DailyQuests.well DailyQuests.headline">
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
<section id="DailyQuests" part="DailyQuests.frame" data-layout="shell" aria-label={title != '' ? title : t("general.quests.name")} aria-busy={loading}>
  <header part="DailyQuests.band" data-layout="cluster" data-gap="loose">
    <h2 part="DailyQuests.title" data-fill>{title != '' ? title : t("general.quests.name")}</h2>
    {#if refresh}<div part="DailyQuests.refresh DailyQuests.label" data-layout="row" data-gap="hair" role="timer"><og-icon class="mark" icon="clock"></og-icon><span class="refresh-text">{refresh}</span></div>{/if}
  </header>
  {#if loading}
    <div class="empty" data-layout="cover" data-fill role="status"><Spinner name={t("general.quests.loading")} /><p class="empty-text">{t("general.quests.loading")}</p></div>
  {:else if quests.length}
    <div class="rows-scroll" data-layout="scroll" data-fill>
      <ul class="rows" data-layout="stack" data-gap="tight" aria-label={title != '' ? title : t("general.quests.name")}>
        {#each quests as q (q.id)}
          <li part="DailyQuests.row" data-layout="row" data-gap="loose" data-state={q.state} aria-label={q.name}>
            <div part="DailyQuests.medal DailyQuests.well" data-layout="row" data-gap="none"><og-icon class="medal-icon" icon={q.icon}></og-icon></div>
            <div class="body" data-layout="stack" data-gap="tight" data-fill>
              <p part="DailyQuests.name DailyQuests.headline">{q.name}</p>
              <ProgressBar value={q.progress} buffer={0} label="" valueText={q.progressText} showValue={true} indeterminate={false} tone={q.state == 'active' ? 'neutral' : 'success'} />
            </div>
            <div class="end" data-layout="stack" data-gap="hair">
              <div part="DailyQuests.reward DailyQuests.figure" data-layout="row" data-gap="hair" aria-label={t("general.quests.reward", { reward: q.reward })}><og-icon class="mark" icon={q.rewardIcon}></og-icon><span class="figure">{q.reward}</span></div>
              {#if q.state == 'done'}<button id="Claim" variant="primary" onclick={claim}>{t("general.quests.claim")}</button>{/if}
              {#if q.state == 'claimed'}<div class="claimed" data-layout="row" data-gap="hair" data-tone="success"><og-icon class="mark" icon="check"></og-icon><span>{t("general.quests.claimed")}</span></div>{/if}
            </div>
          </li>
        {/each}
      </ul>
    </div>
  {:else}
    <div class="empty" data-layout="cover" data-fill role="status"><og-icon class="empty-mark" icon="flag"></og-icon><p class="empty-text">{t("general.quests.empty")}</p></div>
  {/if}
</section>

<style>
[part="DailyQuests.band"] { width: stretch; flex: none; }
[part="DailyQuests.title"] { margin: 0; }
[part="DailyQuests.refresh DailyQuests.label"] { max-width: stretch; }
.refresh-text { text-align: end; }
.mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
.rows { width: stretch; margin: 0; padding: 0; list-style: none; }
[part="DailyQuests.row"] { width: stretch; padding: var(--space-sm) var(--space-md); }
[part~="DailyQuests.medal"] { flex: none; width: var(--size-lg); height: var(--size-lg); justify-content: center; }
.medal-icon { width: var(--size-sm); height: var(--size-sm); }
[part~="DailyQuests.name"] { margin: 0; text-align: start; overflow-wrap: anywhere; }
.end { flex: none; align-items: flex-end; max-width: 40%; }
[part="DailyQuests.reward DailyQuests.figure"] { max-width: stretch; }
[part="DailyQuests.reward DailyQuests.figure"] .figure { text-align: end; }
#Claim { min-height: var(--size-touch); }
.claimed { white-space: nowrap; }
.empty { width: stretch; }
.empty-mark { width: var(--size-lg); height: var(--size-lg); }
.empty-text { margin: 0; }
@container (width < 420px) {
  [part="DailyQuests.row"] { padding: var(--space-xs) var(--space-sm); }
  [part~="DailyQuests.medal"] { display: none; }
}
</style>
