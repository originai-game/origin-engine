<meta name="component" content="DailyRewards">
<meta name="strings" content="./general.en.ogt">
<meta name="parts" content="DailyRewards.frame DailyRewards.band DailyRewards.title DailyRewards.refresh DailyRewards.day DailyRewards.label DailyRewards.amount DailyRewards.state DailyRewards.cell DailyRewards.figure">
<meta name="prop:title" content="string" data-doc="The calendar's heading; empty takes the group's (Daily rewards).">
<meta name="prop:refresh" content="string" data-doc="When the next day's reward opens, as the game words it; empty hides it.">
<meta name="prop:days" content="list" data-doc="The calendar's days in order: keyed rows of { id, label, icon, amount, state, milestone }: label the day in words (Day 1); amount the reward in words (x100); state claimed, today, upcoming or missed, each shown by its own word: the game sends claimed for today once its reward is taken; milestone a big reward the pack marks.">
<meta name="prop:canClaim" content="bool" data-doc="Today's reward is waiting: the day whose state is today is a Claim button; otherwise no day is a button.">
<meta name="fixture:default" content='{"title":"","refresh":"Next reward in 9h 40m","days":[{"id":"d1","label":"Day 1","icon":"coin","amount":"x100","state":"claimed","milestone":false},{"id":"d2","label":"Day 2","icon":"potion","amount":"x3","state":"claimed","milestone":false},{"id":"d3","label":"Day 3","icon":"gem","amount":"x20","state":"missed","milestone":false},{"id":"d4","label":"Day 4","icon":"coin","amount":"x300","state":"today","milestone":false},{"id":"d5","label":"Day 5","icon":"key","amount":"x1","state":"upcoming","milestone":false},{"id":"d6","label":"Day 6","icon":"gem","amount":"x50","state":"upcoming","milestone":false},{"id":"d7","label":"Day 7","icon":"crown","amount":"Legend chest","state":"upcoming","milestone":true}],"canClaim":true}'>
<meta name="fixture:disabled" content='{"title":"","refresh":"Next reward in 23h 10m","days":[{"id":"d1","label":"Day 1","icon":"coin","amount":"x100","state":"claimed","milestone":false},{"id":"d2","label":"Day 2","icon":"potion","amount":"x3","state":"claimed","milestone":false},{"id":"d3","label":"Day 3","icon":"gem","amount":"x20","state":"claimed","milestone":true},{"id":"d4","label":"Day 4","icon":"coin","amount":"x300","state":"upcoming","milestone":false}],"canClaim":false}'>
<meta name="fixture:overflow" content='{"title":"Fourteen Days of Treasure from the Royal Vaults","refresh":"","days":[{"id":"d1","label":"Day 1","icon":"coin","amount":"x1,000,000","state":"claimed","milestone":false},{"id":"d2","label":"Day 2","icon":"potion","amount":"A very long reward name","state":"claimed","milestone":false},{"id":"d3","label":"Day 3","icon":"gem","amount":"x20","state":"claimed","milestone":false},{"id":"d4","label":"Day 4","icon":"coin","amount":"x300","state":"claimed","milestone":false},{"id":"d5","label":"Day 5","icon":"key","amount":"x1","state":"claimed","milestone":true},{"id":"d6","label":"Day 6","icon":"gem","amount":"x50","state":"missed","milestone":false},{"id":"d7","label":"Day 7","icon":"crown","amount":"x1","state":"today","milestone":false},{"id":"d8","label":"Day 8","icon":"coin","amount":"x200","state":"upcoming","milestone":false},{"id":"d9","label":"Day 9","icon":"apple","amount":"x5","state":"upcoming","milestone":false},{"id":"d10","label":"Day 10","icon":"trophy","amount":"x1","state":"upcoming","milestone":true},{"id":"d11","label":"Day 11","icon":"coin","amount":"x250","state":"upcoming","milestone":false},{"id":"d12","label":"Day 12","icon":"gem","amount":"x60","state":"upcoming","milestone":false},{"id":"d13","label":"Day 13","icon":"leaf","amount":"x9","state":"upcoming","milestone":false},{"id":"d14","label":"Day 14","icon":"crown","amount":"x1","state":"upcoming","milestone":true}],"canClaim":true}'>

<!--
  A login reward calendar: a grid of day tiles, each with its label, the
  reward's icon and amount, and its state in words and a mark (claimed,
  today, upcoming, missed); milestone days are marked for the pack. The game
  owns the schedule, the rewards and the clock; the calendar reports claim
  (today's key) while canClaim holds: only then is today's tile a button;
  every other day is a labelled tile that takes no focus and reports
  nothing. A day shows the word of the state the game gave it.
-->
<section id="DailyRewards" part="DailyRewards.frame" data-layout="shell" aria-label={title != '' ? title : t("general.rewards.name")}>
  <header part="DailyRewards.band" data-layout="cluster" data-gap="loose">
    <h2 part="DailyRewards.title" data-fill>{title != '' ? title : t("general.rewards.name")}</h2>
    {#if refresh}<div part="DailyRewards.refresh DailyRewards.label" data-layout="row" data-gap="hair" role="timer"><og-icon class="mark" icon="clock"></og-icon><span>{refresh}</span></div>{/if}
  </header>
  <div class="days-scroll" data-layout="scroll" data-fill>
    <ol class="days" data-layout="grid" data-gap="tight">
      {#each days as d (d.id)}
        <li class="slot" data-layout="row" data-gap="none">
          {#if d.state == 'today' && canClaim}
            <button part="DailyRewards.day DailyRewards.cell" data-state={d.state} data-milestone={d.milestone} aria-current="date" aria-label={t("general.rewards.day", { day: d.label, reward: d.amount, state: t("general.rewards.claim") })} onclick={claim}><div class="face" data-layout="stack" data-gap="hair">
              <p part="DailyRewards.label">{d.label}</p>
              <og-icon class="reward" icon={d.icon}></og-icon>
              <p part="DailyRewards.amount DailyRewards.figure" data-fill>{d.amount}</p>
              <div part="DailyRewards.state DailyRewards.label" data-layout="cluster" data-gap="hair" data-state={d.state}>
                <og-icon class="mark" icon="star"></og-icon>
                <span class="state-text">{t("general.rewards.claim")}</span>
              </div>
            </div></button>
          {:else}
            <div part="DailyRewards.day DailyRewards.cell" data-layout="row" data-gap="none" role="group" data-state={d.state} data-milestone={d.milestone} aria-current={d.state == 'today' ? 'date' : 'false'} aria-label={t("general.rewards.day", { day: d.label, reward: d.amount, state: d.state == 'claimed' ? t("general.rewards.claimed") : d.state == 'missed' ? t("general.rewards.missed") : d.state == 'today' ? t("general.rewards.today") : t("general.rewards.upcoming") })}><div class="face" data-layout="stack" data-gap="hair">
              <p part="DailyRewards.label">{d.label}</p>
              <og-icon class="reward" icon={d.icon}></og-icon>
              <p part="DailyRewards.amount DailyRewards.figure" data-fill>{d.amount}</p>
              <div part="DailyRewards.state DailyRewards.label" data-layout="cluster" data-gap="hair" data-state={d.state} data-tone={d.state == 'claimed' ? 'success' : ''}>
                <og-icon class="mark" icon={d.state == 'claimed' ? 'check' : d.state == 'missed' ? 'close' : d.state == 'today' ? 'star' : 'clock'}></og-icon>
                <span class="state-text">{d.state == 'claimed' ? t("general.rewards.claimed") : d.state == 'missed' ? t("general.rewards.missed") : d.state == 'today' ? t("general.rewards.today") : t("general.rewards.upcoming")}</span>
              </div>
            </div></div>
          {/if}
        </li>
      {/each}
    </ol>
  </div>
</section>

<style>
[part="DailyRewards.band"] { width: stretch; flex: none; }
[part="DailyRewards.title"] { margin: 0; }
.mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
.days { grid-template-columns: repeat(7, minmax(0, 1fr)); width: stretch; margin: 0; padding: var(--space-xxs); list-style: none; }
.slot { width: stretch; min-width: 0; }
[part="DailyRewards.day DailyRewards.cell"] { width: stretch; min-width: 0; height: stretch; min-height: calc(var(--size-xxl) * 1.5); padding: var(--space-xs); align-items: stretch; justify-content: flex-start; }
[part="DailyRewards.day DailyRewards.cell"] .face { width: stretch; height: stretch; align-items: center; }
[part="DailyRewards.label"], [part="DailyRewards.amount DailyRewards.figure"] { margin: 0; text-align: center; overflow-wrap: anywhere; }
.reward { width: var(--size-lg); height: var(--size-lg); flex: none; }
[part="DailyRewards.state DailyRewards.label"] { width: stretch; justify-content: center; }
.state-text { text-align: center; }
@container (width < 760px) {
  .days { grid-template-columns: repeat(4, minmax(0, 1fr)); }
  [part="DailyRewards.day DailyRewards.cell"] { min-height: calc(var(--size-xxl) * 1.3); }
  .reward { width: var(--size-md); height: var(--size-md); }
}
@container (width < 400px) {
  .days { grid-template-columns: repeat(3, minmax(0, 1fr)); }
}
</style>
