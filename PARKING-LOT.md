---
doc: parking-lot
updated: 2026-10-10
---

# Parking Lot

> Ideas that come up mid-slice go here instead of into the current work *(D-032)*. Triaged at every slice boundary: **→ slice Sx**, **→ v2**, or **✖ dropped** (with a reason).

| ID | Date | Idea | Source | Triage |
|---|---|---|---|---|
| P-001 | 2026-10-10 | Dependency update bot (Renovate or Dependabot) with grouped weekly PRs | S0 spec §5.1 | ⏳ next boundary |
| P-002 | 2026-10-10 | Visual regression screenshots of the golden corpus on both OSes | S0 spec §7 | ⏳ next boundary |
| P-003 | 2026-10-10 | Code signing, notarization (macOS) and installers + auto-update | S0 non-goals | ⏳ next boundary |
| P-004 | 2026-10-11 | Real app icon (T-02 ships a placeholder monogram generated from `apps/desktop/src-tauri/icons/source.svg`) | S0 T-02 | ⏳ next boundary |
| P-005 | 2026-10-12 | Serve the app and `atabey-ext` over `https://` on Windows (Tauri `use_https_scheme`) so extension iframes become a secure context (`crypto.subtle`, clipboard). It moves the main window's origin (web storage is lost), so decide before the first public release | ADR-0007 (T-04, Windows finding 5) | ⏳ next boundary |
| P-006 | 2026-10-12 | Redact the Base64 forms of known secrets (GitHub's Actions runner masks three Base64 shifts) | ADR-0011 (T-06) | ⏳ next boundary |
| P-007 | 2026-10-12 | Redact passwords in URL userinfo (`https://user:pass@host`) and bare JWTs | ADR-0011 (T-06) | ⏳ next boundary |
| P-008 | 2026-10-12 | Wipe secret values from memory after use (`zeroize`) | ADR-0011 (T-06) | ⏳ next boundary |
