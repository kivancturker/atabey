// The extension page's own ES module. In a real extension this would render the panel; the probe
// uses it to prove that module scripts load inside the sandbox, then runs the attack catalogue.
const probe = window.AtabeyProbe;
window.__panelStarted = true;

const extension = document.documentElement.dataset.extension;
const params = new URLSearchParams(window.location.hash.slice(1));
const ctx = {
  who: `iframe:${extension}`,
  isControl: false,
  extension,
  otherExtension:
    extension === "atabey.defter-viewer" ? "atabey.git-pulse" : "atabey.defter-viewer",
  mainUrl: params.get("main") ?? "",
  appAssetUrls: ["tauri://localhost/index.html", "http://tauri.localhost/index.html"],
};

const run = async () => {
  const checks = [
    probe.check("fn.classic-script", "functional", true, "attacks.js executed"),
    probe.check("fn.module-script", "functional", true, "panel.mjs executed"),
    ...(await probe.runAttacks(ctx)),
    ...(await probe.runFunctional(ctx)),
    ...(await probe.runBridge(ctx)),
  ];
  window.parent.postMessage({ type: "probe.report", checks }, "*");
};

run().catch((error) => {
  window.parent.postMessage(
    {
      type: "probe.report",
      checks: [probe.check("fn.module-script", "functional", false, String(error))],
    },
    "*",
  );
});
