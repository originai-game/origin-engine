<meta name="component" content="RpgTurnOrder">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="TurnOrder.frame TurnOrder.entry">
<meta name="prop:round" content="number" data-doc="The combat round; 0 hides it.">
<meta name="prop:entries" content="list" data-doc="Who acts, in the game's initiative order from the one acting now: each { id, name, portrait, side, current, done }. side is ally, enemy or neutral; portrait a texture (og.load.texture), 0 draws the side's mark; done greys one who has acted this round.">
<meta name="prop:selected" content="string" data-doc="The id of the entry inspected; empty is none.">
<meta name="fixture:default" content='{"round":3,"selected":"","entries":[{"id":"brenna","name":"Brenna","portrait":0,"side":"ally","current":true,"done":false},{"id":"drowned1","name":"Drowned Oarsman","portrait":0,"side":"enemy","current":false,"done":false},{"id":"idris","name":"Idris","portrait":0,"side":"ally","current":false,"done":false},{"id":"heron","name":"Marsh Heron","portrait":0,"side":"neutral","current":false,"done":false},{"id":"sela","name":"Sela","portrait":0,"side":"ally","current":false,"done":true}]}'>
<meta name="fixture:empty" content='{"round":0,"selected":"","entries":[]}'>

<!--
  The rpg kit's initiative strip (README.md): who acts now and who comes
  next, each marked by side, with the round. The game owns initiative and
  the order; the strip reports choose (the entry's key) to inspect one.
  It scrolls sideways when the turn order is longer than the screen.
-->
<div id="RpgTurnOrder" part="TurnOrder.frame" role="list" aria-label={t("rpg.turns.name")} hidden={!entries.length}>
  <div id="Bar">
  <div id="Round" hidden={!round}>
    <span class="caption">{t("rpg.turns.round")}</span>
    <span class="number">{formatNumber(round)}</span>
  </div>
  <scroll id="Strip" style="overflow-x: auto; overflow-y: hidden">
    <div id="Entries">
      {#each entries as e (e.id)}
        <button id="Entry" part="TurnOrder.entry" class="entry" class:ally={e.side == "ally"} class:enemy={e.side == "enemy"} class:neutral={e.side == "neutral"} class:current={e.current} class:done={e.done} variant="quiet" role="listitem" aria-selected={e.id == selected} aria-current={e.current} aria-label={e.name} aria-description={e.side == "enemy" ? t("rpg.turns.enemy") : e.side == "ally" ? t("rpg.turns.ally") : t("rpg.turns.neutral")} animate:flip={{ duration: 260 }} onclick={choose}>
          <img class="face" src={e.portrait} alt="" hidden={!e.portrait}>
          <div class="noface" hidden={!!e.portrait}><icon class="sidemark" icon={e.side == "enemy" ? "skull" : e.side == "ally" ? "shield" : "leaf"}></icon></div>
          <div class="band"></div>
        </button>
      {/each}
    </div>
  </scroll>
  </div>
</div>

<style>
#RpgTurnOrder { width: stretch; }
#Bar { display: flex; width: stretch; gap: var(--space-sm); align-items: center; padding: var(--space-xs) var(--space-sm); }
#Round { gap: 0; align-items: center; flex-shrink: 0; }
.caption { font: var(--text-caption); color: var(--ink-muted); text-transform: uppercase; letter-spacing: 0.08em; }
.number { font: var(--text-lg); font-family: monospace; font-variant-numeric: tabular-nums; color: var(--ink); }
#Strip { width: stretch; }
#Entries { display: flex; gap: var(--space-xs); align-items: end; padding: var(--space-xxs); }
.entry { position: relative; width: var(--size-lg); height: var(--size-lg); min-width: var(--size-touch); min-height: var(--size-touch); flex-shrink: 0; padding: var(--space-xxs); overflow: hidden; background: var(--panel-raised); border-radius: var(--radius-md); }
.entry.current { width: var(--size-xl); height: var(--size-xl); }
.entry.done { opacity: 0.5; }
.face { width: stretch; height: stretch; object-fit: cover; }
.noface { width: stretch; height: stretch; align-items: center; justify-content: center; }
.sidemark { width: stretch; height: stretch; max-width: var(--size-sm); max-height: var(--size-sm); color: var(--ink-muted); }
.band { position: absolute; left: 0; bottom: 0; width: stretch; height: var(--space-xxs); background: var(--ink-faint); }
.entry.ally .band { background: var(--team1); }
.entry.enemy .band { background: var(--danger); }
.entry.enemy .sidemark { color: var(--danger); }
.entry.ally .sidemark { color: var(--team1); }
</style>
