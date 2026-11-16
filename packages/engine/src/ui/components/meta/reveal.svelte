<meta name="component" content="Reveal">
<meta name="strings" content="./meta.en.ogt">
<meta name="parts" content="Reveal.frame Reveal.title Reveal.body Reveal.seal Reveal.status Reveal.rewards Reveal.reward Reveal.well Reveal.icon Reveal.name Reveal.amount Reveal.rarity Reveal.badge Reveal.actions Reveal.open Reveal.skip Reveal.continue">
<meta name="prop:title" content="string" data-default="" data-doc="Heading; empty uses Rewards.">
<meta name="prop:body" content="string" data-default="" data-doc="Optional explanation before the rewards.">
<meta name="prop:state" content="string" data-default="sealed" data-doc="Game-owned sealed/loading/revealing/revealed. No roll, timer or reward mutation runs here.">
<meta name="prop:rewards" content="list" data-default="[]" data-doc="Keyed { id, name, icon, amount, rarity, fresh } rewards, shown only once revealed. rarity is common/uncommon/rare/epic/legendary/mythic, in words and as a part state.">
<meta name="prop:canSkip" content="bool" data-default="true" data-doc="Offer skip while revealing; the game decides what skipping does.">
<meta name="prop:continueLabel" content="string" data-default="" data-doc="Continue wording; empty uses Continue.">
<meta name="prop:disabled" content="bool" data-default="false" data-doc="Prevent opening, skipping, inspecting and continuing.">
<meta name="prop:error" content="string" data-default="" data-doc="A failed reveal with retry; rewards stay hidden until the game resolves it.">
<meta name="action:open" content="Open the sealed rewards">
<meta name="action:skip" content="Finish the reveal early">
<meta name="action:inspect" content="Inspect a revealed reward by its key">
<meta name="action:continue" content="Continue after the reveal">
<meta name="action:retry" content="Retry the reveal">
<meta name="fixture:default" content='{"title":"Expedition rewards","body":"Open the chest to see what your party recovered.","state":"sealed"}'>
<meta name="fixture:loading" content='{"state":"loading"}'>
<meta name="fixture:revealing" content='{"state":"revealing","canSkip":true}'>
<meta name="fixture:revealed" content='{"title":"Expedition complete","state":"revealed","rewards":[{"id":"blade","name":"Ember Blade of the Ninefold Gate","icon":"sword","amount":"1","rarity":"legendary","fresh":true},{"id":"gold","name":"Gold","icon":"coin","amount":"1,250","rarity":"common","fresh":false}]}'>
<meta name="fixture:empty" content='{"state":"revealed","rewards":[]}'>
<meta name="fixture:error" content='{"error":"Your rewards are saved. Reconnect to view them."}'>

<section id="Reveal" part="Reveal.frame" data-state={state} aria-label={title || t("meta.reveal.title")} aria-busy={state == 'loading' || state == 'revealing'}>
  <h2 part="Reveal.title">{title || t("meta.reveal.title")}</h2>{#if body}<p part="Reveal.body" class="body">{body}</p>{/if}
  {#if error}<ErrorState body={error} />
  {:else if state == 'loading' || state == 'revealing'}<div part="Reveal.status" class="status" role="status"><Spinner name={state == 'loading' ? t("meta.loading") : t("meta.reveal.revealing")} /><span>{state == 'loading' ? t("meta.loading") : t("meta.reveal.revealing")}</span></div>
  {:else if state == 'revealed'}
    <ul part="Reveal.rewards" class="rewards">
      {#each rewards as reward (reward.id)}<li class="reward"><button part="Reveal.reward" data-rarity={reward.rarity} disabled={disabled} aria-label={reward.name} onclick={inspect}><div class="face">
        <div part="Reveal.well" class="well"><og-icon part="Reveal.icon" class="icon" icon={reward.icon}></og-icon></div><span part="Reveal.name" class="name">{reward.name}</span>{#if reward.amount}<span part="Reveal.amount">{reward.amount}</span>{/if}<span part="Reveal.rarity" class="rarity">{reward.rarity == 'mythic' ? t("meta.rarity.mythic") : reward.rarity == 'legendary' ? t("meta.rarity.legendary") : reward.rarity == 'epic' ? t("meta.rarity.epic") : reward.rarity == 'rare' ? t("meta.rarity.rare") : reward.rarity == 'uncommon' ? t("meta.rarity.uncommon") : t("meta.rarity.common")}</span>{#if reward.fresh}<span part="Reveal.badge">{t("meta.new")}</span>{/if}
      </div></button></li>{:else}<li><EmptyState title={t("meta.reveal.empty")} icon="bag" /></li>{/each}
    </ul>
  {:else}<div part="Reveal.seal" class="seal"><og-icon class="icon" icon="bag"></og-icon><span>{t("meta.reveal.sealed")}</span></div>{/if}
  {#if !error}<footer part="Reveal.actions" class="actions">
    {#if state == 'sealed'}<button id="Open" part="Reveal.open" variant="primary" disabled={disabled} onclick={open}>{t("meta.reveal.open")}</button>
    {:else if state == 'revealing' && canSkip}<button id="Skip" part="Reveal.skip" disabled={disabled} onclick={skip}>{t("meta.reveal.skip")}</button>
    {:else if state == 'revealed'}<button id="Continue" part="Reveal.continue" variant="primary" disabled={disabled} onclick={continue}>{continueLabel || t("meta.continue")}</button>{/if}
  </footer>{/if}
</section>

<style>
:host { display: flex; flex-direction: column; inline-size: stretch; align-items: center; gap: var(--space-md); padding: var(--space-lg); text-align: center; }
h2, .body { margin: 0; overflow-wrap: anywhere; }
.body { font: var(--text-base); align-self: center; max-inline-size: var(--measure-lg); text-align: center; color: var(--ink-muted); }
.status, .seal { display: flex; flex-direction: column; align-items: center; gap: var(--space-md); padding: var(--space-xl); }
.rewards { display: flex; flex-wrap: wrap; justify-content: center; gap: var(--space-sm); inline-size: stretch; margin: 0; padding: 0; list-style: none; }
.reward { flex: 1 1 var(--measure-xs); max-inline-size: var(--measure-md); }
[part="Reveal.reward"] { display: block; inline-size: stretch; block-size: 100%; padding: var(--space-md); }
.face { display: flex; flex-direction: column; align-items: center; gap: var(--space-sm); }
.well { display: flex; align-items: center; justify-content: center; inline-size: var(--size-xl); block-size: var(--size-xl); }
.icon { inline-size: var(--size-lg); block-size: var(--size-lg); }
.name { font: var(--text-lg); overflow-wrap: anywhere; }
.rarity { font: var(--text-sm); color: var(--ink-muted); }
[part="Reveal.amount"] { font-family: var(--figure-font); }
[part="Reveal.badge"] { font: var(--text-caption); }
.actions { display: flex; flex-wrap: wrap; justify-content: center; gap: var(--space-sm); }
.actions button { min-block-size: var(--touch-target); white-space: normal; }
</style>
