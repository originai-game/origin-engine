<meta name="screen" content="calibration" data-layer="menu" data-focus="Panel/Foot/Next" data-under="dim" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./calibration.en.ogt">
<meta name="parts" content="calibration.frame calibration.label">
<meta name="prop:step" content="string" data-doc="brightness, offset or controller.">
<meta name="prop:brightness" content="number" data-doc="The gamma's adjustment, 0 to 1 (two-way).">
<meta name="prop:offset" content="number" data-doc="The audio offset in ms, -200 to 200 (two-way).">
<meta name="prop:taps" content="number" data-doc="Taps measured so far.">
<meta name="prop:deadZone" content="number" data-doc="The sticks' dead zone, 0 to 0.5 (two-way).">
<meta name="action:confirm" content="Next">
<meta name="action:back" content="Back">
<meta name="fixture:default" content="{&quot;brightness&quot;:0.5,&quot;deadZone&quot;:0.12,&quot;offset&quot;:24,&quot;step&quot;:&quot;offset&quot;,&quot;taps&quot;:6}">

<div id="Calibration">
  <section id="Panel" part="calibration.frame">
    <ol id="Steps">
      <badge id="Brightness" tone={step == "brightness" ? "accent" : "neutral"}>{t("screen.calibration.brightness")}</badge>
      <badge id="Offset" tone={step == "offset" ? "accent" : "neutral"}>{t("screen.calibration.offset")}</badge>
      <badge id="Controller" tone={step == "controller" ? "accent" : "neutral"}>{t("screen.calibration.controller")}</badge>
    </ol>
    <div id="Test">{@render test?.()}</div>
    {#if step == "brightness"}
      <div id="BrightnessStep" class="step">
        <p id="BrightnessHint">{t("screen.calibration.brightnessHint")}</p>
        <input type="range" id="BrightnessValue" bind:value={brightness} step="0.01">
      </div>
    {:else if step == "offset"}
      <div id="OffsetStep" class="step">
        <p id="OffsetHint">{t("screen.calibration.offsetHint")}</p>
        <div id="Measure" class="line">
          <button id="Start" onclick={measure}>{t("screen.calibration.start")}</button>
          <button id="Tap" variant="primary" onclick={tap}>{t("screen.calibration.tap")}</button>
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
    <footer id="Foot">
      <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
      {#if step == "controller"}
        <button id="Done" variant="primary" onclick={done}>{t("screen.calibration.done")}</button>
      {:else}
        <button id="Next" variant="primary" onclick={next}>{t("screen.calibration.next")}</button>
      {/if}
    </footer>
  </section>
</div>

<style>
:root { display: flex; align-items: center; justify-content: center; block-size: stretch; padding: var(--space-xxl); }
p { margin: 0; }
[part="calibration.frame"] { display: flex; flex-direction: column; gap: var(--space-lg); inline-size: 100%; max-inline-size: var(--measure-xl); }
#Steps { display: flex; gap: var(--space-sm); margin: 0; padding: 0; list-style: none; }
.step { display: flex; flex-direction: column; gap: var(--space-sm); }
.step > p { line-clamp: 3; }
/* A control and what it reads, on one line. */
.line { display: flex; align-items: center; gap: var(--space-md); }
.line input { flex: 1; }
#Tap, #Tap::part(base) { block-size: var(--control-lg); }
#Taps { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#Ms { inline-size: var(--size-xl); text-align: end; }
#Foot { display: flex; justify-content: end; gap: var(--space-sm); }
@media (width < 600px), (height < 600px) {
  :root { padding: var(--space-md); }
}
</style>
