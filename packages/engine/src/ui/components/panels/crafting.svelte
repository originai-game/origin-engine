<meta name="component" content="RpgCrafting">
<meta name="strings" content="./economy.en.ogt">
<meta name="parts" content="RpgCrafting.frame RpgCrafting.title RpgCrafting.recipes RpgCrafting.row RpgCrafting.face RpgCrafting.icon RpgCrafting.name RpgCrafting.reason RpgCrafting.detail RpgCrafting.description RpgCrafting.ingredientsTitle RpgCrafting.ingredients RpgCrafting.ingredient RpgCrafting.ingredientName RpgCrafting.requirement RpgCrafting.quantity RpgCrafting.quantityLabel RpgCrafting.less RpgCrafting.more RpgCrafting.cost RpgCrafting.craft RpgCrafting.status RpgCrafting.empty">
<meta name="prop:title" content="string" data-default="" data-doc="The station name; empty uses Crafting.">
<meta name="prop:recipes" content="list" data-default="[]" data-doc="Keyed { id, name, icon, description, canCraft, reason } recipes. Uncraftable recipes remain selectable for inspection; reason explains the restriction.">
<meta name="prop:selected" content="string" data-default="" data-doc="The selected recipe id, or empty for selection guidance. Clear the selection when removing its recipe.">
<meta name="prop:ingredients" content="list" data-default="[]" data-doc="Requirements for the selected recipe and quantity: keyed { id, name, quantity, sufficient }. The game formats quantity (2 / 3) and computes sufficiency.">
<meta name="prop:quantity" content="number" data-default="1" data-doc="The game-owned batch size. Increase/decrease intents carry the selected recipe id; the game validates and updates it.">
<meta name="prop:canIncrease" content="bool" data-default="false" data-doc="The game permits a larger batch.">
<meta name="prop:canDecrease" content="bool" data-default="false" data-doc="The game permits a smaller batch.">
<meta name="prop:cost" content="string" data-default="" data-doc="The complete batch cost as the game words it; empty hides it.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Recipe data is loading; existing recipes remain visible but inactive.">
<meta name="prop:busy" content="bool" data-default="false" data-doc="A craft is in progress; selection, quantities and crafting are inactive.">
<meta name="prop:error" content="string" data-default="" data-doc="The game's failure message; empty hides it without discarding the selection.">
<meta name="fixture:default" content='{"title":"Alchemist’s bench","recipes":[{"id":"draught","name":"Healing draught","icon":"potion","description":"Restores health. Produces 2 draughts.","canCraft":true,"reason":""},{"id":"elixir","name":"Storm elixir","icon":"bolt","description":"A rare catalyst is needed.","canCraft":false,"reason":"Requires Alchemy 4"}],"selected":"draught","ingredients":[{"id":"herb","name":"Silverleaf","quantity":"6 / 2","sufficient":true},{"id":"water","name":"Pure water","quantity":"4 / 1","sufficient":true}],"quantity":1,"canIncrease":true,"canDecrease":false,"cost":"10 gold","loading":false,"busy":false,"error":""}'>
<meta name="fixture:empty" content='{}'>
<meta name="fixture:loading" content='{"loading":true}'>
<meta name="fixture:busy" content='{"busy":true,"selected":"p","recipes":[{"id":"p","name":"Healing draught","icon":"potion","description":"Brewing a batch.","canCraft":true,"reason":""}]}'>
<meta name="fixture:missing" content='{"selected":"e","recipes":[{"id":"e","name":"Storm elixir","icon":"bolt","description":"A rare catalyst is needed.","canCraft":false,"reason":"Missing ingredients"}],"ingredients":[{"id":"c","name":"Storm crystal","quantity":"0 / 1","sufficient":false}],"error":"Not enough ingredients."}'>

<section id="RpgCrafting" part="RpgCrafting.frame" aria-label={title || t("rpg.craft.title")} aria-busy={loading || busy}>
  <h2 part="RpgCrafting.title">{title || t("rpg.craft.title")}</h2>
  {#if error}<p part="RpgCrafting.status" role="alert">{error}</p>{/if}
  {#if loading || busy}<div part="RpgCrafting.status" role="status"><Spinner name={busy ? t("rpg.economy.busy") : t("rpg.economy.loading")} /><span>{busy ? t("rpg.economy.busy") : t("rpg.economy.loading")}</span></div>{/if}
  {#if recipes.length}
    <div part="RpgCrafting.recipes" role="listbox" aria-label={t("rpg.craft.title")}>
      {#each recipes as recipe (recipe.id)}
        <button part="RpgCrafting.row" role="option" aria-selected={recipe.id == selected} disabled={loading || busy} onclick={choose}><div part="RpgCrafting.face"><og-icon part="RpgCrafting.icon" icon={recipe.icon}></og-icon><div part="RpgCrafting.name">{recipe.name}</div>{#if !recipe.canCraft}<div part="RpgCrafting.reason">{recipe.reason || t("rpg.economy.unavailable")}</div>{/if}</div></button>
      {/each}
    </div>
    {#if !selected}<p part="RpgCrafting.empty">{t("rpg.craft.choose")}</p>{/if}
    {#each recipes as recipe (recipe.id)}
      {#if recipe.id == selected}<section part="RpgCrafting.detail" aria-label={recipe.name}>
        {#if recipe.description}<p part="RpgCrafting.description">{recipe.description}</p>{/if}
        {#if ingredients.length}<h3 part="RpgCrafting.ingredientsTitle">{t("rpg.craft.ingredients")}</h3><ul part="RpgCrafting.ingredients">{#each ingredients as ingredient (ingredient.id)}<li part="RpgCrafting.ingredient" data-sufficient={ingredient.sufficient}><div part="RpgCrafting.ingredientName">{ingredient.name}</div><div part="RpgCrafting.requirement">{ingredient.quantity}{#if !ingredient.sufficient}<og-icon icon="close" title={t("rpg.craft.missing")}></og-icon>{/if}</div></li>{/each}</ul>{/if}
        <div part="RpgCrafting.quantity"><button part="RpgCrafting.less" aria-label={t("rpg.economy.less")} disabled={!canDecrease || loading || busy} onclick={decrease}>{t("rpg.economy.minus")}</button><div part="RpgCrafting.quantityLabel">{t("rpg.economy.quantity", { quantity })}</div><button part="RpgCrafting.more" aria-label={t("rpg.economy.more")} disabled={!canIncrease || loading || busy} onclick={increase}>{t("rpg.economy.plus")}</button></div>
        {#if cost}<p part="RpgCrafting.cost">{cost}</p>{/if}
        {#if !recipe.canCraft}<p part="RpgCrafting.reason">{recipe.reason || t("rpg.economy.unavailable")}</p>{/if}
        <button part="RpgCrafting.craft" class="primary" disabled={!recipe.canCraft || loading || busy} onclick={craft}>{t("rpg.craft.make")}</button>
      </section>{/if}
    {/each}
  {:else if !loading}<p part="RpgCrafting.empty" role="status">{t("rpg.craft.empty")}</p>{/if}
</section>

<style>
:host { display: flex; flex-direction: column; inline-size: stretch; max-block-size: stretch; overflow-y: auto; gap: var(--space-md); }
h2, h3, p { margin: 0; }
[part="RpgCrafting.recipes"] { display: flex; flex-direction: column; gap: var(--space-xs); }
[part="RpgCrafting.row"] { min-block-size: var(--size-touch); padding: var(--space-sm); }
[part="RpgCrafting.face"] { display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-sm); }
[part="RpgCrafting.icon"] { inline-size: var(--size-sm); block-size: var(--size-sm); flex: none; }
[part="RpgCrafting.name"] { flex: 1; text-align: start; }
[part="RpgCrafting.reason"], [part="RpgCrafting.empty"] { color: var(--ink-muted); font: var(--text-caption); }
[part="RpgCrafting.detail"] { display: flex; flex-direction: column; gap: var(--space-sm); padding: var(--space-md); background: var(--well); border-radius: var(--radius-control); }
[part="RpgCrafting.ingredients"] { display: flex; flex-direction: column; gap: var(--space-xs); list-style: none; margin: 0; padding: 0; }
[part="RpgCrafting.ingredient"] { display: flex; justify-content: space-between; align-items: center; gap: var(--space-sm); }
[part="RpgCrafting.requirement"] { display: flex; align-items: center; gap: var(--space-xs); flex: none; font-variant-numeric: tabular-nums; }
[part="RpgCrafting.requirement"] og-icon { inline-size: var(--size-xs); block-size: var(--size-xs); }
[part="RpgCrafting.ingredient"][data-sufficient=false] { color: var(--danger); }
[part="RpgCrafting.quantity"] { display: flex; justify-content: center; align-items: center; gap: var(--space-md); }
[part="RpgCrafting.less"], [part="RpgCrafting.more"] { min-inline-size: var(--size-touch); min-block-size: var(--size-touch); }
[part="RpgCrafting.craft"] { min-block-size: var(--size-touch); }
[part="RpgCrafting.status"] { padding: var(--space-sm); background: var(--well); border-radius: var(--radius-control); }
[part="RpgCrafting.status"][role=status] { display: flex; align-items: center; gap: var(--space-sm); }
[part="RpgCrafting.status"][role=alert] { color: var(--danger); }
</style>
