<meta name="component" content="Ribbon">
<meta name="prop:text" content="string" data-doc="The ribbon's words (NEW, SALE).">
<meta name="prop:corner" content="string" data-doc="Where it sits: topRight (empty), topLeft, bottomRight, bottomLeft or center (a stamp).">
<meta name="prop:tone" content="string" data-doc="Its tone: primary (empty), success, warn, danger or info.">
<meta name="fixture:default" content='{"text":"New","corner":"","tone":"danger"}'>

<div id="Corner" class:topLeft={corner == 'topLeft'} class:bottomRight={corner == 'bottomRight'} class:bottomLeft={corner == 'bottomLeft'} class:center={corner == 'center'} class:success={tone == 'success'} class:warn={tone == 'warn'} class:danger={tone == 'danger'} class:info={tone == 'info'}>
  <div id="Ribbon" role="status" aria-label={text}><span id="Text">{text}</span></div>
</div>

<style>
#Corner { width: 160px; height: 110px; background: var(--panel); border-radius: 8px; overflow: hidden; --tone: var(--primary); }
#Corner.success { --tone: var(--success); }
#Corner.warn { --tone: var(--warn); }
#Corner.danger { --tone: var(--danger); }
#Corner.info { --tone: var(--info); }
#Ribbon { position: absolute; top: 18px; inset-inline-start: 50px; width: 150px; padding: 3px 0; align-items: center; background: linear-gradient(180deg, #ffffff2e, #00000040), var(--tone); border: 1px solid #ffffff55; box-shadow: 0 2px 4px #00000073; rotate: 45deg; animation: slap 260ms ease-out; }
.topLeft #Ribbon { inset-inline-start: -40px; rotate: -45deg; }
.bottomRight #Ribbon { top: 70px; rotate: -45deg; }
.bottomLeft #Ribbon { top: 70px; inset-inline-start: -40px; rotate: 45deg; }
.center #Ribbon { top: 38px; inset-inline-start: 20px; width: 120px; rotate: -8deg; border-radius: 4px; animation: stamp 260ms ease-out; }
#Text { font: var(--text-caption); text-transform: uppercase; color: var(--on-primary); white-space: nowrap; }
@keyframes slap { from { opacity: 0; } to { opacity: 1; } }
@keyframes stamp { from { scale: 2.4; opacity: 0; } to { scale: 1; opacity: 1; } }
</style>
