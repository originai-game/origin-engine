<meta name="component" content="banner">
<meta name="parts" content="banner.frame banner.title banner.subtitle">
<meta name="prop:title" content="string" data-doc="Its headline (Level up!).">
<meta name="prop:subtitle" content="string" data-doc="A line under it; empty: none.">
<meta name="fixture:default" content="{&quot;title&quot;:&quot;Level up!&quot;,&quot;subtitle&quot;:&quot;Level 12&quot;}">

<section id="Banner" part="banner.frame" role="status">
  <h2 id="Title" part="banner.title">{$title}</h2>
  <p id="Subtitle" part="banner.subtitle" hidden={$subtitle == ""}>{$subtitle}</p>
</section>

<style>
#Banner { display: flex; flex-direction: column; align-items: center; animation: enter var(--motion-named-panel-duration) ease-out both; }
#Subtitle { margin: 0; color: var(--ink-muted); }
@keyframes enter { from { scale: 0.8; opacity: 0; -og-sound: reward; } to { scale: 1; opacity: 1; } }
</style>
