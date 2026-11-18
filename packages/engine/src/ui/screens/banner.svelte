<meta name="component" content="banner">
<meta name="parts" content="banner.frame banner.title banner.subtitle banner.rule">
<meta name="prop:title" content="string" data-doc="Its headline (Level up!).">
<meta name="prop:subtitle" content="string" data-doc="A line under it; empty: none.">
<meta name="fixture:default" content="{&quot;title&quot;:&quot;Level up!&quot;,&quot;subtitle&quot;:&quot;Level 12&quot;}">

<!--
  A call across the scene with no plate: the headline in large HUD type
  (white, one stroke under the fill and a soft shadow), a lit gold rule
  under it and the line beneath in quieter HUD type. It rises in with the
  reward sound; under reduced motion it fades in place.
-->
<section id="Banner" part="banner.frame" class="bare" role="status">
  <h2 id="Title" part="banner.title">{title}</h2>
  <div id="Rule" part="banner.rule"></div>
  <p id="Subtitle" part="banner.subtitle" hidden={subtitle == ""}>{subtitle}</p>
</section>

<style>
/* HUD type over the scene: one stroke under the fill and one soft shadow. */
:host { color: var(--hud-ink); text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--alert-stroke-color); paint-order: stroke fill; }
#Banner { display: flex; flex-direction: column; align-items: center; gap: var(--space-xs); padding-inline: var(--space-lg); text-align: center; animation: enter var(--motion-named-panel-duration) ease-out both; }
#Title { margin: 0; max-inline-size: var(--measure-xl); font-size: var(--alert-size-banner); font-weight: var(--alert-weight); line-height: var(--font-leading-tight); text-wrap: balance; overflow-wrap: anywhere; }
#Rule { inline-size: min(100%, var(--measure-md)); block-size: var(--stroke-thick); background: var(--alert-rule); }
#Subtitle { margin: 0; font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-gold); text-wrap: balance; overflow-wrap: anywhere; }
@keyframes enter { from { scale: 0.8; opacity: 0; -og-sound: reward; } to { scale: 1; opacity: 1; } }
@keyframes appear { from { opacity: 0; -og-sound: reward; } to { opacity: 1; } }
@media (prefers-reduced-motion: reduce) { #Banner { animation-name: appear; } }
@media (width < 600px) { #Title { font-size: var(--alert-size-banner-compact); } }
</style>
