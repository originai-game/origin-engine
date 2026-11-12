<meta name="component" content="RpgQuestLog">
<meta name="strings" content="./rpg.en.ogt">
<meta name="prop:title" content="string" data-doc="The log's heading (Quest Journal).">
<meta name="prop:groups" content="list" data-doc="The groups in order, keyed rows of { id, label, done }: done marks the group of finished quests, shown struck and quiet.">
<meta name="prop:quests" content="list" data-doc="The quests, keyed rows of { id, group, name, tracked, fresh }: group a group's id; tracked the one the HUD follows; fresh an update the player has not read.">
<meta name="prop:selected" content="string" data-doc="The quest shown in detail; empty is none.">
<meta name="action:details" content="Track">
<meta name="fixture:default" content='{"title":"Quest Journal","groups":[{"id":"main","label":"Main Quests","done":false},{"id":"done","label":"Completed","done":true}],"quests":[{"id":"crown","group":"main","name":"The Hollow Crown","tracked":true,"fresh":false},{"id":"heir","group":"done","name":"The Lost Heir","tracked":false,"fresh":false}],"selected":"crown"}'>
<meta name="fixture:empty" content='{"title":"Quest Journal","groups":[],"quests":[],"selected":""}'>

<!--
  The quest list, grouped as the game groups it (main, side, finished):
  each group a small-caps heading with a marker, each quest a row. The
  selected row is the one in detail; the tracked one carries the filled
  marker. Reports choose (a quest's key) and track (X or pad Y on the
  focused quest, its key). It fills its height and scrolls inside it.
-->
<section id="RpgQuestLog" class="rpg-quest-log" aria-label={title}>
  <header class="rpg-band"><h2 class="rpg-title">{title}</h2></header>
  {#if quests.length}
    <scroll class="groups">
      {#each groups as g (g.id)}
        <section class="group" data-done={g.done} aria-label={g.label}>
          <div class="group-head" role="heading"><div class="group-mark"></div><span class="group-label">{g.label}</span></div>
          <div class="rows" role="listbox" aria-label={g.label}>
            {#each quests as q (q.id)}
              {#if q.group == g.id}
                <button class="quest" role="option" data-done={g.done} data-tracked={q.tracked} data-fresh={q.fresh} aria-selected={q.id == selected} aria-label={q.name} onclick={choose} ondetails={track}><div class="face">
                  <div class="quest-mark"></div>
                  <span class="quest-name">{q.name}</span>
                  <div class="quest-open"></div>
                </div></button>
              {/if}
            {/each}
          </div>
        </section>
      {/each}
    </scroll>
  {:else}
    <div class="empty" role="status"><p class="empty-text">{t("rpg.quests.empty")}</p></div>
  {/if}
</section>

<style>
.rpg-quest-log { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-xs); }
.rpg-band { display: flex; width: stretch; align-items: center; flex: none; }
.groups { width: stretch; flex: 1; min-height: 0; gap: var(--space-sm); }
.group { display: flex; flex-direction: column; width: stretch; gap: var(--space-xxs); }
.group-head { display: flex; margin: 0; align-items: center; gap: var(--space-sm); padding: var(--space-xs) var(--space-sm); }
.group-mark { width: var(--space-md); height: var(--space-md); flex: none; }
.rows { display: flex; flex-direction: column; width: stretch; }
.quest { width: stretch; min-height: var(--size-touch); align-items: center; gap: var(--space-sm); padding: var(--space-xs) var(--space-sm) var(--space-xs) var(--space-md); justify-content: flex-start; }
.quest .face { display: flex; width: stretch; align-items: center; gap: var(--space-sm); }
.quest-mark { width: var(--space-lg); height: var(--space-lg); flex: none; }
.quest-name { width: stretch; text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.quest-open { display: none; width: var(--space-md); height: var(--space-md); flex: none; }
.quest[aria-selected=true] .quest-open { display: block; }
.quest-open:dir(rtl) { transform: scaleX(-1); }
@media (pointer: fine) and (width >= 600px) { .quest { min-height: calc(var(--size-touch) * 0.78); } }
.empty { display: flex; flex-direction: column; width: stretch; flex: 1; align-items: center; justify-content: center; }
.empty-text { margin: 0; }
</style>
