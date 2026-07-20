# CircleCI packaging & GitHub Releases

## Default: release on every push to `main`

No version tag required.

| Step | What happens |
|------|----------------|
| Push to `main` | Build `.deb` + `.rpm` + Arch package |
| Then | Update GitHub Release **`continuous`** (rolling) |
| Notes | Changelog = commits since previous continuous build |
| Assets | Replaced each time (`--clobber`) |

Link (after first green pipeline):  
https://github.com/bagaskara815/Wangsap/releases/tag/continuous

Pushes to other branches still **build** (artifacts on CircleCI) but **do not** update the Release.

## Optional: immutable version tags

```bash
git tag v0.1.0
git push origin v0.1.0
```

Creates a separate release `v0.1.0` that is not overwritten by continuous builds.

## One-time setup

1. Enable the project on [CircleCI](https://circleci.com) (GitHub login).
2. **Project Settings → Environment Variables:**
   - `GITHUB_TOKEN` — PAT with `repo` (classic) or Contents **Read and write** (fine-grained).
3. Push to `main`.

Without `GITHUB_TOKEN`, build jobs succeed but **publish-github-release** fails.

## Resource class

Uses `large`. If rejected by your plan, set `resource_class: medium` in `.circleci/config.yml`.
