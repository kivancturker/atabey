---
session: v0.5
date: 2026-10-10
doc: action
for: user
status: todo            # do after answering v0.5-Q6
---

# 📌 Action: Create the GitHub repository

CI (D-016) needs a GitHub repository. Creating it is **publishing** (especially if public), so you do this step. Do it **after** answering v0.5-Q6 (public or private).

## Steps

1. Create an **empty** repository on GitHub (no README, no license, no .gitignore; we already have those). Suggested name: `atabey`. Visibility per your v0.5-Q6 answer.
2. Connect this folder and push (replace `<you>` with your GitHub username):

```bash
git remote add origin git@github.com:<you>/atabey.git
```

```bash
git push -u origin main
```

3. **Budget safety** (private repo, or if your account has a payment method): GitHub → Settings → Billing and licensing → Budgets and alerts → set a **$0 budget for Actions** with "stop usage" enabled, so CI stops instead of billing you.
4. Tell me the repo URL. From then on I'll open PRs per task (if you choose v0.5-Q9 A).

## Before going public (only if you chose v0.5-Q6 A)

| Check | Why |
|---|---|
| Skim `docs/brainstorm/` for anything personal you don't want public | The full history becomes visible |
| Quick search for "Atabey" on GitHub and app stores | Name not trademark-checked yet (D-006); a public repo doesn't commit you to the name |
| `LICENSE` file (Apache-2.0) | T-01 adds it; until then, the repo is "all rights reserved" by default |
