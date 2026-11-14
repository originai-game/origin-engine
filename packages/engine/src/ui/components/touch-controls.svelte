<meta name="component" content="TouchControls">
<meta name="strings" content="./touch-controls.en.ogt">
<meta name="parts" content="TouchControls.frame TouchControls.stick TouchControls.actions TouchControls.look">
<meta name="prop:move" content="string" data-doc="The two-axis og-input action the stick at the start side feeds (move); empty leaves the stick out.">
<meta name="prop:pad" content="bool" data-doc="A d-pad in the stick's place: eight ways, each axis -1, 0 or 1.">
<meta name="prop:look" content="string" data-doc="A second stick at the end side and the two-axis action it feeds (look); empty leaves it out.">
<meta name="prop:deadZone" content="number" data-doc="The sticks' dead zone as a part of their radius, up to 0.95; 0 is the engine's 0.15.">
<meta name="prop:actions" content="list" data-doc="The buttons at the end side, the first nearest the corner: keyed rows of { id, action, label, icon, primary, charge, disabled }, each as ActionButton takes them.">
<meta name="prop:always" content="bool" data-doc="Shown under a mouse too (a touch laptop, a preview). Otherwise the controls show only where the primary pointer is a finger.">
<meta name="prop:disabled" content="bool" data-doc="Every control feeds nothing and shows faded (a cutscene, a menu over the HUD).">
<meta name="fixture:default" content='{"move":"move","pad":false,"look":"","deadZone":0,"always":true,"disabled":false,"actions":[{"id":"jump","action":"jump","label":"Jump","icon":"bolt","primary":true,"charge":-1,"disabled":false},{"id":"dash","action":"dash","label":"Dash","icon":"fire","primary":false,"charge":-1,"disabled":false},{"id":"use","action":"use","label":"Use","icon":"key","primary":false,"charge":-1,"disabled":false}]}'>
<meta name="fixture:pad" content='{"move":"move","pad":true,"look":"","deadZone":0,"always":true,"disabled":false,"actions":[{"id":"a","action":"confirm","label":"A","icon":"","primary":true,"charge":-1,"disabled":false},{"id":"b","action":"back","label":"B","icon":"","primary":false,"charge":-1,"disabled":false}]}'>
<meta name="fixture:twin" content='{"move":"move","pad":false,"look":"look","deadZone":0.2,"always":true,"disabled":false,"actions":[{"id":"fire","action":"fire","label":"Fire","icon":"bomb","primary":true,"charge":-1,"disabled":false},{"id":"ult","action":"ultimate","label":"Ultimate","icon":"star","primary":false,"charge":0.6,"disabled":true}]}'>
<meta name="fixture:many" content='{"move":"move","pad":false,"look":"","deadZone":0,"always":true,"disabled":false,"actions":[{"id":"attack","action":"attack","label":"Attack","icon":"sword","primary":true,"charge":-1,"disabled":false},{"id":"block","action":"block","label":"Block","icon":"shield","primary":false,"charge":-1,"disabled":false},{"id":"dodge","action":"dodge","label":"Dodge","icon":"bolt","primary":false,"charge":-1,"disabled":false},{"id":"heal","action":"heal","label":"Heal","icon":"potion","primary":false,"charge":-1,"disabled":false},{"id":"bag","action":"bag","label":"Bag","icon":"bag","primary":false,"charge":-1,"disabled":false}]}'>
<meta name="fixture:disabled" content='{"move":"move","pad":false,"look":"","deadZone":0,"always":true,"disabled":true,"actions":[{"id":"jump","action":"jump","label":"Jump","icon":"bolt","primary":true,"charge":-1,"disabled":false}]}'>
<meta name="fixture:empty" content='{"move":"","pad":false,"look":"","deadZone":0,"always":true,"disabled":false,"actions":[]}'>

<!--
  The on-screen controls of a game played with thumbs: a stick or a d-pad
  at the start side, the action buttons at the end side with the first
  nearest the corner (a row of three, the rest in rows above), and a second
  stick beside them when the game looks with one. Each control is an input
  source of the og-input action it names, so the game reads the same
  actions as from keys or a pad and handles no event; the component reports
  none. It shows only where the primary pointer is a finger, unless always
  is set, and a control that is not shown or is disabled feeds nothing. The
  page places it: it is as wide as its box and stands on its bottom edge.
-->
<div id="TouchControls" part="TouchControls.frame" class="bare" class:always={always} role="group" aria-label={t("component.touch-controls.name")}>
  <div id="Start">
    {#if move}
      {#if pad}
        <dpad id="Pad" part="TouchControls.stick" action={move} disabled={disabled} aria-label={t("component.touch-controls.move")}></dpad>
      {:else}
        <joystick id="Stick" part="TouchControls.stick" action={move} data-dead-zone={deadZone} disabled={disabled} aria-label={t("component.touch-controls.move")}></joystick>
      {/if}
    {/if}
  </div>
  <div id="End">
    <div id="Actions" part="TouchControls.actions">
      {#each actions as a (a.id)}
        <ActionButton action={a.action} label={a.label} icon={a.icon} primary={a.primary} charge={a.charge} disabled={disabled || a.disabled} />
      {/each}
    </div>
    {#if look}
      <joystick id="Look" part="TouchControls.look" action={look} data-dead-zone={deadZone} disabled={disabled} aria-label={t("component.touch-controls.look")}></joystick>
    {/if}
  </div>
</div>

<style>
#TouchControls { display: flex; width: stretch; justify-content: space-between; align-items: end; gap: var(--space-lg); padding: var(--space-lg); }
@media (pointer: fine) {
  #TouchControls { display: none; }
  #TouchControls.always { display: flex; }
}
#Start { display: flex; align-items: end; }
#End { display: flex; align-items: end; gap: var(--space-md); }
#Actions { display: flex; flex-direction: row-reverse; flex-wrap: wrap-reverse; align-items: flex-start; gap: var(--space-sm); max-width: calc(var(--action-size) * 3.25 + var(--space-sm) * 2); }
@media (width < 600px) {
  #TouchControls { padding: var(--space-md); gap: var(--space-md); }
  #End { gap: var(--space-sm); }
}
</style>
