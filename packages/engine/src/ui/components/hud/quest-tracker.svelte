<meta name="component" content="RpgQuestTracker">
<meta name="strings" content="./quest-tracker.en.ogt">
<meta name="parts" content="RpgQuestTracker.frame RpgQuestTracker.title RpgQuestTracker.row RpgQuestTracker.caption RpgQuestTracker.figure">
<meta name="prop:name" content="string" data-doc="The tracked quest's name, on the plate; empty hides the tracker.">
<meta name="prop:steps" content="list" data-doc="Its current objectives, keyed rows of { id, text, done }: done ticks and quiets one.">
<meta name="prop:distance" content="string" data-doc="How far its marker is, as the game words it (312 m); empty hides it.">
<meta name="fixture:default" content='{"name":"The Fallen Shrine","steps":[{"id":"a","text":"Reach the Ashen Spire","done":true},{"id":"b","text":"Investigate the Bloodstone Circle","done":false}],"distance":"312 m"}'>
<meta name="fixture:none" content='{"name":"","steps":[],"distance":""}'>

<!--
  The tracked quest on the HUD: its name on a plate, its objectives with
  their ticks, and the distance to its marker. Read only; the journal owns
  choosing what to track.
-->
<section id="RpgQuestTracker" part="RpgQuestTracker.frame" class="hud" data-layout="stack" data-empty={!name} role="status" aria-label={t("rpg.quests.tracked")}>
  {#if name}
  <h3 part="RpgQuestTracker.title">{name}</h3>
  <ul class="tracker-steps" data-layout="stack">
    {#each steps as s (s.id)}
      <li part="RpgQuestTracker.row" data-layout="row" data-done={s.done}><og-icon class="tracker-mark" data-tone={s.done ? 'success' : null} icon={s.done ? 'check' : 'flag'}></og-icon><span class="tracker-text" data-fill>{s.text}</span></li>
    {/each}
  </ul>
  {#if distance}<div class="tracker-distance" data-layout="row" data-gap="tight"><og-icon class="tracker-arrow" icon="map"></og-icon><span class="tracker-far" part="RpgQuestTracker.caption RpgQuestTracker.figure">{distance}</span></div>{/if}
  {/if}
</section>

<style>
:host { inline-size: stretch; }
[part="RpgQuestTracker.frame"][data-empty=true] { display: none; }
[part="RpgQuestTracker.title"] { margin: 0; text-align: start; }
.tracker-steps { margin: 0; padding: 0; list-style: none; }
.tracker-mark { width: var(--size-xs); height: var(--size-xs); flex: none; }
.tracker-distance { padding-block-start: var(--space-xs); }
.tracker-arrow { width: var(--space-md); height: var(--space-md); }
</style>
