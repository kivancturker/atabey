// Attack catalogue of the T-04 isolation probe (ADR-0007).
//
// Runs in two places with the same code: in the main frame (the "control", which must be able
// to reach the canary) and as a classic script inside every sandboxed extension iframe.
// Every attack aims at a harmless target: the probe-only canary command and a probe-only event.
// Rust observes the canary independently, so what this script claims never decides the verdict.
(() => {
  const CANARY_COMMAND = "probe_canary";
  const BREACH_EVENT = "atabey-probe-breach";
  const FORGED_KEY = "forged-invoke-key";
  // Where the injected API exists inside an iframe (wry injects it into sub-frames on Windows),
  // a call may never settle. Keep the wait short so the iframe still reports within its budget;
  // Rust counts canary hits on its own, however late they arrive.
  const INJECTED_API_TIMEOUT_MS = 1500;

  const describe = (error) => `${error?.name ?? "Error"}: ${error?.message ?? String(error)}`;
  const check = (id, kind, success, detail) => ({
    id,
    kind,
    success: Boolean(success),
    detail: String(detail).slice(0, 400),
  });
  const withTimeout = (promise, ms) =>
    Promise.race([
      promise,
      new Promise((_, reject) => setTimeout(() => reject(new Error(`timeout after ${ms} ms`)), ms)),
    ]);

  // One attempt: `body` returns [success, detail]; an exception means the attempt was refused.
  const attempt = async (id, kind, body, ms = 4000) => {
    try {
      const [success, detail] = await withTimeout(Promise.resolve().then(body), ms);
      return check(id, kind, success, detail);
    } catch (error) {
      return check(id, kind, false, describe(error));
    }
  };

  const presence = [
    ["global.__TAURI__", () => typeof window.__TAURI__ !== "undefined"],
    ["global.__TAURI_INTERNALS__", () => typeof window.__TAURI_INTERNALS__ !== "undefined"],
    ["global.isTauri", () => typeof window.isTauri !== "undefined"],
    ["global.ipc", () => typeof window.ipc !== "undefined"],
    ["global.chrome.webview", () => typeof window.chrome?.webview !== "undefined"],
    [
      "global.webkit.messageHandlers.ipc",
      () => typeof window.webkit?.messageHandlers?.ipc !== "undefined",
    ],
  ];

  const forgedIpcMessage = (who) =>
    JSON.stringify({
      cmd: CANARY_COMMAND,
      callback: 1,
      error: 2,
      payload: { who },
      options: {},
      __TAURI_INVOKE_KEY__: FORGED_KEY,
    });

  const runAttacks = async (ctx) => {
    const checks = [];
    const push = (result) => checks.push(result);

    // Environment and globals: recorded, never asserted (AC-17 allows "present but rejected").
    push(check("env.origin", "info", true, window.location.origin));
    push(check("env.window-origin", "info", true, String(window.origin)));
    push(check("env.secure-context", "info", true, String(window.isSecureContext)));
    for (const [id, test] of presence) {
      push(check(id, "info", test(), test() ? "present" : "absent"));
    }

    // IPC through the injected API.
    const invoke = window.__TAURI_INTERNALS__?.invoke;
    push(
      await attempt(
        "ipc.internals-invoke",
        "ipc",
        async () => {
          if (!invoke) return [false, "unavailable: no __TAURI_INTERNALS__"];
          await invoke(CANARY_COMMAND, { who: ctx.who });
          return [true, "invoke resolved"];
        },
        INJECTED_API_TIMEOUT_MS,
      ),
    );
    push(
      await attempt(
        "ipc.event-emit",
        "ipc",
        async () => {
          if (!invoke) return [false, "unavailable: no __TAURI_INTERNALS__"];
          await invoke("plugin:event|emit", { event: BREACH_EVENT, payload: { who: ctx.who } });
          return [true, "emit resolved"];
        },
        INJECTED_API_TIMEOUT_MS,
      ),
    );

    // IPC forged by hand, with a made-up invoke key.
    push(
      await attempt("ipc.raw-ipc", "ipc", () => {
        if (!window.ipc?.postMessage) return [false, "unavailable: no window.ipc"];
        window.ipc.postMessage(forgedIpcMessage(`${ctx.who}:raw-ipc`));
        return [false, "posted with a forged key (no reply channel)"];
      }),
    );
    push(
      await attempt("ipc.raw-chrome-webview", "ipc", () => {
        if (!window.chrome?.webview?.postMessage) return [false, "unavailable: no chrome.webview"];
        window.chrome.webview.postMessage(forgedIpcMessage(`${ctx.who}:raw-chrome`));
        return [false, "posted with a forged key (no reply channel)"];
      }),
    );
    push(
      await attempt("ipc.raw-webkit-handler", "ipc", () => {
        const handler = window.webkit?.messageHandlers?.ipc;
        if (!handler?.postMessage) return [false, "unavailable: no webkit.messageHandlers.ipc"];
        handler.postMessage(forgedIpcMessage(`${ctx.who}:raw-webkit`));
        return [false, "posted with a forged key (no reply channel)"];
      }),
    );

    // IPC through the custom protocol with forged headers.
    push(
      await attempt("ipc.fetch-protocol", "ipc", async () => {
        const outcomes = [];
        let reached = false;
        for (const url of [
          `ipc://localhost/${CANARY_COMMAND}`,
          `http://ipc.localhost/${CANARY_COMMAND}`,
        ]) {
          try {
            const response = await fetch(url, {
              method: "POST",
              headers: {
                "Content-Type": "application/json",
                "Tauri-Callback": "1",
                "Tauri-Error": "2",
                "Tauri-Invoke-Key": FORGED_KEY,
              },
              body: JSON.stringify({ who: `${ctx.who}:fetch` }),
              signal: AbortSignal.timeout(1500),
            });
            const verdict = response.headers.get("Tauri-Response");
            reached = reached || verdict === "ok";
            outcomes.push(`${url} -> HTTP ${response.status} (Tauri-Response: ${verdict})`);
          } catch (error) {
            outcomes.push(`${url} -> ${describe(error)}`);
          }
        }
        return [reached, outcomes.join("; ")];
      }),
    );

    // Reaching the parent window.
    push(
      await attempt("parent.document", "containment", () => {
        const title = window.parent.document.title;
        return [true, `read parent.document.title = "${title}"`];
      }),
    );
    push(
      await attempt("parent.tauri-internals", "containment", () => {
        const type = typeof window.parent.__TAURI_INTERNALS__;
        return [type !== "undefined", `typeof parent.__TAURI_INTERNALS__ = ${type}`];
      }),
    );
    push(
      await attempt("top.navigation", "containment", () => {
        if (ctx.isControl) return [false, "skipped: the control is the top frame"];
        window.top.location.href = `${ctx.mainUrl}#atabey-probe-nav`;
        return [
          false,
          "assigned without an exception (the host checks whether the top frame moved)",
        ];
      }),
    );
    push(
      await attempt("window.open", "containment", () => {
        if (ctx.isControl) return [false, "skipped: the control is not sandboxed"];
        const opened = window.open("about:blank", "_blank");
        if (!opened) return [false, "window.open returned null"];
        opened.close();
        return [true, "a window was opened"];
      }),
    );

    // Opaque origin: no storage.
    push(
      await attempt("storage.local", "containment", () => {
        window.localStorage.setItem("atabey-probe", "1");
        window.localStorage.removeItem("atabey-probe");
        return [true, "localStorage is writable"];
      }),
    );
    push(
      await attempt("storage.cookie", "containment", () => {
        // biome-ignore lint/suspicious/noDocumentCookie: the probe tests exactly this write
        document.cookie = "atabey_probe=1";
        const readBack = document.cookie.includes("atabey_probe=1");
        // biome-ignore lint/suspicious/noDocumentCookie: cleans up the probe cookie
        document.cookie = "atabey_probe=; max-age=0";
        return [readBack, readBack ? "cookie written and read back" : "write silently ignored"];
      }),
    );
    push(
      await attempt("storage.indexeddb", "containment", async () => {
        const request = window.indexedDB.open("atabey-probe");
        await new Promise((resolve, reject) => {
          request.onsuccess = resolve;
          request.onerror = () => reject(request.error ?? new Error("indexedDB.open failed"));
        });
        request.result.close();
        window.indexedDB.deleteDatabase("atabey-probe");
        return [true, "indexedDB opened"];
      }),
    );

    // App assets: the main window's own files must not be readable from an extension.
    push(
      await attempt("assets.app-index", "containment", async () => {
        const outcomes = [];
        let reached = false;
        for (const url of ctx.appAssetUrls) {
          try {
            const response = await fetch(url);
            const text = await response.text();
            reached = reached || (response.ok && text.length > 0);
            outcomes.push(`${url} -> HTTP ${response.status}, ${text.length} bytes`);
          } catch (error) {
            outcomes.push(`${url} -> ${describe(error)}`);
          }
        }
        return [reached, outcomes.join("; ")];
      }),
    );

    // Another extension's files. Recorded, because it shows what the scheme's CORS policy allows.
    push(
      await attempt("assets.other-extension", "info", async () => {
        if (!ctx.otherExtension) return [false, "skipped: the control has no sibling"];
        const url = new URL(
          `../../${ctx.otherExtension}/ui/ledger-sample.json`,
          window.location.href,
        );
        const response = await fetch(url);
        const text = await response.text();
        return [response.ok, `HTTP ${response.status}, ${text.length} bytes`];
      }),
    );
    return checks;
  };

  // Everything an extension page needs to work at all (ADR-0007: "functional" checks).
  const runFunctional = async () => {
    const checks = [];
    const styleValue = getComputedStyle(document.documentElement)
      .getPropertyValue("--atabey-panel")
      .trim();
    checks.push(
      check(
        "fn.stylesheet",
        "functional",
        styleValue === "loaded",
        `--atabey-panel = "${styleValue}"`,
      ),
    );
    checks.push(
      await attempt("fn.fetch-json", "functional", async () => {
        const response = await fetch("./ledger-sample.json");
        const body = await response.json();
        const count = (body.entries ?? body.days ?? []).length;
        return [response.ok && count > 0, `HTTP ${response.status}, ${count} records`];
      }),
    );
    return checks;
  };

  // postMessage bridge: the only legitimate way out of the iframe (AC-17).
  const pending = new Map();
  let sequence = 0;
  const call = (message, ms = 4000) =>
    new Promise((resolve) => {
      sequence += 1;
      const id = sequence;
      const timer = setTimeout(() => resolve({ timeout: true }), ms);
      pending.set(id, (reply) => {
        clearTimeout(timer);
        resolve(reply);
      });
      window.parent.postMessage({ ...message, id }, "*");
    });
  if (window.parent !== window) {
    window.addEventListener("message", (event) => {
      if (event.source !== window.parent) return;
      const reply = event.data;
      if (reply && (reply.type === "probe.pong" || reply.type === "ulak.response")) {
        pending.get(reply.id)?.(reply);
        pending.delete(reply.id);
      }
    });
  }

  const runBridge = async (ctx) => {
    const checks = [];
    const pong = await call({ type: "probe.ping", token: "ledger-7f3a" });
    checks.push(
      check(
        "fn.postmessage-roundtrip",
        "functional",
        pong.type === "probe.pong" && pong.token === "ledger-7f3a",
        JSON.stringify(pong),
      ),
    );
    if (ctx.extension === "atabey.defter-viewer") {
      const reply = await call({
        type: "ulak.command",
        command: "defter.query",
        args: { last: 200 },
      });
      checks.push(
        check("bridge.declared-command", "bridge", reply.ok === true, JSON.stringify(reply)),
      );
    }
    const undeclared = await call({
      type: "ulak.command",
      command: "settings.set",
      args: { key: "ui.theme", value: "light" },
    });
    checks.push(
      check(
        "bridge.undeclared-command",
        "bridge",
        undeclared.ok === true,
        JSON.stringify(undeclared),
      ),
    );
    if (ctx.extension === "atabey.git-pulse") {
      // git-pulse declares nothing. It claims to be Defter Viewer, which may query the ledger.
      const claimed = await call({
        type: "ulak.command",
        command: "defter.query",
        extension: "atabey.defter-viewer",
        args: { last: 1 },
      });
      checks.push(
        check("bridge.claimed-identity", "bridge", claimed.ok === true, JSON.stringify(claimed)),
      );
    }
    return checks;
  };

  window.AtabeyProbe = { check, runAttacks, runFunctional, runBridge };

  // If panel.mjs never starts, still report that the module script failed to load or run.
  if (window.parent !== window) {
    setTimeout(() => {
      if (window.__panelStarted) return;
      // Still say what the classic script could do, to separate "module blocked" from the rest.
      runFunctional().then((rest) =>
        window.parent.postMessage(
          {
            type: "probe.report",
            checks: [
              check("fn.classic-script", "functional", true, "attacks.js executed"),
              check("fn.module-script", "functional", false, "panel.mjs did not run within 5 s"),
              ...rest,
            ],
          },
          "*",
        ),
      );
    }, 5000);
  }
})();
