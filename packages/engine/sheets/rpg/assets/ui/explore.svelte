<meta name="component" content="ExplorePage">

<!-- The explore page, as the approved HUD mockup lays it out over a stage:
     the leader's vitals, effects and companions top left; the boss's bar
     top centre over the finisher prompt; the minimap, the place and the
     tracked quest top right over the combo; along the foot the loot feed,
     the skill bar and the input hints, and the experience line under
     them. The lock-on, the damage dealt and a heal stand round the target
     at the screen's centre, and nothing else comes near the fight. The art
     is sample art of the sheet only. -->
<div id="Page">
  <img id="Stage" src={$hudStage} alt="">
  <div id="Scrim"></div>
  <div id="Hud">
    <div id="Left">
      <div id="Leader"><RpgLeader name={$leader.name} level={$leader.level} portrait={$leader.portrait} health={$leader.health} healthMax={$leader.healthMax} ghost={$leader.ghost} mana={$leader.mana} manaMax={$leader.manaMax} stamina={$leader.stamina} staminaMax={$leader.staminaMax} segment={$leader.segment} low={$leader.low} /></div>
      <div id="Effects"><RpgEffects effects={$effects} /></div>
      <div id="Companions"><RpgCompanions members={$companions} low={0.25} /></div>
    </div>
    <div id="Middle">
      <div id="Boss"><BossBar name={$boss.name} epithet={$boss.epithet} value={$boss.value} max={$boss.max} ghost={$boss.ghost} phases={$bossPhases} phase={$boss.phase} stagger={$boss.stagger} staggerCells={$boss.staggerCells} weaknesses={$weaknesses} /></div>
      <div id="Fight">
        <div id="Lock"><RpgLockOn target={$target} /></div>
        <div id="Dealt"><RpgHits hits={$hits} /></div>
        <div id="Heal"><RpgHits hits={$heals} /></div>
      </div>
      <div id="Prompt"><RpgPrompt key={$prompt.key} text={$prompt.text} disabled={$prompt.disabled} reason={$prompt.reason} /></div>
    </div>
    <div id="Right">
      <div id="Map"><RpgMinimap map={$minimap.map} centerX={$minimap.centerX} centerY={$minimap.centerY} zoom={$minimap.zoom} heading={$minimap.heading} place={$minimap.place} clock={$minimap.clock} blips={$blips} /></div>
      <div id="Quest"><RpgQuestTracker label={$quest.label} name={$quest.name} steps={$questSteps} distance={$quest.distance} /></div>
      <div id="Combo"><RpgCombo count={$combo.count} word={$combo.word} remaining={$combo.remaining} multiplier={$combo.multiplier} /></div>
    </div>
    <div id="Foot">
      <div id="Loot"><RpgLootFeed items={$loot} /></div>
      <div id="Skills"><RpgSkills slots={$slots} /></div>
      <div id="Hints"><RpgHints hints={$hints} /></div>
    </div>
    <div id="Xp"><RpgExperience level={$xp.level} value={$xp.value} /></div>
  </div>
</div>

<style>
#Page { position: fixed; inset: 0; overflow: hidden; background: #05070c; --hud-accent: rgb(127 224 255); }
#Stage { position: absolute; left: 0; top: 0; width: stretch; height: stretch; object-fit: cover; }
/* The frame darkens toward the edges the HUD stands at, never a flat band. */
#Scrim { position: absolute; left: 0; top: 0; width: stretch; height: stretch; background: radial-gradient(120% 90% at 50% 45%, transparent 55%, rgb(3 5 10 / 0.55) 100%), linear-gradient(180deg, rgb(3 5 10 / 0.5), transparent 16%, transparent 76%, rgb(3 5 10 / 0.62)); }
/* Three columns round an empty middle, each stacked on its own, over the
   skill bar's row and the experience line along the foot. Only the boss's
   bar, the fight and the prompt sit in the middle column, so the fight
   stays clear. The side columns take their content's width up
   to a share of the screen (at a large text scale their names scroll),
   the middle the rest. The skill bar is centred in its row between
   the loot feed (at the row's top) and the input hints (at its foot);
   when the row cannot hold all three the hints go up a line (wrap-reverse:
   flex-start is the row's foot). */
#Hud { position: relative; display: grid; width: stretch; height: stretch; box-sizing: border-box; column-gap: var(--space-xl); row-gap: var(--space-xs);
  grid-template-columns: fit-content(32%) minmax(0, 1fr) fit-content(26%); grid-template-rows: minmax(0, 1fr) auto auto;
  grid-template-areas: "left middle right" "foot foot foot" "xp xp xp";
  padding: calc(env(-og-hud-inset-top) + var(--space-lg)) calc(env(-og-hud-inset-right) + var(--space-xl)) env(-og-hud-inset-bottom) calc(env(-og-hud-inset-left) + var(--space-xl)); }
#Left { grid-area: left; }
#Middle { grid-area: middle; }
#Right { grid-area: right; }
#Foot { grid-area: foot; display: flex; flex-wrap: wrap-reverse; align-items: flex-start; row-gap: var(--space-sm); min-width: 0; }
#Loot { flex: 1 1 0; align-self: flex-end; }
#Hints { flex: 1 1 0; padding-bottom: var(--space-xs); }
#Xp { grid-area: xp; display: flex; margin-inline: calc(-1 * (env(-og-hud-inset-left) + var(--space-xl))) calc(-1 * (env(-og-hud-inset-right) + var(--space-xl))); }
@media (width >= 600px) {
  #Left, #Middle, #Right { display: flex; flex-direction: column; min-width: 0; min-height: 0; }
  #Left { row-gap: var(--space-sm); align-items: flex-start; }
  #Middle { align-items: center; }
  #Right { row-gap: var(--space-xs); align-items: flex-end; }
}
#Leader, #Effects, #Companions, #Loot, #Boss, #Prompt, #Skills, #Map, #Quest, #Combo, #Hints { display: flex; min-width: 0; max-width: 100%; }
#Effects { padding-left: calc(80 * var(--font-step-px)); }
#Companions { padding-top: var(--space-xs); }
#Boss { justify-content: center; }
#Prompt { margin-top: auto; padding-bottom: var(--space-md); justify-content: center; }
#Skills { flex: none; justify-content: center; }
#Quest { padding-top: var(--space-xs); }
#Combo { margin-top: auto; }
#Hints { justify-content: flex-end; }
/* The fight: the middle column's free box between the boss's bar and the
   prompt. The damage dealt stands a step under its top, right of its
   middle, bounded by the box (a long call breaks between its words), the older figures
   stepping back left; the lock-on round the target under the newest
   figure's call; the heal at the box's lower left, by the hero. Nothing
   leaves the box, so no figure reaches another plate at any text scale. */
#Fight { position: relative; flex: 1 1 0; align-self: stretch; min-height: 0; }
#Lock { position: absolute; left: calc(50% - var(--size-xxl) * 0.45); top: calc(var(--space-lg) + 60 * var(--font-step-px)); }
#Dealt { position: absolute; display: flex; left: calc(50% - 120 * var(--font-step-px)); right: 0; top: var(--space-lg); }
#Heal { position: absolute; display: flex; left: 20%; bottom: var(--space-xl); }
/* On a phone the columns dissolve into full-width rows: the boss's bar
   across the top, the leader and its effects beside the minimap, the
   companions side by side, the step in play; then the fight's band, which
   nothing else shares (the newest figure stands right of the lock-on);
   then the combo, the prompt, the skill bar and the experience line. The
   loot feed and the input hints are left out. */
@media (width < 600px) {
  #Hud { column-gap: var(--space-sm); row-gap: var(--space-xs);
    grid-template-columns: minmax(0, 1fr) auto;
    grid-template-rows: auto auto auto auto auto minmax(0, 1fr) auto auto auto auto;
    grid-template-areas: "boss boss" "leader map" "effects map" "companions companions" "quest quest" "fight fight" "combo combo" "prompt prompt" "skills skills" "xp xp";
    padding: calc(env(-og-hud-inset-top) + var(--space-sm)) calc(env(-og-hud-inset-right) + var(--space-md)) env(-og-hud-inset-bottom) calc(env(-og-hud-inset-left) + var(--space-md)); }
  #Left, #Middle, #Right, #Foot { display: contents; }
  #Xp { margin-inline: calc(-1 * (env(-og-hud-inset-left) + var(--space-md))) calc(-1 * (env(-og-hud-inset-right) + var(--space-md))); }
  #Boss { grid-area: boss; padding-bottom: var(--space-xs); }
  #Leader { grid-area: leader; align-items: center; }
  #Map { grid-area: map; align-items: flex-start; }
  #Effects { grid-area: effects; padding-left: 0; }
  #Companions { grid-area: companions; padding-top: 0; }
  #Quest { grid-area: quest; padding-top: 0; }
  #Loot { display: none; }
  #Combo { grid-area: combo; margin-top: 0; padding-bottom: 0; justify-content: flex-end; }
  #Prompt { grid-area: prompt; margin-top: 0; padding-bottom: 0; }
  #Skills { grid-area: skills; padding-bottom: var(--space-xs); }
  #Hints { display: none; }
  #Fight { grid-area: fight; }
  #Lock { left: calc(50% - var(--size-xxl) * 0.3); top: calc(var(--space-lg) + 12 * var(--font-step-px)); }
  #Dealt { left: calc(50% + 32 * var(--font-step-px)); }
}
</style>
