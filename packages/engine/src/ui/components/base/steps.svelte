<meta name="component" content="Steps">
<meta name="strings" content="./states.en.ogt">
<meta name="prop:title" content="string" data-default="" data-doc="Optional visible heading; the accessible fallback is Progress steps.">
<meta name="prop:steps" content="list" data-default="[]" data-doc="Keyed { id, label, detail, number, state, disabled } rows. number is the game's formatted marker, state upcoming/current/done/error. The game chooses order and allowed navigation.">
<meta name="prop:column" content="bool" data-default="false" data-doc="Stack steps vertically instead of a wrapping horizontal sequence.">
<meta name="action:choose" content="Choose a step, by its stable key">
<meta name="fixture:default" content='{"title":"Prepare your expedition","steps":[{"id":"party","label":"Party","detail":"3 players ready","number":"1","state":"done","disabled":false},{"id":"route","label":"Route","detail":"Choose a destination","number":"2","state":"current","disabled":false},{"id":"launch","label":"Launch","detail":"Waiting for a route","number":"3","state":"upcoming","disabled":true}],"column":false}'>
<meta name="fixture:empty" content='{"steps":[]}'>
<meta name="fixture:column" content='{"title":"Prepare your expedition","column":true,"steps":[{"id":"party","label":"Party","detail":"3 players ready","number":"1","state":"done","disabled":false},{"id":"route","label":"Route","detail":"Choose a destination","number":"2","state":"current","disabled":false},{"id":"launch","label":"Launch","detail":"Waiting for a route","number":"3","state":"upcoming","disabled":true}]}'>
<meta name="fixture:error" content='{"steps":[{"id":"gear","label":"Equipment","number":"1","state":"done"},{"id":"network","label":"Connection","detail":"Reconnect to continue","number":"2","state":"error"},{"id":"ready","label":"Ready","number":"3","state":"upcoming","disabled":true}]}'>
<meta name="fixture:overflow" content='{"title":"An expedition across the northern territories","steps":[{"id":"one","label":"Choose your character and starting equipment","detail":"Every choice can be changed later in the lobby.","number":"1","state":"done"},{"id":"two","label":"Invite the rest of your expedition party","detail":"Waiting for players to accept their invitations.","number":"2","state":"current"},{"id":"three","label":"Confirm the expedition destination","number":"3","state":"upcoming","disabled":true}]}'>

<!-- A rail of numbered marks: each step is a mark, the reach to the next
     one (full once the step is done) and its words. It lies along a row
     and stands up as a column where `column` asks or the container is
     narrow. -->
<div id="StepsShell" layout="stack" fill>
<nav id="Steps" part="frame" variant="bare" layout="stack" gap="loose" class:column={column} aria-label={title || t("base.steps.title")}>
  {#if title}<h3 part="title" class="title">{title}</h3>{/if}
  <ol part="list" class="steps">
    {#each steps as step, i (step.id)}<li part="step" class="step" class:last={i == steps.length - 1} class:state-done={(step.state) == 'done'} class:state-upcoming={(step.state) == 'upcoming'}>
      <button part="row" class="item" disabled={step.disabled} aria-current={step.state == 'current' ? 'step' : null} aria-label={step.label} aria-description={step.detail} onclick={choose}>
        <div class="face">
          <div class="rail">
            <div part="dot" class="mark cover" layout="stack" fill tone={step.state == 'done' ? 'success' : step.state == 'error' ? 'danger' : step.state == 'current' ? 'primary' : null} play={step.state == 'current' ? 'pop' : null}>{#if step.state == 'done' || step.state == 'error'}<og-icon class="icon" icon={step.state == 'done' ? 'check' : 'close'}></og-icon>{:else}<span>{step.number}</span>{/if}</div>
            <div part="track" class="reach"><div part="fill" class="done" tone="success"></div></div>
          </div>
          <div class="copy" layout="stack" gap="hair">
            <span part="caption" class="state" tone={step.state == 'done' ? 'success' : step.state == 'error' ? 'danger' : step.state == 'current' ? 'primary' : null}>{step.state == 'done' ? t("base.steps.done") : step.state == 'error' ? t("base.steps.error") : step.state == 'current' ? t("base.steps.current") : t("base.steps.upcoming")}</span>
            <span part="headline" class="label">{step.label}</span>
            {#if step.detail}<span part="caption" class="detail">{step.detail}</span>{/if}
          </div>
        </div>
      </button>
    </li>{:else}<li part="caption" class="empty" role="status">{t("base.steps.empty")}</li>{/each}
  </ol>
</nav>
</div>

<style>
.cover { justify-content: center; align-items: center; }
#StepsShell { block-size: auto; }
.title { --title-size: var(--title-small-size); }
.steps { display: flex; margin: 0; padding: 0; list-style: none; }
.step { display: flex; flex: 1 1 0; min-inline-size: 0; }
.item { display: block; align-items: flex-start; inline-size: stretch; min-block-size: var(--touch-target); padding: var(--space-sm); text-align: start; --row-selected-fill: var(--row-fill); --row-selected-edge: none; }
.item:disabled { --headline-ink: var(--ink-disabled); }
.face { display: grid; align-self: flex-start; grid-template-columns: minmax(0, 1fr); gap: var(--space-sm) var(--space-md); inline-size: stretch; }
.rail { display: flex; align-items: center; gap: var(--space-xs); }
.mark { flex: none; inline-size: var(--size-md); block-size: var(--size-md); }
.step.state-upcoming .mark { --dot-fill: var(--well-fill); --dot-rim: var(--cell-rim); --dot-rim-width: var(--cell-rim-width); --dot-ink: var(--ink-label); }
.step.state-upcoming .item { --headline-ink: var(--ink-label); }
.icon { inline-size: var(--size-xs); block-size: var(--size-xs); }
.reach { display: flex; flex: 1 1 0; min-inline-size: var(--space-md); block-size: var(--stroke-thick); margin-inline-end: calc(var(--space-xs) - var(--space-sm) * 2); overflow: hidden; --track-fill: var(--cell-rim); --track-shadow: none; }
.done { flex: 0 0 0; }
.step.state-done .done { flex: 1 1 0; }
.step.last .reach { visibility: hidden; }
.copy { min-inline-size: 0; }
.label, .detail, .state { overflow-wrap: anywhere; }
.column .steps { flex-direction: column; }
.column .face { grid-template-columns: auto minmax(0, 1fr); }
.column .rail { flex-direction: column; }
.column .reach { flex-direction: column; inline-size: var(--stroke-thick); block-size: auto; min-inline-size: 0; min-block-size: var(--space-md); margin-inline-end: 0; margin-block-end: calc(var(--space-xs) - var(--space-sm) * 2); }
.steps:size(compact) { flex-direction: column; }
.face:size(compact) { grid-template-columns: auto minmax(0, 1fr); }
.rail:size(compact) { flex-direction: column; }
.reach:size(compact) { flex-direction: column; inline-size: var(--stroke-thick); block-size: auto; min-inline-size: 0; min-block-size: var(--space-md); margin-inline-end: 0; margin-block-end: calc(var(--space-xs) - var(--space-sm) * 2); }
</style>
