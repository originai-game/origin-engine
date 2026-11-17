<meta name="component" content="Nameplate">
<meta name="parts" content="Nameplate.frame Nameplate.icon Nameplate.badge Nameplate.name Nameplate.subtitle Nameplate.well Nameplate.fill Nameplate.shield Nameplate.cast Nameplate.spell Nameplate.track Nameplate.tag Nameplate.caption">
<meta name="prop:entity" content="number" data-doc="The unit it sits over, by its bits (0: use at).">
<meta name="prop:at" content="string" data-default="0 0 0" data-doc="A world point it sits over without an entity (x y z).">
<meta name="prop:height" content="number" data-doc="World units above the point (its head).">
<meta name="prop:name" content="string" data-doc="The name.">
<meta name="prop:title" content="string" data-doc="A line under it (a guild, a role); empty is none.">
<meta name="prop:level" content="string" data-doc="Its level on a badge; empty is none.">
<meta name="prop:reaction" content="string" data-doc="Its colour: neutral (empty), friendly, hostile or player.">
<meta name="prop:health" content="number" data-default="-1" data-doc="Health 0 to 1 for a bar under the name; negative draws none.">
<meta name="prop:shield" content="number" data-doc="An absorb over the health, 0 to 1.">
<meta name="prop:castName" content="string" data-doc="What it is casting; empty is nothing.">
<meta name="prop:castProgress" content="number" data-doc="How far the cast is, 0 to 1.">
<meta name="prop:icon" content="string" data-doc="A class, quest or role icon before the name; empty is none.">
<meta name="prop:boss" content="bool" data-doc="A boss or elite: a gold frame.">
<meta name="prop:targeted" content="bool" data-doc="The player's target: larger, outlined, drawn over the rest.">
<meta name="prop:maxDistance" content="number" data-default="80" data-doc="Hidden past this many world units (80).">
<meta name="fixture:default" content='{"entity":0,"at":"0 0 0","height":2.1,"name":"Grimtooth","title":"Warlord","level":"32","reaction":"hostile","health":0.62,"shield":0.15,"castName":"Cleave","castProgress":0.4,"icon":"skull","boss":true,"targeted":true,"maxDistance":80}'>

<anchor id="Nameplate" entity={entity || null} at={at} offset={"0 " + height + " 0"} occlusion="fade" declutter="stack" priority={targeted ? 1100 : 100} max-distance={maxDistance}>
  <div id="Plate" part="Nameplate.frame" class="bare" data-layout="stack" data-gap="hair" class:friendly={reaction == 'friendly'} class:hostile={reaction == 'hostile'} class:player={reaction == 'player'} class:boss={boss} class:targeted={targeted}>
    <div id="Head" data-layout="row" data-gap="tight">
      {#if icon}<og-icon id="Icon" part="Nameplate.icon" icon={icon}></og-icon>{/if}
      {#if level}<span id="Level" part="Nameplate.badge">{level}</span>{/if}
      <div part="Nameplate.tag"><span id="Name" class="name" part="Nameplate.name">{name}</span></div>
    </div>
    {#if title}<div part="Nameplate.tag"><span id="Title" class="subtitle" part="Nameplate.subtitle Nameplate.caption">{title}</span></div>{/if}
    {#if health >= 0}<div id="Health" part="Nameplate.well Nameplate.track" data-layout="overlay" style:--hp={health} style:--shield={shield}>
      <div id="Fill" part="Nameplate.fill" data-anchor="top-start"></div>
      <div id="Shield" part="Nameplate.shield Nameplate.fill" data-anchor="top-start"></div>
    </div>{/if}
    {#if castName}<div id="Cast" part="Nameplate.well Nameplate.track" data-layout="overlay" style:--cast={castProgress}>
      <div id="CastFill" part="Nameplate.cast Nameplate.fill" data-anchor="top-start" data-tone="warn"></div>
      <span id="CastName" class="spell" part="Nameplate.spell" data-anchor="center">{castName}</span>
    </div>{/if}
  </div>
</anchor>

<style>
:host { text-shadow: var(--tag-halo); }
[part="Nameplate.frame"] { align-items: center; --tone: var(--warn); --bar: var(--size-xxl); --fill-color: var(--tone); --layout-anchor-inset: 0; }
[part="Nameplate.frame"].friendly { --tone: var(--success); }
[part="Nameplate.frame"].hostile { --tone: var(--danger); }
[part="Nameplate.frame"].player { --tone: var(--info); }
[part="Nameplate.frame"].targeted { scale: var(--motion-scale-swell); z-index: 1; }
[part="Nameplate.icon"] { width: var(--size-xs); height: var(--size-xs); color: var(--fill-color); }
.boss [part="Nameplate.badge"] { border: var(--cell-rim-width) solid var(--cell-rim); --cell-rim: var(--gold); }
/* Over the scene: the tag tokens outline it, and give it a ground where a pack sets one. */
[part="Nameplate.name"] { --name-size: var(--font-step-sm); --name-ink: var(--tone); white-space: nowrap; }
.targeted [part="Nameplate.name"] { --name-ink: var(--scene-ink); }
[part~="Nameplate.subtitle"] { white-space: nowrap; }
[part~="Nameplate.well"] { width: var(--bar); overflow: hidden; }
#Health { height: var(--space-sm); border: var(--cell-rim-width) solid var(--cell-rim); --cell-rim: transparent; }
.targeted #Health { --cell-rim: var(--highlight); }
.boss #Health { --cell-rim: var(--gold); }
#Fill { block-size: 100%; inline-size: calc(var(--hp) * 100%); }
#Shield { block-size: 100%; inset-inline-start: calc(var(--hp) * 100%); inline-size: calc(var(--shield) * 100%); --fill-color: var(--scene-ink); }
#Cast { height: var(--space-md); }
#CastFill { block-size: 100%; inline-size: calc(var(--cast) * 100%); }
/* Across the bar (a name longer than it ends in an ellipsis), its glyphs
   inked for the warn fill as far as the cast has run and in the well's
   ink past it: it reads over both. */
[part="Nameplate.spell"] { width: var(--bar); text-align: center; font: var(--caption-text); white-space: nowrap; text-overflow: ellipsis; color: transparent; background: linear-gradient(90deg, var(--ink-on-warn) calc(var(--cast) * 100%), var(--ink) calc(var(--cast) * 100%)); background-clip: text; }
</style>
