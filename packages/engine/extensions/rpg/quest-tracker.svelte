<meta name="component" content="RpgQuestTracker">
<meta name="strings" content="./rpg.en.ogt">
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
<section id="RpgQuestTracker" class="rpg-quest-tracker" role="status" aria-label={t("rpg.quests.tracked")} hidden={!name}>
  <h3 class="tracker-name">{name}</h3>
  <ul class="tracker-steps">
    {#each steps as s (s.id)}
      <li class="tracker-step" data-done={s.done}><div class="tracker-mark"></div><span class="tracker-text">{s.text}</span></li>
    {/each}
  </ul>
  {#if distance}<div class="tracker-distance"><div class="tracker-arrow"></div><span class="tracker-far">{distance}</span></div>{/if}
</section>

<style>
.rpg-quest-tracker { display: flex; flex-direction: column; width: stretch; gap: var(--space-xs); padding: var(--space-sm) var(--space-md); }
.tracker-name { margin: 0; text-align: center; }
.tracker-steps { display: flex; flex-direction: column; margin: 0; padding: 0; list-style: none; gap: var(--space-xxs); }
.tracker-step { display: flex; align-items: center; gap: var(--space-sm); }
.tracker-mark { width: var(--space-md); height: var(--space-md); flex: none; }
.tracker-text { width: stretch; }
.tracker-distance { display: flex; align-items: center; justify-content: center; gap: var(--space-xs); }
.tracker-arrow { width: var(--space-md); height: var(--space-md); }
</style>
