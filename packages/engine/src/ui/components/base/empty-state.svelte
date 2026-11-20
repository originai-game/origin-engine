<meta name="component" content="EmptyState">
<meta name="strings" content="./states.en.ogt">
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

<!-- Where content would be, there is a recess: a mark on a disc, why the
     area is empty, and the one thing to do about it. -->
<section id="EmptyState" part="frame" layout="stack" play="enter" role="status" aria-label={title || t("base.empty.title")}>
  {#if icon}<div part="dot" class="disc cover" layout="stack" fill><og-icon part="caption" class="mark" icon={icon}></og-icon></div>{/if}
  <div class="words" layout="stack" gap="tight">
    <h3 part="headline" class="title">{title || t("base.empty.title")}</h3>
    {#if body}<p part="body" class="body">{body}</p>{/if}
    {#if hint}<p part="caption" class="hint">{hint}</p>{/if}
  </div>
  {#if actionLabel}<button id="Action" part="action" class="go action" variant="primary" disabled={disabled} onclick={activate}>{actionLabel}</button>{/if}
</section>

<style>
.cover { justify-content: center; align-items: center; }
#EmptyState { align-items: center; padding: var(--space-xl); text-align: center; --layout-gap: var(--layout-gap-wide); --frame-fill: var(--well-fill); --frame-shadow: var(--well-shadow); --frame-rim-width: 0px; --frame-radius: var(--card-radius); }
.disc { flex: none; inline-size: var(--size-xl); block-size: var(--size-xl); --dot-fill: var(--card-fill); --dot-rim: var(--card-rim); --dot-rim-width: var(--card-rim-width); }
.mark { inline-size: var(--size-md); block-size: var(--size-md); }
.words { align-items: center; max-inline-size: var(--measure-lg); }
.title { margin: 0; --headline-text: var(--text-lg); }
.body, .hint { margin: 0; text-align: center; overflow-wrap: anywhere; }
.action { max-inline-size: 100%; min-block-size: var(--touch-target); white-space: normal; }
</style>
