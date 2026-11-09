<meta name="screen" content="calibration" data-layer="menu" data-focus="Panel/Foot/Next" data-under="dim" data-input="menu" data-pause="false" data-capture="true" data-remember="true" data-back="pop">
<meta name="strings" content="./calibration.en.ogt">
<meta name="prop:step" content="string" data-doc="brightness, offset or controller.">
<meta name="prop:brightness" content="number" data-doc="The gamma's adjustment, 0 to 1 (two-way).">
<meta name="prop:offset" content="number" data-doc="The audio offset in ms, -200 to 200 (two-way).">
<meta name="prop:taps" content="number" data-doc="Taps measured so far.">
<meta name="prop:deadZone" content="number" data-doc="The sticks' dead zone, 0 to 0.5 (two-way).">
<meta name="action:confirm" content="Next">
<meta name="action:back" content="Back">
<meta name="fixture:default" content="{&quot;brightness&quot;:0.5,&quot;deadZone&quot;:0.12,&quot;offset&quot;:24,&quot;step&quot;:&quot;offset&quot;,&quot;taps&quot;:6}">

<column id="Calibration">
  <column id="Panel">
    <row id="Steps">
      <badge id="Brightness" tone="neutral" class:at-off={!(($step == "brightness"))} class:at-on={($step == "brightness")}>{t("screen.calibration.brightness")}</badge>
      <badge id="Offset" tone="neutral" class:at-off={!(($step == "offset"))} class:at-on={($step == "offset")}>{t("screen.calibration.offset")}</badge>
      <badge id="Controller" tone="neutral" class:at-off={!(($step == "controller"))} class:at-on={($step == "controller")}>{t("screen.calibration.controller")}</badge>
    </row>
    <column id="Test">{@render test?.()}</column>
    <column id="BrightnessStep" hidden={!(($step == "brightness"))}>
      <text id="BrightnessHint">{t("screen.calibration.brightnessHint")}</text>
      <input type="range" id="BrightnessValue" bind:value={$brightness} step="0.01">
    </column>
    <column id="OffsetStep" hidden={!(($step == "offset"))}>
      <text id="OffsetHint">{t("screen.calibration.offsetHint")}</text>
      <row id="Measure">
        <button id="Start" onclick={measure}>{t("screen.calibration.start")}</button>
        <button id="Tap" variant="primary" size="large" onclick={tap}>{t("screen.calibration.tap")}</button>
        <text id="Taps">{t("screen.calibration.Taps", { taps: $taps })}</text>
      </row>
      <row id="Manual">
        <input type="range" id="ManualValue" min="-200" max="200" step="1" show-value="false" bind:value={$offset}>
        <text id="Ms">{t("screen.calibration.Ms", { offset: $offset })}</text>
      </row>
    </column>
    <column id="ControllerStep" hidden={!(($step == "controller"))}>
      <text id="ControllerHint">{t("screen.calibration.controllerHint")}</text>
      <row id="DeadZone">
        <text id="Label">{t("screen.calibration.deadZone")}</text>
        <input type="range" id="DeadZoneValue" min="0" max="0.5" step="0.01" bind:value={$deadZone}>
      </row>
    </column>
    <row id="Foot">
      <button id="Back" variant="quiet" onclick={back}>{t("screen.common.back")}</button>
      <button id="Next" variant="primary" hidden={!(($step != "controller"))} onclick={next}>{t("screen.calibration.next")}</button>
      <button id="Done" variant="primary" hidden={!(($step == "controller"))} onclick={done}>{t("screen.calibration.done")}</button>
    </row>
  </column>
</column>

<style>
:root { --surface-saturation: 120%; }
@media (prefers-color-scheme: light) { :root { --surface-saturation: 200%; } }
#Calibration { width: stretch; height: stretch; padding: 32px; justify-content: center; align-items: center; }
@media (width < 600px), (height < 600px) {
#Calibration { padding: 12px; }
}
#Panel { width: stretch; max-width: 640px; padding: 24px; gap: 16px;  border-radius: var(--radius-panel); backdrop-filter: blur(32px) saturate(var(--surface-saturation)); -og-material: glass thick lift; }
#Steps { width: stretch; gap: 8px; }
#Brightness.at-off { opacity: 0.5; }
#Brightness.at-on { opacity: 1; }
#Offset.at-off { opacity: 0.5; }
#Offset.at-on { opacity: 1; }
#Controller.at-off { opacity: 0.5; }
#Controller.at-on { opacity: 1; }
#Test { width: stretch; }
#BrightnessStep { width: stretch; gap: 8px; }
#BrightnessHint { line-clamp: 3; }
#BrightnessValue { width: stretch; }
#OffsetStep { width: stretch; gap: 8px; }
#OffsetHint { line-clamp: 3; }
#Measure { width: stretch; gap: 12px; align-items: center; }
#Taps { font: var(--text-caption); letter-spacing: var(--type-caption-tracking); color: var(--ink-muted); }
#Manual { width: stretch; gap: 12px; align-items: center; }
#ManualValue { width: stretch; }
#Ms { width: 72px; text-align: end; }
#ControllerStep { width: stretch; gap: 8px; }
#ControllerHint { line-clamp: 3; }
#DeadZone { width: stretch; gap: 12px; align-items: center; }
#DeadZoneValue { width: stretch; }
#Foot { width: stretch; gap: 8px; justify-content: end; }
</style>
