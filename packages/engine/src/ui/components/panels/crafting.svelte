<meta name="component" content="RpgCrafting">
<meta name="strings" content="./economy.en.ogt">
<meta name="prop:title" content="string" data-default="" data-doc="The station name; empty uses Crafting.">
<meta name="prop:recipes" content="list" data-default="[]" data-doc="Keyed { id, name, icon, description, canCraft, reason, rarity, held } recipes. Uncraftable recipes remain selectable for inspection; reason explains the restriction. rarity (common … mythic, empty common) colours the row's edge, the socket and the name; held, how many of the result the player has as the game words it, is shown under the result (empty hides it).">
<meta name="prop:selected" content="string" data-default="" data-doc="The selected recipe id, or empty for selection guidance. Clear the selection when removing its recipe.">
<meta name="prop:ingredients" content="list" data-default="[]" data-doc="Requirements for the selected recipe and quantity: keyed { id, name, quantity, sufficient }. The game formats quantity (2 / 3) and computes sufficiency.">
<meta name="prop:quantity" content="number" data-default="1" data-doc="The game-owned batch size. Increase/decrease intents carry the selected recipe id; the game validates and updates it.">
<meta name="prop:canIncrease" content="bool" data-default="false" data-doc="The game permits a larger batch.">
<meta name="prop:canDecrease" content="bool" data-default="false" data-doc="The game permits a smaller batch.">
<meta name="prop:cost" content="string" data-default="" data-doc="The complete batch cost as the game words it; empty hides it.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Recipe data is loading; existing recipes remain visible but inactive.">
<meta name="prop:busy" content="bool" data-default="false" data-doc="A craft is in progress; selection, quantities and crafting are inactive.">
<meta name="prop:error" content="string" data-default="" data-doc="The game's failure message; empty hides it without discarding the selection.">
<meta name="fixture:default" content='{"title":"Alchemist’s bench","recipes":[{"id":"draught","name":"Healing draught","icon":"potion","description":"Restores 240 health over 4 s. Produces 2 draughts.","canCraft":true,"reason":"","rarity":"uncommon","held":"You hold 5"},{"id":"elixir","name":"Storm elixir","icon":"bolt","description":"A rare catalyst is needed.","canCraft":false,"reason":"Requires Alchemy 4","rarity":"epic","held":""}],"selected":"draught","ingredients":[{"id":"herb","name":"Silverleaf","quantity":"6 / 2","sufficient":true},{"id":"water","name":"Pure water","quantity":"4 / 1","sufficient":true}],"quantity":1,"canIncrease":true,"canDecrease":false,"cost":"10 gold","loading":false,"busy":false,"error":""}'>
<meta name="fixture:empty" content='{}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:busy" content='{"busy":true,"selected":"p","recipes":[{"id":"p","name":"Healing draught","icon":"potion","description":"Brewing a batch.","canCraft":true,"reason":"","rarity":"uncommon","held":""}]}'>
<meta name="fixture:missing" content='{"selected":"e","recipes":[{"id":"e","name":"Storm elixir","icon":"bolt","description":"A rare catalyst is needed.","canCraft":false,"reason":"Missing ingredients","rarity":"legendary","held":""}],"ingredients":[{"id":"c","name":"Storm crystal","quantity":"0 / 1","sufficient":false}],"error":"Not enough ingredients."}'>

<!--
  A crafting station: a band with the station's name, the recipes as rows
  (the result's icon in a socket, its rarity on the row's edge and in its
  name, why it cannot be made in words), and beside them (under them when
  narrow) the selected recipe: its art large over a glow of its rarity,
  what it does and how many the player holds, the ingredients as have /
  need with a short one marked in words, the batch stepper with the cost,
  and Craft. The game decides every requirement; the panel reports choose,
  increase, decrease and craft, each with the recipe's key.
-->
<section class="frame" id="RpgCrafting" part="frame" layout="stack" fill gap="loose" aria-label={title || t("rpg.craft.title")} aria-busy={loading || busy}>
  <div class="lit"></div>
  <header part="band" class="band" layout="stack" gap="hair">
    <span part="kicker" class="kicker">{t("rpg.craft.title")}</span>
    <h2 class="title" part="title">{title || t("rpg.craft.title")}</h2>
  </header>
  {#if error}<p part="status" class="alert" tone="danger" role="alert">{error}</p>{/if}
  {#if loading || busy}<div part="well" class="wait status" layout="row" role="status"><Spinner name={busy ? t("rpg.economy.busy") : t("rpg.economy.loading")} /><span>{busy ? t("rpg.economy.busy") : t("rpg.economy.loading")}</span></div>{/if}
  {#if recipes.length}
    <div class="cols" fill>
      <div part="recipes" class="recipes" layout="scroll" role="listbox" aria-label={t("rpg.craft.title")}>
        {#each recipes as recipe (recipe.id)}
          <button part="row" class="row" role="option" tier={recipe.rarity || 'common'} aria-selected={recipe.id == selected} disabled={loading || busy} onclick={choose} state:selected={recipe.id == selected} class:not-can={!(recipe.canCraft)}><div part="face" class="face">
            <div part="socket" class="socket cover" layout="stack" fill><og-icon class="icon" part="icon" icon={recipe.icon}></og-icon></div>
            <div class="words" layout="stack" gap="hair"><span part="name" class="name">{recipe.name}</span>{#if !recipe.canCraft}<span part="reason" class="reason">{recipe.reason || t("rpg.economy.unavailable")}</span>{/if}</div>
          </div></button>
        {/each}
      </div>
      {#if !selected}<p part="caption" class="guide empty">{t("rpg.craft.choose")}</p>{/if}
      {#each recipes as recipe (recipe.id)}
        {#if recipe.id == selected}<section part="detail" class="detail" tier={recipe.rarity || 'common'} layout="stack" gap="none" aria-label={recipe.name}>
          <div class="detail-inner" layout="scroll" gap="loose">
            <div part="hero" class="hero"><div class="hero-glow"></div><og-icon class="hero-art" icon={recipe.icon}></og-icon></div>
            <div part="result" class="result" layout="stack" gap="hair">
              <h3 class="result-name">{recipe.name}</h3>
              {#if recipe.description}<p class="description" part="caption">{recipe.description}</p>{/if}
              {#if recipe.held}<p part="figure" class="held">{recipe.held}</p>{/if}
            </div>
            {#if ingredients.length}<div class="needs" layout="stack" gap="tight"><h4 part="ingredientsTitle" class="kicker">{t("rpg.craft.ingredients")}</h4><ul part="ingredients" class="ingredients">{#each ingredients as ingredient (ingredient.id)}<li part="ingredient" class="ingredient" class:not-sufficient={!(ingredient.sufficient)}><span part="ingredientName" class="ing-name">{ingredient.name}</span><span part="figure" class="have requirement">{ingredient.quantity}</span>{#if !ingredient.sufficient}<span class="short">{t("rpg.craft.missing")}</span>{/if}</li>{/each}</ul></div>{/if}
            {#if !recipe.canCraft}<p part="reason" class="reason">{recipe.reason || t("rpg.economy.unavailable")}</p>{/if}
          </div>
          <div class="make" layout="stack" gap="tight">
              <div class="batch" layout="row"><div part="quantity" class="quantity" layout="row" gap="none"><button part="less" class="step" aria-label={t("rpg.economy.less")} disabled={!canDecrease || loading || busy} onclick={decrease}>{t("rpg.economy.minus")}</button><div part="figure" class="qty quantityLabel" aria-label={t("rpg.economy.quantity", { quantity })}>{formatNumber(quantity)}</div><button part="more" class="step" aria-label={t("rpg.economy.more")} disabled={!canIncrease || loading || busy} onclick={increase}>{t("rpg.economy.plus")}</button></div>{#if cost}<p part="figure" class="cost">{cost}</p>{/if}</div>
              <button part="craft" class="go craft" variant="primary" disabled={!recipe.canCraft || loading || busy} aria-label={t("rpg.craft.make")} onclick={craft}><kbd class="cap" data-action="details"></kbd><span>{t("rpg.craft.make")}</span></button>
          </div>
        </section>{/if}
      {/each}
    </div>
  {:else if !loading}<p part="caption" class="guide empty" role="status">{t("rpg.craft.empty")}</p>{/if}
</section>

<style>
.cover { justify-content: center; align-items: center; }
.frame { position: relative; inline-size: stretch; max-block-size: stretch; min-block-size: 0; border-radius: var(--inv-frame-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.lit { position: absolute; inset-inline: var(--inv-frame-cut) 30%; top: 0; block-size: var(--stroke-hairline); background: var(--inv-lit-edge); pointer-events: none; }
h2, h3, h4, p { margin: 0; }
.band { flex: none; width: stretch; }
.kicker { font-size: var(--inv-kicker-size); font-weight: var(--inv-name-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--inv-kicker-ink); white-space: nowrap; }
.title { overflow-wrap: anywhere; }
.alert { flex: none; padding: var(--space-xs) var(--space-sm); background: var(--inv-alert-fill); box-shadow: var(--inv-alert-rim); }
.wait { flex: none; padding: var(--space-sm); }
/* Two columns, the recipes and the selected one; one over the other when narrow. */
.cols { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1.15fr); column-gap: var(--space-lg); row-gap: var(--space-md); min-block-size: 0; width: stretch; }
.recipes { gap: var(--space-xs); min-block-size: 0; align-content: start; }
.row { --inv-rarity: var(--common); --inv-socket-fill: var(--cell-common-fill); --inv-socket-rim: var(--cell-common-shadow); flex: none; width: stretch; min-block-size: var(--size-touch); padding: var(--space-xs) var(--space-sm) var(--space-xs) var(--space-md); justify-content: flex-start; border: none; background: var(--inv-row-fill); box-shadow: var(--inv-row-rim); border-inline-start: var(--inv-edge); border-inline-start-color: var(--inv-rarity); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.row:hover { background: var(--inv-row-hover-fill); }
.row:state(selected) { background: var(--inv-row-on-fill); box-shadow: var(--inv-row-on-rim); }
.row.rarity-uncommon, .detail.rarity-uncommon { --inv-rarity: var(--uncommon); --inv-socket-fill: var(--cell-uncommon-fill); --inv-socket-rim: var(--cell-uncommon-shadow); }
.row.rarity-rare, .detail.rarity-rare { --inv-rarity: var(--rare); --inv-socket-fill: var(--cell-rare-fill); --inv-socket-rim: var(--cell-rare-shadow); }
.row.rarity-epic, .detail.rarity-epic { --inv-rarity: var(--epic); --inv-socket-fill: var(--cell-epic-fill); --inv-socket-rim: var(--cell-epic-shadow); }
.row.rarity-legendary, .detail.rarity-legendary { --inv-rarity: var(--legendary); --inv-socket-fill: var(--cell-legendary-fill); --inv-socket-rim: var(--cell-legendary-shadow); }
.row.rarity-mythic, .detail.rarity-mythic { --inv-rarity: var(--mythic); --inv-socket-fill: var(--cell-mythic-fill); --inv-socket-rim: var(--cell-mythic-shadow); }
.face { display: flex; align-items: center; gap: var(--space-sm); width: stretch; min-width: 0; }
.socket { flex: none; inline-size: calc(44 * var(--font-step-px)); block-size: calc(44 * var(--font-step-px)); background: var(--inv-socket-fill); box-shadow: var(--inv-socket-rim); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.icon { inline-size: 74%; block-size: 74%; filter: var(--inv-slot-art-shadow); }
.row.not-can .socket { opacity: var(--disabled-content-opacity); }
.words { min-width: 0; align-items: flex-start; text-align: start; }
.name { overflow-wrap: anywhere; font-weight: var(--inv-name-weight); color: var(--inv-rarity); }
.row.rarity-common .name, .detail.rarity-common .result-name { color: var(--ink-bright); }
.reason { color: var(--inv-broken-ink); overflow-wrap: anywhere; }
.guide { padding: var(--space-lg); text-align: center; }
/* The selected recipe: its art over a glow of its rarity, what it makes,
   what it needs and the batch, Craft at its foot. */
.detail { min-block-size: 0; }
.detail-inner { flex: 0 1 auto; width: stretch; min-block-size: 0; }
.hero { flex: none; display: flex; align-items: center; justify-content: center; block-size: var(--inv-hero-size); position: relative; color: var(--inv-rarity); }
.hero-glow { position: absolute; inset: 0; background: var(--inv-hero-glow); opacity: 0.35; }
.hero-art { position: relative; }
.hero-art { inline-size: var(--inv-hero-art-size); block-size: var(--inv-hero-art-size); filter: var(--inv-slot-art-shadow); }
.result-name { font-size: var(--inv-figure-xl-size); color: var(--inv-rarity); overflow-wrap: anywhere; }
.held { font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); }
.ingredients { display: grid; grid-template-columns: repeat(auto-fill, minmax(calc(130 * var(--font-step-px)), 1fr)); gap: var(--space-xs); margin: 0; padding: 0; list-style: none; }
.ingredient { display: flex; flex-direction: column; gap: var(--space-xxs); padding: var(--space-xs) var(--space-sm); background: var(--inv-row-fill); box-shadow: var(--inv-row-rim); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.ingredient.not-sufficient { background: var(--inv-short-fill); box-shadow: var(--inv-short-rim); }
.ing-name { color: var(--ink-label); overflow-wrap: anywhere; }
.have { font-size: var(--inv-figure-lg-size); font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); font-variant-numeric: var(--hud-figure-numeric); white-space: nowrap; }
.ingredient.not-sufficient .have, .short { color: var(--inv-broken-ink); }
/* The batch and Craft follow what the recipe needs; when that is too
   tall it scrolls above them, so Craft stays in view. */
.make { flex: none; width: stretch; padding-block-start: var(--space-sm); border-block-start: var(--inv-rule); }
.batch { align-items: center; justify-content: space-between; flex-wrap: wrap; gap: var(--space-sm); }
.quantity { flex: none; }
.step { min-inline-size: var(--size-touch); min-block-size: var(--size-touch); border-radius: 0; }
.qty { display: grid; place-items: center; min-inline-size: calc(64 * var(--font-step-px)); min-block-size: var(--size-touch); font-size: var(--inv-figure-lg-size); font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); background: var(--inv-well-fill); box-shadow: var(--inv-well-rim); }
.cost { font-size: var(--inv-figure-lg-size); font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); white-space: nowrap; }
.craft { width: stretch; min-block-size: calc(var(--size-touch) * 1.15); gap: var(--space-xs); }
.cap { flex: none; }
@container (width < 600px) {
  /* Narrow: the recipes over the selected one, in one scroller. */
  .cols { display: flex; flex-direction: column; }
  .recipes { flex: 0 1 auto; max-block-size: 40%; }
  .detail { flex: 1 1 0; }
  /* The batch and Craft stay at the foot while what is above scrolls. */
  .hero { block-size: var(--inv-hero-compact-size); }
  .hero-art { inline-size: var(--inv-hero-compact-art-size); block-size: var(--inv-hero-compact-art-size); }
}
</style>
