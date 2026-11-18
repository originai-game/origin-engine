<meta name="component" content="RpgParty">
<meta name="strings" content="./rpg.en.ogt">
<meta name="parts" content="RpgParty.frame RpgParty.cell RpgParty.badge RpgParty.headline RpgParty.track RpgParty.fill RpgParty.caption">
<meta name="prop:members" content="list" data-doc="Keyed rows of { id, name, portrait, level, health, healthMax, mana, manaMax, down }: portrait a texture (og.load.texture), 0 none; down greys the card and marks it.">
<meta name="prop:selected" content="string" data-doc="The member chosen; empty is none.">
<meta name="fixture:default" content='{"members":[{"id":"a","name":"Aerin","portrait":0,"level":8,"health":62,"healthMax":72,"mana":48,"manaMax":72,"down":false},{"id":"l","name":"Lys","portrait":0,"level":8,"health":0,"healthMax":60,"mana":0,"manaMax":80,"down":true}],"selected":"a"}'>
<meta name="fixture:empty" content='{"members":[],"selected":""}'>

<!--
  The party as a row of portrait cards: a level seal, the portrait, the
  name, and health and mana bars with their figures. Reports choose (a
  member's key). The cards share the width evenly.
-->
<section id="RpgParty" part="RpgParty.frame" class="bare" aria-label={t("rpg.party.name")}>
  {#if members.length}
    <div class="cards" data-layout="grid" role="listbox" aria-label={t("rpg.party.name")}>
      {#each members as m (m.id)}
        <button part="RpgParty.cell" role="option" data-down={m.down} aria-selected={m.id == selected} aria-label={m.name} onclick={choose}><div class="face" data-layout="stack" data-gap="hair">
          <div class="picture">{#if m.portrait}<img class="portrait" src={m.portrait} alt="">{/if}</div>
          <div class="seal" part="RpgParty.badge" data-layout="cover" data-gap="none" aria-label={m.down ? t("rpg.party.down") : t("rpg.party.level", { level: m.level })}>{#if !m.down}<span class="seal-lv">{t("rpg.party.lv")}</span><span class="seal-level">{formatNumber(m.level)}</span>{:else}<og-icon icon="skull"></og-icon>{/if}</div>
          <span class="member-name" part="RpgParty.headline">{m.name}</span>
          <div class="bar health" part="RpgParty.track" data-layout="row" role="meter" aria-label={t("rpg.party.health")} style:--fill={m.healthMax > 0 ? m.health / m.healthMax : 0}><div class="bar-fill" part="RpgParty.fill"></div><div class="bar-figure" part="RpgParty.badge"><span>{t("rpg.party.figure", { now: m.health, max: m.healthMax })}</span></div></div>
          <div class="bar mana" part="RpgParty.track" data-layout="row" role="meter" aria-label={t("rpg.party.mana")} style:--fill={m.manaMax > 0 ? m.mana / m.manaMax : 0}><div class="bar-fill" part="RpgParty.fill"></div><div class="bar-figure" part="RpgParty.badge"><span>{t("rpg.party.figure", { now: m.mana, max: m.manaMax })}</span></div></div>
        </div></button>
      {/each}
    </div>
  {:else}
    <p class="empty-text" part="RpgParty.caption" role="status">{t("rpg.party.empty")}</p>
  {/if}
</section>

<style>
/* Drawn over the scene: the style's tag halo holds its text there. */
:host { text-shadow: var(--tag-halo); }
:host { inline-size: stretch; }
.cards { grid-template-columns: none; grid-auto-flow: column; grid-auto-columns: 1fr; width: stretch; }
[part="RpgParty.cell"] { position: relative; width: stretch; padding: var(--space-xs); align-items: stretch; justify-content: flex-start; min-width: 0; }
[part="RpgParty.cell"] .face { width: stretch; }
.picture { width: stretch; aspect-ratio: 1.15; overflow: hidden; }
@media (width >= 600px) and (height >= 500px) { .picture { aspect-ratio: 1.5; } }
.portrait { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.seal { position: absolute; inset-inline-start: var(--space-xs); top: var(--space-xs); width: var(--size-md); height: max-content; min-height: var(--size-md); }
.seal og-icon { width: var(--size-xs); height: var(--size-xs); }
.member-name { padding: var(--space-xxs) var(--space-xs); text-align: center; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
.bar { position: relative; height: calc(var(--space-md) * 1.25); min-block-size: var(--size-sm); justify-content: center; overflow: hidden; --fill-color: var(--health); }
.bar-fill { position: absolute; inset-inline-start: 0; top: 0; bottom: 0; inline-size: calc(var(--fill) * 100%); }
.mana { --fill-color: var(--mana); }
.bar-figure { position: relative; white-space: nowrap; padding-inline: var(--space-xs); }
.empty-text { margin: 0; }
@media (height < 500px) { .picture { aspect-ratio: 2.4; } }
@media (600px <= width) and (500px <= height < 800px) { .picture { aspect-ratio: 2; } }
</style>
