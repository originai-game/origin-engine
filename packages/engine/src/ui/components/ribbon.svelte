<meta name="component" content="Ribbon">
<meta name="parts" content="Ribbon.frame Ribbon.ribbon Ribbon.text">
<meta name="prop:text" content="string" data-doc="The ribbon's words (NEW, SALE).">
<meta name="prop:corner" content="string" data-doc="Where it sits: topRight (empty), topLeft, bottomRight, bottomLeft or center (a stamp).">
<meta name="prop:tone" content="string" data-doc="Its tone: primary (empty), success, warn, danger or info.">
<meta name="fixture:default" content='{"text":"New","corner":"","tone":"danger"}'>

<div id="Corner" part="Ribbon.frame" class:topLeft={corner == 'topLeft'} class:bottomRight={corner == 'bottomRight'} class:bottomLeft={corner == 'bottomLeft'} class:center={corner == 'center'} class:success={tone == 'success'} class:warn={tone == 'warn'} class:danger={tone == 'danger'} class:info={tone == 'info'}>
  <div id="Ribbon" part="Ribbon.ribbon" role="status" aria-label={text}><span id="Text" part="Ribbon.text">{text}</span></div>
</div>

<style>
[part="Ribbon.frame"] { width: var(--size-4xl); height: calc(var(--size-xxl) + var(--space-lg)); padding: 0; overflow: hidden; --tone: var(--primary); }
[part="Ribbon.frame"].success { --tone: var(--success); }
[part="Ribbon.frame"].warn { --tone: var(--warn); }
[part="Ribbon.frame"].danger { --tone: var(--danger); }
[part="Ribbon.frame"].info { --tone: var(--info); }
[part="Ribbon.ribbon"] { position: absolute; top: var(--space-lg); inset-inline-start: var(--space-3xl); width: var(--size-4xl); padding: var(--space-xxs) 0; align-items: center; background: linear-gradient(180deg, var(--sheen), var(--shadow-30)), var(--tone); border: var(--stroke-hairline) solid var(--sheen); box-shadow: 0 var(--space-xxs) var(--space-xs) var(--shadow-45); rotate: 45deg; animation-name: slap; animation-duration: var(--motion-duration-base); animation-timing-function: var(--motion-easing-out); }
.topLeft [part="Ribbon.ribbon"] { inset-inline-start: calc(var(--space-3xl) * -1); rotate: -45deg; }
.bottomRight [part="Ribbon.ribbon"] { top: var(--size-xl); rotate: -45deg; }
.bottomLeft [part="Ribbon.ribbon"] { top: var(--size-xl); inset-inline-start: calc(var(--space-3xl) * -1); rotate: 45deg; }
.center [part="Ribbon.ribbon"] { top: calc(var(--space-xl) + var(--space-lg)); inset-inline-start: var(--space-lg); width: var(--size-3xl); rotate: -8deg; border-radius: var(--radius-control); animation-name: stamp; }
[part="Ribbon.text"] { font: var(--text-caption); text-transform: uppercase; color: var(--on-primary); white-space: nowrap; }
@keyframes slap { from { opacity: 0; } to { opacity: 1; } }
@keyframes stamp { from { scale: 2.4; opacity: 0; } to { scale: 1; opacity: 1; } }
</style>
