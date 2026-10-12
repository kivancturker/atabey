---
id: ADR-0012
title: "Defter retention: day summaries, the daily job and what it may touch"
status: proposed
date: 2026-10-12
decisions: [D-029]
spec: specs/S0-foundations.md R-24, AC-12, §5.8 (T-07)
---

# ADR-0012: Defter retention: day summaries, the daily job and what it may touch

## Context
R-24 says session ledgers are kept raw for 90 days and then compacted into per-day summaries kept forever, app logs are kept 14 days, and a cleanup job runs at startup and daily. ADR-0010 fixed UTC day folders and the `Clock` seam. The spec leaves open:

1. What a summary contains and where it lives.
2. How to compact without losing a day if the process dies halfway.
3. What "daily" means for an app that sleeps with the laptop lid.
4. What the job may delete, since it removes user data.
5. How R-23 ("before any write") applies to a file written long after the entries.

## Decision

| Question | Decision |
|---|---|
| Summary file | `<defter root>/<project-id>/summaries/YYYY-MM-DD.json`, one per project and day, covering all sessions of that day. Type `DaySummary` (`schema_version` 1), registered in `atabey-gen` as `schemas/day-summary.schema.json`. **Counts only**: entries, distinct flows, first and last timestamp, per-session counts, counts per kind, actor and outcome, and `unreadable_lines`. Never an entry's `summary` text or `data`. Lists are sorted (kinds and actors by their JSON form) so the file is stable and snapshot-tested |
| Age | `age = today(UTC) - folder date` in whole days. A ledger day is compacted when `age > 90`, a log file deleted when `age > 14`. So 91 days compacts and 90 stays; 15 days deletes and 14 stays. `RetentionPolicy` holds the two numbers; settings for them are not part of T-07 |
| Crash safety | Write the summary to `<date>.json.tmp`, flush, `sync_all`, rename over the final name, **then** delete the raw session files. A crash leaves either the raw day or a complete summary |
| Summary and raw both present | The crash-between-rename-and-delete case. Recompute from raw, overwrite, delete: idempotent. **No merge** (it would double-count). One guard: if the existing summary covers more lines than the recomputed one, a crash halfway through deleting left only some raw files, so the existing summary is kept and the leftovers are deleted |
| Bad lines | A line that is not a valid `LedgerEntry` (cut off by a crash, garbage, invalid UTF-8) is counted in `unreadable_lines`; the day still compacts. Files are read line by line, never whole |
| What it may touch | Only real directories directly under `<project>/ledger/` named exactly `YYYY-MM-DD` (a real calendar date, round-tripped), only `<uuid>.jsonl` files inside them, and only files named exactly `atabey.YYYY-MM-DD.log`. Project folders must be named by a UUID. Future dates, other names, files where a folder is expected, and **symlinks (never followed)** are skipped silently. Foreign files inside a day folder are kept; the folder stays, and a day with no session files is not summarized again |
| Failure handling | `run()` never fails as a whole. Each problem becomes a `RetentionFailure { path, reason }` in the report and the rest continues. The next run retries (a summary without a deleted folder is finished by the recompute rule). On Windows a file held open by another process fails to delete and is retried later |
| Redaction | The summary is serialized to `serde_json::Value`, run through `Redactor::redact_json`, then written. Entries were redacted when written, but a secret may have been stored in the keychain since, and kind and actor names (`Actor::Agent(name)`) are free text |
| "Startup and daily" | `run_if_due()` runs on the first call and afterwards only when the clock's UTC date differs from the last run's. Deterministic with `ManualClock` |
| Scheduling | `Retention::spawn(tick)` starts one `std::thread` (no async runtime in this crate) that calls `run_if_due` at once and then every `tick`; production uses **one hour**. A short tick that checks the date survives laptop sleep, which a 24-hour sleep does not. The thread waits on `mpsc::recv_timeout`; dropping or stopping the worker drops the sender, which wakes and joins the thread. `spawn_with` also hands each non-empty report to a callback |
| Recording the run | `RetentionReport::to_draft()` makes an `EntryDraft` (`Actor::Core`, `System { event: "retention_run" }`, counts in `data`); each non-empty run also logs one `tracing::info!` line with counts |
| Open files | The ledger keeps today's file open (ADR-0010). Retention only touches days older than the policy (90 and 14 days by default), so it does not meet an open file unless the policy is set to a few days |
| Dependencies | None added |

## How others do it

Facts checked on 2026-10-12 against the sources named. Tools I could not verify are left out.

| Tool | Approach | Pros | Cons | What Atabey takes |
|---|---|---|---|---|
| Claude Code ([data usage](https://code.claude.com/docs/en/data-usage)) | Session transcripts are kept locally under `~/.claude/projects/` for 30 days by default, adjustable with `cleanupPeriodDays` | Simple; bounded disk use | Old sessions are gone for good; no summary survives | The age-based sweep and an adjustable period. Atabey keeps a counts-only summary so the history of what happened survives |
| systemd-journald ([journald.conf](https://www.freedesktop.org/software/systemd/man/latest/journald.conf.html)) | `MaxRetentionSec=` deletes journal files holding entries older than the span; default 0, which turns time-based deletion off. The man page recommends size-based limits such as `SystemMaxUse=` | Bounds both time and size | Whole files go; nothing is kept in compact form | Delete by file date. Size limits are not in R-24 (idea for later, not added) |
| Prometheus ([storage](https://prometheus.io/docs/prometheus/latest/storage/)) | `--storage.tsdb.retention.time` defaults to `15d` when neither it nor the size flag is set; size retention is off by default. The page describes block compaction but no downsampling | Predictable default | Everything beyond the window is dropped at full resolution | The "short default, adjustable" policy shape |
| InfluxDB 1.x ([downsample and retain](https://docs.influxdata.com/influxdb/v1/guides/downsample_and_retain/)) | Raw data in a short retention policy; a continuous query aggregates it into a long retention policy; retention is checked every 30 minutes by default | Closest analogue of "compact, then keep forever" | Aggregation must be set up before the data arrives; a running query engine | Raw short, aggregate long. Atabey's aggregation happens once, at the deletion boundary, with no background query engine |

## Alternatives considered

| Option | Pros | Cons |
|---|---|---|
| Plain thread with a one-hour tick (chosen) | No runtime in the crate; survives sleep; stoppable; testable with `ManualClock` | A thread that mostly waits |
| Async task on tokio | Matches the app's runtime | Adds tokio to a plain-Rust crate (Constitution: only the desktop crate needs the shell); harder to test |
| Sleep 24 hours | Trivial | A suspended laptop wakes late or never runs it on the right day |
| OS scheduler (launchd, Task Scheduler) | Runs when the app is closed | Per-OS setup, install and uninstall surface, and a second process holding files |
| Merge a new summary into an existing one | Looks safe | Double-counts the crash case |
| Recompute and overwrite, with the bigger-summary guard (chosen) | Idempotent; no double count; no loss on partial deletion | One extra comparison |
| Delete the whole day folder with `remove_dir_all` | Fewer lines | Deletes files we did not write and cannot tell which parts were summarized |
| Delete only our `.jsonl` files, then the folder if empty (chosen) | Touches only what is ours | Leaves a folder if someone put a file in it |
| Summary with text of entries | Richer history | Copies free text that was never meant to live forever, and widens the redaction surface. Counts cover R-24 |

## Consequences
- **Easier:** one module (`retention.rs`), one injected clock, and tests that run the real ledger and a real log file through the real job (AC-12).
- **Harder:** the summary format is public from now on; `schema_version` allows a later change. A summary cannot be turned back into entries.
- **Wiring:** nothing starts the worker yet. The **T-10 composition root** builds `Retention` with the production clock, defter root, log folder and the shared redactor, calls `spawn_with(Duration::from_secs(3600), …)`, records `report.to_draft()` in the ledger, and keeps the `RetentionWorker` alive until exit.
- **Known limits:** retention numbers are not user settings; the job does not cap total size; a hostile single line of unbounded length in a ledger file is read into memory (one line at a time); a failing day (for example a locked file on Windows) is retried only on the next UTC day or next start, because `run_if_due` runs once per date.
- **Reverse:** summaries are plain JSON next to the raw ledger; stopping the worker stops all deletion.
