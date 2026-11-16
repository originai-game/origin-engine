<meta name="component" content="Steps">
<meta name="strings" content="./states.en.ogt">
<meta name="parts" content="Steps.frame Steps.title Steps.list Steps.step Steps.item Steps.cell Steps.mark Steps.dot Steps.label Steps.headline Steps.detail Steps.state Steps.caption Steps.empty">
<meta name="prop:title" content="string" data-default="" data-doc="Optional visible heading; the accessible fallback is Progress steps.">
<meta name="prop:steps" content="list" data-default="[]" data-doc="Keyed { id, label, detail, number, state, disabled } rows. number is the game's formatted marker, state upcoming/current/done/error. The game chooses order and allowed navigation.">
<meta name="prop:column" content="bool" data-default="false" data-doc="Stack steps vertically instead of a wrapping horizontal sequence.">
<meta name="action:choose" content="Choose a step, by its stable key">
<meta name="fixture:default" content='{"title":"Prepare your expedition","steps":[{"id":"party","label":"Party","detail":"3 players ready","number":"1","state":"done","disabled":false},{"id":"route","label":"Route","detail":"Choose a destination","number":"2","state":"current","disabled":false},{"id":"launch","label":"Launch","detail":"Waiting for a route","number":"3","state":"upcoming","disabled":true}],"column":false}'>
<meta name="fixture:empty" content='{"steps":[]}'>
<meta name="fixture:error" content='{"column":true,"steps":[{"id":"gear","label":"Equipment","number":"1","state":"done"},{"id":"network","label":"Connection","detail":"Reconnect to continue","number":"2","state":"error"},{"id":"ready","label":"Ready","number":"3","state":"upcoming","disabled":true}]}'>
<meta name="fixture:overflow" content='{"title":"An expedition across the northern territories","steps":[{"id":"one","label":"Choose your character and starting equipment","detail":"Every choice can be changed later in the lobby.","number":"1","state":"done"},{"id":"two","label":"Invite the rest of your expedition party","detail":"Waiting for players to accept their invitations.","number":"2","state":"current"},{"id":"three","label":"Confirm the expedition destination","number":"3","state":"upcoming","disabled":true}]}'>

<nav id="Steps" part="Steps.frame" class:column={column} aria-label={title || t("base.steps.title")}>
  {#if title}<h3 part="Steps.title Steps.headline">{title}</h3>{/if}
  <ol part="Steps.list" class="steps">
    {#each steps as step (step.id)}<li part="Steps.step" class="step" data-state={step.state}>
      <button part="Steps.item Steps.cell" class="item" disabled={step.disabled} aria-current={step.state == 'current' ? 'step' : null} aria-label={step.label} aria-description={step.detail} onclick={choose}>
        <div class="face">
        <div part="Steps.mark Steps.dot" class="mark" data-tone={step.state == 'done' ? 'success' : step.state == 'error' ? 'danger' : null}>{#if step.state == 'done' || step.state == 'error'}<og-icon class="icon" icon={step.state == 'done' ? 'check' : 'close'}></og-icon>{:else}<span>{step.number}</span>{/if}</div>
        <div class="copy"><span part="Steps.label Steps.headline" class="label">{step.label}</span>{#if step.detail}<span part="Steps.detail Steps.caption" class="detail">{step.detail}</span>{/if}<span part="Steps.state Steps.caption" class="state" data-tone={step.state == 'error' ? 'danger' : null}>{step.state == 'done' ? t("base.steps.done") : step.state == 'error' ? t("base.steps.error") : step.state == 'current' ? t("base.steps.current") : t("base.steps.upcoming")}</span></div>
        </div>
      </button>
    </li>{:else}<li part="Steps.empty Steps.caption" class="empty" role="status">{t("base.steps.empty")}</li>{/each}
  </ol>
</nav>

<style>
:host { display: flex; flex-direction: column; gap: var(--space-sm); inline-size: stretch; }
[part="Steps.title"] { margin: 0; }
.steps { display: flex; flex-wrap: wrap; gap: var(--space-sm); margin: 0; padding: 0; list-style: none; }
.step { flex: 1 1 var(--measure-xs); }
.item { display: block; inline-size: stretch; min-block-size: var(--touch-target); padding: var(--space-sm); text-align: start; }
.face { display: flex; align-items: start; gap: var(--space-sm); inline-size: stretch; }
.mark { display: flex; flex: none; inline-size: var(--size-md); block-size: var(--size-md); align-items: center; justify-content: center; }
.icon { inline-size: var(--size-xs); block-size: var(--size-xs); }
.copy { display: flex; flex-direction: column; gap: var(--space-xxs); flex: 1; min-inline-size: 0; }
.label, .detail, .state { overflow-wrap: anywhere; }
.column .steps { flex-direction: column; }
.column .step { flex: none; }
</style>
