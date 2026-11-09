<meta name="component" content="RpgJournal">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="Journal.quest Journal.objective Journal.category Journal.frame Journal.entry">
<meta name="prop:title" content="string" data-doc="The journal's heading (Journal, Quests).">
<meta name="prop:filters" content="list" data-doc="The quest groups as tabs, a keyed collection of { id, label, count } (Active, Completed, Failed); count is the game's.">
<meta name="prop:filter" content="string" data-doc="The id of the group shown.">
<meta name="prop:quests" content="list" data-doc="The quests shown, in the game's order: each { id, name, region, level, kind, tracked, fresh, state }. kind is main, side or task; state active, done or failed; level the game's recommended level as words (Lv 8), empty for none; fresh marks an unread update.">
<meta name="prop:selected" content="string" data-doc="The id of the quest whose entry is open; empty is none. Selection follows the id through reorders and removals.">
<meta name="prop:detailName" content="string" data-doc="The open quest's name; empty hides the entry.">
<meta name="prop:detailMeta" content="string" data-doc="Its line under the name (Main quest · Saltmarsh · Lv 8).">
<meta name="prop:detailText" content="string" data-doc="Its journal text, newest entry as the game writes it.">
<meta name="prop:objectives" content="list" data-doc="Its objectives in the game's order: each { id, text, progress, done, failed, optional }; progress is the game's count as words (2 / 5), empty for none.">
<meta name="prop:rewards" content="list" data-doc="Its rewards: each { id, icon, label } (850 XP, 120 gold, an item's name).">
<meta name="prop:tracked" content="bool" data-doc="The open quest is the one tracked.">
<meta name="prop:canTrack" content="bool" data-doc="Track is offered (an active quest); false greys it.">
<meta name="prop:canLocate" content="bool" data-doc="Show on map is offered: the quest has a place.">
<meta name="action:details" content="Track">
<meta name="action:tabPrev" content="Previous group">
<meta name="action:tabNext" content="Next group">
<meta name="fixture:default" content='{"title":"Journal","filter":"active","filters":[{"id":"active","label":"Active","count":3},{"id":"done","label":"Completed","count":12},{"id":"failed","label":"Failed","count":1}],"quests":[{"id":"ferry","name":"The Drowned Ferryman","region":"Saltmarsh","level":"Lv 8","kind":"main","tracked":true,"fresh":false,"state":"active"},{"id":"bells","name":"Bells Beneath the Abbey","region":"Greyfen","level":"Lv 6","kind":"side","tracked":false,"fresh":true,"state":"active"},{"id":"pelts","name":"Wolf Pelts for Marta","region":"Hollin","level":"","kind":"task","tracked":false,"fresh":false,"state":"active"}],"selected":"ferry","detailName":"The Drowned Ferryman","detailMeta":"Main quest · Saltmarsh · Lv 8","detailText":"The ferryman’s lantern still burns over black water. Old Ysolde says he took a debt to the bottom with him.","objectives":[{"id":"o1","text":"Speak with Old Ysolde","progress":"","done":true,"failed":false,"optional":false},{"id":"o2","text":"Light the marsh beacons","progress":"2 / 3","done":false,"failed":false,"optional":false},{"id":"o3","text":"Find the ferryman’s ledger","progress":"","done":false,"failed":false,"optional":true}],"rewards":[{"id":"xp","icon":"star","label":"850 XP"},{"id":"gold","icon":"coin","label":"120 gold"}],"tracked":true,"canTrack":true,"canLocate":true}'>
<meta name="fixture:empty" content='{"title":"Journal","filter":"active","filters":[{"id":"active","label":"Active","count":0}],"quests":[],"selected":"","detailName":"","detailMeta":"","detailText":"","objectives":[],"rewards":[],"tracked":false,"canTrack":false,"canLocate":false}'>

<!--
  The rpg kit's journal (README.md): quest groups as tabs, the quest list
  with its kind, region, level and tracked mark, and the open quest's
  entry with objectives, rewards and actions. The game owns every rule:
  what a group holds, the order, progress, and which quest can be
  tracked; the journal reports choose (the quest's key), category, track
  and locate. The list sits beside the entry on a wide screen and over it
  on a narrow one.
-->
<section id="RpgJournal" part="Journal.frame" aria-label={title}>
  <header id="Head">
    <h2 id="Title">{title}</h2>
  </header>
  <div id="Groups" role="tablist" aria-label={t("rpg.journal.groups")} ontabNext={categoryNext} ontabPrev={categoryPrev}>
    <kbd id="PrevKey" data-action="tabPrev"></kbd>
    {#each filters as f (f.id)}
      <button id="Group" part="Journal.category" class="group" variant="quiet" role="tab" aria-selected={f.id == filter} onclick={category}>
        <span class="label">{f.label}</span><span class="count">{formatNumber(f.count)}</span>
      </button>
    {/each}
    <kbd id="NextKey" data-action="tabNext"></kbd>
  </div>
  <div id="Body">
    <div id="List">
      {#if !quests.length}
        <div id="Empty" role="status"><icon class="mark" icon="flag"></icon><p class="quiet">{t("rpg.journal.empty")}</p></div>
      {:else}
        <scroll id="Scroll">
          <div id="Quests" role="listbox" aria-label={title}>
            {#each quests as q (q.id)}
              <button id="Quest" part="Journal.quest" class="quest" class:main={q.kind == "main"} class:done={q.state == "done"} class:failed={q.state == "failed"} role="option" aria-selected={q.id == selected} aria-label={q.name} aria-description={q.region} onclick={choose}>
                <div class="kind"><icon class="kindmark" icon={q.kind == "main" ? "crown" : q.kind == "side" ? "flag" : "leaf"}></icon></div>
                <div class="what">
                  <span class="name">{q.name}</span>
                  <span class="where">{q.region}</span>
                </div>
                <span class="level number" hidden={!q.level}>{q.level}</span>
                <div class="pin" hidden={!q.tracked} aria-label={t("rpg.journal.tracked")}><icon icon="star"></icon></div>
                <div class="fresh" hidden={!q.fresh} aria-label={t("rpg.journal.updated")}></div>
              </button>
            {/each}
          </div>
        </scroll>
      {/if}
    </div>
    <article id="Entry" part="Journal.entry" hidden={!detailName} aria-label={detailName}>
      <div id="EntryHead">
        <h3 id="Name">{detailName}</h3>
        <p class="quiet">{detailMeta}</p>
      </div>
      <scroll id="EntryScroll">
        <div id="EntryBody">
          <p id="Text">{detailText}</p>
          <h4 class="caption">{t("rpg.journal.objectives")}</h4>
          <ul id="Objectives">
            {#each objectives as o (o.id)}
              <li id="Objective" part="Journal.objective" class="objective" class:done={o.done} class:failed={o.failed} class:optional={o.optional} aria-checked={o.done}>
                <div class="tick"><icon class="state" icon={o.failed ? "close" : "check"}></icon></div>
                <span class="text">{o.text}</span>
                <span class="tag" hidden={!o.optional}>{t("rpg.journal.optional")}</span>
                <span class="progress number" hidden={!o.progress}>{o.progress}</span>
              </li>
            {/each}
          </ul>
          <div id="Rewards" hidden={!rewards.length}>
            <h4 class="caption">{t("rpg.journal.rewards")}</h4>
            <div id="RewardRow">
              {#each rewards as r (r.id)}
                <div class="reward"><icon class="rewardmark" icon={r.icon}></icon><span>{r.label}</span></div>
              {/each}
            </div>
          </div>
        </div>
      </scroll>
      <div id="Actions">
        <button id="Locate" variant="secondary" hidden={!canLocate} onclick={locate}><icon icon="map"></icon><span>{t("rpg.journal.locate")}</span></button>
        <button id="Track" variant="primary" disabled={!canTrack} aria-pressed={tracked} onclick={track} ondetails={track}><kbd id="TrackKey" data-action="details"></kbd><span id="TrackLabel">{tracked ? t("rpg.journal.untrack") : t("rpg.journal.track")}</span></button>
      </div>
    </article>
  </div>
</section>

<style>
#RpgJournal { width: stretch; height: stretch; gap: var(--space-md); padding: var(--space-lg); background: var(--panel); border-radius: var(--radius-lg); container-type: inline-size; }
#Head { display: flex; width: stretch; align-items: center; gap: var(--space-md); }
#Title { margin: 0; font: var(--text-xl); font-family: fantasy; color: var(--ink); }
.number { font-family: monospace; font-variant-numeric: tabular-nums; white-space: nowrap; }
#Groups { display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-xs); width: stretch; border-bottom: var(--stroke-hairline) solid var(--line); padding-bottom: var(--space-xs); }
.group { min-height: var(--size-touch); padding: 0 var(--space-md); gap: var(--space-xs); }
.group[aria-selected=true] { color: var(--highlight); box-shadow: inset 0 calc(-1 * var(--stroke-thick)) 0 0 var(--highlight); }
.group .count { font: var(--text-caption); color: var(--ink-faint); }
#Body { display: flex; width: stretch; height: stretch; gap: var(--space-lg); align-items: stretch; }
#List { width: calc(17 * var(--space-lg)); flex-shrink: 0; height: stretch; }
#Scroll { width: stretch; height: stretch; }
#Quests { width: stretch; gap: var(--space-xxs); padding-inline-end: var(--space-sm); }
.quest { position: relative; width: stretch; min-height: var(--size-touch); padding: var(--space-sm) var(--space-md); gap: var(--space-sm); align-items: center; justify-content: start; text-align: start; }
.kind { width: var(--space-lg); height: var(--space-lg); align-items: center; justify-content: center; flex-shrink: 0; }
.kindmark { width: var(--space-md); height: var(--space-md); color: var(--ink-faint); }
.quest.main .kindmark { color: var(--gold); }
.what { width: stretch; gap: 0; min-width: 0; }
.name { color: var(--ink); overflow-wrap: anywhere; }
.where { font: var(--text-caption); color: var(--ink-muted); }
.quest.done .name, .quest.failed .name { color: var(--ink-muted); }
.quest.failed .name { text-decoration: line-through; }
.level { font: var(--text-caption); font-family: monospace; color: var(--ink-muted); }
.pin { width: var(--space-lg); height: var(--space-lg); align-items: center; justify-content: center; color: var(--highlight); flex-shrink: 0; }
.pin icon { width: var(--space-md); height: var(--space-md); }
.fresh { position: absolute; left: var(--space-xxs); top: var(--space-xxs); width: var(--space-sm); height: var(--space-sm); border-radius: var(--radius-pill); background: var(--accent); }
#Empty { width: stretch; padding: var(--space-xl); gap: var(--space-sm); align-items: center; }
.mark { width: var(--size-lg); height: var(--size-lg); color: var(--ink-faint); }
.quiet { margin: 0; color: var(--ink-muted); font: var(--text-sm); }
.caption { margin: 0; font: var(--text-caption); color: var(--ink-muted); text-transform: uppercase; letter-spacing: 0.08em; }
#Entry { width: stretch; height: stretch; min-width: 0; gap: var(--space-md); padding: var(--space-lg); background: var(--panel-raised); border-radius: var(--radius-md); }
#EntryHead { width: stretch; gap: var(--space-xxs); border-bottom: var(--stroke-hairline) solid var(--line); padding-bottom: var(--space-sm); }
#Name { margin: 0; font: var(--text-lg); font-family: fantasy; color: var(--ink); }
#EntryScroll { width: stretch; height: stretch; }
#EntryBody { width: stretch; gap: var(--space-md); padding-inline-end: var(--space-sm); }
#Text { margin: 0; width: stretch; color: var(--ink); font: var(--text-reading); }
#Objectives { margin: 0; padding: 0; list-style: none; width: stretch; gap: var(--space-xs); }
.objective { display: flex; width: stretch; gap: var(--space-sm); align-items: center; }
.tick { width: var(--space-lg); height: var(--space-lg); flex-shrink: 0; align-items: center; justify-content: center; border: var(--stroke-thin) solid var(--line-strong); border-radius: var(--radius-sm); }
.state { width: var(--space-md); height: var(--space-md); color: var(--success); opacity: 0; }
.objective.done .state { opacity: 1; }
.objective.failed .state { opacity: 1; color: var(--danger); }
.objective .text { width: stretch; color: var(--ink); }
.objective.done .text { color: var(--ink-muted); }
.objective.failed .text { color: var(--ink-muted); text-decoration: line-through; }
.tag { font: var(--text-caption); color: var(--ink-muted); font-style: italic; }
.progress { color: var(--highlight); }
#Rewards { width: stretch; gap: var(--space-xs); }
#RewardRow { display: flex; flex-wrap: wrap; gap: var(--space-xs) var(--space-lg); width: stretch; }
.reward { display: flex; gap: var(--space-xs); align-items: center; color: var(--ink); }
.rewardmark { width: var(--space-lg); height: var(--space-lg); color: var(--gold); }
#Actions { display: flex; flex-wrap: wrap; gap: var(--space-sm); width: stretch; justify-content: end; }
#Actions button { min-height: var(--size-touch); }
@container (width < 640px) {
  #Body { flex-direction: column; }
  #List { width: stretch; height: auto; }
  #Scroll { max-height: calc(4 * var(--size-touch) + 3 * var(--space-xxs)); }
  #Entry { height: auto; }
  #EntryScroll { height: auto; }
  #Actions button { flex: 1 1 auto; }
}
@media (pointer: coarse) { kbd { display: none; } }
</style>
