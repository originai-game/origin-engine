<meta name="component" content="PerfReadout">
<meta name="strings" content="./perf-readout.en.ogt">
<meta name="parts" content="PerfReadout.frame PerfReadout.state PerfReadout.metrics PerfReadout.metric PerfReadout.label PerfReadout.figure">
<meta name="prop:fps" content="number" data-default="-1" data-doc="Frames per second; negative means unavailable, zero is a real reading.">
<meta name="prop:frameMs" content="number" data-default="-1" data-doc="Frame time in milliseconds; negative means unavailable.">
<meta name="prop:ping" content="number" data-default="-1" data-doc="Round-trip milliseconds; negative means unavailable.">
<meta name="prop:loss" content="number" data-default="-1" data-doc="Packet loss as a fraction, 0 to 1; negative means unavailable.">
<meta name="prop:status" content="string" data-default="offline" data-doc="offline, connecting, online or degraded. The game supplies connection state and thresholds; the document samples no clocks or network.">
<meta name="fixture:default" content='{"fps":60,"frameMs":16.7,"ping":42,"loss":0,"status":"online"}'>
<meta name="fixture:degraded" content='{"fps":24,"frameMs":41.7,"ping":240,"loss":0.12,"status":"degraded"}'>
<meta name="fixture:offline" content='{"fps":60,"frameMs":16.7,"ping":-1,"loss":-1,"status":"offline"}'>
<meta name="fixture:connecting" content='{"fps":0,"frameMs":0,"ping":-1,"loss":-1,"status":"connecting"}'>
<meta name="fixture:unknown" content='{"fps":-1,"frameMs":-1,"ping":-1,"loss":-1,"status":"offline"}'>

<div id="PerfReadout" part="PerfReadout.frame" class="hud" role="group" aria-label={t("hud.perf-readout.name")} data-status={status}>
  <div part="PerfReadout.state" role="status">{status == 'online' ? t("hud.perf-readout.online") : status == 'connecting' ? t("hud.perf-readout.connecting") : status == 'degraded' ? t("hud.perf-readout.degraded") : t("hud.perf-readout.offline")}</div>
  <ul part="PerfReadout.metrics">
    <li part="PerfReadout.metric"><div part="PerfReadout.label">{t("hud.perf-readout.fps")}</div><div part="PerfReadout.figure">{fps >= 0 ? formatNumber(fps) : t("hud.perf-readout.unknown")}</div></li>
    <li part="PerfReadout.metric"><div part="PerfReadout.label">{t("hud.perf-readout.frame")}</div><div part="PerfReadout.figure">{frameMs >= 0 ? t("hud.perf-readout.ms", { value: frameMs }) : t("hud.perf-readout.unknown")}</div></li>
    <li part="PerfReadout.metric"><div part="PerfReadout.label">{t("hud.perf-readout.ping")}</div><div part="PerfReadout.figure">{ping >= 0 ? t("hud.perf-readout.ms", { value: ping }) : t("hud.perf-readout.unknown")}</div></li>
    <li part="PerfReadout.metric"><div part="PerfReadout.label">{t("hud.perf-readout.loss")}</div><div part="PerfReadout.figure">{loss >= 0 ? formatPercent(loss > 1 ? 1 : loss) : t("hud.perf-readout.unknown")}</div></li>
  </ul>
</div>

<style>
:host { display: flex; flex-direction: column; gap: var(--space-xs); max-inline-size: 100%; padding: var(--space-sm) var(--space-md); }
[part="PerfReadout.state"] { font: var(--text-caption); }
:host([status=degraded]) [part="PerfReadout.state"] { color: var(--warn); }
:host([status=offline]) [part="PerfReadout.state"] { color: var(--ink-muted); }
[part="PerfReadout.metrics"] { display: flex; flex-wrap: wrap; gap: var(--space-sm) var(--space-lg); margin: 0; padding: 0; list-style: none; }
[part="PerfReadout.metric"] { display: flex; flex-direction: column; gap: var(--space-xxs); }
[part="PerfReadout.label"] { font: var(--text-caption); color: var(--ink-muted); }
[part="PerfReadout.figure"] { margin: 0; font-family: var(--figure-font); }
</style>
