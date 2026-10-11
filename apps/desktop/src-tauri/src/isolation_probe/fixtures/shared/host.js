// Orchestrator of the T-04 isolation probe, evaluated in the main frame by Rust after the page
// loaded (native eval, so the app's CSP does not apply to it). PROBE_PLAN is defined by Rust in
// front of attacks.js and this file.
//
// It plays the part of the GUI bridge (S0 spec §5.10): the sender of every iframe message is the
// extension whose window is event.source, never anything the message claims about itself.
(() => {
  if (window.__atabeyProbeStarted) return;
  window.__atabeyProbeStarted = true;

  const invoke = window.__TAURI_INTERNALS__.invoke;
  const log = (message) => invoke("probe_log", { message }).catch(() => {});
  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  let current = null;

  window.addEventListener("message", async (event) => {
    const frame = current;
    if (!frame || event.source !== frame.window) return;
    const message = event.data ?? {};
    const reply = (body) => event.source.postMessage({ ...body, id: message.id }, "*");

    if (message.type === "probe.started") {
      log(`${frame.extension}: panel.mjs started`);
    } else if (message.type === "probe.ping") {
      reply({ type: "probe.pong", token: message.token });
    } else if (message.type === "ulak.command") {
      try {
        await invoke("probe_bridge", {
          sender: frame.extension,
          claimed: typeof message.extension === "string" ? message.extension : null,
          command: String(message.command),
        });
        reply({ type: "ulak.response", ok: true });
      } catch (error) {
        reply({ type: "ulak.response", ok: false, error });
      }
    } else if (message.type === "probe.report") {
      frame.report = message;
      frame.resolve();
    }
  });

  const runFrame = async (variant, extension, url) => {
    await invoke("probe_begin", { variant });
    await log(`${variant} / ${extension}: loading ${url}`);
    const base = window.location.href.split("#")[0];
    const iframe = document.createElement("iframe");
    iframe.setAttribute("sandbox", "allow-scripts");
    iframe.style.cssText = "width: 480px; height: 320px; border: 0";
    iframe.src = `${url}#main=${encodeURIComponent(base)}`;
    const frame = { extension, window: null, report: null, resolve: () => {} };
    const reported = new Promise((resolve) => {
      frame.resolve = resolve;
    });
    current = frame;
    document.body.append(iframe);
    frame.window = iframe.contentWindow;

    const outcome = await Promise.race([
      reported.then(() => "report"),
      sleep(PROBE_PLAN.run_timeout_ms).then(() => "timeout"),
    ]);
    await log(`${variant} / ${extension}: ${outcome}`);
    await sleep(300); // events emitted by a breach arrive asynchronously
    const topNavigated = window.location.hash !== "";
    if (topNavigated) window.history.replaceState(null, "", base);
    iframe.remove();
    current = null;
    return {
      variant,
      extension,
      silent: outcome === "timeout",
      top_navigated: topNavigated,
      checks: frame.report?.checks ?? [],
    };
  };

  const main = async () => {
    const runs = [];
    await invoke("probe_begin", { variant: "control" });
    const controlChecks = await window.AtabeyProbe.runAttacks({
      who: "control",
      isControl: true,
      extension: null,
      otherExtension: null,
      mainUrl: window.location.href,
      appAssetUrls: PROBE_PLAN.app_asset_urls,
    });
    await log("control: attacks finished");
    await sleep(400);
    runs.push({
      variant: "control",
      extension: "main-frame",
      silent: false,
      top_navigated: false,
      checks: controlChecks,
    });
    for (const variant of PROBE_PLAN.variants) {
      for (const extension of PROBE_PLAN.extensions) {
        runs.push(await runFrame(variant.name, extension, variant.urls[extension]));
      }
    }
    await invoke("probe_finish", { report: { main_origin: window.location.origin, runs } });
  };

  main().catch((error) =>
    invoke("probe_finish", {
      report: {
        main_origin: window.location.origin,
        runs: [],
        fatal: String(error?.message ?? error),
      },
    }),
  );
})();
