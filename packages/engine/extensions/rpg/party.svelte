<meta name="component" content="RpgParty">
<meta name="strings" content="./rpg.en.ogt">
<meta name="prop:members" content="list" data-doc="Keyed rows of { id, name, portrait, level, health, healthMax, mana, manaMax, down }: portrait a texture (og.load.texture), 0 none; down greys the card and marks it.">
<meta name="prop:selected" content="string" data-doc="The member chosen; empty is none.">
<meta name="fixture:default" content='{"members":[{"id":"a","name":"Aerin","portrait":0,"level":8,"health":62,"healthMax":72,"mana":48,"manaMax":72,"down":false},{"id":"l","name":"Lys","portrait":0,"level":8,"health":0,"healthMax":60,"mana":0,"manaMax":80,"down":true}],"selected":"a"}'>
<meta name="fixture:empty" content='{"members":[],"selected":""}'>

<!--
  The party as a row of portrait cards: a level seal, the portrait, the
  name, and health and mana bars with their figures. Reports choose (a
  member's key). The cards share the width evenly.
-->
<section id="RpgParty" class="rpg-party" aria-label={t("rpg.party.name")}>
  {#if members.length}
    <div class="cards" role="listbox" aria-label={t("rpg.party.name")}>
      {#each members as m (m.id)}
        <button class="member" role="option" data-down={m.down} aria-selected={m.id == selected} aria-label={m.name} onclick={choose}><div class="face">
          <div class="picture"><img class="portrait" src={m.portrait} alt="" hidden={!m.portrait}></div>
          <div class="seal" aria-label={m.down ? t("rpg.party.down") : t("rpg.party.level", { level: m.level })}><span class="seal-lv" hidden={m.down}>{t("rpg.party.lv")}</span><span class="seal-level" hidden={m.down}>{formatNumber(m.level)}</span></div>
          <span class="member-name">{m.name}</span>
          <div class="bar health" role="meter" aria-label={t("rpg.party.health")} style:--fill={m.healthMax > 0 ? m.health / m.healthMax : 0}><div class="bar-fill"></div><span class="bar-figure">{t("rpg.party.figure", { now: m.health, max: m.healthMax })}</span></div>
          <div class="bar mana" role="meter" aria-label={t("rpg.party.mana")} style:--fill={m.manaMax > 0 ? m.mana / m.manaMax : 0}><div class="bar-fill"></div><span class="bar-figure">{t("rpg.party.figure", { now: m.mana, max: m.manaMax })}</span></div>
        </div></button>
      {/each}
    </div>
  {:else}
    <p class="empty-text" role="status">{t("rpg.party.empty")}</p>
  {/if}
</section>

<style>
.rpg-party { width: stretch; }
.cards { display: grid; grid-auto-flow: column; grid-auto-columns: 1fr; gap: var(--space-sm); width: stretch; }
.member { position: relative; width: stretch; padding: var(--space-xs); gap: var(--space-xxs); align-items: stretch; justify-content: flex-start; }
.member .face { display: flex; flex-direction: column; width: stretch; gap: var(--space-xxs); }
.picture { width: stretch; aspect-ratio: 0.9; overflow: hidden; }
@media (width >= 600px) { .picture { aspect-ratio: 1.5; } }
.portrait { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.seal { position: absolute; left: var(--space-xxs); top: var(--space-xxs); width: var(--size-md); height: var(--size-md); align-items: center; justify-content: center; gap: 0; }
.member-name { padding: var(--space-xxs) var(--space-xs); text-align: center; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.bar { display: flex; position: relative; width: stretch; height: calc(var(--space-md) * 1.25); align-items: center; justify-content: center; }
.bar-fill { position: absolute; left: 0; top: 0; width: calc(var(--fill) * 100%); height: stretch; }
.bar-fill:dir(rtl) { left: auto; right: 0; }
.bar-figure { position: relative; font-variant-numeric: tabular-nums; white-space: nowrap; }
.empty-text { margin: 0; }
</style>
