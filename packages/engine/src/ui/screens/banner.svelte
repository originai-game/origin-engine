<meta name="component" content="banner">
<meta name="prop:title" content="string" data-doc="Its headline (Level up!).">
<meta name="prop:subtitle" content="string" data-doc="A line under it; empty: none.">
<meta name="fixture:default" content="{&quot;title&quot;:&quot;Level up!&quot;,&quot;subtitle&quot;:&quot;Level 12&quot;}">

<section id="Banner" role="status">
  <h2 id="Title">{$title}</h2>
  <p id="Subtitle" hidden={$subtitle == ""}>{$subtitle}</p>
</section>

<style>
#Banner { display: flex; flex-direction: column; align-items: center; padding: 16px 48px; background: var(--panel-raised); border-radius: var(--radius-panel); animation: enter 420ms ease-out both; }
#Title, #Subtitle { margin: 0; }
#Subtitle { color: var(--ink-muted); }
@keyframes enter { from { scale: 0.8; opacity: 0; -og-sound: reward; } to { scale: 1; opacity: 1; } }
</style>
