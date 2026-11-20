<meta name="component" content="PrimitivesPage">

<section class="page">
  <h1>Items and players</h1>
  <p class="intro">Selectable items keep their count, shortcut and cooldown readable. Locked, loading and unavailable slots do not activate.</p>
  <h2>Item slots</h2>
  <div class="gallery">
    <ItemSlot name="Healing draught" icon="potion" count={12} rarity="rare" keyLabel="Q" showName={true} />
    <ItemSlot name="Storm crystal" icon="gem" count={3} rarity="epic" cooldown={3} cooldownMax={8} selected={true} keyLabel="2" showName={true} />
    <ItemSlot name="Sealed relic" icon="key" rarity="legendary" locked={true} showName={true} />
    <ItemSlot name="Spent potion" icon="potion" disabled={true} showName={true} />
    <ItemSlot loading={true} showName={true} />
    <ItemSlot empty={true} showName={true} />
  </div>
  <h2>Avatars</h2>
  <div class="gallery">
    <Avatar name="Brenna Vale" portrait={$charPortrait} level={12} status="online" />
    <Avatar name="Idris" level={8} status="away" />
    <Avatar name="Sela" status="busy" frame="veteran" />
    <Avatar name="Maren" status="offline" />
    <Avatar name="Visitor" loading={true} />
    <Avatar />
    <div class="identity"><Avatar name="Brenna Vale" portrait={$charPortrait} showName={false} level={12} status="online" /><h2>Brenna Vale</h2></div>
  </div>
  <h2>HUD blocks</h2>
  <p class="intro">The action HUD's plates are built from these blocks: a portrait, a bar and a skill slot. A game uses them by name to lay out a plate of its own, as the last row does.</p>
  <div class="stage">
    <div class="tall"><RpgPortrait portrait={$charPortrait} dimmed={false} /></div>
    <RpgPortrait portrait={0} dimmed={false} />
    <RpgPortrait portrait={$charPortrait} dimmed={true} />
    <div class="bars">
      <div class="fat"><RpgBar value={2184} max={3420} kind="health" ghost={2668} segment={0} low={false} showValue={false} name="" /></div>
      <div class="fat"><RpgBar value={420} max={3420} kind="health" ghost={900} segment={0} low={true} showValue={false} name="" /></div>
      <RpgBar value={72} max={100} kind="mana" ghost={0} segment={0} low={false} showValue={false} name="" />
      <RpgBar value={5} max={8} kind="stamina" ghost={0} segment={1} low={false} showValue={false} name="" />
      <RpgBar value={62} max={100} kind="xp" ghost={0} segment={0} low={false} showValue={false} name="" />
    </div>
    <RpgSkillSlot name="Ember slash" icon={0} glyph="sword" key="1" kind="skill" cooldown={0} time="" count={0} ready={true} unaffordable={false} reason="" charges={0} chargesMax={0} charge={0} />
    <RpgSkillSlot name="Arc step" icon={0} glyph="star" key="2" kind="skill" cooldown={0.4} time="3" count={0} ready={false} unaffordable={false} reason="" charges={1} chargesMax={3} charge={0} />
    <RpgSkillSlot name="Void rend" icon={0} glyph="skull" key="3" kind="skill" cooldown={0} time="" count={0} ready={false} unaffordable={true} reason="No mana" charges={0} chargesMax={0} charge={0} />
    <div class="small"><RpgSkillSlot name="Healing tonic" icon={0} glyph="potion" key="Z" kind="item" cooldown={0} time="" count={5} ready={false} unaffordable={false} reason="" charges={0} chargesMax={0} charge={0} /></div>
    <div class="large"><RpgSkillSlot name="Cataclysm" icon={0} glyph="fire" key="R" kind="ultimate" cooldown={0} time="" count={0} ready={false} unaffordable={false} reason="" charges={0} chargesMax={0} charge={0.58} /></div>
  </div>
  <div class="stage">
    <div class="target">
      <div class="target-bars">
        <span class="target-name">Ashen Colossus</span>
        <div class="fat"><RpgBar value={1380} max={2400} kind="health" ghost={1700} segment={0} low={false} showValue={false} name="Ashen Colossus" /></div>
        <RpgBar value={3} max={6} kind="stamina" ghost={0} segment={1} low={false} showValue={false} name="Guard" />
      </div>
      <RpgPortrait portrait={0} dimmed={false} />
    </div>
    <div class="quick">
      <RpgSkillSlot name="Healing tonic" icon={0} glyph="potion" key="Z" kind="item" cooldown={0} time="" count={5} ready={false} unaffordable={false} reason="" charges={0} chargesMax={0} charge={0} />
      <RpgSkillSlot name="Ether" icon={0} glyph="drop" key="X" kind="item" cooldown={0} time="" count={2} ready={false} unaffordable={false} reason="" charges={0} chargesMax={0} charge={0} />
      <RpgSkillSlot name="Ward" icon={0} glyph="shield" key="C" kind="item" cooldown={0.7} time="9" count={1} ready={false} unaffordable={false} reason="" charges={0} chargesMax={0} charge={0} />
    </div>
  </div>
</section>
<style>
.page { display: flex; flex-direction: column; width: stretch; height: stretch; overflow-y: auto; padding: var(--space-lg); gap: var(--space-md); background: var(--panel); color: var(--ink); }
h1, h2, .intro { margin: 0; }
.intro { max-inline-size: var(--measure-md); color: var(--ink-muted); }
.gallery { display: flex; flex-wrap: wrap; align-items: flex-start; gap: var(--space-lg); padding: var(--space-xs); }
.identity { display: flex; align-items: center; gap: var(--space-md); }
/* The HUD's blocks stand over the scene: a dark ground like one. A block's
   size is a token set on its place. */
.stage { display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-lg); padding: var(--space-lg); background: linear-gradient(180deg, #4b4038, #16120e); color: var(--hud-ink); }
.tall { --rpg-portrait-size: calc(68 * var(--font-step-px)); }
.bars { display: flex; flex-direction: column; gap: var(--space-xs); inline-size: calc(240 * var(--font-step-px)); max-inline-size: 100%; }
.fat { --rpg-bar-size: calc(16 * var(--font-step-px)); }
.small { --rpg-skill-slot-size: calc(var(--size-xl) * 0.7); }
.large { --rpg-skill-slot-size: calc(var(--size-xl) * 1.3); }
/* A plate laid out by the sheet from the same blocks: a target's bars
   beside its portrait, and a column of quick items. */
.target { display: flex; align-items: center; gap: var(--space-sm); inline-size: calc(300 * var(--font-step-px)); max-inline-size: 100%; --rpg-portrait-size: calc(52 * var(--font-step-px)); }
.target-bars { display: flex; flex-direction: column; gap: var(--space-xxs); flex: 1 1 0; min-inline-size: 0; }
.target-name { align-self: flex-end; font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); text-shadow: var(--hud-shadow); }
.quick { display: flex; gap: var(--space-md); --rpg-skill-slot-size: calc(var(--size-xl) * 0.6); }@media (width < 600px) { .page { gap: var(--space-sm); } .gallery { gap: var(--space-md); --avatar-size: var(--size-xl); } }
</style>
