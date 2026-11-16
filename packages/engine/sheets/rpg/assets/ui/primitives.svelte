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
  </div>
</section>

<style>
.page { display: flex; flex-direction: column; width: stretch; height: stretch; overflow-y: auto; padding: var(--space-lg); gap: var(--space-md); background: var(--panel); color: var(--ink); }
h1, h2, .intro { margin: 0; }
.intro { max-inline-size: var(--measure-md); color: var(--ink-muted); }
.gallery { display: flex; flex-wrap: wrap; align-items: flex-start; gap: var(--space-lg); padding: var(--space-xs); }
@media (width < 600px) { .page { gap: var(--space-sm); } .gallery { gap: var(--space-md); --avatar-size: var(--size-xl); } }
</style>
