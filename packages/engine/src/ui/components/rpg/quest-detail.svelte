<meta name="component" content="RpgQuestDetail">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgQuestDetail.frame RpgQuestDetail.band RpgQuestDetail.title RpgQuestDetail.row RpgQuestDetail.reward RpgQuestDetail.figure RpgQuestDetail.scenery RpgQuestDetail.headline RpgQuestDetail.caption RpgQuestDetail.dot RpgQuestDetail.well RpgQuestDetail.picture">
<meta name="prop:name" content="string" data-doc="The quest's name, in the heading band.">
<meta name="prop:summary" content="string" data-doc="Its story so far, as the game writes it.">
<meta name="prop:objectives" content="list" data-doc="Keyed rows of { id, text, done, optional, step }: step 0 is a stage, 1 a task under it; done ticks it, optional marks it so.">
<meta name="prop:rewards" content="list" data-doc="Keyed rows of { id, icon, label, detail }: detail a second, quieter line (Unique relic); empty hides it.">
<meta name="prop:scenery" content="number" data-doc="A texture beside the rewards (og.load.texture), with its own space; 0 is none.">
<meta name="fixture:default" content='{"name":"The Hollow Crown","summary":"The crown is said to lie within the ruins.","objectives":[{"id":"o1","text":"Enter the ruins","done":false,"optional":false,"step":0},{"id":"o2","text":"Find the crown","done":true,"optional":false,"step":1}],"rewards":[{"id":"g","icon":"coin","label":"1,200 Gold","detail":""}],"scenery":0}'>

<!--
  One quest: its name on the heading band, the story so far, its
  objectives as stages with tasks under them, and its rewards. Read only;
  the log beside it owns choosing and tracking. Fills its height; the
  body scrolls inside it.
-->
<article id="RpgQuestDetail" part="RpgQuestDetail.frame" data-layout="shell" data-gap="loose" aria-label={name}>
  <header part="RpgQuestDetail.band" data-layout="row"><div class="band-mark"></div><h2 part="RpgQuestDetail.title">{name}</h2></header>
  <div class="body" data-layout="scroll" data-gap="tight">
    <p class="summary">{summary}</p>
    <div class="rpg-rule" role="separator"></div>
    {#if objectives.length}<section class="part" data-layout="stack" aria-label={t("rpg.quest.objectives")}>
      <h3 class="part-head" part="RpgQuestDetail.headline">{t("rpg.quest.objectives")}</h3>
      <ul class="objectives" data-layout="stack" data-gap="hair">
        {#each objectives as o (o.id)}
          <li part="RpgQuestDetail.row" data-layout="row" data-step={o.step} data-done={o.done} data-optional={o.optional}>
            <div class="objective-mark" part="RpgQuestDetail.dot" data-tone={o.done ? 'success' : null}>{#if o.done}<og-icon icon="check"></og-icon>{/if}</div>
            <span class="objective-text" data-fill>{o.text}</span>
            {#if o.optional}<span class="objective-optional" part="RpgQuestDetail.caption">{t("rpg.quest.optional")}</span>{/if}
          </li>
        {/each}
      </ul>
    </section>{/if}
    {#if rewards.length}<div class="rpg-rule" role="separator"></div>
    <section class="part rewards-part" part="RpgQuestDetail.well" data-layout="stack" aria-label={t("rpg.quest.rewards")}>
      <h3 class="part-head" part="RpgQuestDetail.headline">{t("rpg.quest.rewards")}</h3>
      <div class="reward-body" data-layout="row">
      <ul class="rewards" data-layout="stack" data-gap="hair" data-fill>
        {#each rewards as r (r.id)}
          <li part="RpgQuestDetail.reward RpgQuestDetail.figure" data-layout="row" data-gap="loose"><og-icon class="reward-mark" icon={r.icon}></og-icon><div class="reward-text" data-layout="stack" data-gap="none" data-fill><span class="reward-label" part="RpgQuestDetail.headline">{r.label}</span>{#if r.detail}<span class="reward-detail" part="RpgQuestDetail.caption">{r.detail}</span>{/if}</div></li>
        {/each}
      </ul>
      {#if scenery}<img part="RpgQuestDetail.scenery RpgQuestDetail.picture" class="scenery" data-muted="true" src={scenery} alt="">{/if}
      </div>
    </section>{/if}
  </div>
</article>

<style>
[part="RpgQuestDetail.band"] { width: stretch; flex: none; }
.band-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
.body { padding: 0 var(--space-sm); }
.summary { margin: 0; }
.part { position: relative; flex: none; }
.part-head { margin: 0; }
.objectives, .rewards { margin: 0; padding: 0; list-style: none; width: stretch; }
[part="RpgQuestDetail.row"] { padding: var(--space-xxs) 0; }
[part="RpgQuestDetail.row"][data-step="1"] { padding-inline-start: var(--space-xl); }
.objective-mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
.objective-mark og-icon { width: 100%; height: 100%; }
.rewards-part { padding: var(--space-sm); }
[part="RpgQuestDetail.reward RpgQuestDetail.figure"] { padding: var(--space-xxs) 0; }
.reward-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
.reward-body { align-items: flex-start; }
.reward-text { overflow-wrap: anywhere; }
.scenery { inline-size: 28%; block-size: var(--size-xxl); flex: none; object-fit: cover; object-position: right bottom; }
</style>
