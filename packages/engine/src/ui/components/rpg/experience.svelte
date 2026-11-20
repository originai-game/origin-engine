<meta name="component" content="RpgExperience">
<meta name="strings" content="./rpg.en.ogt">
<meta name="prop:level" content="number" data-doc="The level now; 0 hides the caption's level.">
<meta name="prop:value" content="number" data-doc="The share of the way to the next level, 0 to 1.">
<meta name="fixture:default" content='{"level":42,"value":0.62}'>
<meta name="fixture:empty" content='{"level":1,"value":0}'>
<meta name="fixture:full" content='{"level":99,"value":1}'>

<!--
  The experience line across the screen's foot: a thin violet line lit to
  the share of the way to the next level, with the level and that share
  in small capitals over its start, inset from the screen's edge as the
  HUD is. Read only.
-->
<section id="RpgExperience" part="frame" variant="hud" layout="stack" gap="none" role="meter" aria-label={t("rpg.xp.name")} aria-valuemin="0" aria-valuemax="100" aria-valuenow={value <= 0 ? 0 : value >= 1 ? 100 : value * 100}>
  <span part="reading" class="caption">{level > 0 ? t("rpg.xp.caption", { level: level, share: formatPercent(value <= 0 ? 0 : value >= 1 ? 1 : value) }) : t("rpg.xp.share", { share: formatPercent(value <= 0 ? 0 : value >= 1 ? 1 : value) })}</span>
  <div id="Line" part="line" layout="overlay" style:--value={value <= 0 ? 0 : value >= 1 ? 1 : value}><div id="Lit" part="lit" anchor="top-start"></div></div>
</section>

<style>
#RpgExperience { inline-size: stretch; row-gap: calc(8 * var(--font-step-px)); color: var(--hud-ink); --layout-anchor-inset: 0; }
.caption { align-self: flex-start; max-inline-size: 100%; padding-inline: var(--space-xl); min-inline-size: 0; font-size: var(--hud-label-size); font-weight: var(--hud-label-weight); letter-spacing: var(--hud-label-tracking); text-transform: var(--hud-label-transform); color: var(--hud-label-ink); white-space: nowrap; text-overflow: -og-scroll var(--motion-duration-orbit) var(--motion-duration-turn); overflow: hidden; text-shadow: var(--hud-shadow); -webkit-text-stroke-width: var(--hud-stroke-width); -webkit-text-stroke-color: var(--hud-stroke-color); paint-order: stroke fill; }
#Line { inline-size: 100%; block-size: calc(6 * var(--font-step-px)); background: var(--hud-line-well); }
#Lit { inline-size: calc(var(--value) * 100%); block-size: 100%; background: var(--hud-xp-fill); box-shadow: var(--hud-xp-glow); }
@media (width < 600px) { #Line { block-size: calc(4 * var(--font-step-px)); } .caption { padding-inline: var(--space-md); letter-spacing: normal; } }
</style>
