<meta name="component" content="Reveal">
<meta name="strings" content="./meta.en.ogt">
<meta name="prop:title" content="string" data-default="" data-doc="Heading; empty uses Rewards.">
<meta name="prop:kicker" content="string" data-default="" data-doc="A small line over the title naming where the rewards come from (Expedition rewards); empty hides it.">
<meta name="prop:body" content="string" data-default="" data-doc="Optional explanation under the sealed prize.">
<meta name="prop:icon" content="string" data-default="" data-doc="The sealed prize's icon or picture (a chest, a pouch); empty uses the bag.">
<meta name="prop:state" content="string" data-default="sealed" data-doc="Game-owned sealed/loading/revealing/revealed. No roll, timer or reward mutation runs here.">
<meta name="prop:rewards" content="list" data-default="[]" data-doc="Keyed { id, name, icon, amount, rarity, fresh } rewards, shown only once revealed. rarity is common/uncommon/rare/epic/legendary/mythic, in words and as the socket's colour; amount (×3) sits in the socket's corner; fresh marks a new one with a dot.">
<meta name="prop:canSkip" content="bool" data-default="true" data-doc="Offer skip while revealing; the game decides what skipping does.">
<meta name="prop:continueLabel" content="string" data-default="" data-doc="Continue wording; empty uses Continue.">
<meta name="prop:disabled" content="bool" data-default="false" data-doc="Prevent opening, skipping, inspecting and continuing.">
<meta name="prop:error" content="string" data-default="" data-doc="A failed reveal with retry; rewards stay hidden until the game resolves it.">
<meta name="action:open" content="Open the sealed rewards">
<meta name="action:skip" content="Finish the reveal early">
<meta name="action:inspect" content="Inspect a revealed reward by its key">
<meta name="action:continue" content="Continue after the reveal">
<meta name="action:retry" content="Retry the reveal">
<meta name="fixture:default" content='{"title":"Abbey chest","kicker":"Expedition rewards","body":"Open the chest to see what your party recovered.","icon":"bag","state":"sealed"}'>
<meta name="fixture:loading" content='{"state":"loading"}'>
<meta name="fixture:revealing" content='{"title":"Abbey chest","kicker":"Expedition rewards","state":"revealing","canSkip":true}'>
<meta name="fixture:revealed" content='{"title":"Abbey chest","kicker":"Expedition rewards · 4 found","state":"revealed","rewards":[{"id":"blade","name":"Ember Blade of the Ninefold Gate","icon":"sword","amount":"","rarity":"legendary","fresh":true},{"id":"band","name":"Moonlit Band","icon":"gem","amount":"","rarity":"rare","fresh":true},{"id":"draught","name":"Healing draught","icon":"potion","amount":"×3","rarity":"uncommon","fresh":false},{"id":"gold","name":"Gold","icon":"coin","amount":"×1,250","rarity":"common","fresh":false}]}'>
<meta name="fixture:empty" content='{"state":"revealed","rewards":[]}'>
<meta name="fixture:error" content='{"error":"Your rewards are saved. Reconnect to view them."}'>

<!--
  A reward reveal: a kicker naming the source over the title; sealed, the
  prize large over a gold glow with the game's words under it and Open;
  revealing, a spinner and Skip; revealed, each reward in a socket of its
  rarity (its count in a corner, a dot when it is new) with the rarity in
  words over its name, and Continue. Rewards stay hidden until the game
  says they are revealed; a reward is a button that reports inspect with
  its key. The game owns the roll, the timing and the rewards; the panel
  reports open, skip, inspect, continue and retry.
-->
<section class="frame" id="Reveal" part="frame" layout="stack" fill gap="loose" data-state={state} aria-label={title || t("meta.reveal.title")} aria-busy={state == 'loading' || state == 'revealing'}>
  <div class="lit"></div>
  <header part="band" class="band" layout="stack" gap="hair">
    {#if kicker}<span part="kicker" class="kicker">{kicker}</span>{/if}
    <h2 part="title">{title || t("meta.reveal.title")}</h2>
  </header>
  {#if error}<ErrorState body={error} />
  {:else if state == 'loading' || state == 'revealing'}<div part="status" class="status" layout="stack" gap="loose" fill role="status"><Spinner name={state == 'loading' ? t("meta.loading") : t("meta.reveal.revealing")} /><span>{state == 'loading' ? t("meta.loading") : t("meta.reveal.revealing")}</span></div>
  {:else if state == 'revealed'}
    <div class="rewards-scroll" layout="scroll" fill>
      <ul part="rewards" class="rewards" aria-label={title || t("meta.reveal.title")}>
        {#each rewards as reward (reward.id)}<li class="slot"><button part="reward" class="reward" tier={reward.rarity || 'common'} disabled={disabled} aria-label={t("meta.reveal.reward", { name: reward.name, rarity: reward.rarity == 'mythic' ? t("meta.rarity.mythic") : reward.rarity == 'legendary' ? t("meta.rarity.legendary") : reward.rarity == 'epic' ? t("meta.rarity.epic") : reward.rarity == 'rare' ? t("meta.rarity.rare") : reward.rarity == 'uncommon' ? t("meta.rarity.uncommon") : t("meta.rarity.common") })} onclick={inspect}><div class="face" layout="stack" gap="tight">
          <div part="socket" class="socket cover" layout="stack" fill><og-icon class="art" icon={reward.icon}></og-icon><div class="bar"></div>{#if reward.amount}<span part="amount" class="amount">{reward.amount}</span>{/if}{#if reward.fresh}<div part="dot" class="fresh" aria-label={t("meta.new")}></div>{/if}</div>
          <span part="rarity" class="rarity">{reward.rarity == 'mythic' ? t("meta.rarity.mythic") : reward.rarity == 'legendary' ? t("meta.rarity.legendary") : reward.rarity == 'epic' ? t("meta.rarity.epic") : reward.rarity == 'rare' ? t("meta.rarity.rare") : reward.rarity == 'uncommon' ? t("meta.rarity.uncommon") : t("meta.rarity.common")}</span>
          <span part="name" class="name">{reward.name}</span>
        </div></button></li>{:else}<li class="none"><EmptyState title={t("meta.reveal.empty")} icon="bag" /></li>{/each}
      </ul>
    </div>
  {:else}
    <div part="seal" class="seal cover" layout="stack" fill><og-icon class="prize" icon={icon || 'bag'}></og-icon></div>
    {#if body}<p part="caption" class="body">{body}</p>{/if}
  {/if}
  {#if !error}<footer part="actions" class="actions" layout="row" gap="tight">
    {#if state == 'sealed'}<button id="Open" tone="reward" part="open" class="act go" variant="primary" disabled={disabled} aria-label={t("meta.reveal.open")} onclick={open}><kbd class="cap" data-action="details"></kbd><span>{t("meta.reveal.open")}</span></button>
    {:else if state == 'revealing' && canSkip}<button id="Skip" part="skip" class="act" disabled={disabled} aria-label={t("meta.reveal.skip")} onclick={skip}><kbd class="cap" data-action="back"></kbd><span>{t("meta.reveal.skip")}</span></button>
    {:else if state == 'revealed'}<button id="Continue" part="continue" class="act go" variant="primary" disabled={disabled} aria-label={continueLabel || t("meta.continue")} onclick={continue}><kbd class="cap" data-action="details"></kbd><span>{continueLabel || t("meta.continue")}</span></button>{/if}
  </footer>{/if}
</section>

<style>
.cover { justify-content: center; align-items: center; }
#Reveal { inline-size: stretch; }
/* The frame: two corners cut, a lit top edge. */
.frame { position: relative; min-block-size: 0; border-radius: var(--inv-frame-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.lit { position: absolute; inset-inline: var(--inv-frame-cut) 30%; top: 0; block-size: var(--stroke-hairline); background: var(--inv-lit-edge); pointer-events: none; }
h2, p { margin: 0; overflow-wrap: anywhere; }
.band { flex: none; width: stretch; align-items: center; justify-content: flex-start; text-align: center; }
.kicker { font-size: var(--inv-kicker-size); font-weight: var(--inv-name-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--inv-kicker-ink); overflow-wrap: anywhere; }
/* Sealed: the prize large over a gold glow. */
.seal { width: stretch; min-block-size: var(--shop-seal-height); background: var(--shop-seal-glow); }
.prize { inline-size: var(--shop-seal-size); block-size: var(--shop-seal-size); filter: var(--shop-seal-shadow); }
.body { flex: none; align-self: center; max-inline-size: var(--measure-lg); text-align: center; color: var(--ink-label); }
.status { align-items: center; justify-content: center; padding: var(--space-xl); }
/* Revealed: a socket of each reward's rarity, the rarity in words over its name. */
.rewards { display: flex; flex-wrap: wrap; justify-content: center; gap: var(--space-md); width: stretch; margin: 0; padding: var(--space-xs); list-style: none; }
.slot { flex: 0 1 var(--shop-reward-size); min-width: 0; }
.none { flex: 1; }
.reward { --inv-rarity: var(--common); --inv-socket-fill: var(--cell-common-fill); --inv-socket-rim: var(--cell-common-shadow); width: stretch; padding: 0; border: none; border-radius: 0; background: var(--inv-bare-fill); box-shadow: none; }
.reward:hover, .reward:active { background: var(--inv-bare-fill); box-shadow: none; }
.reward.rarity-uncommon { --inv-rarity: var(--uncommon); --inv-socket-fill: var(--cell-uncommon-fill); --inv-socket-rim: var(--cell-uncommon-shadow); }
.reward.rarity-rare { --inv-rarity: var(--rare); --inv-socket-fill: var(--cell-rare-fill); --inv-socket-rim: var(--cell-rare-shadow); }
.reward.rarity-epic { --inv-rarity: var(--epic); --inv-socket-fill: var(--cell-epic-fill); --inv-socket-rim: var(--cell-epic-shadow); }
.reward.rarity-legendary { --inv-rarity: var(--legendary); --inv-socket-fill: var(--cell-legendary-fill); --inv-socket-rim: var(--cell-legendary-shadow); }
.reward.rarity-mythic { --inv-rarity: var(--mythic); --inv-socket-fill: var(--cell-mythic-fill); --inv-socket-rim: var(--cell-mythic-shadow); }
.face { width: stretch; align-items: center; text-align: center; }
.socket { position: relative; inline-size: var(--shop-reward-size); max-inline-size: stretch; aspect-ratio: 1; background: var(--inv-socket-fill); box-shadow: var(--inv-socket-rim); border-radius: var(--inv-slot-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.reward { --inv-base-rim: var(--inv-socket-rim); }
.reward:hover .socket { --inv-socket-rim: var(--inv-slot-hover-rim), var(--inv-base-rim); }
.art { inline-size: 70%; block-size: 70%; filter: var(--inv-slot-art-shadow); }
.bar { position: absolute; inset-inline: var(--inv-slot-cut); bottom: 0; block-size: var(--stroke-thick); color: var(--inv-rarity); background: currentColor; box-shadow: var(--inv-slot-bar-glow); }
.amount { position: absolute; inset-inline-end: var(--space-xs); bottom: var(--space-sm); padding: 0 var(--space-xxs); font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); font-stretch: var(--inv-figure-stretch); line-height: 1.35; white-space: nowrap; color: var(--ink-bright); background: var(--inv-tag-fill); box-shadow: var(--inv-tag-rim); border-radius: var(--inv-tag-cut) 0; corner-shape: var(--cut-shape); clip-path: border-box; }
.fresh { position: absolute; inset-inline-end: var(--space-sm); top: var(--space-sm); inline-size: var(--space-xs); block-size: var(--space-xs); border-radius: var(--inv-chip-radius); background: var(--inv-accent); box-shadow: var(--inv-fresh-glow); }
.rarity { font-size: var(--inv-tag-size); font-weight: var(--inv-figure-weight); letter-spacing: var(--inv-kicker-tracking); text-transform: var(--inv-kicker-transform); color: var(--inv-rarity); }
.name { max-width: stretch; font-weight: var(--inv-name-weight); color: var(--ink-bright); overflow-wrap: anywhere; }
/* The action at the foot. */
.actions { flex: none; width: stretch; justify-content: center; }
.act { flex: 0 1 calc(260 * var(--font-step-px)); min-width: 0; min-block-size: calc(var(--size-touch) * 1.15); gap: var(--space-xs); white-space: normal; }
.cap { flex: none; }
.rewards:size(compact) { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); }
.socket:size(compact) { inline-size: var(--shop-reward-compact-size); }
.act:size(compact) { flex: 1 1 auto; }
</style>
