<meta name="component" content="RpgQuestLog">
<meta name="strings" content="./quest-log.en.ogt">
<meta name="parts" content="RpgQuestLog.frame RpgQuestLog.band RpgQuestLog.title RpgQuestLog.heading RpgQuestLog.row RpgQuestLog.dot RpgQuestLog.caption">
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
<section id="RpgQuestLog" part="RpgQuestLog.frame" data-layout="stack" aria-label={title}>
  <header part="RpgQuestLog.band" data-layout="row"><h2 part="RpgQuestLog.title">{title}</h2></header>
  {#if quests.length}
    <div class="groups" data-layout="scroll" data-gap="wide" data-fill>
      {#each groups as g (g.id)}
        <section class="group" data-layout="stack" data-gap="tight" data-done={g.done} aria-label={g.label}>
          <div part="RpgQuestLog.heading" data-layout="row" role="heading"><div part="RpgQuestLog.dot" class="group-mark"></div><span part="RpgQuestLog.caption" class="group-label">{g.label}</span></div>
          <div class="rows" data-layout="stack" data-gap="none" role="listbox" aria-label={g.label}>
            {#each quests as q (q.id)}
              {#if q.group == g.id}
                <button part="RpgQuestLog.row" role="option" data-done={g.done} data-tracked={q.tracked} data-fresh={q.fresh} aria-selected={q.id == selected} aria-label={q.name} onclick={choose} ondetails={track}><div class="face" data-layout="row">
                  <div part="RpgQuestLog.dot" class="quest-mark" data-tracked={q.tracked}></div>
                  <span class="quest-name" data-fill>{q.name}</span>
                  <div class="quest-open"></div>
                </div></button>
              {/if}
            {/each}
          </div>
        </section>
      {/each}
    </div>
  {:else}
    <div class="empty" data-layout="cover" data-fill role="status"><p class="empty-text">{t("rpg.quests.empty")}</p></div>
  {/if}
</section>

<style>
:host { inline-size: stretch; block-size: stretch; }
[part="RpgQuestLog.band"] { width: stretch; flex: none; }
.group { flex: none; }
[part="RpgQuestLog.heading"] { margin: 0; padding: var(--space-xs) var(--space-sm); }
.group-mark { --dot-fill: var(--primary); width: var(--space-xs); height: var(--space-md); flex: none; }
.rows { width: stretch; }
[part="RpgQuestLog.row"] { width: stretch; min-height: var(--size-touch); padding: var(--space-xs) var(--space-sm) var(--space-xs) var(--space-md); justify-content: flex-start; min-width: 0; }
[part="RpgQuestLog.row"] .face { width: stretch; }
.quest-mark { --dot-fill: transparent; --dot-rim-width: var(--stroke-regular); --dot-rim: var(--line-strong); width: var(--space-sm); height: var(--space-sm); flex: none; }
.quest-mark[data-tracked=true] { --dot-fill: var(--primary); --dot-rim: var(--primary); }
.quest-name { text-align: start; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.quest-open { display: none; width: var(--space-md); height: var(--space-md); flex: none; }
[part="RpgQuestLog.row"][aria-selected=true] .quest-open { display: block; }
.quest-open:dir(rtl) { transform: scaleX(-1); }
@media (pointer: fine) and (width >= 600px) { [part="RpgQuestLog.row"] { min-height: calc(var(--size-touch) * 0.78); } }
.empty { width: stretch; }
.empty-text { margin: 0; }
</style>
