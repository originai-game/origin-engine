<meta name="component" content="hud">
<meta name="prop:shown" content="bool" data-doc="The HUD is drawn; H turns it off.">
<meta name="prop:clock" content="string" data-doc="The time of day, as the clock shows it (06:54).">
<meta name="prop:day" content="string" data-doc="The season and the time of day's name (Summer · Dawn).">
<meta name="prop:sky" content="string" data-doc="The weather, in a word (Clear).">
<meta name="prop:quest" content="string" data-doc="The day's task list's name.">
<meta name="prop:steps" content="list" data-doc="Its tasks, keyed rows of { id, text, done }.">
<meta name="prop:slots" content="list" data-doc="The hotbar's slots, as AbilityBar takes them.">
<meta name="fixture:default" content='{"shown":true,"clock":"06:54","day":"Summer · Dawn","sky":"Clear","quest":"Morning rounds","steps":[{"id":"a","text":"Watch the sun come up","done":true},{"id":"b","text":"Count the flock","done":false}],"slots":[{"id":"weather","icon":"drop","name":"Weather","action":"weather","hotkey":"1","cooldown":0,"cooldownMax":0,"cost":"","unaffordable":false,"locked":false,"active":false,"count":0}]}'>

<!--
  The cosy HUD: the clock top left, the day's tasks top right, the hotbar
  along the bottom. It reads what the game passes and reports `use` with a
  slot's id; H hides it.
-->
<div id="Hud">
{#if $shown}
  <section id="Clock" class="hud" data-layout="stack" data-gap="hair" role="status">
    <div id="Now" data-layout="row" data-gap="tight"><og-icon id="ClockIcon" icon="clock"></og-icon><span id="Time">{$clock}</span></div>
    <span id="Day">{$day}</span>
    <span id="Sky">{$sky}</span>
  </section>
  <div id="Tasks"><RpgQuestTracker name={$quest} steps={$steps} distance="" /></div>
  <div id="Hotbar"><AbilityBar slots={$slots} column={false} /></div>
{/if}
</div>

<style>
#Clock {
  position: fixed;
  left: calc(env(-og-hud-inset-left) + var(--space-lg));
  top: calc(env(-og-hud-inset-top) + var(--space-lg));
  padding: var(--space-sm) var(--space-lg);
  background: var(--panel);
  border-radius: var(--radius-panel);
  outline: 1px solid var(--line); outline-offset: -1px;
}
#ClockIcon { width: var(--size-xs); height: var(--size-xs); }
#Time { font: var(--text-display-sm); font-family: var(--font-mono); white-space: nowrap; }
#Day { font: var(--text-body); white-space: nowrap; }
#Sky { font: var(--text-meta); color: var(--ink-muted); white-space: nowrap; }
#Tasks {
  position: fixed;
  right: calc(env(-og-hud-inset-right) + var(--space-lg));
  top: calc(env(-og-hud-inset-top) + var(--space-lg));
  inline-size: var(--measure-sm);
}
#Hotbar {
  position: fixed;
  left: 0; right: 0;
  bottom: calc(env(-og-hud-inset-bottom) + var(--space-lg));
  display: flex;
  justify-content: center;
}
</style>
