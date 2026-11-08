<meta name="component" content="Nameplate">
<meta name="prop:entity" content="number" data-doc="The unit it sits over, by its bits (0: use at).">
<meta name="prop:at" content="string" data-doc="A world point it sits over without an entity (x y z).">
<meta name="prop:height" content="number" data-doc="World units above the point (its head).">
<meta name="prop:name" content="string" data-doc="The name.">
<meta name="prop:title" content="string" data-doc="A line under it (a guild, a role); empty is none.">
<meta name="prop:level" content="string" data-doc="Its level on a badge; empty is none.">
<meta name="prop:reaction" content="string" data-doc="Its colour: neutral (empty), friendly, hostile or player.">
<meta name="prop:health" content="number" data-doc="Health 0 to 1 for a bar under the name; negative draws none.">
<meta name="prop:shield" content="number" data-doc="An absorb over the health, 0 to 1.">
<meta name="prop:castName" content="string" data-doc="What it is casting; empty is nothing.">
<meta name="prop:castProgress" content="number" data-doc="How far the cast is, 0 to 1.">
<meta name="prop:icon" content="string" data-doc="A class, quest or role icon before the name; empty is none.">
<meta name="prop:boss" content="bool" data-doc="A boss or elite: a gold frame.">
<meta name="prop:targeted" content="bool" data-doc="The player's target: larger, outlined, drawn over the rest.">
<meta name="prop:maxDistance" content="number" data-doc="Hidden past this many world units (80).">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","height":2.1,"name":"Grimtooth","title":"Warlord","level":"32","reaction":"hostile","health":0.62,"shield":0.15,"castName":"Cleave","castProgress":0.4,"icon":"skull","boss":true,"targeted":true,"maxDistance":80}'>

<anchor id="Nameplate" entity={entity || null} at={at} offset={"0 " + height + " 0"} occlusion="fade" declutter="stack" priority={targeted ? 1100 : 100} max-distance={maxDistance}>
  <div id="Plate" class:friendly={reaction == 'friendly'} class:hostile={reaction == 'hostile'} class:player={reaction == 'player'} class:boss={boss} class:targeted={targeted}>
    <div id="Head">
      <icon id="Icon" icon={icon} hidden={!icon}></icon>
      <span id="Level" hidden={!level}>{level}</span>
      <span id="Name">{name}</span>
    </div>
    <span id="Title" hidden={!title}>{title}</span>
    <div id="Health" hidden={health < 0} style:--hp={health} style:--shield={shield}>
      <div id="Fill"></div>
      <div id="Shield"></div>
    </div>
    <div id="Cast" hidden={!castName} style:--cast={castProgress}>
      <div id="CastFill"></div>
      <span id="CastName">{castName}</span>
    </div>
  </div>
</anchor>

<style>
#Nameplate { align-items: center; }
#Plate { align-items: center; gap: 2px; --tone: var(--warn); }
#Plate.friendly { --tone: var(--success); }
#Plate.hostile { --tone: var(--danger); }
#Plate.player { --tone: var(--info); }
#Plate.targeted { scale: 1.15; z-index: 1; }
#Head { display: flex; gap: 4px; align-items: center; }
#Icon { width: 14px; height: 14px; color: var(--tone); }
#Level { font: var(--text-caption); padding: 0 4px; border-radius: 4px; background: #000000b3; color: var(--ink); }
.boss #Level { background: #6b4a00ff; border: 1px solid var(--gold); }
#Name { font: var(--text-sm); color: var(--tone); white-space: nowrap; text-shadow: 0 1px 2px #000000ff; }
.targeted #Name { color: var(--ink); }
#Title { font: var(--text-caption); color: var(--ink-muted); white-space: nowrap; text-shadow: 0 1px 2px #000000ff; }
#Health { width: 96px; height: 7px; border-radius: 2px; overflow: hidden; background: #000000b3; border: 1px solid #00000000; }
.targeted #Health { border: 1px solid #ffffffcc; }
.boss #Health { border: 1px solid var(--gold); }
#Fill { width: calc(var(--hp) * 96px); height: stretch; background: var(--tone); }
#Shield { position: absolute; top: 0; inset-inline-start: calc(var(--hp) * 96px); width: calc(var(--shield) * 96px); height: stretch; background: #ffffffb3; }
#Cast { width: 96px; height: 12px; border-radius: 2px; overflow: hidden; background: #000000b3; align-items: center; justify-content: center; }
#CastFill { position: absolute; top: 0; inset-inline-start: 0; width: calc(var(--cast) * 96px); height: stretch; background: var(--warn); }
#CastName { font: var(--text-caption); color: var(--ink); white-space: nowrap; }
</style>
