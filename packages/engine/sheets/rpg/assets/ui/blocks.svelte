<meta name="component" content="BlocksPage">

<!-- A game's own HUD, laid out by the game from the RPG HUD's blocks
     alone (HudPortrait, HudBar, HudSlot, HudEffect, HudHint,
     HudLootEntry), unlike the explore page's ready-made plates: the
     player's frame and its skill strip joined at the screen's foot, the
     target's frame at the top with its effects, the party as a column of
     small frames on the left, the loot on the right and the hints under
     it. The page owns every place and size (the blocks' size tokens);
     the blocks only draw. The art is sample art of the sheet only. -->
<div id="Page">
  <img id="Stage" src={$hudStage} alt="">
  <div id="Scrim"></div>

  <div id="Target">
    <div id="TargetPortrait"><HudPortrait portrait={0} dimmed={false} /></div>
    <div id="TargetBody">
      <span class="name">{$boss.name}</span>
      <div id="TargetHealth"><HudBar value={$boss.value} max={$boss.max} kind="health" ghost={$boss.ghost} segment={0} low={false} showValue={true} name={$boss.name} /></div>
      <div id="TargetEffects">
        {#each $effects as e (e.id)}<HudEffect name={e.name} icon={e.icon} glyph={e.glyph} remaining={e.remaining} label={e.label} harmful={e.harmful} />{/each}
      </div>
    </div>
  </div>

  <ul id="Party">
    {#each $companions as m (m.id)}
    <li class="member" class:fallen={m.fallen}>
      <HudPortrait portrait={m.portrait} dimmed={m.fallen} />
      <div class="member-bars">
        <span class="member-name">{m.name}</span>
        <HudBar value={m.health} max={m.healthMax} kind="health" ghost={m.ghost} segment={0} low={!m.fallen && m.health <= m.healthMax * 0.25} showValue={false} name="" />
        <div class="member-mana"><HudBar value={m.mana} max={m.manaMax} kind="mana" ghost={0} segment={0} low={false} showValue={false} name="" /></div>
      </div>
    </li>
    {/each}
  </ul>

  <div id="Side">
    {#each $loot as p (p.id)}<HudLootEntry name={p.name} icon={p.icon} glyph={p.glyph} rarity={p.rarity} label={p.label} quantity={p.quantity} currency={p.currency} fading={p.fading} />{/each}
    <div id="Hints">{#each $hints as h (h.id)}<HudHint key={h.key} label={h.label} />{/each}</div>
  </div>

  <div id="Dock">
    <div id="Player">
      <div id="PlayerPortrait"><HudPortrait portrait={$leader.portrait} dimmed={false} /></div>
      <div id="PlayerBars">
        <div id="PlayerHealth"><HudBar value={$leader.health} max={$leader.healthMax} kind="health" ghost={$leader.ghost} segment={0} low={false} showValue={true} name="" /></div>
        <div id="PlayerMana"><HudBar value={$leader.mana} max={$leader.manaMax} kind="mana" ghost={0} segment={0} low={false} showValue={false} name="" /></div>
        <div id="PlayerStamina"><HudBar value={$leader.stamina} max={$leader.staminaMax} kind="stamina" ghost={0} segment={$leader.segment} low={false} showValue={false} name="" /></div>
      </div>
    </div>
    <ul id="Strip">
      {#each $slots as s (s.id)}<li class="slot" class:kind-item={(s.kind) == 'item'}><HudSlot name={s.name} icon={s.icon} glyph={s.glyph} key={s.key} kind={s.kind} cooldown={s.cooldown} time={s.time} count={s.count} ready={s.ready} unaffordable={s.unaffordable} reason={s.reason} charges={s.charges} chargesMax={s.chargesMax} charge={s.charge} /></li>{/each}
    </ul>
  </div>
</div>

<style>
#Page { position: fixed; inset: 0; overflow: hidden; background: #05070c; color: var(--hud-ink); --hud-accent: rgb(127 224 255); }
#Stage { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
#Scrim { position: absolute; left: 0; top: 0; width: stretch; height: stretch; background: linear-gradient(180deg, rgb(3 5 10 / 0.5), transparent 20%, transparent 70%, rgb(3 5 10 / 0.7)); }
.name, .member-name { font-size: var(--hud-size-name); font-weight: var(--hud-label-weight); white-space: nowrap; text-overflow: ellipsis; overflow: hidden; text-shadow: var(--hud-shadow); }
ul { margin: 0; padding: 0; list-style: none; }

/* The target's frame, top centre: portrait, name over a thick bar with its
   figure, its effects under the bar. */
#Target { position: absolute; top: var(--space-lg); left: 50%; translate: -50% 0; display: flex; align-items: center; gap: var(--space-sm); inline-size: min(calc(420 * var(--font-step-px)), 70%); }
#TargetPortrait { --hud-portrait-size: calc(56 * var(--font-step-px)); }
#TargetBody { display: flex; flex-direction: column; gap: var(--space-xxs); flex: 1 1 0; min-inline-size: 0; }
#TargetHealth { --hud-bar-size: calc(20 * var(--font-step-px)); }
#TargetEffects { display: flex; gap: calc(6 * var(--font-step-px)); padding-block-start: var(--space-xxs); }

/* The party, a column of small frames on the left. */
#Party { position: absolute; top: calc(30% - 40 * var(--font-step-px)); left: var(--space-xl); display: flex; flex-direction: column; gap: var(--space-sm); inline-size: calc(200 * var(--font-step-px)); max-inline-size: 40%; --hud-portrait-size: calc(36 * var(--font-step-px)); }
.member { display: flex; align-items: center; gap: var(--space-xs); }
.member.fallen { opacity: 0.6; }
.member-bars { display: flex; flex-direction: column; gap: calc(3 * var(--font-step-px)); flex: 1 1 0; min-inline-size: 0; --hud-bar-size: calc(7 * var(--font-step-px)); }
.member-mana { --hud-bar-size: calc(4 * var(--font-step-px)); }

/* The loot and the hints on the right. */
#Side { position: absolute; right: var(--space-xl); bottom: calc(150 * var(--font-step-px)); display: flex; flex-direction: column; align-items: flex-end; gap: calc(5 * var(--font-step-px)); max-inline-size: 34%; }
#Hints { display: flex; gap: var(--space-md); padding-block-start: var(--space-sm); }

/* The dock: the player's frame joined to an even strip of slots. */
#Dock { position: absolute; bottom: var(--space-lg); left: 50%; translate: -50% 0; display: flex; align-items: flex-end; gap: var(--space-lg); max-inline-size: calc(100% - 2 * var(--space-lg)); }
#Player { display: flex; align-items: center; gap: var(--space-sm); inline-size: calc(250 * var(--font-step-px)); flex: 0 1 auto; min-inline-size: 0; }
#PlayerPortrait { filter: var(--hud-lift); --hud-portrait-size: calc(72 * var(--font-step-px)); }
#PlayerBars { display: flex; flex-direction: column; gap: calc(4 * var(--font-step-px)); flex: 1 1 0; min-inline-size: 0; }
#PlayerHealth { --hud-bar-size: calc(18 * var(--font-step-px)); }
#PlayerMana { --hud-bar-size: calc(8 * var(--font-step-px)); }
#PlayerStamina { --hud-bar-size: calc(6 * var(--font-step-px)); inline-size: 70%; }
/* Every slot one size, the ultimate too: this game's strip is even. */
#Strip { display: flex; align-items: flex-end; gap: calc(6 * var(--font-step-px)); --hud-slot-size: calc(var(--size-xl) * 0.75); }

/* On a phone the dock stacks, the party and the side drop to the essentials. */
#Target:size(compact) { inline-size: calc(100% - 2 * var(--space-md)); top: var(--space-sm); }
#Party:size(compact) { left: var(--space-md); top: calc(110 * var(--font-step-px)); inline-size: calc(150 * var(--font-step-px)); }
#Side:size(compact) { right: var(--space-md); bottom: calc(220 * var(--font-step-px)); max-inline-size: 60%; }
#Hints:size(compact) { display: none; }
#Dock:size(compact) { flex-direction: column; align-items: stretch; gap: var(--space-sm); inline-size: calc(100% - 2 * var(--space-md)); }
#Player:size(compact) { inline-size: auto; }
#PlayerPortrait:size(compact) { --hud-portrait-size: calc(48 * var(--font-step-px)); }
#Strip:size(compact) { justify-content: center; --hud-slot-size: calc(var(--size-xl) * 0.55); }
.slot.kind-item:size(compact) { display: none; }
</style>
