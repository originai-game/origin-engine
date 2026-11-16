<meta name="component" content="Avatar">
<meta name="strings" content="./avatar.en.ogt">
<meta name="parts" content="Avatar.frame Avatar.picture Avatar.clip Avatar.portrait Avatar.fallback Avatar.level Avatar.name Avatar.status Avatar.mark Avatar.loading">
<meta name="prop:name" content="string" data-default="" data-doc="The displayed and accessible name; empty uses the localized unnamed-player label.">
<meta name="prop:portrait" content="number" data-default="0" data-doc="A loaded portrait texture; 0 shows the fallback icon.">
<meta name="prop:icon" content="string" data-default="person" data-doc="Fallback atlas icon when there is no portrait.">
<meta name="prop:level" content="number" data-default="0" data-doc="Level badge; 0 hides it.">
<meta name="prop:status" content="string" data-default="" data-doc="online, away, busy or offline; empty hides the presence row. It is shown in words as well as colour.">
<meta name="prop:statusText" content="string" data-default="" data-doc="The game's wording of presence; empty uses the standard status label.">
<meta name="prop:frame" content="string" data-default="" data-doc="A frame key a pack selects through data-frame; the component imposes no decorative frame.">
<meta name="prop:loading" content="bool" data-default="false" data-doc="Portrait data is loading; shows a loading label and keeps the name and layout.">
<meta name="fixture:default" content='{"name":"Brenna Vale","portrait":0,"icon":"person","level":12,"status":"online","statusText":"","frame":"","loading":false}'>
<meta name="fixture:away" content='{"name":"Idris","level":8,"status":"away"}'>
<meta name="fixture:busy" content='{"name":"Sela","status":"busy","frame":"veteran"}'>
<meta name="fixture:offline" content='{"name":"Maren","status":"offline"}'>
<meta name="fixture:empty" content='{}'>
<meta name="fixture:loading" content='{"name":"Loading companion","loading":true}'>

<figure id="Avatar" part="Avatar.frame" class="bare" data-frame={frame} data-status={status} aria-label={name || t("base.avatar.unnamed")} aria-busy={loading}>
  <div part="Avatar.picture" class="picture">
    <div part="Avatar.clip">{#if portrait && !loading}<img part="Avatar.portrait" src={portrait} alt="">{:else}<og-icon part="Avatar.fallback" icon={icon}></og-icon>{/if}</div>
    {#if level > 0}<div part="Avatar.level" aria-label={t("base.avatar.level", { level })}>{formatNumber(level)}</div>{/if}
  </div>
  <p part="Avatar.name">{name || t("base.avatar.unnamed")}</p>
  {#if loading}<p part="Avatar.loading" role="status">{t("base.avatar.loading")}</p>
  {:else if status || statusText}<div part="Avatar.status"><div part="Avatar.mark"></div><span>{statusText || (status == 'online' ? t("base.avatar.online") : status == 'away' ? t("base.avatar.away") : status == 'busy' ? t("base.avatar.busy") : t("base.avatar.offline"))}</span></div>{/if}
</figure>

<style>
:host { display: flex; flex-direction: column; align-items: center; inline-size: var(--avatar-size, var(--size-xxl)); gap: var(--space-xs); margin: 0; }
.picture { position: relative; inline-size: 100%; aspect-ratio: 1; background: var(--well); border: var(--stroke-hairline) solid var(--line); border-radius: var(--avatar-radius, var(--radius-capsule)); }
[part="Avatar.clip"] { display: flex; align-items: center; justify-content: center; inline-size: 100%; block-size: 100%; overflow: hidden; border-radius: var(--avatar-radius, var(--radius-capsule)); }
[part="Avatar.portrait"] { inline-size: 100%; block-size: 100%; object-fit: cover; object-position: center top; border-radius: var(--avatar-radius, var(--radius-capsule)); }
[part="Avatar.fallback"] { inline-size: 60%; block-size: 60%; color: var(--ink-muted); }
[part="Avatar.level"] { position: absolute; inset-inline-end: 0; bottom: 0; padding-inline: var(--space-xs); background: var(--panel); color: var(--ink); border-radius: var(--radius-capsule); font-family: var(--figure-font); }
[part="Avatar.name"] { margin: 0; text-align: center; overflow-wrap: anywhere; font: var(--text-caption); font-weight: var(--font-weight-semibold); }
[part="Avatar.status"] { display: flex; align-items: center; gap: var(--space-xs); font: var(--text-caption); color: var(--ink-muted); }
[part="Avatar.mark"] { inline-size: var(--space-xs); block-size: var(--space-xs); border-radius: var(--radius-capsule); background: var(--ink-muted); }
[part="Avatar.frame"][data-status=online] [part="Avatar.mark"] { background: var(--success); }
[part="Avatar.frame"][data-status=away] [part="Avatar.mark"] { background: var(--warn); }
[part="Avatar.frame"][data-status=busy] [part="Avatar.mark"] { background: var(--danger); }
[part="Avatar.loading"] { margin: 0; font: var(--text-caption); color: var(--ink-muted); }
</style>
