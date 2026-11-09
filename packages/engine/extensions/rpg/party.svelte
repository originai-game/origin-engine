<meta name="component" content="RpgParty">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="Party.member Party.portrait Party.health Party.resource">
<meta name="prop:members" content="list" data-doc="The party in the game's order: each { id, name, portrait, level, health, healthMax, resource, resourceMax, status, active, down }. portrait is a texture (og.load.texture), 0 draws the person icon; resource is mana, rage or the like (resourceMax 0 hides it); status the game's conditions as words (Poisoned · Blessed), empty for none; active the one acting; down a member at 0 health.">
<meta name="prop:selected" content="string" data-doc="The id of the member the player controls or inspects.">
<meta name="prop:orientation" content="string" data-doc="vertical (default): a column of members, each a portrait beside its name and bars, for a side of the screen; horizontal: a row of portrait cards, for a phone's edge or the bottom of a HUD.">
<meta name="prop:compact" content="bool" data-doc="Portraits and bars only, for a HUD corner: names and status go to the accessible description.">
<meta name="fixture:default" content='{"selected":"brenna","orientation":"vertical","compact":false,"members":[{"id":"brenna","name":"Brenna","portrait":0,"level":7,"health":48,"healthMax":64,"resource":0,"resourceMax":0,"status":"","active":true,"down":false},{"id":"idris","name":"Idris","portrait":0,"level":7,"health":21,"healthMax":38,"resource":14,"resourceMax":30,"status":"Concentrating","active":false,"down":false},{"id":"sela","name":"Sela","portrait":0,"level":6,"health":0,"healthMax":42,"resource":0,"resourceMax":0,"status":"Unconscious","active":false,"down":true}]}'>
<meta name="fixture:empty" content='{"selected":"","orientation":"vertical","compact":false,"members":[]}'>

<!--
  The rpg kit's party (README.md): a column of members, each a portrait
  with its health and resource bars, level, name and conditions. The game
  computes every number; the party reports choose (the member's key).
  orientation horizontal lays the members out as a row of portrait cards,
  for a phone's edge or a HUD's bottom.
-->
<nav id="RpgParty" class:compact={compact} class:horizontal={orientation == "horizontal"} aria-label={t("rpg.party.name")}>
  {#each members as m (m.id)}
    <button id="Member" part="Party.member" class="member" class:active={m.active} class:down={m.down} variant="quiet" aria-selected={m.id == selected} aria-label={m.name} aria-description={t("rpg.party.health", { health: m.health, max: m.healthMax }) + (m.status ? " · " + m.status : "")} onclick={choose}>
      <div class="card">
      <div class="portrait" part="Party.portrait">
        <img class="face" src={m.portrait} alt="" hidden={!m.portrait}>
        <div class="noface" hidden={!!m.portrait}><icon class="placeholder" icon="person"></icon></div>
        <span class="level number">{formatNumber(m.level)}</span>
        <div class="downmark" hidden={!m.down}><icon icon="skull"></icon></div>
      </div>
      <div class="info">
        <span class="name">{m.name}</span>
        <div class="bars">
          <meter class="health" part="Party.health" value={m.health} min="0" max={m.healthMax} low={m.healthMax * 0.3} high={m.healthMax * 0.6} optimum={m.healthMax} aria-label={t("rpg.party.hp")}></meter>
          <meter class="resource" part="Party.resource" value={m.resource} min="0" max={m.resourceMax} hidden={!m.resourceMax} aria-label={t("rpg.party.resource")}></meter>
        </div>
        <span class="hp number">{formatNumber(m.health)} / {formatNumber(m.healthMax)}</span>
        <span class="status" hidden={!m.status}>{m.status}</span>
      </div>
      </div>
    </button>
  {:else}
    <p class="quiet" role="status">{t("rpg.party.empty")}</p>
  {/each}
</nav>

<style>
#RpgParty { width: stretch; gap: var(--space-xs); container-type: inline-size; }
.member { width: stretch; min-height: var(--size-touch); padding: var(--space-xs); }
.card { display: flex; width: stretch; gap: var(--space-sm); align-items: center; }
.portrait { position: relative; width: var(--size-xl); height: var(--size-xl); flex-shrink: 0; overflow: hidden; background: var(--panel-raised); border-radius: var(--radius-md); padding: var(--space-xxs); }
.face { width: stretch; height: stretch; object-fit: cover; }
.noface { width: stretch; height: stretch; align-items: center; justify-content: center; }
.placeholder { width: var(--size-sm); height: var(--size-sm); color: var(--ink-faint); }
.level { position: absolute; left: 0; bottom: 0; min-width: var(--space-lg); padding: 0 var(--space-xxs); font: var(--text-caption); font-family: monospace; color: var(--ink); background: var(--panel-raised); border-radius: var(--radius-xs); text-align: center; }
.downmark { position: absolute; left: 0; top: 0; width: stretch; height: stretch; align-items: center; justify-content: center; background: var(--scrim); }
.downmark icon { width: var(--size-sm); height: var(--size-sm); color: var(--danger); }
.member.down .face { filter: grayscale(1); }
.info { width: stretch; gap: var(--space-xxs); min-width: 0; }
.name { color: var(--ink); font-family: fantasy; overflow-wrap: anywhere; }
.member.active .name { color: var(--highlight); }
.bars { width: stretch; gap: 2px; }
.health { width: stretch; height: var(--space-sm); }
.resource { width: stretch; height: var(--space-xs); }
.number { font-family: monospace; font-variant-numeric: tabular-nums; white-space: nowrap; }
.hp { font: var(--text-caption); font-family: monospace; color: var(--ink-muted); }
.status { font: var(--text-caption); color: var(--warn); font-style: italic; }
.quiet { margin: 0; color: var(--ink-muted); font: var(--text-sm); }
.compact .info .name, .compact .hp, .compact .status { display: none; }
.compact .member { width: auto; }
#RpgParty.horizontal { flex-direction: row; flex-wrap: wrap; }
.horizontal .member { flex: 1 1 0; min-width: calc(4 * var(--space-lg)); }
.horizontal .card { flex-direction: column; align-items: stretch; gap: var(--space-xs); }
.horizontal .portrait { width: stretch; height: auto; aspect-ratio: 1; }
.horizontal .status, .horizontal .hp { display: none; }
.horizontal .name { font: var(--text-sm); font-family: fantasy; text-align: center; }
</style>
