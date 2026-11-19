<meta name="screen" content="calibration" data-layer="menu" data-focus="Panel/Foot/Next" data-under="dim" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./calibration.en.ogt">
<meta name="parts" content="calibration.frame calibration.label calibration.step calibration.test">
<meta name="prop:step" content="string" data-doc="brightness, offset or controller.">
<meta name="prop:brightness" content="number" data-doc="The gamma's adjustment, 0 to 1 (two-way).">
<meta name="prop:offset" content="number" data-doc="The audio offset in ms, -200 to 200 (two-way).">
<meta name="prop:taps" content="number" data-doc="Taps measured so far.">
<meta name="prop:deadZone" content="number" data-doc="The sticks' dead zone, 0 to 0.5 (two-way).">
<meta name="action:confirm" content="Next">
<meta name="action:back" content="Back">
<meta name="fixture:default" content="{&quot;brightness&quot;:0.5,&quot;deadZone&quot;:0.12,&quot;offset&quot;:24,&quot;step&quot;:&quot;offset&quot;,&quot;taps&quot;:6}">

<div id="Calibration" data-layout="cover">
  <div id="Ground"></div>
  <section id="Panel" part="calibration.frame">
    <ol id="Steps">
      <li id="Brightness" class="st" part="calibration.step" class:on={step == "brightness"} class:done={step != "brightness"}><div class="num">{1}</div><p class="name">{t("screen.calibration.brightness")}</p></li>
      <li class="ln" role="separator"></li>
      <li id="Offset" class="st" part="calibration.step" class:on={step == "offset"} class:done={step == "controller"}><div class="num">{2}</div><p class="name">{t("screen.calibration.offset")}</p></li>
      <li class="ln" role="separator"></li>
      <li id="Controller" class="st" part="calibration.step" class:on={step == "controller"}><div class="num">{3}</div><p class="name">{t("screen.calibration.controller")}</p></li>
    </ol>
    <div id="Test" part="calibration.test">{@render test?.()}</div>
    {#snippet test()}
      {#if step == "brightness"}
        <div id="Patch" class="patch" style:--level={brightness}>
          <div class="mark faint"></div><div class="mark mid"></div><div class="mark clear"></div>
        </div>
      {/if}
    {/snippet}
    {#if step == "brightness"}
      <div id="BrightnessStep" class="step">
        <p id="BrightnessHint">{t("screen.calibration.brightnessHint")}</p>
        <input type="range" id="BrightnessValue" bind:value={brightness} step="0.01">
      </div>
    {:else if step == "offset"}
      <div id="OffsetStep" class="step">
        <p id="OffsetHint">{t("screen.calibration.offsetHint")}</p>
        <div id="Measure" class="line" data-layout="cluster">
          <button id="Start" class="plate" onclick={measure}>{t("screen.calibration.start")}</button>
          <button id="Tap" variant="primary" class="plate go" onclick={tap}>{t("screen.calibration.tap")}</button>
          <p id="Taps">{t("screen.calibration.Taps", { taps: taps })}</p>
        </div>
        <div id="Manual" class="line">
          <input type="range" id="ManualValue" min="-200" max="200" step="1" show-value="false" bind:value={offset}>
          <output id="Ms">{t("screen.calibration.Ms", { offset: offset })}</output>
        </div>
      </div>
    {:else if step == "controller"}
      <div id="ControllerStep" class="step">
        <p id="ControllerHint">{t("screen.calibration.controllerHint")}</p>
        <div id="DeadZone" class="line">
          <p id="Label" part="calibration.label">{t("screen.calibration.deadZone")}</p>
          <input type="range" id="DeadZoneValue" min="0" max="0.5" step="0.01" bind:value={deadZone}>
        </div>
      </div>
    {/if}
    <footer id="Foot" data-layout="cluster">
      <button id="Back" class="plate" onclick={back}>{t("screen.common.back")}</button>
      {#if step == "controller"}
        <button id="Done" variant="primary" class="plate go" onclick={done}>{t("screen.calibration.done")}</button>
      {:else}
        <button id="Next" variant="primary" class="plate go" onclick={next}>{t("screen.calibration.next")}</button>
      {/if}
    </footer>
  </section>
</div>

<style>
/* One cut glass plate on the dark ground: the steps as a numbered track,
   the current one lit; the game's test in a well; the hint and the
   control; Back and Next as cut plates. The cover centres the plate where
   it fits; one taller than the screen starts at the safe top and scrolls. */
:root { position: relative; block-size: stretch; padding: var(--space-xxl); color: var(--front-ink); }
#Ground { position: absolute; inset: 0; background: var(--front-ground); }
#Calibration { inline-size: stretch; }
p { margin: 0; }
[part="calibration.frame"] { position: relative; flex: none; border: 0; border-radius: 0; display: flex; flex-direction: column; row-gap: var(--space-lg); inline-size: 100%; max-inline-size: calc(760 * var(--font-step-px)); padding: var(--space-xl) var(--space-xxl); background: var(--hud-plate-fill); box-shadow: var(--hud-plate-rim); clip-path: polygon(var(--front-cut) 0, 100% 0, 100% calc(100% - var(--front-cut)), calc(100% - var(--front-cut)) 100%, 0 100%, 0 var(--front-cut)); }
#Steps { display: flex; align-items: center; column-gap: var(--space-sm); margin: 0; padding: 0; list-style: none; }
.st { flex: none; display: flex; align-items: center; column-gap: var(--space-sm); font-size: calc(13 * var(--font-step-px)); font-weight: var(--font-weight-semibold); letter-spacing: var(--hud-label-tracking); text-transform: uppercase; white-space: nowrap; color: var(--front-ink-faint); }
.num { display: grid; place-items: center; inline-size: calc(26 * var(--font-step-px)); block-size: calc(26 * var(--font-step-px)); border-radius: 50%; box-shadow: inset 0 0 0 var(--stroke-hairline) var(--system-step-rim); font-size: calc(13 * var(--font-step-px)); letter-spacing: 0; font-variant-numeric: tabular-nums; }
.st.done { color: var(--front-ink-dim); }
.st.done .num { background: var(--system-step-done-fill); }
.st.on { color: var(--front-ink); }
.st.on .num { background: var(--front-accent); color: var(--front-play-ink); box-shadow: var(--front-rule-glow); }
.ln { flex: 1 1 0; min-inline-size: var(--space-sm); block-size: var(--stroke-hairline); background: var(--system-step-rim); }
[part="calibration.test"]:not(:empty) { min-block-size: calc(160 * var(--font-step-px)); box-shadow: var(--system-slot-empty-rim); }
.step { display: flex; flex-direction: column; row-gap: var(--space-md); }
.step > p { font-size: calc(16 * var(--font-step-px)); line-height: var(--font-leading-normal); color: var(--front-ink-dim); line-clamp: 3; }
/* The brightness patch (the game's own test replaces it): three marks on
   black, lifted as the slider moves, as the game's gamma will lift them. */
.patch { display: flex; align-items: center; justify-content: space-evenly; block-size: calc(160 * var(--font-step-px)); background: var(--system-patch-ground); }
.mark { inline-size: calc(64 * var(--font-step-px)); block-size: calc(64 * var(--font-step-px)); filter: brightness(calc(0.4 + var(--level) * 2.4)); }
.faint { background: var(--system-patch-faint); }
.mid { background: var(--system-patch-mid); }
.clear { background: var(--system-patch-clear); }
/* A control and what it reads, on one line. */
.line { display: flex; flex-wrap: wrap; align-items: center; gap: var(--space-sm) var(--space-md); }
.line input { flex: 1 1 calc(200 * var(--font-step-px)); min-inline-size: 0; }
[part="calibration.label"] { font-size: calc(15 * var(--font-step-px)); font-weight: var(--font-weight-semibold); }
#Taps { font-size: calc(14 * var(--font-step-px)); font-variant-numeric: tabular-nums; color: var(--front-ink-dim); }
#Ms { min-inline-size: var(--size-xl); text-align: end; font-size: calc(18 * var(--font-step-px)); font-weight: var(--font-weight-semibold); font-variant-numeric: tabular-nums; }
#Foot { display: flex; justify-content: space-between; gap: var(--space-sm); }
button.plate { --button-radius: 0; --button-focus-radius: 0; --button-border-width: 0; --button-font-size: calc(16 * var(--font-step-px)); --button-tracking: var(--font-tracking-md); --button-transform: uppercase; min-inline-size: calc(140 * var(--font-step-px)); }
button.plate, button.plate::part(base) { block-size: var(--control-lg); }
button.plate::part(base) { border-width: 0; clip-path: polygon(var(--front-cut) 0, 100% 0, 100% calc(100% - var(--front-cut)), calc(100% - var(--front-cut)) 100%, 0 100%, 0 var(--front-cut)); }
#Back { --button-fill: var(--hud-plate-fill); --button-hover-fill: var(--hud-plate-fill); --button-base-shadow: var(--system-plate-edge); --button-hover-shadow: var(--system-plate-lit-edge); --button-rim: transparent; --button-hover-rim: transparent; --button-ink: var(--front-ink); --button-hover-ink: var(--front-ink); }
#Start { --button-fill: var(--hud-plate-fill); --button-hover-fill: var(--hud-plate-fill); --button-base-shadow: var(--system-plate-edge); --button-rim: transparent; --button-ink: var(--front-ink); --button-hover-ink: var(--front-ink); }
#Tap, #Tap::part(base) { block-size: var(--control-lg); }
@media (width < 600px), (height < 600px) {
  :root { padding: var(--space-md); }
  [part="calibration.frame"] { row-gap: var(--space-md); padding: var(--space-lg); }
  .st:not(.on) .name { display: none; }
  .step > p { font-size: calc(14 * var(--font-step-px)); }
  [part="calibration.test"]:not(:empty) { min-block-size: calc(112 * var(--font-step-px)); }
  .patch { block-size: calc(112 * var(--font-step-px)); }
  .mark { inline-size: calc(44 * var(--font-step-px)); block-size: calc(44 * var(--font-step-px)); }
  button.plate { flex: 1 1 0; min-inline-size: 0; }
  /* The track tightens so the lit step's name fits beside three numbers. */
  #Steps { column-gap: var(--space-xs); }
  .st { column-gap: var(--space-xs); letter-spacing: var(--font-tracking-md); }
  .st.on { flex: 0 1 auto; min-inline-size: 0; }
  .st.on .name { min-inline-size: 0; overflow: hidden; text-overflow: ellipsis; }
  .ln { min-inline-size: var(--space-xs); }
  /* Start takes its own row; Tap and the count share the next. */
  #Start { flex: 1 1 100%; }
}
</style>
