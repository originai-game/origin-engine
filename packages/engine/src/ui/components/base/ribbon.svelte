<meta name="component" content="Ribbon">
<meta name="parts" content="Ribbon.frame Ribbon.ribbon Ribbon.text">
<meta name="prop:text" content="string" data-doc="The ribbon's words (NEW, SALE).">
<meta name="prop:corner" content="string" data-doc="Where it sits: topRight (empty), topLeft, bottomRight, bottomLeft or center (a stamp).">
<meta name="prop:tone" content="string" data-doc="Its tone: primary (empty), success, warn, danger or info.">
<meta name="fixture:default" content='{"text":"New","corner":"","tone":"danger"}'>
<meta name="fixture:topLeft" content='{"text":"Sale","corner":"topLeft","tone":"success"}'>
<meta name="fixture:bottomRight" content='{"text":"−50%","corner":"bottomRight","tone":"warn"}'>
<meta name="fixture:bottomLeft" content='{"text":"Rank 1","corner":"bottomLeft","tone":"info"}'>
<meta name="fixture:stamp" content='{"text":"Sold out","corner":"center","tone":"primary"}'>
<meta name="fixture:overflow" content='{"text":"Limited edition of the Ninefold Gate","corner":"","tone":"danger"}'>

<!-- A strip across a corner of whatever it lies over (or a stamp across
     its middle). Its box only holds and cuts the strip: it draws nothing
     of its own, so the card under it shows. A game puts the box in the
     corner it names of a positioned card that clips (`overflow: hidden`),
     and the card's own corner cuts the strip's ends. -->
<div id="Corner" part="Ribbon.frame" class="bare" class:topLeft={corner == 'topLeft'} class:bottomRight={corner == 'bottomRight'} class:bottomLeft={corner == 'bottomLeft'} class:center={corner == 'center'}>
  <div id="Ribbon" part="Ribbon.ribbon" data-layout="stack" data-gap="none" data-tone={tone || 'primary'} data-motion={corner == 'center' ? 'stamp' : 'enter'} role="status" aria-label={text}><span id="Text" class="text" part="Ribbon.text">{text}</span></div>
</div>

<style>
:host { position: relative; inline-size: var(--size-4xl); block-size: calc(var(--size-xxl) + var(--space-lg)); overflow: hidden; pointer-events: none; }
[part="Ribbon.ribbon"] { position: absolute; top: var(--space-lg); inset-inline-start: var(--space-3xl); width: var(--size-4xl); padding: var(--space-xxs) 0; align-items: center; rotate: 45deg; }
.topLeft [part="Ribbon.ribbon"] { inset-inline-start: calc(var(--space-3xl) * -1); rotate: -45deg; }
.bottomRight [part="Ribbon.ribbon"] { top: var(--size-xl); rotate: -45deg; }
.bottomLeft [part="Ribbon.ribbon"] { top: var(--size-xl); inset-inline-start: calc(var(--space-3xl) * -1); rotate: 45deg; }
.center [part="Ribbon.ribbon"] { top: calc(var(--space-xl) + var(--space-lg)); inset-inline-start: var(--space-lg); width: var(--size-3xl); rotate: -8deg; }
[part="Ribbon.text"] { max-inline-size: calc(var(--size-4xl) - var(--space-3xl) * 2); white-space: nowrap; text-overflow: ellipsis; }
</style>
