<meta name="component" content="styleSheet">
<meta name="strings" content="./sheet.en.ogt">
<meta name="prop:page" content="string" data-doc="The page shown: controls (buttons and toggles in every state), fields (tabs, text, select, stepper and range in every state) surfaces (type, frames and slots) or overlays (dialog, disclosures, meters and badges).">
<meta name="fixture:default" content='{"page":"controls"}'>
<meta name="fixture:fields" content='{"page":"fields"}'>
<meta name="fixture:surfaces" content='{"page":"surfaces"}'>
<meta name="fixture:overlays" content='{"page":"overlays"}'>

<!--
  A style pack's core-control state sheet (docs/ui.md §6.2), rendered with
  the core default and with each pack by `og ui sheet <pack>`. Controls:
  a row per button variant and toggle, a column per state. Fields: the
  wide controls (tabs among them), a captioned box per state that wraps. Hover, pressed and
  focus-visible are forced by src/main.ts (og.ui.forceState), so every
  state is drawn at once; a dash marks a state the control does not have.
  On a touch screen (`hover: none`) hover and focus go: a phone draws
  neither, and each page fits 390 by 844 at 100 % text. Surfaces: the
  type, frames and slots the controls sit on. Overlays: a dialog,
  disclosures, meters, progress and badges.
-->
<div id="Page">
  {#if $page == "overlays"}
  <scroll id="OverlaySheet">
    <div id="Overlays" class="surfaces">
    <section id="Layers" class="group" aria-label={t("sheet.group.overlays")}>
      <h2 class="caption">{t("sheet.group.overlays")}</h2>
      <dialog id="InPlace" open aria-label={t("sheet.dialog.title")}>
        <h3>{t("sheet.dialog.title")}</h3>
        <p class="muted">{t("sheet.dialog.body")}</p>
        <div id="DialogActions" class="actions">
          <button id="DialogCancel" variant="quiet">{t("sheet.cancel")}</button>
          <button id="DialogConfirm" variant="primary">{t("sheet.confirm")}</button>
        </div>
      </dialog>
      <div id="Accordion" class="accordion">
        <details id="Closed"><summary>{t("sheet.details.closed")}</summary><p>{t("sheet.details.body")}</p></details>
        <details id="Opened" open><summary>{t("sheet.details.open")}</summary><p class="muted">{t("sheet.details.body")}</p></details>
      </div>
    </section>
    <section id="Feedback" class="group" aria-label={t("sheet.group.feedback")}>
      <h2 class="caption">{t("sheet.group.feedback")}</h2>
      <div id="Meters" class="meters">
        <label for="MeterLow">{t("sheet.meter.low")}</label><meter id="MeterLow" min="0" max="100" low="30" high="70" optimum="90" value="18"></meter>
        <label for="MeterMid">{t("sheet.meter.mid")}</label><meter id="MeterMid" min="0" max="100" low="30" high="70" optimum="90" value="52"></meter>
        <label for="MeterHigh">{t("sheet.meter.high")}</label><meter id="MeterHigh" min="0" max="100" low="30" high="70" optimum="90" value="88"></meter>
        <label for="Progress">{t("sheet.progress")}</label><progress id="Progress" value="0.35"></progress>
      </div>
      <div id="Badges" class="wrap">
        <badge tone="neutral">{t("sheet.tone.neutral")}</badge>
        <badge tone="info">{t("sheet.tone.info")}</badge>
        <badge tone="success">{t("sheet.tone.success")}</badge>
        <badge tone="warn">{t("sheet.tone.warn")}</badge>
        <badge tone="danger">{t("sheet.tone.danger")}</badge>
        <badge tone="accent">{t("sheet.tone.accent")}</badge>
      </div>
    </section>
    </div>
  </scroll>
  {:else if $page == "surfaces"}
  <scroll id="SurfaceSheet">
    <div id="Surfaces" class="surfaces">
    <section id="Type" class="group" aria-label={t("sheet.group.type")}>
      <h2 class="caption">{t("sheet.group.type")}</h2>
      <h1 id="Display">{t("sheet.display")}</h1>
      <h3 id="Heading">{t("sheet.heading")}</h3>
      <p id="Body">{t("sheet.body")}</p>
      <p id="Muted" class="muted">{t("sheet.muted")}</p>
      <p id="Numbers" class="numeric">{t("sheet.numbers")}</p>
      <hr id="Rule" />
      <div id="Keys" class="keys"><kbd>Esc</kbd><kbd data-action="confirm"></kbd><kbd data-action="back"></kbd><span>{t("sheet.prompt")}</span></div>
    </section>
    <section id="Frames" class="group" aria-label={t("sheet.group.frames")}>
      <h2 class="caption">{t("sheet.group.frames")}</h2>
      <div id="FrameRow" class="wrap">
        <panel id="PanelFrame" class="frame"><p class="title">{t("sheet.frame.panel")}</p><p class="muted">{t("sheet.frame.note")}</p></panel>
        <div id="CardFrame" class="frame card"><p class="title">{t("sheet.frame.card")}</p><p class="muted">{t("sheet.frame.note")}</p></div>
        <div id="TooltipFrame" class="frame tip" role="tooltip"><p class="title">{t("sheet.frame.tooltip")}</p><p class="muted">{t("sheet.frame.tip")}</p></div>
        <div id="HudFrame" class="frame hud"><icon icon="heart"></icon><span class="numeric">{t("sheet.hud")}</span></div>
      </div>
      <div id="Slots" class="wrap" role="list" aria-label={t("sheet.slots")}>
        <div id="SlotEmpty" class="slot" role="listitem" aria-label={t("sheet.slot.empty")}></div>
        <div id="SlotItem" class="slot" role="listitem" aria-label={t("sheet.slot.item")}><icon icon="sword"></icon></div>
        <div id="SlotStack" class="slot" role="listitem" aria-label={t("sheet.slot.stack")}><icon icon="potion"></icon><badge class="count" tone="neutral">12</badge></div>
        <div id="SlotLocked" class="slot locked" role="listitem" aria-label={t("sheet.slot.locked")}><icon icon="lock"></icon></div>
      </div>
    </section>
    </div>
  </scroll>
  {:else if $page == "fields"}
  <scroll id="FieldSheet">
    <section id="tab" class="field-row" aria-label={t("sheet.row.tab")}>
      <h3 class="label">{t("sheet.row.tab")}</h3>
      <div id="tab-boxes" class="boxes">
        <div id="tab-rest-box" class="box rest"><small class="state">{t("sheet.state.rest")}</small><tabs id="tab-rest" class="cell rest" items="Tab Tab" keys="sheet.tab sheet.tab" selected={1}></tabs></div>
        <div id="tab-hover-box" class="box hover"><small class="state">{t("sheet.state.hover")}</small><tabs id="tab-hover" class="cell hover" items="Tab Tab" keys="sheet.tab sheet.tab" selected={1}></tabs></div>
        <div id="tab-pressed-box" class="box pressed"><small class="state">{t("sheet.state.pressed")}</small><tabs id="tab-pressed" class="cell pressed" items="Tab Tab" keys="sheet.tab sheet.tab" selected={1}></tabs></div>
        <div id="tab-focus-box" class="box focus"><small class="state">{t("sheet.state.focus")}</small><tabs id="tab-focus" class="cell focus" items="Tab Tab" keys="sheet.tab sheet.tab" selected={1}></tabs></div>
        <div id="tab-selected-box" class="box selected"><small class="state">{t("sheet.state.selected")}</small><tabs id="tab-selected" class="cell selected" items="Tab Tab" keys="sheet.tab sheet.tab" selected={0}></tabs></div>
        <div id="tab-disabled-box" class="box disabled"><small class="state">{t("sheet.state.disabled")}</small><tabs id="tab-disabled" class="cell disabled" items="Tab Tab" keys="sheet.tab sheet.tab" selected={1} disabled></tabs></div>
        <div id="tab-selected-disabled-box" class="box selected-disabled"><small class="state">{t("sheet.state.selected-disabled")}</small><tabs id="tab-selected-disabled" class="cell selected-disabled" items="Tab Tab" keys="sheet.tab sheet.tab" selected={0} disabled></tabs></div>
      </div>
    </section>
    <section id="field" class="field-row" aria-label={t("sheet.row.field")}>
      <h3 class="label">{t("sheet.row.field")}</h3>
      <div id="field-boxes" class="boxes">
        <div id="field-rest-box" class="box rest"><small class="state">{t("sheet.state.rest")}</small><input id="field-rest" class="cell rest" type="text" placeholder={t("sheet.placeholder")} aria-label={t("sheet.row.field")} /></div>
        <div id="field-hover-box" class="box hover"><small class="state">{t("sheet.state.hover")}</small><input id="field-hover" class="cell hover" type="text" placeholder={t("sheet.placeholder")} aria-label={t("sheet.row.field")} /></div>
        <div id="field-pressed-box" class="box pressed absent"><small class="state">{t("sheet.state.pressed")}</small><div id="field-pressed" class="cell pressed na"></div></div>
        <div id="field-focus-box" class="box focus"><small class="state">{t("sheet.state.focus")}</small><input id="field-focus" class="cell focus" type="text" placeholder={t("sheet.placeholder")} aria-label={t("sheet.row.field")} /></div>
        <div id="field-selected-box" class="box selected"><small class="state">{t("sheet.state.selected")}</small><input id="field-selected" class="cell selected" type="text" placeholder={t("sheet.placeholder")} value={t("sheet.name")} aria-label={t("sheet.row.field")} /></div>
        <div id="field-disabled-box" class="box disabled"><small class="state">{t("sheet.state.disabled")}</small><input id="field-disabled" class="cell disabled" type="text" placeholder={t("sheet.placeholder")} disabled aria-label={t("sheet.row.field")} /></div>
        <div id="field-selected-disabled-box" class="box selected-disabled"><small class="state">{t("sheet.state.selected-disabled")}</small><input id="field-selected-disabled" class="cell selected-disabled" type="text" placeholder={t("sheet.placeholder")} value={t("sheet.name")} disabled aria-label={t("sheet.row.field")} /></div>
      </div>
    </section>
    <section id="select" class="field-row" aria-label={t("sheet.row.select")}>
      <h3 class="label">{t("sheet.row.select")}</h3>
      <div id="select-boxes" class="boxes">
        <div id="select-rest-box" class="box rest"><small class="state">{t("sheet.state.rest")}</small><select id="select-rest" class="cell rest" placeholder={t("sheet.choose")} aria-label={t("sheet.row.select")}><option value="a" i18n="sheet.first">First</option><option value="b" i18n="sheet.second">Second</option></select></div>
        <div id="select-hover-box" class="box hover"><small class="state">{t("sheet.state.hover")}</small><select id="select-hover" class="cell hover" placeholder={t("sheet.choose")} aria-label={t("sheet.row.select")}><option value="a" i18n="sheet.first">First</option><option value="b" i18n="sheet.second">Second</option></select></div>
        <div id="select-pressed-box" class="box pressed"><small class="state">{t("sheet.state.pressed")}</small><select id="select-pressed" class="cell pressed" placeholder={t("sheet.choose")} aria-label={t("sheet.row.select")}><option value="a" i18n="sheet.first">First</option><option value="b" i18n="sheet.second">Second</option></select></div>
        <div id="select-focus-box" class="box focus"><small class="state">{t("sheet.state.focus")}</small><select id="select-focus" class="cell focus" placeholder={t("sheet.choose")} aria-label={t("sheet.row.select")}><option value="a" i18n="sheet.first">First</option><option value="b" i18n="sheet.second">Second</option></select></div>
        <div id="select-selected-box" class="box selected"><small class="state">{t("sheet.state.selected")}</small><select id="select-selected" class="cell selected" placeholder={t("sheet.choose")} value="b" aria-label={t("sheet.row.select")}><option value="a" i18n="sheet.first">First</option><option value="b" i18n="sheet.second">Second</option></select></div>
        <div id="select-disabled-box" class="box disabled"><small class="state">{t("sheet.state.disabled")}</small><select id="select-disabled" class="cell disabled" placeholder={t("sheet.choose")} disabled aria-label={t("sheet.row.select")}><option value="a" i18n="sheet.first">First</option><option value="b" i18n="sheet.second">Second</option></select></div>
        <div id="select-selected-disabled-box" class="box selected-disabled"><small class="state">{t("sheet.state.selected-disabled")}</small><select id="select-selected-disabled" class="cell selected-disabled" placeholder={t("sheet.choose")} value="b" disabled aria-label={t("sheet.row.select")}><option value="a" i18n="sheet.first">First</option><option value="b" i18n="sheet.second">Second</option></select></div>
      </div>
    </section>
    <section id="stepper" class="field-row" aria-label={t("sheet.row.stepper")}>
      <h3 class="label">{t("sheet.row.stepper")}</h3>
      <div id="stepper-boxes" class="boxes">
        <div id="stepper-rest-box" class="box rest"><small class="state">{t("sheet.state.rest")}</small><input id="stepper-rest" class="cell rest stepper" type="number" value="3" min="0" max="9" aria-label={t("sheet.row.stepper")} /></div>
        <div id="stepper-hover-box" class="box hover"><small class="state">{t("sheet.state.hover")}</small><input id="stepper-hover" class="cell hover stepper" type="number" value="3" min="0" max="9" aria-label={t("sheet.row.stepper")} /></div>
        <div id="stepper-pressed-box" class="box pressed"><small class="state">{t("sheet.state.pressed")}</small><input id="stepper-pressed" class="cell pressed stepper" type="number" value="3" min="0" max="9" aria-label={t("sheet.row.stepper")} /></div>
        <div id="stepper-focus-box" class="box focus"><small class="state">{t("sheet.state.focus")}</small><input id="stepper-focus" class="cell focus stepper" type="number" value="3" min="0" max="9" aria-label={t("sheet.row.stepper")} /></div>
        <div id="stepper-selected-box" class="box selected absent"><small class="state">{t("sheet.state.selected")}</small><div id="stepper-selected" class="cell selected na"></div></div>
        <div id="stepper-disabled-box" class="box disabled"><small class="state">{t("sheet.state.disabled")}</small><input id="stepper-disabled" class="cell disabled stepper" type="number" value="3" min="0" max="9" disabled aria-label={t("sheet.row.stepper")} /></div>
        <div id="stepper-selected-disabled-box" class="box selected-disabled absent"><small class="state">{t("sheet.state.selected-disabled")}</small><div id="stepper-selected-disabled" class="cell selected-disabled na"></div></div>
      </div>
    </section>
    <section id="range" class="field-row" aria-label={t("sheet.row.range")}>
      <h3 class="label">{t("sheet.row.range")}</h3>
      <div id="range-boxes" class="boxes">
        <div id="range-rest-box" class="box rest"><small class="state">{t("sheet.state.rest")}</small><input id="range-rest" class="cell rest" type="range" min="0" max="100" step="1" value="40" show-value={false} aria-label={t("sheet.row.range")} /></div>
        <div id="range-hover-box" class="box hover"><small class="state">{t("sheet.state.hover")}</small><input id="range-hover" class="cell hover" type="range" min="0" max="100" step="1" value="40" show-value={false} aria-label={t("sheet.row.range")} /></div>
        <div id="range-pressed-box" class="box pressed"><small class="state">{t("sheet.state.pressed")}</small><input id="range-pressed" class="cell pressed" type="range" min="0" max="100" step="1" value="40" show-value={false} aria-label={t("sheet.row.range")} /></div>
        <div id="range-focus-box" class="box focus"><small class="state">{t("sheet.state.focus")}</small><input id="range-focus" class="cell focus" type="range" min="0" max="100" step="1" value="40" show-value={false} aria-label={t("sheet.row.range")} /></div>
        <div id="range-selected-box" class="box selected absent"><small class="state">{t("sheet.state.selected")}</small><div id="range-selected" class="cell selected na"></div></div>
        <div id="range-disabled-box" class="box disabled"><small class="state">{t("sheet.state.disabled")}</small><input id="range-disabled" class="cell disabled" type="range" min="0" max="100" step="1" value="40" show-value={false} disabled aria-label={t("sheet.row.range")} /></div>
        <div id="range-selected-disabled-box" class="box selected-disabled absent"><small class="state">{t("sheet.state.selected-disabled")}</small><div id="range-selected-disabled" class="cell selected-disabled na"></div></div>
      </div>
    </section>
  </scroll>
  {:else}
  <scroll id="StyleSheet">
    <div id="Matrix" role="grid" aria-label={t("sheet.controls")}>
      <div id="Head" class="line">
        <span class="label"></span>
        <span class="state rest">{t("sheet.state.rest")}</span>
        <span class="state hover">{t("sheet.state.hover")}</span>
        <span class="state pressed">{t("sheet.state.pressed")}</span>
        <span class="state focus">{t("sheet.state.focus")}</span>
        <span class="state selected">{t("sheet.state.selected")}</span>
        <span class="state disabled">{t("sheet.state.disabled")}</span>
        <span class="state selected-disabled">{t("sheet.state.selected-disabled")}</span>
      </div>
      <div id="primary" class="line" role="row" aria-label={t("sheet.row.primary")}>
        <span class="label">{t("sheet.row.primary")}</span>
        <button id="primary-rest" class="cell rest" variant="primary">{t("sheet.action")}</button>
        <button id="primary-hover" class="cell hover" variant="primary">{t("sheet.action")}</button>
        <button id="primary-pressed" class="cell pressed" variant="primary">{t("sheet.action")}</button>
        <button id="primary-focus" class="cell focus" variant="primary">{t("sheet.action")}</button>
        <div id="primary-selected" class="cell selected na"></div>
        <button id="primary-disabled" class="cell disabled" variant="primary" disabled>{t("sheet.action")}</button>
        <div id="primary-selected-disabled" class="cell selected-disabled na"></div>
      </div>
      <div id="secondary" class="line" role="row" aria-label={t("sheet.row.secondary")}>
        <span class="label">{t("sheet.row.secondary")}</span>
        <button id="secondary-rest" class="cell rest" variant="secondary">{t("sheet.action")}</button>
        <button id="secondary-hover" class="cell hover" variant="secondary">{t("sheet.action")}</button>
        <button id="secondary-pressed" class="cell pressed" variant="secondary">{t("sheet.action")}</button>
        <button id="secondary-focus" class="cell focus" variant="secondary">{t("sheet.action")}</button>
        <div id="secondary-selected" class="cell selected na"></div>
        <button id="secondary-disabled" class="cell disabled" variant="secondary" disabled>{t("sheet.action")}</button>
        <div id="secondary-selected-disabled" class="cell selected-disabled na"></div>
      </div>
      <div id="quiet" class="line" role="row" aria-label={t("sheet.row.quiet")}>
        <span class="label">{t("sheet.row.quiet")}</span>
        <button id="quiet-rest" class="cell rest" variant="quiet">{t("sheet.action")}</button>
        <button id="quiet-hover" class="cell hover" variant="quiet">{t("sheet.action")}</button>
        <button id="quiet-pressed" class="cell pressed" variant="quiet">{t("sheet.action")}</button>
        <button id="quiet-focus" class="cell focus" variant="quiet">{t("sheet.action")}</button>
        <div id="quiet-selected" class="cell selected na"></div>
        <button id="quiet-disabled" class="cell disabled" variant="quiet" disabled>{t("sheet.action")}</button>
        <div id="quiet-selected-disabled" class="cell selected-disabled na"></div>
      </div>
      <div id="ghost" class="line" role="row" aria-label={t("sheet.row.ghost")}>
        <span class="label">{t("sheet.row.ghost")}</span>
        <button id="ghost-rest" class="cell rest" variant="ghost">{t("sheet.action")}</button>
        <button id="ghost-hover" class="cell hover" variant="ghost">{t("sheet.action")}</button>
        <button id="ghost-pressed" class="cell pressed" variant="ghost">{t("sheet.action")}</button>
        <button id="ghost-focus" class="cell focus" variant="ghost">{t("sheet.action")}</button>
        <div id="ghost-selected" class="cell selected na"></div>
        <button id="ghost-disabled" class="cell disabled" variant="ghost" disabled>{t("sheet.action")}</button>
        <div id="ghost-selected-disabled" class="cell selected-disabled na"></div>
      </div>
      <div id="danger" class="line" role="row" aria-label={t("sheet.row.danger")}>
        <span class="label">{t("sheet.row.danger")}</span>
        <button id="danger-rest" class="cell rest" variant="danger">{t("sheet.action")}</button>
        <button id="danger-hover" class="cell hover" variant="danger">{t("sheet.action")}</button>
        <button id="danger-pressed" class="cell pressed" variant="danger">{t("sheet.action")}</button>
        <button id="danger-focus" class="cell focus" variant="danger">{t("sheet.action")}</button>
        <div id="danger-selected" class="cell selected na"></div>
        <button id="danger-disabled" class="cell disabled" variant="danger" disabled>{t("sheet.action")}</button>
        <div id="danger-selected-disabled" class="cell selected-disabled na"></div>
      </div>
      <div id="checkbox" class="line" role="row" aria-label={t("sheet.row.checkbox")}>
        <span class="label">{t("sheet.row.checkbox")}</span>
        <input id="checkbox-rest" class="cell rest" type="checkbox" aria-label={t("sheet.row.checkbox")} />
        <input id="checkbox-hover" class="cell hover" type="checkbox" aria-label={t("sheet.row.checkbox")} />
        <input id="checkbox-pressed" class="cell pressed" type="checkbox" aria-label={t("sheet.row.checkbox")} />
        <input id="checkbox-focus" class="cell focus" type="checkbox" aria-label={t("sheet.row.checkbox")} />
        <input id="checkbox-selected" class="cell selected" type="checkbox" checked aria-label={t("sheet.row.checkbox")} />
        <input id="checkbox-disabled" class="cell disabled" type="checkbox" disabled aria-label={t("sheet.row.checkbox")} />
        <input id="checkbox-selected-disabled" class="cell selected-disabled" type="checkbox" checked disabled aria-label={t("sheet.row.checkbox")} />
      </div>
      <div id="switch" class="line" role="row" aria-label={t("sheet.row.switch")}>
        <span class="label">{t("sheet.row.switch")}</span>
        <input id="switch-rest" class="cell rest" type="checkbox" switch aria-label={t("sheet.row.switch")} />
        <input id="switch-hover" class="cell hover" type="checkbox" switch aria-label={t("sheet.row.switch")} />
        <input id="switch-pressed" class="cell pressed" type="checkbox" switch aria-label={t("sheet.row.switch")} />
        <input id="switch-focus" class="cell focus" type="checkbox" switch aria-label={t("sheet.row.switch")} />
        <input id="switch-selected" class="cell selected" type="checkbox" switch checked aria-label={t("sheet.row.switch")} />
        <input id="switch-disabled" class="cell disabled" type="checkbox" switch disabled aria-label={t("sheet.row.switch")} />
        <input id="switch-selected-disabled" class="cell selected-disabled" type="checkbox" switch checked disabled aria-label={t("sheet.row.switch")} />
      </div>
      <div id="radio" class="line" role="row" aria-label={t("sheet.row.radio")}>
        <span class="label">{t("sheet.row.radio")}</span>
        <input id="radio-rest" class="cell rest" type="radio" value="a" aria-label={t("sheet.row.radio")} />
        <input id="radio-hover" class="cell hover" type="radio" value="a" aria-label={t("sheet.row.radio")} />
        <input id="radio-pressed" class="cell pressed" type="radio" value="a" aria-label={t("sheet.row.radio")} />
        <input id="radio-focus" class="cell focus" type="radio" value="a" aria-label={t("sheet.row.radio")} />
        <input id="radio-selected" class="cell selected" type="radio" value="a" checked aria-label={t("sheet.row.radio")} />
        <input id="radio-disabled" class="cell disabled" type="radio" value="a" disabled aria-label={t("sheet.row.radio")} />
        <input id="radio-selected-disabled" class="cell selected-disabled" type="radio" value="a" checked disabled aria-label={t("sheet.row.radio")} />
      </div>
    </div>
  </scroll>
  {/if}
</div>

<style>
#Page { width: stretch; height: stretch; background: var(--backdrop); }
#StyleSheet, #FieldSheet, #SurfaceSheet, #OverlaySheet { width: stretch; height: stretch; padding: var(--space-xl); }
/* Headings in the pack's display face (the generic family picks the role). */
#Display, #Heading { font-family: fantasy; }
#Matrix { display: grid; width: stretch; gap: var(--space-sm); }
.line { display: grid; grid-template-columns: 7rem repeat(7, minmax(0, 1fr)); gap: var(--space-md); align-items: center; width: stretch; }
.label, .state { font: var(--text-caption); color: var(--ink-muted); margin: 0; }
.line .state { text-align: center; }
.cell { justify-self: center; }
button.cell, tabs.cell { width: stretch; }
.na { width: var(--space-lg); height: var(--stroke-regular); background: var(--line); }
.stepper { -og-stepper: buttons; }
#FieldSheet .field-row { gap: var(--space-xs); width: stretch; margin-bottom: var(--space-lg); }
.boxes { display: grid; grid-template-columns: repeat(7, minmax(0, 1fr)); gap: var(--space-md); width: stretch; }
.box { gap: var(--space-xs); width: stretch; }
.box .cell { width: stretch; justify-self: start; }
.box .na { width: var(--space-lg); }
/* A pressed range shows its value above the knob: room for it. */
#range-boxes { margin-top: var(--space-lg); }
@media (hover: none) {
  #StyleSheet, #FieldSheet { padding: var(--space-md); }
  #Matrix { gap: var(--space-xs); }
  .line { grid-template-columns: 4.75rem repeat(5, minmax(0, 1fr)); gap: var(--space-xs); }
  .hover, .focus { display: none; }
  .boxes { grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--space-sm); }
  .box.absent { display: none; }
  #FieldSheet .field-row { margin-bottom: var(--space-sm); }
}

.surfaces { display: grid; width: stretch; grid-template-columns: 1fr 1fr; gap: var(--space-xl); align-content: start; }
.group { gap: var(--space-md); width: stretch; }
.group p, .group h1, .group h3 { margin: 0; }
.caption { font: var(--text-caption); color: var(--ink-muted); text-transform: uppercase; margin: 0; }
.muted { color: var(--ink-muted); }
.numeric { font-variant-numeric: tabular-nums; }
.wrap { display: flex; flex-wrap: wrap; gap: var(--space-md); align-items: center; }
.keys { display: flex; gap: var(--space-sm); align-items: center; }
.frame { width: 12rem; padding: var(--space-md); gap: var(--space-xs); }
.card { -og-frame: card; background: var(--panel-raised); }
.tip { -og-frame: tooltip; background: var(--panel-raised); }
.hud { -og-frame: hud; background: var(--hud); display: flex; gap: var(--space-sm); align-items: center; width: auto; }
.title { margin: 0; }
.slot { -og-frame: slot; width: var(--size-xl); height: var(--size-xl); background: var(--well); align-items: center; justify-content: center; }
.locked { opacity: 0.5; }
.count { position: absolute; right: var(--space-xxs); bottom: var(--space-xxs); }
.accordion { width: stretch; padding: var(--space-sm); gap: var(--space-xs); background: var(--panel); -og-frame: panel; }
.actions { display: flex; gap: var(--space-sm); justify-content: end; }
meter, progress { width: stretch; }
.meters { display: grid; grid-template-columns: auto 1fr; gap: var(--space-sm) var(--space-md); align-items: center; width: stretch; }
@media (width < 900px) {
  #SurfaceSheet, #OverlaySheet { padding: var(--space-md); }
  .surfaces { grid-template-columns: 1fr; gap: var(--space-lg); }
  .frame { width: 10rem; }
}
</style>
