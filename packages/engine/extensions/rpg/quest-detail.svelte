<meta name="component" content="RpgQuestDetail">
<meta name="strings" content="./rpg.en.ogt">
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
<article id="RpgQuestDetail" class="rpg-quest-detail" aria-label={name}>
  <header class="rpg-band"><div class="band-mark"></div><h2 class="rpg-title">{name}</h2></header>
  <scroll class="body">
    <p class="summary">{summary}</p>
    <div class="rpg-rule" role="separator"></div>
    <section class="part" aria-label={t("rpg.quest.objectives")} hidden={!objectives.length}>
      <h3 class="part-head">{t("rpg.quest.objectives")}</h3>
      <ul class="objectives">
        {#each objectives as o (o.id)}
          <li class="objective" data-step={o.step} data-done={o.done} data-optional={o.optional}>
            <div class="objective-mark"></div>
            <span class="objective-text">{o.text}</span>
            <span class="objective-optional" hidden={!o.optional}>{t("rpg.quest.optional")}</span>
          </li>
        {/each}
      </ul>
    </section>
    <div class="rpg-rule" role="separator" hidden={!rewards.length}></div>
    <section class="part rewards-part" aria-label={t("rpg.quest.rewards")} hidden={!rewards.length}>
      <img class="scenery" src={scenery} alt="" hidden={!scenery}>
      <h3 class="part-head">{t("rpg.quest.rewards")}</h3>
      <ul class="rewards">
        {#each rewards as r (r.id)}
          <li class="reward"><icon class="reward-mark" icon={r.icon}></icon><div class="reward-text"><span class="reward-label">{r.label}</span><span class="reward-detail" hidden={!r.detail}>{r.detail}</span></div></li>
        {/each}
      </ul>
    </section>
  </scroll>
</article>

<style>
.rpg-quest-detail { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-sm); }
.rpg-band { display: flex; width: stretch; align-items: center; gap: var(--space-sm); flex: none; }
.band-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
.body { width: stretch; flex: 1; min-height: 0; gap: var(--space-sm); padding: 0 var(--space-sm); }
.summary { margin: 0; }
.part { display: flex; flex-direction: column; position: relative; width: stretch; gap: var(--space-xs); }
.part-head { margin: 0; }
.objectives, .rewards { display: flex; flex-direction: column; margin: 0; padding: 0; list-style: none; width: stretch; gap: var(--space-xxs); }
.objective { display: flex; align-items: center; gap: var(--space-sm); padding: var(--space-xxs) 0; }
.objective[data-step="1"] { padding-inline-start: var(--space-xl); }
.objective-mark { width: var(--space-lg); height: var(--space-lg); flex: none; }
.objective-text { width: stretch; }
.reward { display: flex; align-items: center; gap: var(--space-md); padding: var(--space-xxs) 0; }
.reward-mark { width: var(--size-sm); height: var(--size-sm); flex: none; }
.reward-text { display: flex; flex-direction: column; gap: 0; }
.scenery { position: absolute; right: 0; bottom: 0; width: 55%; height: stretch; object-fit: cover; object-position: right bottom; }
</style>
