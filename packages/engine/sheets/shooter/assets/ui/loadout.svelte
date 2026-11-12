<meta name="component" content="LoadoutPage">

<div id="Page">
  <div id="Main"><Loadout title={$title} categories={$categories} slots={$items} selected={$selected} /></div>
  <div id="Feed"><Killfeed rows={$feed} /></div>
</div>

<style>
#Page { display: flex; width: stretch; height: stretch; gap: var(--space-xl); padding: calc(env(safe-area-inset-top) + var(--space-xl)) var(--space-xl) var(--space-xl) var(--space-xl); background: var(--backdrop); }
#Main { width: stretch; height: stretch; }
#Feed { width: calc(var(--size-xxl) * 4); }
/* A landscape phone: a tighter frame and a narrower feed leave the list its width. */
@media (height < 600px) and (orientation: landscape) {
  #Page { padding: calc(env(safe-area-inset-top) + var(--space-md)) var(--space-lg) var(--space-md) var(--space-lg); gap: var(--space-lg); }
  #Feed { width: calc(var(--size-xxl) * 3); }
}
@media (width < 600px) {
  #Page { flex-direction: column; padding: calc(env(safe-area-inset-top) + var(--space-md)) var(--space-md) var(--space-md) var(--space-md); gap: var(--space-md); }
  #Feed { order: -1; width: stretch; }
}
</style>
