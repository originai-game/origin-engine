<meta name="component" content="RpgQuestTracker">
<meta name="strings" content="./quest-tracker.en.ogt">
<meta name="parts" content="RpgQuestTracker.frame RpgQuestTracker.legend RpgQuestTracker.quest RpgQuestTracker.distance RpgQuestTracker.step RpgQuestTracker.marker RpgQuestTracker.text RpgQuestTracker.progress">
<meta name="prop:label" content="string" data-doc="The tracker's heading as the game words it (Side quest); empty takes the component's Main quest.">
<meta name="prop:name" content="string" data-doc="The tracked quest's name; empty hides the tracker.">
<meta name="prop:steps" content="list" data-doc="Its objectives in order, keyed rows of { id, text, done, tag }: done ones are struck and dimmed with a filled diamond; the first not done is the step in play (lit, on data-current); the ones after it are upcoming (dim); tag its progress as the game words it (1 / 3) on a tag, empty for none. A phone shows the step in play alone.">
<meta name="prop:distance" content="string" data-doc="How far its marker is, as the game words it (312 m), on a tag beside the name; empty hides it.">
<meta name="fixture:default" content='{"label":"","name":"The Ninefold Gate","steps":[{"id":"a","text":"Reach the highland ruins","done":true,"tag":""},{"id":"b","text":"Break the Colossus’s guard","done":false,"tag":"1 / 3"},{"id":"c","text":"Light the three rune pillars","done":false,"tag":""}],"distance":"312 m"}'>
<meta name="fixture:none" content='{"label":"","name":"","steps":[],"distance":""}'>
<meta name="fixture:done" content='{"label":"Side quest","name":"The Sunken Bell","steps":[{"id":"a","text":"Speak with the oarsman","done":true,"tag":""},{"id":"b","text":"Find the sunken belfry","done":true,"tag":""}],"distance":""}'>
<meta name="fixture:overflow" content='{"label":"Main quest of the second act of the long campaign","name":"The Ninefold Gate beneath the Ashen Vale and the Drowned Abbey","steps":[{"id":"a","text":"Break the guard of the Ashen Colossus before the ninth bell tolls over the vale","done":false,"tag":"12,345 / 99,999"},{"id":"b","text":"Light the three rune pillars","done":false,"tag":""}],"distance":"12,345 km"}'>

<!--
  The tracked quest on the HUD: a heading in gold, the quest's name, then
  its objectives, the done ones struck through, the one in play lit with
  its progress on a tag and the upcoming ones dim. Read only; the journal
  owns choosing what to track.
-->
<section id="RpgQuestTracker" part="RpgQuestTracker.frame" class="hud bare" data-layout="stack" data-empty={!name} role="status" aria-label={label != '' ? label : t("rpg.quests.main")}>
  {#if name}
  <span part="RpgQuestTracker.legend" class="legend">{label != '' ? label : t("rpg.quests.main")}</span>
  <div id="Head" data-layout="row" data-gap="tight">
    <span part="RpgQuestTracker.quest" class="quest">{name}</span>
    {#if distance}<span part="RpgQuestTracker.distance" class="tag">{distance}</span>{/if}
  </div>
  <ul class="steps" data-layout="stack">
    {#each steps as s, i (s.id)}
      <li part="RpgQuestTracker.step" class="step" data-layout="row" data-gap="tight" data-done={s.done} data-current={!s.done && (i == 0 || steps[i - 1]['done'])}>
        <div part="RpgQuestTracker.marker" class="marker"></div>
        <span part="RpgQuestTracker.text" class="text">{s.text}</span>
        {#if s.tag}<span part="RpgQuestTracker.progress" class="tag">{s.tag}</span>{/if}
      </li>
    {/each}
  </ul>
  {/if}
</section>

<style>
:host { inline-size: calc(250 * var(--font-step-px)); max-inline-size: stretch; row-gap: calc(5 * var(--font-step-px)); color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
[part="RpgQuestTracker.frame"][data-empty=true] { display: none; }
.legend { min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-gold); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
#Head { align-items: baseline; min-inline-size: 0; }
.quest { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-size-title); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.steps { margin: 0; padding: 0; list-style: none; row-gap: calc(4 * var(--font-step-px)); }
.step { align-items: center; min-inline-size: 0; opacity: 0.66; }
.step[data-current=true] { opacity: 1; }
.step[data-done=true] { opacity: 0.55; }
/* A diamond: hollow gold for a step to do, filled once done. */
.marker { flex: none; inline-size: calc(7 * var(--font-step-px)); block-size: calc(7 * var(--font-step-px)); margin-inline: calc(2 * var(--font-step-px)); transform: rotate(45deg); box-shadow: var(--hud-step-rim); }
.step[data-done=true] .marker { background: var(--hud-gold); }
.text { flex: 0 1 auto; min-inline-size: 0; font-size: var(--hud-size-xs); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; }
.step[data-done=true] .text { text-decoration-line: var(--hud-done-line); }
.tag { flex: none; max-inline-size: 45%; padding: 0 var(--space-xs); font-size: var(--hud-tag-size); font-weight: var(--hud-figure-weight); font-stretch: var(--hud-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; -webkit-text-stroke-width: 0; text-shadow: none; background: var(--hud-tag-fill); box-shadow: var(--hud-tag-rim); border-radius: var(--hud-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
/* On a phone the tracker is one line: the step in play. */
@media (width < 600px) {
  :host { inline-size: stretch; }
  .legend, #Head, .step:not([data-current=true]) { display: none; }
}
</style>
