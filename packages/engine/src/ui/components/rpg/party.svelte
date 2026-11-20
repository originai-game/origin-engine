<meta name="component" content="RpgParty">
<meta name="strings" content="./rpg.en.ogt">
<meta name="prop:members" content="list" data-doc="Keyed rows of { id, name, portrait, level, health, healthMax, mana, manaMax, down }: portrait a texture (og.load.texture), 0 none; down greys the card and marks it.">
<meta name="prop:selected" content="string" data-doc="The member chosen; empty is none.">
<meta name="fixture:default" content='{"members":[{"id":"a","name":"Aerin","portrait":0,"level":8,"health":62,"healthMax":72,"mana":48,"manaMax":72,"down":false},{"id":"l","name":"Lys","portrait":0,"level":8,"health":0,"healthMax":60,"mana":0,"manaMax":80,"down":true}],"selected":"a"}'>
<meta name="fixture:empty" content='{"members":[],"selected":""}'>

<!--
  The party as a row of portrait cards: a level seal, the portrait, the
  name, and a health and a mana HudBar with their figures, in the page's
  health and mana colours. Reports choose (a member's key). The cards
  share the width evenly.
-->
<section id="RpgParty" part="frame" variant="bare" aria-label={t("rpg.party.name")}>
  {#if members.length}
    <div gap="base" class="cards grid" role="listbox" aria-label={t("rpg.party.name")}>
      {#each members as m (m.id)}
        <button class="cell" part="cell" role="option" data-down={m.down} aria-selected={m.id == selected} aria-label={m.name} onclick={choose}><div class="face" layout="stack" gap="hair">
          <div class="picture">{#if m.portrait}<img class="portrait" src={m.portrait} alt="">{/if}</div>
          <div class="seal cover" part="badge" layout="stack" fill gap="none" aria-label={m.down ? t("rpg.party.down") : t("rpg.party.level", { level: m.level })}>{#if !m.down}<span class="seal-lv">{t("rpg.party.lv")}</span><span class="seal-level">{formatNumber(m.level)}</span>{:else}<og-icon icon="skull"></og-icon>{/if}</div>
          <span class="member-name" part="headline">{m.name}</span>
          <div class="bar" part="health"><HudBar value={m.health} max={m.healthMax} kind="health" ghost={0} segment={0} low={false} showValue={true} name="" /></div>
          <div class="bar mana" part="mana"><HudBar value={m.mana} max={m.manaMax} kind="mana" ghost={0} segment={0} low={false} showValue={true} name="" /></div>
        </div></button>
      {/each}
    </div>
  {:else}
    <p class="empty-text" part="caption" role="status">{t("rpg.party.empty")}</p>
  {/if}
</section>

<style>
.cover { justify-content: center; align-items: center; }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(var(--layout-tile), 100%), 1fr)); }
/* Drawn over the scene: the style's tag halo holds its text there. */
#RpgParty { text-shadow: var(--tag-halo); --scene-halo: var(--tag-halo); }
#RpgParty { inline-size: stretch; }
.cards { grid-template-columns: none; grid-auto-flow: column; grid-auto-columns: 1fr; width: stretch; }
.cell { position: relative; width: stretch; padding: var(--space-xs); align-items: stretch; justify-content: flex-start; min-width: 0; }
.cell .face { width: stretch; }
.picture { width: stretch; aspect-ratio: 1.15; overflow: hidden; }
.picture:not(:size(compact)) { aspect-ratio: 1.5; }
.portrait { width: stretch; height: stretch; object-fit: cover; object-position: center top; }
.seal { position: absolute; inset-inline-start: var(--space-xs); top: var(--space-xs); width: var(--size-md); height: max-content; min-height: var(--size-md); }
.seal og-icon { width: var(--size-xs); height: var(--size-xs); }
.member-name { padding: var(--space-xxs) var(--space-xs); text-align: center; white-space: nowrap; text-overflow: ellipsis; overflow: hidden; }
/* Each bar's place: thick enough for its figure, filled in the page's
   health and mana instead of the HUD's. */
.bar { margin-inline: var(--space-xs); --hud-bar-size: max(calc(var(--space-md) * 1.25), var(--size-sm)); --hud-life-fill: var(--fill-gloss), var(--health); --hud-life-glow: none; }
.mana { --hud-mana-fill: var(--fill-gloss), var(--mana); --hud-mana-glow: none; }
.empty-text { margin: 0; }
.picture:size(short) { aspect-ratio: 2.4; }
@media (600px <= width) and (500px <= height < 800px) { .picture { aspect-ratio: 2; } }
</style>
