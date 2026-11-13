<meta name="component" content="Nameplate">
<meta name="parts" content="Nameplate.frame Nameplate.icon Nameplate.badge Nameplate.name Nameplate.subtitle Nameplate.well Nameplate.fill Nameplate.shield Nameplate.cast Nameplate.spell">
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
  <div id="Plate" part="Nameplate.frame" class="bare" class:friendly={reaction == 'friendly'} class:hostile={reaction == 'hostile'} class:player={reaction == 'player'} class:boss={boss} class:targeted={targeted}>
    <div id="Head">
      <og-icon id="Icon" part="Nameplate.icon" icon={icon} hidden={!icon}></og-icon>
      <span id="Level" part="Nameplate.badge" hidden={!level}>{level}</span>
      <span id="Name" part="Nameplate.name">{name}</span>
    </div>
    <span id="Title" part="Nameplate.subtitle" hidden={!title}>{title}</span>
    <div id="Health" part="Nameplate.well" hidden={health < 0} style:--hp={health} style:--shield={shield}>
      <div id="Fill" part="Nameplate.fill"></div>
      <div id="Shield" part="Nameplate.shield"></div>
    </div>
    <div id="Cast" part="Nameplate.well" hidden={!castName} style:--cast={castProgress}>
      <div id="CastFill" part="Nameplate.cast"></div>
      <span id="CastName" part="Nameplate.spell">{castName}</span>
    </div>
  </div>
</anchor>

<style>
#Nameplate { align-items: center; }
[part="Nameplate.frame"] { align-items: center; gap: var(--space-xxs); --tone: var(--warn); --bar: var(--size-xxl); }
[part="Nameplate.frame"].friendly { --tone: var(--success); }
[part="Nameplate.frame"].hostile { --tone: var(--danger); }
[part="Nameplate.frame"].player { --tone: var(--info); }
[part="Nameplate.frame"].targeted { scale: var(--motion-scale-swell); z-index: 1; }
#Head { display: flex; gap: var(--space-xs); align-items: center; }
[part="Nameplate.icon"] { width: var(--size-xs); height: var(--size-xs); color: var(--tone); }
[part="Nameplate.badge"] { font: var(--text-caption); padding: 0 var(--space-xs); }
.boss [part="Nameplate.badge"] { border: var(--stroke-hairline) solid var(--gold); }
[part="Nameplate.name"] { font: var(--text-sm); color: var(--tone); white-space: nowrap; text-shadow: 0 var(--stroke-hairline) var(--stroke-regular) var(--scene-shadow); }
.targeted [part="Nameplate.name"] { color: var(--scene-ink); }
[part="Nameplate.subtitle"] { font: var(--text-caption); color: var(--ink-muted); white-space: nowrap; text-shadow: 0 var(--stroke-hairline) var(--stroke-regular) var(--scene-shadow); }
[part="Nameplate.well"] { width: var(--bar); overflow: hidden; }
#Health { height: var(--space-sm); border: var(--stroke-hairline) solid transparent; }
.targeted #Health { border: var(--stroke-hairline) solid var(--highlight); }
.boss #Health { border: var(--stroke-hairline) solid var(--gold); }
[part="Nameplate.fill"] { width: calc(var(--hp) * var(--bar)); height: stretch; background: var(--tone); }
[part="Nameplate.shield"] { position: absolute; top: 0; inset-inline-start: calc(var(--hp) * var(--bar)); width: calc(var(--shield) * var(--bar)); height: stretch; background: var(--scene-ink); }
#Cast { height: var(--space-md); align-items: center; justify-content: center; }
[part="Nameplate.cast"] { position: absolute; top: 0; inset-inline-start: 0; width: calc(var(--cast) * var(--bar)); height: stretch; background: var(--warn); }
[part="Nameplate.spell"] { font: var(--text-caption); color: var(--ink); white-space: nowrap; }
</style>
