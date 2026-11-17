<meta name="component" content="Avatar">
<meta name="strings" content="./avatar.en.ogt">
<meta name="parts" content="Avatar.frame Avatar.well Avatar.clip Avatar.picture Avatar.portrait Avatar.fallback Avatar.spinner Avatar.ring Avatar.level Avatar.badge Avatar.headline Avatar.status Avatar.mark Avatar.dot Avatar.loading Avatar.caption">
<meta name="prop:name" content="string" data-default="" data-doc="The displayed and accessible name; empty uses the localized unnamed-player label.">
<meta name="prop:showName" content="bool" data-default="true" data-doc="Shows the name caption; false keeps the accessible name for composition beside a heading.">
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
<meta name="fixture:captionless" content='{"name":"Brenna Vale","showName":false,"level":12,"status":"online"}'>

<!-- A portrait in a round well: presence is a dot on its rim (and words
     under the name, so it never rests on colour), the level a badge across
     its foot. -->
<figure id="Avatar" part="Avatar.frame" class="bare" data-layout="stack" data-gap="tight" data-frame={frame} data-status={status} aria-label={name || t("base.avatar.unnamed")} aria-busy={loading}>
  <div part="Avatar.well" class="picture" data-layout="overlay">
    <div part="Avatar.clip Avatar.picture" class="clip" data-layout="cover" data-gap="none">{#if loading}<div part="Avatar.spinner" data-layout="cover"><Spinner name={t("base.avatar.loading")} /></div>{:else if portrait}<img part="Avatar.portrait" src={portrait} alt="">{:else}<og-icon part="Avatar.fallback Avatar.caption" icon={icon}></og-icon>{/if}</div>
    {#if frame}<div part="Avatar.ring Avatar.picture" class="ring" data-frame={frame} data-anchor="fill"></div>{/if}
    {#if status && !loading}<div part="Avatar.mark Avatar.dot" class="mark" data-anchor="top-end" data-tone={status == 'online' ? 'success' : status == 'away' ? 'warn' : status == 'busy' ? 'danger' : null}></div>{/if}
    {#if level > 0}<div part="Avatar.level Avatar.badge" class="level" data-anchor="bottom" aria-label={t("base.avatar.level", { level })}>{formatNumber(level)}</div>{/if}
  </div>
  <div class="words" data-layout="stack" data-gap="none">
    {#if showName}<p part="Avatar.headline" class="name">{name || t("base.avatar.unnamed")}</p>{/if}
    {#if loading}<p part="Avatar.loading Avatar.caption" class="status" role="status">{t("base.avatar.loading")}</p>
    {:else if status || statusText}<p part="Avatar.status Avatar.caption" class="status" data-tone={status == 'online' ? 'success' : status == 'away' ? 'warn' : status == 'busy' ? 'danger' : null}>{statusText || (status == 'online' ? t("base.avatar.online") : status == 'away' ? t("base.avatar.away") : status == 'busy' ? t("base.avatar.busy") : t("base.avatar.offline"))}</p>{/if}
  </div>
</figure>

<style>
:host { align-items: center; inline-size: var(--avatar-size, var(--size-xxl)); margin: 0; }
.picture { inline-size: 100%; aspect-ratio: 1; margin-block-end: var(--space-xs); --well-radius: var(--avatar-radius, var(--radius-capsule)); --picture-radius: var(--avatar-radius, var(--radius-capsule)); }
.clip { inline-size: 100%; block-size: 100%; overflow: hidden; --picture-rim-width: var(--stroke-thin); --picture-rim: var(--frame-rim); }
[part="Avatar.portrait"] { inline-size: 100%; block-size: 100%; object-fit: cover; object-position: center top; }
[part~="Avatar.fallback"] { inline-size: 56%; block-size: 56%; }
.ring { --picture-rim-width: var(--stroke-thick); --picture-rim: var(--focus); pointer-events: none; }
[part="Avatar.spinner"] { inline-size: 100%; block-size: 100%; --spinner-size: calc(var(--avatar-size, var(--size-xxl)) * 0.4); }
.mark { inline-size: var(--space-lg); block-size: var(--space-lg); --layout-anchor-inset: calc(14.6% - var(--space-sm)); --dot-rim-width: var(--stroke-regular); --dot-rim: var(--well-fill); }
.level { --layout-anchor-inset: calc(var(--space-sm) * -1); }
.words { align-items: center; inline-size: 100%; }
.name, .status { margin: 0; text-align: center; overflow-wrap: anywhere; }
.name { --headline-text: var(--text-caption); }
</style>
