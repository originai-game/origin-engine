<meta name="component" content="RpgQuestDetail">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgQuestDetail.frame RpgQuestDetail.band RpgQuestDetail.title RpgQuestDetail.row RpgQuestDetail.reward RpgQuestDetail.figure">
<meta name="prop:name" content="string" data-doc="The quest's name, in the heading band.">
<meta name="prop:summary" content="string" data-doc="Its story so far, as the game writes it.">
<meta name="prop:objectives" content="list" data-doc="Keyed rows of { id, text, done, optional, step }: step 0 is a stage, 1 a task under it; done ticks it, optional marks it so.">
<meta name="prop:rewards" content="list" data-doc="Keyed rows of { id, icon, label, detail }: detail a second, quieter line (Unique relic); empty hides it.">
<meta name="prop:scenery" content="number" data-doc="A texture shown faintly behind the rewards (og.load.texture); 0 is none.">
<meta name="fixture:default" content='{"name":"The Hollow Crown","summary":"The crown is said to lie within the ruins.","objectives":[{"id":"o1","text":"Enter the ruins","done":false,"optional":false,"step":0},{"id":"o2","text":"Find the crown","done":true,"optional":false,"step":1}],"rewards":[{"id":"g","icon":"coin","label":"1,200 Gold","detail":""}],"scenery":0}'>

<!--
  One quest: its name on the heading band, the story so far, its
  objectives as stages with tasks under them, and its rewards. Read only;
  the log beside it owns choosing and tracking. Fills its height; the
  body scrolls inside it.
-->
<article id="RpgQuestDetail" part="RpgQuestDetail.frame" aria-label={name}>
  <header part="RpgQuestDetail.band"><div class="band-mark"></div><h2 part="RpgQuestDetail.title">{name}</h2></header>
  <div class="body">
    <p class="summary">{summary}</p>
    <div class="rpg-rule" role="separator"></div>
    {#if objectives.length}<section class="part" aria-label={t("rpg.quest.objectives")}>
      <h3 class="part-head">{t("rpg.quest.objectives")}</h3>
      <ul class="objectives">
        {#each objectives as o (o.id)}
          <li part="RpgQuestDetail.row" data-step={o.step} data-done={o.done} data-optional={o.optional}>
            <div class="objective-mark"></div>
            <span class="objective-text">{o.text}</span>
            {#if o.optional}<span class="objective-optional">{t("rpg.quest.optional")}</span>{/if}
          </li>
        {/each}
      </ul>
    </section>{/if}
    {#if rewards.length}<div class="rpg-rule" role="separator"></div>
    <section class="part rewards-part" aria-label={t("rpg.quest.rewards")}>
      {#if scenery}<img class="scenery" src={scenery} alt="">{/if}
      <h3 class="part-head">{t("rpg.quest.rewards")}</h3>
      <ul class="rewards">
        {#each rewards as r (r.id)}
          <li part="RpgQuestDetail.reward RpgQuestDetail.figure"><og-icon class="reward-mark" icon={r.icon}></og-icon><div class="reward-text"><span class="reward-label">{r.label}</span>{#if r.detail}<span class="reward-detail">{r.detail}</span>{/if}</div></li>
        {/each}
      </ul>
    </section>{/if}
  </div>
</article>

<style>
:host { display: flex; flex-direction: column; inline-size: stretch; block-size: stretch; gap: var(--space-md); }
[part="RpgQuestDetail.band"] { display: flex; width: stretch; align-items: center; gap: var(--space-sm); flex: none; }
.band-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
.body { display: flex; flex-direction: column; flex: 1; min-block-size: 0; overflow-y: auto; gap: var(--space-sm); padding: 0 var(--space-sm); }
.summary { margin: 0; }
.part { display: flex; flex-direction: column; position: relative; flex: none; gap: var(--space-sm); }
.part-head { margin: 0; }
.objectives, .rewards { display: flex; flex-direction: column; margin: 0; padding: 0; list-style: none; width: stretch; gap: var(--space-xxs); }
[part="RpgQuestDetail.row"] { display: flex; align-items: center; gap: var(--space-sm); padding: var(--space-xxs) 0; }
[part="RpgQuestDetail.row"][data-step="1"] { padding-inline-start: var(--space-xl); }
.objective-mark { width: var(--space-sm); height: var(--space-sm); flex: none; border: var(--stroke-regular) solid var(--line-strong); border-radius: var(--radius-capsule); }
[part="RpgQuestDetail.row"][data-done=true] .objective-mark { background: var(--success); border-color: var(--success); }
.objective-optional, .reward-detail { color: var(--ink-muted); font: var(--text-caption); }
.reward-label { font-weight: var(--font-weight-semibold); }
.rewards-part { padding: var(--space-md); background: var(--well); border-radius: var(--radius-panel); }
.objective-text { flex: 1; min-width: 0; }
[part="RpgQuestDetail.reward RpgQuestDetail.figure"] { display: flex; align-items: center; gap: var(--space-md); padding: var(--space-xxs) 0; }
.reward-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
.reward-text { display: flex; flex-direction: column; gap: 0; }
.scenery { position: absolute; right: 0; top: 0; bottom: 0; width: 55%; height: 100%; object-fit: cover; object-position: right bottom; opacity: var(--state-disabled-opacity); }
</style>
