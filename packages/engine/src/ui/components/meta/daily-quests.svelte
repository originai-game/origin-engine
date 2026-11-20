<meta name="component" content="DailyQuests">
<meta name="strings" content="./general.en.ogt">
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
  counts it down), then one entry per quest: its icon in an octagon socket,
  its name over a leaning bar with the count beside it, and its prize in a
  dark box at the end. A done quest is lit gold, its prize box glows and
  a gold Claim stands beside it; a claimed one dims behind a Claimed stamp.
  On a narrow list the socket goes and the prize and Claim take a line of
  their own. The game owns progress, rewards and the clock; the list
  reports claim (a quest's key).
-->
<section id="DailyQuests" part="frame" layout="stack" fill aria-label={title != '' ? title : t("general.quests.name")} aria-busy={loading}>
  <header class="band" part="band" layout="wrap" gap="loose">
    <h2 class="title" part="title" fill>{title != '' ? title : t("general.quests.name")}</h2>
    {#if refresh}<div part="label" class="timer refresh" layout="row" gap="tight" role="timer" aria-label={refresh}><og-icon class="mark" icon="clock"></og-icon><span class="refresh-text">{refresh}</span></div>{/if}
  </header>
  {#if loading}
    <div class="empty cover" layout="stack" fill role="status"><Spinner name={t("general.quests.loading")} /><p class="empty-text">{t("general.quests.loading")}</p></div>
  {:else if quests.length}
    <div class="rows-scroll" layout="scroll" fill>
      <ul class="rows" layout="stack" gap="tight" aria-label={title != '' ? title : t("general.quests.name")}>
        {#each quests as q (q.id)}
          <li part="row" class="quest" layout="row" gap="loose" aria-label={q.name} class:state-claimed={(q.state) == 'claimed'} class:state-done={(q.state) == 'done'}>
            <div part="well" class="socket medal" layout="overlay" class:state-claimed={(q.state) == 'claimed'} class:state-done={(q.state) == 'done'}><div class="socket-face cover" anchor="fill" layout="stack"><og-icon class="medal-icon" icon={q.state == 'claimed' ? 'check' : q.icon} data-principal></og-icon></div></div>
            <div class="body" layout="stack" gap="tight" fill>
              <p class="name" part="name">{q.name}</p>
              <div class="progress" layout="row" gap="tight" class:state-claimed={(q.state) == 'claimed'} class:state-done={(q.state) == 'done'}>
                <div class="bar" fill><ProgressBar value={q.progress} buffer={0} label="" valueText={q.progressText} showValue={false} indeterminate={false} tone={q.state == 'claimed' ? 'success' : ''} /></div>
                <span class="count">{q.progressText}</span>
              </div>
            </div>
            <div class="end" layout="row" gap="tight">
              <div part="figure" class="prize reward" layout="stack" gap="none" aria-label={t("general.quests.reward", { reward: q.reward })} class:state-claimed={(q.state) == 'claimed'} class:state-done={(q.state) == 'done'}><og-icon class="prize-icon" icon={q.rewardIcon}></og-icon><span class="prize-figure">{q.reward}</span></div>
              {#if q.state == 'done'}<button id="Claim" tone="reward" class="claim go" variant="primary" play="pop" onclick={claim}>{t("general.quests.claim")}</button>{/if}
              {#if q.state == 'claimed'}<div part="claimed" class="stamp" layout="row" gap="hair" play="stamp"><og-icon class="mark" icon="check"></og-icon><span>{t("general.quests.claimed")}</span></div>{/if}
            </div>
          </li>
        {/each}
      </ul>
    </div>
  {:else}
    <div class="empty cover" layout="stack" fill role="status"><og-icon class="empty-mark" icon="flag"></og-icon><p class="empty-text">{t("general.quests.empty")}</p></div>
  {/if}
</section>

<style>
.cover { justify-content: center; align-items: center; }
.band { width: stretch; flex: none; }
/* The title keeps its words whole and takes its own width first: the tag
   wraps under it when both do not fit. */
.title { margin: 0; flex: 1 1 auto; }
/* The refresh time on a dark cut tag. */
.timer { max-width: stretch; padding: var(--space-xxs) var(--space-sm); background: var(--quest-well-fill); box-shadow: var(--quest-well-rim); border-radius: var(--quest-cut-sm) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.refresh-text { text-align: end; min-inline-size: 0; overflow-wrap: anywhere; color: var(--ink-bright); }
.mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
.rows { width: stretch; margin: 0; padding: 0 0 var(--space-xs); list-style: none; }
/* An entry: a card with two corners cut, washed gold from its leading
   edge while its reward waits, dimmed once it is taken. */
.quest { width: stretch; min-width: 0; padding: var(--space-sm) var(--space-md) var(--space-sm) var(--space-sm); background: var(--quest-entry-fill); box-shadow: var(--quest-entry-rim); border-radius: var(--quest-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.quest:hover { background: var(--quest-entry-fill); }
.quest.state-done { background: var(--quest-ready-fill); box-shadow: var(--quest-ready-rim); }
.quest.state-claimed .socket, .quest.state-claimed .body, .quest.state-claimed .prize { opacity: var(--quest-taken-opacity); }
/* The quest's icon in an octagon with a lit edge, gold while its reward waits. */
.socket { flex: none; width: var(--size-xl); height: var(--size-xl); border-radius: 0; background: var(--quest-socket-rim); box-shadow: none; clip-path: polygon(22% 0, 78% 0, 100% 22%, 100% 78%, 78% 100%, 22% 100%, 0 78%, 0 22%); --layout-anchor-inset: var(--stroke-thin); }
.socket-face { background: var(--quest-socket-fill); clip-path: polygon(22% 0, 78% 0, 100% 22%, 100% 78%, 78% 100%, 22% 100%, 0 78%, 0 22%); color: var(--ink-bright); }
.socket.state-done { background: var(--quest-socket-gold-rim); }
.socket.state-done .socket-face { background: var(--quest-socket-gold-fill); color: var(--quest-gold); }
.socket.state-claimed { background: var(--quest-socket-off-rim); }
.medal-icon { width: var(--size-sm); height: var(--size-sm); }
.body { min-width: 0; }
.name { margin: 0; text-align: start; overflow-wrap: anywhere; font: var(--headline-text); font-weight: var(--headline-weight); color: var(--ink-bright); }
.progress { width: stretch; min-width: 0; }
.progress.state-done { --primary: var(--quest-gold); }
.bar { min-width: 0; }
.count { flex: none; white-space: nowrap; font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); color: var(--ink-bright); }
.progress.state-done .count { color: var(--quest-gold); }
.end { flex: none; max-width: 45%; }
/* The prize in a dark cut box, glowing gold while it waits. */
.prize { flex: none; min-inline-size: calc(var(--size-xl) * 1.4); max-inline-size: stretch; min-block-size: var(--size-xl); padding: var(--space-xxs) var(--space-sm); align-items: center; justify-content: center; background: var(--quest-well-fill); box-shadow: var(--quest-well-rim); border-radius: var(--quest-cut-sm) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.prize.state-done { background: var(--quest-well-ready-fill); box-shadow: var(--quest-well-ready-rim); color: var(--quest-gold); }
.prize-icon { width: var(--size-sm); height: var(--size-sm); flex: none; }
.prize-figure { text-align: center; overflow-wrap: anywhere; font-size: var(--quest-size-figure); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); }
/* Claim: a gold face that pops in. */
.claim { flex: none; min-height: var(--size-touch);  }
.stamp { flex: none; white-space: nowrap; font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--tone-success); }
.empty { width: stretch; }
.empty-mark { width: var(--size-lg); height: var(--size-lg); }
.empty-text { margin: 0; }
/* A narrow list: no socket; the prize and Claim take a line of their own. */
@container (width < 480px) {
  .quest { flex-wrap: wrap; row-gap: var(--space-xs); padding: var(--space-sm); }
  .socket { display: none; }
  .end { width: stretch; max-width: stretch; justify-content: space-between; }
  .prize { flex-direction: row; min-block-size: 0; column-gap: var(--space-xs); padding: var(--space-xxs) var(--space-md); }
}
</style>
