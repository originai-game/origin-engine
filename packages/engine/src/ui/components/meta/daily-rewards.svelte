<meta name="component" content="DailyRewards">
<meta name="strings" content="./general.en.ogt">
<meta name="parts" content="DailyRewards.frame DailyRewards.band DailyRewards.title DailyRewards.refresh DailyRewards.day DailyRewards.label DailyRewards.amount DailyRewards.state DailyRewards.cell DailyRewards.figure">
<meta name="prop:title" content="string" data-doc="The calendar's heading; empty takes the group's (Daily rewards).">
<meta name="prop:refresh" content="string" data-doc="When the next day's reward opens, as the game words it; empty hides it.">
<meta name="prop:days" content="list" data-doc="The calendar's days in order: keyed rows of { id, label, icon, amount, state, milestone }: label the day in words (Day 1); amount the reward in words (x100); state claimed, today, upcoming or missed, each shown by its own word: the game sends claimed for today once its reward is taken; milestone a big reward, lit violet with a ribbon.">
<meta name="prop:canClaim" content="bool" data-doc="Today's reward is waiting: the day whose state is today is a Claim button; otherwise no day is a button.">
<meta name="fixture:default" content='{"title":"","refresh":"Next reward in 9h 40m","days":[{"id":"d1","label":"Day 1","icon":"coin","amount":"x100","state":"claimed","milestone":false},{"id":"d2","label":"Day 2","icon":"potion","amount":"x3","state":"claimed","milestone":false},{"id":"d3","label":"Day 3","icon":"gem","amount":"x20","state":"missed","milestone":false},{"id":"d4","label":"Day 4","icon":"coin","amount":"x300","state":"today","milestone":false},{"id":"d5","label":"Day 5","icon":"key","amount":"x1","state":"upcoming","milestone":false},{"id":"d6","label":"Day 6","icon":"gem","amount":"x50","state":"upcoming","milestone":false},{"id":"d7","label":"Day 7","icon":"crown","amount":"Legend chest","state":"upcoming","milestone":true}],"canClaim":true}'>
<meta name="fixture:disabled" content='{"title":"","refresh":"Next reward in 23h 10m","days":[{"id":"d1","label":"Day 1","icon":"coin","amount":"x100","state":"claimed","milestone":false},{"id":"d2","label":"Day 2","icon":"potion","amount":"x3","state":"claimed","milestone":false},{"id":"d3","label":"Day 3","icon":"gem","amount":"x20","state":"claimed","milestone":true},{"id":"d4","label":"Day 4","icon":"coin","amount":"x300","state":"upcoming","milestone":false}],"canClaim":false}'>
<meta name="fixture:overflow" content='{"title":"Fourteen Days of Treasure from the Royal Vaults","refresh":"","days":[{"id":"d1","label":"Day 1","icon":"coin","amount":"x1,000,000","state":"claimed","milestone":false},{"id":"d2","label":"Day 2","icon":"potion","amount":"A very long reward name","state":"claimed","milestone":false},{"id":"d3","label":"Day 3","icon":"gem","amount":"x20","state":"claimed","milestone":false},{"id":"d4","label":"Day 4","icon":"coin","amount":"x300","state":"claimed","milestone":false},{"id":"d5","label":"Day 5","icon":"key","amount":"x1","state":"claimed","milestone":true},{"id":"d6","label":"Day 6","icon":"gem","amount":"x50","state":"missed","milestone":false},{"id":"d7","label":"Day 7","icon":"crown","amount":"x1","state":"today","milestone":false},{"id":"d8","label":"Day 8","icon":"coin","amount":"x200","state":"upcoming","milestone":false},{"id":"d9","label":"Day 9","icon":"apple","amount":"x5","state":"upcoming","milestone":false},{"id":"d10","label":"Day 10","icon":"trophy","amount":"x1","state":"upcoming","milestone":true},{"id":"d11","label":"Day 11","icon":"coin","amount":"x250","state":"upcoming","milestone":false},{"id":"d12","label":"Day 12","icon":"gem","amount":"x60","state":"upcoming","milestone":false},{"id":"d13","label":"Day 13","icon":"leaf","amount":"x9","state":"upcoming","milestone":false},{"id":"d14","label":"Day 14","icon":"crown","amount":"x1","state":"upcoming","milestone":true}],"canClaim":true}'>

<!--
  A login reward calendar: a grid of day tiles with two corners cut, each a
  header strip with its day, the reward's icon large in the middle, its
  amount, and a foot with its state in words and a mark. Today is lit gold
  (a gold header and foot, a glow round the tile); a day taken carries a
  check stamp over its dimmed art; a missed day is hatched with its word in
  danger; a milestone is violet with a ribbon. The game owns the schedule,
  the rewards and the clock; the calendar reports claim (today's key) while
  canClaim holds: only then is today's tile a button; every other day is a
  labelled tile that takes no focus and reports nothing. A day shows the
  word of the state the game gave it. Seven across on a desktop, four on a
  narrow panel, three on a phone, where a last day alone on its row takes
  the whole row.
-->
<section id="DailyRewards" part="DailyRewards.frame" data-layout="shell" aria-label={title != '' ? title : t("general.rewards.name")}>
  <header part="DailyRewards.band" data-layout="cluster" data-gap="loose">
    <h2 part="DailyRewards.title" data-fill>{title != '' ? title : t("general.rewards.name")}</h2>
    {#if refresh}<div part="DailyRewards.refresh DailyRewards.label" class="timer" data-layout="row" data-gap="tight" role="timer" aria-label={refresh}><og-icon class="mark" icon="clock"></og-icon><span class="refresh-text">{refresh}</span></div>{/if}
  </header>
  <div class="days-scroll" data-layout="scroll" data-fill>
    <ol class="days" data-layout="grid" data-gap="tight">
      {#each days as d (d.id)}
        <li class="slot" data-layout="row" data-gap="none">
          {#if d.state == 'today' && canClaim}
            <button part="DailyRewards.day DailyRewards.cell" class="day" data-state={d.state} data-milestone={d.milestone} aria-current="date" aria-label={t("general.rewards.day", { day: d.label, reward: d.amount, state: t("general.rewards.claim") })} onclick={claim}><div class="face" data-layout="stack" data-gap="none">
              <p part="DailyRewards.label" class="head">{d.label}</p>
              <div class="art" data-layout="cover" data-fill><og-icon class="reward" icon={d.icon} data-principal></og-icon></div>
              <p part="DailyRewards.amount DailyRewards.figure" class="amount">{d.amount}</p>
              <div part="DailyRewards.state DailyRewards.label" class="foot" data-layout="row" data-gap="hair" data-state={d.state} data-motion="pop">
                <og-icon class="mark" icon="star"></og-icon>
                <span class="state-text">{t("general.rewards.claim")}</span>
              </div>
            </div></button>
          {:else}
            <div part="DailyRewards.day DailyRewards.cell" class="day" data-layout="row" data-gap="none" role="group" data-state={d.state} data-milestone={d.milestone} aria-current={d.state == 'today' ? 'date' : 'false'} aria-label={t("general.rewards.day", { day: d.label, reward: d.amount, state: d.state == 'claimed' ? t("general.rewards.claimed") : d.state == 'missed' ? t("general.rewards.missed") : d.state == 'today' ? t("general.rewards.today") : t("general.rewards.upcoming") })}><div class="face" data-layout="stack" data-gap="none">
              <p part="DailyRewards.label" class="head">{d.label}</p>
              <div class="art" data-layout="overlay" data-fill>
                <div class="art-icon" data-layout="cover" data-anchor="fill"><og-icon class="reward" icon={d.icon} data-principal></og-icon></div>
                {#if d.state == 'claimed'}<div class="stamp" data-layout="cover" data-anchor="center" data-motion="stamp"><og-icon class="stamp-mark" icon="check" data-principal></og-icon></div>{/if}
              </div>
              <p part="DailyRewards.amount DailyRewards.figure" class="amount">{d.amount}</p>
              <div part="DailyRewards.state DailyRewards.label" class="foot" data-layout="row" data-gap="hair" data-state={d.state}>
                {#if d.state != 'upcoming'}<og-icon class="mark" icon={d.state == 'claimed' ? 'check' : d.state == 'missed' ? 'close' : 'star'}></og-icon>{/if}
                <span class="state-text">{d.state == 'claimed' ? t("general.rewards.claimed") : d.state == 'missed' ? t("general.rewards.missed") : d.state == 'today' ? t("general.rewards.today") : t("general.rewards.upcoming")}</span>
              </div>
            </div></div>
          {/if}
          {#if d.milestone}<span class="ribbon">{t("general.rewards.milestone")}</span>{/if}
        </li>
      {/each}
    </ol>
  </div>
</section>

<style>
[part="DailyRewards.band"] { width: stretch; flex: none; }
/* The title keeps its words whole and takes its own width first: the tag
   wraps under it when both do not fit. */
[part="DailyRewards.title"] { margin: 0; flex: 1 1 auto; }
/* The next reward's time on a dark cut tag. */
.timer { max-width: stretch; padding: var(--space-xxs) var(--space-sm); background: var(--quest-well-fill); box-shadow: var(--quest-well-rim); border-radius: var(--quest-cut-sm) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.refresh-text { text-align: end; min-inline-size: 0; overflow-wrap: anywhere; color: var(--ink-bright); }
.mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
.days { grid-template-columns: repeat(7, minmax(0, 1fr)); width: stretch; margin: 0; padding: var(--space-xxs) 0 var(--space-xs); list-style: none; }
.slot { position: relative; width: stretch; min-width: 0; align-items: stretch; }
/* A day: a tile with two corners cut, its header strip, art and foot. */
.day { width: stretch; min-width: 0; height: stretch; min-height: calc(var(--size-xxl) * 1.8); padding: 0; align-items: stretch; justify-content: flex-start; border: 0; background: var(--quest-day-fill); box-shadow: var(--quest-entry-rim); border-radius: var(--quest-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.day:hover { box-shadow: var(--quest-entry-rim); }
.day .face { width: stretch; height: stretch; align-items: stretch; }
.head { margin: 0; padding: var(--space-xxs) var(--space-xxs); text-align: center; overflow-wrap: anywhere; font: var(--caption-text); font-weight: var(--quest-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--quest-ink-quiet); background: var(--quest-day-head-fill); }
.art { width: stretch; min-height: var(--size-xl); background: var(--quest-day-art-fill); }
.art-icon { width: stretch; height: stretch; }
.reward { width: var(--size-lg); height: var(--size-lg); flex: none; color: var(--ink-bright); }
.amount { margin: 0; padding: 0 var(--space-xxs) var(--space-xs); text-align: center; overflow-wrap: anywhere; --figure-size: var(--quest-size-figure); --figure-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); color: var(--ink-bright); }
.foot { width: stretch; justify-content: center; padding: var(--space-xxs); font: var(--caption-text); font-weight: var(--quest-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--quest-ink-faint); }
.state-text { text-align: center; min-inline-size: 0; overflow-wrap: anywhere; }
/* Taken: the art and amount dim under a check stamp; the foot in the success colour. */
.day[data-state=claimed] .art-icon, .day[data-state=claimed] .amount { opacity: var(--quest-day-dim-opacity); }
.day[data-state=claimed] .foot { color: var(--tone-success); background: var(--quest-day-taken-foot-fill); }
.stamp { width: var(--size-xl); height: var(--size-xl); border-radius: var(--quest-stamp-radius); background: var(--quest-stamp-fill); color: var(--quest-stamp-ink); box-shadow: var(--quest-stamp-shadow); rotate: -8deg; }
.stamp-mark { width: var(--size-sm); height: var(--size-sm); }
/* Missed: hatched, its word in danger. */
.day[data-state=missed] { background: var(--quest-day-missed-fill); }
.day[data-state=missed] .art-icon, .day[data-state=missed] .amount { opacity: var(--quest-day-dim-opacity); }
.day[data-state=missed] .foot { color: var(--tone-danger); background: var(--quest-day-missed-foot-fill); }
/* A milestone: violet, with a ribbon. */
.day[data-milestone=true] { background: var(--quest-day-milestone-fill); box-shadow: var(--quest-day-milestone-rim); }
.day[data-milestone=true] .head { color: var(--quest-day-milestone-ink); background: var(--quest-day-milestone-head-fill); }
.day[data-milestone=true] .reward { color: var(--quest-day-milestone-ink); filter: var(--quest-day-milestone-glow); }
.ribbon { position: absolute; inset-inline-end: 0; top: calc(var(--size-md) + var(--space-xxs)); padding: 0 var(--space-xs) 0 var(--space-sm); font: var(--caption-text); font-weight: var(--quest-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; color: var(--quest-ribbon-ink); background: var(--quest-ribbon-fill); clip-path: polygon(var(--quest-cut-sm) 0, 100% 0, 100% 100%, var(--quest-cut-sm) 100%, 0 50%); pointer-events: none; }
/* Today: lit gold, a gold header and foot. */
.day[data-state=today] { background: var(--quest-day-today-fill); box-shadow: var(--quest-day-today-rim); }
.day[data-state=today] .head { color: var(--quest-gold-ink); background: var(--quest-gold); }
.day[data-state=today] .reward { color: var(--quest-gold); filter: var(--quest-socket-glow); }
.day[data-state=today] .foot { color: var(--quest-gold-ink); background: var(--quest-claim-fill); }
button.day[data-state=today]:hover { box-shadow: var(--quest-day-today-hover-rim); }
button.day[data-state=today]:active { background: var(--quest-day-today-fill); }
@container (width < 760px) {
  .days { grid-template-columns: repeat(4, minmax(0, 1fr)); }
  .day { min-height: calc(var(--size-xxl) * 1.6); }
  .reward { width: var(--size-md); height: var(--size-md); }
  .stamp { width: var(--size-lg); height: var(--size-lg); }
  .stamp-mark { width: var(--size-xs); height: var(--size-xs); }
}
@container (width < 400px) {
  .days { grid-template-columns: repeat(3, minmax(0, 1fr)); }
  /* A last day alone on its row takes the whole row. */
  .slot:nth-child(3n+1):last-child { grid-column: 1 / -1; }
  /* A narrow tile keeps its state word whole: the mark above it, no tracking. */
  .foot { flex-direction: column; letter-spacing: normal; }
  .state-text { white-space: nowrap; }
}
/* Too narrow for the word: the mark alone (the day's name carries the state). */
@container (width < 300px) {
  .state-text { display: none; }
}
</style>
