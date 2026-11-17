<meta name="component" content="EmptyState">
<meta name="strings" content="./states.en.ogt">
<meta name="parts" content="EmptyState.frame EmptyState.mark EmptyState.title EmptyState.body EmptyState.hint EmptyState.action EmptyState.headline EmptyState.caption">
<meta name="prop:title" content="string" data-default="" data-doc="Heading; empty uses Nothing here yet.">
<meta name="prop:body" content="string" data-default="" data-doc="Why this area is empty.">
<meta name="prop:hint" content="string" data-default="" data-doc="An optional next step, in quieter text.">
<meta name="prop:icon" content="string" data-default="bag" data-doc="An icon above the heading; empty hides it.">
<meta name="prop:actionLabel" content="string" data-default="" data-doc="The optional action's words; empty hides the button.">
<meta name="prop:disabled" content="bool" data-default="false" data-doc="The action is unavailable.">
<meta name="action:activate" content="Take the suggested action">
<meta name="fixture:default" content='{"title":"No saved builds","body":"Save your equipment to switch builds between matches.","hint":"Your current equipment will not change.","icon":"bag","actionLabel":"Save this build","disabled":false}'>
<meta name="fixture:minimal" content='{}'>
<meta name="fixture:disabled" content='{"title":"No saved builds","body":"Finish the tutorial before saving a build.","actionLabel":"Save build","disabled":true}'>
<meta name="fixture:overflow" content='{"title":"No discoveries in the northern reaches of the Sunken Kingdom","body":"Explore the coast and investigate the old watchtowers to add discoveries to this collection.","hint":"Progress is shared across every character on your account.","actionLabel":"Return to the world map and choose a destination"}'>

<section id="EmptyState" part="EmptyState.frame" data-layout="stack" role="status" aria-label={title || t("base.empty.title")}>
  {#if icon}<og-icon part="EmptyState.mark EmptyState.caption" class="mark" icon={icon}></og-icon>{/if}
  <h3 part="EmptyState.title EmptyState.headline">{title || t("base.empty.title")}</h3>
  {#if body}<p part="EmptyState.body" class="body">{body}</p>{/if}
  {#if hint}<p part="EmptyState.hint EmptyState.caption" class="hint">{hint}</p>{/if}
  {#if actionLabel}<button id="Action" part="EmptyState.action" disabled={disabled} onclick={activate}>{actionLabel}</button>{/if}
</section>

<style>
:host { align-items: center; padding: var(--space-xl); text-align: center; }
.mark { inline-size: var(--size-lg); block-size: var(--size-lg); }
[part~="EmptyState.title"] { margin: 0; --headline-text: var(--text-lg); }
.body, .hint { margin: 0; align-self: center; max-inline-size: var(--measure-lg); text-align: center; overflow-wrap: anywhere; }
[part="EmptyState.action"] { max-inline-size: 100%; min-block-size: var(--touch-target); white-space: normal; margin-top: var(--space-xs); }
</style>
