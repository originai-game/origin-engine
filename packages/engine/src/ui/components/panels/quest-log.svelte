<meta name="component" content="RpgQuestLog">
<meta name="strings" content="./quest-log.en.ogt">
<meta name="parts" content="RpgQuestLog.frame RpgQuestLog.band RpgQuestLog.title RpgQuestLog.heading RpgQuestLog.row RpgQuestLog.dot RpgQuestLog.caption RpgQuestLog.fresh">
<meta name="prop:title" content="string" data-doc="The log's heading (Quest Journal).">
<meta name="prop:groups" content="list" data-doc="The groups in order, keyed rows of { id, label, done }: done marks the group of finished quests, shown struck and quiet.">
<meta name="prop:quests" content="list" data-doc="The quests, keyed rows of { id, group, name, tracked, fresh }: group a group's id; tracked the one the HUD follows; fresh an update the player has not read.">
<meta name="prop:selected" content="string" data-doc="The quest shown in detail; empty is none.">
<meta name="action:details" content="Track">
<meta name="fixture:default" content='{"title":"Quest Journal","groups":[{"id":"main","label":"Main Quests","done":false},{"id":"done","label":"Completed","done":true}],"quests":[{"id":"crown","group":"main","name":"The Hollow Crown","tracked":true,"fresh":false},{"id":"heir","group":"done","name":"The Lost Heir","tracked":false,"fresh":false}],"selected":"crown"}'>
<meta name="fixture:empty" content='{"title":"Quest Journal","groups":[],"quests":[],"selected":""}'>
<meta name="fixture:overflow" content='{"title":"Quest Journal of the Long Campaign","groups":[{"id":"main","label":"Main Quests of the Second Act","done":false},{"id":"side","label":"Side Quests","done":false},{"id":"done","label":"Completed","done":true}],"quests":[{"id":"crown","group":"main","name":"The Hollow Crown","tracked":true,"fresh":false},{"id":"gate","group":"main","name":"Ashes of the Ninefold Gate beneath the Drowned Abbey","tracked":false,"fresh":true},{"id":"bell","group":"side","name":"The Sunken Bell","tracked":false,"fresh":false},{"id":"lantern","group":"side","name":"A Lantern for the Ferryman’s Daughter","tracked":false,"fresh":true},{"id":"heir","group":"done","name":"The Lost Heir","tracked":false,"fresh":false},{"id":"mill","group":"done","name":"Wolves at the Mill","tracked":false,"fresh":false}],"selected":"crown"}'>

<!--
  The quest list, grouped as the game groups it (main, side, finished):
  each group a gold label in small capitals behind a short gold rule (a
  finished group's quiet), each quest a row with its mark. The mark is a
  diamond: hollow, lit gold on the quest the HUD tracks, filled in the
  success colour once the quest is done. The selected row (the one in
  detail) is washed gold from its leading edge with a gold bar on it and a
  chevron toward the detail; a fresh update carries a New tag. Finished
  quests are struck and quiet. Reports choose (a quest's key) and track (X
  or pad Y on the focused quest, its key). It fills its height and scrolls
  inside it.
-->
<section id="RpgQuestLog" part="RpgQuestLog.frame" data-layout="shell" aria-label={title}>
  <header part="RpgQuestLog.band" data-layout="row"><h2 part="RpgQuestLog.title" data-fill>{title}</h2></header>
  {#if quests.length}
    <div class="groups" data-layout="scroll" data-gap="loose" data-fill>
      {#each groups as g (g.id)}
        <section class="group" data-layout="stack" data-gap="hair" data-done={g.done} aria-label={g.label}>
          <div part="RpgQuestLog.heading" class="heading" data-layout="row" data-gap="tight" data-done={g.done} role="heading"><div class="group-mark" data-done={g.done}></div><span part="RpgQuestLog.caption" class="group-label">{g.label}</span></div>
          <div class="rows" data-layout="stack" data-gap="none" role="listbox" aria-label={g.label}>
            {#each quests as q (q.id)}
              {#if q.group == g.id}
                <button part="RpgQuestLog.row" class="quest" role="option" data-done={g.done} data-tracked={q.tracked} data-fresh={q.fresh} aria-selected={q.id == selected} aria-label={q.name} onclick={choose} ondetails={track}><div class="face" data-layout="row" data-gap="loose">
                  <div part="RpgQuestLog.dot" class="quest-mark" data-tracked={q.tracked} data-done={g.done}></div>
                  <span class="quest-name" data-fill>{q.name}</span>
                  {#if q.fresh && !g.done}<span part="RpgQuestLog.fresh" class="fresh" data-motion="pop">{t("rpg.quests.fresh")}</span>{/if}
                  {#if q.id == selected}<og-icon class="quest-open" icon="chevronRight"></og-icon>{/if}
                </div></button>
              {/if}
            {/each}
          </div>
        </section>
      {/each}
    </div>
  {:else}
    <div class="empty" data-layout="cover" data-fill role="status"><og-icon class="empty-mark" icon="flag"></og-icon><p class="empty-text">{t("rpg.quests.empty")}</p></div>
  {/if}
</section>

<style>
:host { inline-size: stretch; block-size: stretch; }
[part="RpgQuestLog.band"] { width: stretch; flex: none; }
[part="RpgQuestLog.title"] { margin: 0; min-inline-size: 0; }
.groups { padding-block: var(--space-xs); }
.group { flex: none; width: stretch; }
/* A group's label: gold small capitals behind a short gold rule; a finished group's quiet. */
.heading { margin: 0; padding: var(--space-xs) var(--space-sm) var(--space-xxs); }
.group-mark { flex: none; width: var(--space-sm); height: var(--stroke-thick); background: var(--quest-gold); }
.group-mark[data-done=true] { background: var(--ink-done); }
.group-label { min-inline-size: 0; font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--quest-gold); white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.heading[data-done=true] .group-label { color: var(--ink-done); }
.rows { width: stretch; }
.quest { width: stretch; min-width: 0; min-height: var(--size-touch); padding: 0 var(--space-sm) 0 var(--space-md); justify-content: flex-start; }
.quest[aria-selected=true] { background: var(--quest-selected-fill); box-shadow: var(--quest-selected-rim); }
.quest .face { width: stretch; min-width: 0; }
/* The mark: a hollow diamond, lit gold when tracked, filled once done. */
.quest-mark { flex: none; width: var(--space-sm); height: var(--space-sm); margin-inline: var(--space-xxs); rotate: 45deg; border-radius: 0; --dot-fill: transparent; box-shadow: var(--quest-mark-rim); }
.quest-mark[data-tracked=true] { --dot-fill: var(--quest-gold); box-shadow: var(--quest-mark-lit); }
.quest-mark[data-done=true] { --dot-fill: var(--quest-mark-done); box-shadow: none; }
.quest-name { min-inline-size: 0; text-align: start; font-weight: var(--headline-weight); white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
/* The chosen quest's name scrolls when it does not fit; the others end in an ellipsis. */
.quest[aria-selected=true] .quest-name { text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); }
.quest[data-done=true] .quest-name { color: var(--ink-done); text-decoration-line: var(--hud-done-line); }
/* A fresh update: a gold tag with two corners cut. */
.fresh { flex: none; padding: 0 var(--space-xs); font: var(--caption-text); font-weight: var(--hud-figure-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); white-space: nowrap; color: var(--quest-gold-ink); background: var(--quest-gold); border-radius: var(--quest-cut-sm) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.quest-open { flex: none; width: var(--size-xs); height: var(--size-xs); color: var(--quest-gold); }
.quest-open:dir(rtl) { scale: -1 1; }
@media (pointer: fine) and (width >= 600px) { .quest { min-height: calc(var(--size-touch) * 0.82); } }
.empty { width: stretch; }
.empty-mark { width: var(--size-lg); height: var(--size-lg); }
.empty-text { margin: 0; }
</style>
