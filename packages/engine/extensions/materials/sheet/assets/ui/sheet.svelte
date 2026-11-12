<meta name="component" content="materialsSheet">

<div id="Page">
  <div id="Plain" class="card"><div class="art"><og-icon class="icon" icon="crown"></og-icon></div><span class="title">Common</span><span class="note">no material</span></div>
  <div id="Foil" class="card"><div class="art"><og-icon class="icon" icon="crown"></og-icon></div><span class="title">Foil</span><span class="note">foil</span></div>
  <div id="Holo" class="card"><div class="art"><og-icon class="icon" icon="crown"></og-icon></div><span class="title">Holo</span><span class="note">holo</span></div>
  <div id="Burn" class="card" style:--burn={$burn}><div class="art"><og-icon class="icon" icon="crown"></og-icon></div><span class="title">Burn</span><span class="note">burn</span></div>
  <div id="Rarity" class="card"><div class="art"><og-icon class="icon" icon="crown"></og-icon></div><span class="title">Legendary</span><span class="note">rarity</span></div>
</div>

<style>
#Page { display: flex; flex-direction: row; flex-wrap: wrap; width: stretch; height: stretch; padding: 24px; gap: 16px; align-items: center; justify-content: center; background: var(--backdrop); }
.card { width: 168px; height: 236px; padding: 14px; gap: 10px; align-items: center; border-radius: 12px; background: var(--panel-raised); }
.art { width: stretch; height: 132px; align-items: center; justify-content: center; border-radius: 8px; background: var(--primary); }
.icon { width: 64px; height: 64px; }
.title { font: var(--text-lg); }
.note { font: var(--text-caption); }
/* The band held still across the art, so every capture shows it. */
#Foil { -og-material: "materials/materials/foil.ogt" 0.7 0 0 0 1.5 0 0 0 0 -0.12 0 0 0.85 0.92 1 0; }
#Holo { -og-material: "materials/materials/holo.ogt"; }
/* The burn's progress from the store (the `burn:` command). */
#Burn { -og-material: "materials/materials/burn.ogt" var(--burn) 0 0 0 6 0 0 0 40 0 0 0 1 0.36 0.05 0; }
#Rarity { -og-material: "materials/materials/rarity.ogt"; }
</style>
