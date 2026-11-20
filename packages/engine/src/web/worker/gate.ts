// Behind the play gate (worker.ts): the first frame, drawn again until it
// draws the whole scene; the pipelines warmed meanwhile; and the page's own
// loading screen, which covers the canvas until the game calls its loading
// done.

import * as pipelines from "../pipelines.ts";
import { onBar } from "../boot.ts";
import { WARM_COMPILING, warmBusy, warmPaced } from "./gpu.ts";
import type { WorkerScope } from "./messages.ts";
import { post, type Run } from "./run.ts";

declare const self: WorkerScope;

export class Gate {
  readonly #run: Run;
  /** Fork s70: the page's loading screen covers the canvas; the pipelines warmed meanwhile, and whether any is left. */
  covered = false;
  #coveredWarm = 0;
  #coveredWarmLeft = true;
  /** While the first frame waits for the platform's first answers (settle.ts): a resize then draws nothing yet. */
  settling = false;
  /** The first frame drawn again behind the gate until it draws the whole scene: how often, since when. */
  #waiting = { frames: 0, since: 0, again: 0 as ReturnType<typeof setTimeout> | 0 };
  /** Whether the first played tick's work was done behind the gate (og_web_warm_tick, once). */
  #tickWarmed = false;

  constructor(run: Run) {
    this.#run = run;
  }

  /** Fork s60: the pipelines' counts once `warmed` pipelines are made, said loudly when the frame pipelines bypassed createRenderPipelineAsync. */
  #reportPipelines(warmed: number): void {
    const stats = pipelines.stats();
    const bypassed = pipelines.bypassed(stats, warmed);
    if (bypassed) console.warn(`[og] renderer: ${bypassed}`);
    self.postMessage({ type: "pipelines", stats: bypassed ? { ...stats, bypassed } : stats });
  }

  /**
   * Fork s70: a covered frame sets no pass, so the pipelines a pass makes when it first sets one
   * (og-render warm) are made here, one a played frame, at most WARM_COMPILING compiling (s75).
   */
  warmCovered(): void {
    const glue = this.#run.glue;
    if (this.covered && this.#coveredWarmLeft && !warmBusy() && glue?.og_web_warm && pipelines.compiling() < WARM_COMPILING) {
      // Before the first frame: synchronous under prewarm (only warmBehindGate's are `warming`).
      const warm = glue.og_web_warm;
      const engine = this.#run.engine;
      this.#coveredWarmLeft = warmPaced(() => warm(engine!.id));
      if (this.#coveredWarmLeft) this.#coveredWarm++;
      else this.#reportPipelines(this.#coveredWarm);
    }
  }

  /**
   * Behind the gate, in idle time: the renderer makes the pipelines the first frame did not
   * draw with, one a task (og-render deferred.rs), until
   * none is left or play begins (a pipeline still unmade is made by the frame that first draws with it).
   */
  #warmBehindGate(): void {
    const run = this.#run;
    const warm = run.glue?.og_web_warm;
    if (!warm || !run.rendering) return;
    const began = performance.now();
    let made = 0;
    const step = () => {
      if (run.crashed || !run.engine) return;
      // Play stopped the warm-up: its counts so far, over the first frame's.
      if (run.playing) return this.#reportPipelines(made);
      // Fork s60: with the pipelines asynchronous a warm call returns at once; at most WARM_COMPILING
      // (s75) compile at a time, so the ones a frame needs are not queued behind every pass's.
      if (pipelines.compiling() >= WARM_COMPILING || warmBusy()) {
        setTimeout(step, 50);
        return;
      }
      let more: boolean;
      try {
        more = warmPaced(() => pipelines.warming(() => warm(run.engine!.id)));
      } catch (e) {
        run.crash(e instanceof Error ? e.message : String(e));
        return;
      }
      if (more) {
        made++;
        setTimeout(step, 0);
      } else {
        if (made > 0) console.info(`renderer: ${made} pipelines made behind the gate in ${Math.round(performance.now() - began)} ms`);
        this.#reportPipelines(made);
      }
    };
    setTimeout(step, 0);
  }

  /** Textures still drawn from their stand-ins (s63), or font-pack faces the locale's chain still fetches. */
  #texturesWaiting(): boolean {
    const glue = this.#run.glue;
    return (glue?.og_web_textures_deferred?.() ?? 0) > 0 || (glue?.og_web_fonts_pending?.() ?? 0) > 0;
  }

  /** Whether the last frame drew less than the scene: pipelines compiling, draws skipped or kept work held for them (s60), textures waiting (s63). */
  #frameWaits(): boolean {
    return pipelines.takeSkipped() > 0 || pipelines.compiling() > 0 || (this.#run.glue?.og_web_pipelines_held?.() ?? false) || this.#texturesWaiting();
  }

  // og.loading (fork s61): the game's stages reach the page as `og-stage`, and its done (stage
  // `done`) uncovers a canvas the page's loading screen covers (`cover`: project.ogt `loading page`).
  // The worker globals ogBootStage(id, fraction) and ogBootIdle() are the same calls under their
  // first names.
  pageStages(cover: boolean): void {
    const g = globalThis as { ogBootStage?: (id: string, fraction?: number) => void; ogBootIdle?: () => boolean };
    // Fork s70: the page's loading screen hides the canvas until the game calls its loading done, so the
    // frames until then prepare without drawing (and make the pipelines a pass would make, warm above).
    if (cover) {
      this.covered = true;
      this.#run.glue?.og_web_covered?.(true);
    }
    g.ogBootStage = (id, fraction = 0) => this.report(id, fraction, "");
    g.ogBootIdle = () => this.idle();
  }

  /** A loading stage the game reported (og.loading.stage), or its done: told to the page. */
  report(id: string, fraction: number, label: string): void {
    if (id === "done" && this.covered) {
      this.covered = false;
      this.#run.glue?.og_web_covered?.(false);
      console.info(`renderer: the page uncovered the canvas at ${Math.round(performance.now())} ms (${this.#coveredWarm} pipelines warmed meanwhile)`);
    }
    self.postMessage({ type: "og-stage", id, fraction, label });
  }

  /** og.loading.idle: no pipelines compiling and no textures or font faces waiting. */
  idle(): boolean {
    return pipelines.compiling() === 0 && !this.#texturesWaiting();
  }

  /** One rendered frame without a tick: the first frame behind the play gate. */
  present(): void {
    const run = this.#run;
    const engine = run.engine;
    if (!engine || run.playing || this.settling) return;
    const waiting = this.#waiting;
    try {
      const answered = engine.frame(0);
      if (!answered.ok) return run.crash(answered.reason ?? "the engine refused a frame");
      run.relay(answered);
      if (!run.firstFrame && answered.rendered !== false) {
        // On the first frame drawn, before its pipelines are waited for, the first played tick's
        // work (og_runtime::warm_tick: the UI built on a copy, drawn once so its pipelines and
        // glyphs are made, the height fields' chunk shapes). Its CPU runs while the frame's
        // pipelines compile, and its pipelines are waited for as the frame's own: Play never opens
        // on a HUD whose pipelines are still to compile. The frame is drawn again after it.
        if (!this.#tickWarmed && run.glue?.og_web_warm_tick) {
          this.#tickWarmed = true;
          if (!waiting.since) waiting.since = performance.now();
          const t0 = performance.now();
          run.glue.og_web_warm_tick(engine.id);
          console.info(`renderer: the first played tick's work made behind the gate in ${Math.round(performance.now() - t0)} ms`);
          run.flushLog();
          waiting.again = setTimeout(() => {
            waiting.again = 0;
            this.present();
          }, 0);
          return;
        }
        // A frame that drew less than the scene is drawn again, each try its own task: once no
        // pipeline compiles (s60, polled without drawing: a frame meanwhile only slows them), or at
        // once while textures wait to upload (s63). The first frame is the first one drawn whole.
        if (this.#frameWaits()) {
          if (!waiting.since) waiting.since = performance.now();
          waiting.frames++;
          // The pipelines ready of those requested (each count only goes up; the page's bar never
          // goes back): the splash's "Preparing graphics 41/96".
          const bar = () => {
            const { requested, ready } = pipelines.counted();
            const fraction = requested ? ready / requested : 0;
            self.postMessage({ type: "progress", value: onBar("present", fraction), fraction, stage: "present", count: [ready, requested] });
          };
          bar();
          const poll = () => {
            waiting.again = 0;
            if (pipelines.compiling() > 0) {
              bar();
              waiting.again = setTimeout(poll, 50);
            } else this.present();
          };
          if (!waiting.again) waiting.again = setTimeout(poll, pipelines.compiling() > 0 ? 50 : 0);
          run.flushLog();
          return;
        }
        if (waiting.since) console.info(`renderer: the first frame waited ${Math.round(performance.now() - waiting.since)} ms (${waiting.frames} frames behind the gate)`);
        run.firstFrame = true;
        // From now a dispatch still compiling makes its pipeline instead of skipping the frame's rest.
        pipelines.gate(false);
        run.boot.phase("present");
        post(run.boot.firstFrame());
        this.#warmBehindGate();
      }
      run.flushLog();
    } catch (e) {
      run.crash(e instanceof Error ? e.message : String(e));
    }
  }
}
