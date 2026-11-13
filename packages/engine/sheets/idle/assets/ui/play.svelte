<meta name="component" content="IdlePlayPage">

<!-- The normal page, an idle game's main screen: the resource bar across the
     top (one store full), the generators (running cycles, a locked one, the
     selected one) and the upgrade shop (×1 chosen; affordable, short, maxed
     and locked rows). On a phone they stack, each scrolling in its share. -->
<div id="Page">
  <div id="Bar"><IdleResources currencies={$resources_default_currencies} loading={$resources_default.loading} /></div>
  <div id="Main">
    <div id="Generators"><IdleGenerators title={$generators_default.title} generators={$generators_default_generators} selected={$generators_default.selected} /></div>
    <div id="Shop"><IdlePurchases title={$purchases_default.title} amounts={$purchases_default_amounts} amount={$purchases_default.amount} upgrades={$purchases_default_upgrades} /></div>
  </div>
</div>

<style>
#Page { display: flex; flex-direction: column; width: stretch; height: stretch; gap: var(--space-lg); padding: var(--space-xl); }
#Bar { width: stretch; flex: none; }
#Main { display: flex; width: stretch; flex: 1; min-height: 0; gap: var(--space-xl); }
#Generators { flex: 1; min-width: 0; height: stretch; }
#Shop { flex: 1; min-width: 0; height: stretch; }
@media (width < 600px) {
  #Page { gap: var(--space-sm); padding: var(--space-sm); }
  #Main { flex-direction: column; gap: var(--space-sm); }
  #Generators, #Shop { flex: 1; min-height: 0; width: stretch; height: auto; }
}
</style>
