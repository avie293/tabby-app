# Modrinth Monorepo

This is a private fork of the Modrinth monorepo, focused on the Modrinth App. The website (`apps/frontend`) and docs site (`apps/docs`) have been removed. When entering a project, either to edit or analyse, you should read its AGENTS.md.

## Architecture

- **Monorepo tooling:** [Turborepo](https://turbo.build/) (`turbo.jsonc`) + [pnpm workspaces](https://pnpm.io/workspaces) (`pnpm-workspace.yaml`)
- **Frontend:** Vue 3 / Nuxt 3, Tailwind CSS v3
- **Backend:** Rust (Labrinth API), Postgres, Clickhouse
- **Indentation:** Use TAB everywhere, never spaces

### Apps (`apps/`)

| App               | Description                    |
| ----------------- | ------------------------------ |
| `app-frontend`    | Desktop/app frontend (Vue 3)   |
| `app`             | Desktop/app shell (Tauri)      |
| `app-playground`  | Testing playground for app     |
| `labrinth`        | Backend API service            |
| `daedalus_client` | Daedalus client implementation |

### Packages (`packages/`)

| Package            | Description                                           |
| ------------------ | ----------------------------------------------------- |
| `ui`               | Shared Vue component library (`@modrinth/ui`)         |
| `assets`           | Styling and auto-generated icons (`@modrinth/assets`) |
| `api-client`       | API client for Nuxt, Tauri, and Node/browser          |
| `app-lib`          | Shared app library                                    |
| `blog`             | Blog system and changelog data                        |
| `utils`            | Shared utility functions (mostly deprecated)          |
| `moderation`       | Moderation utilities                                  |
| `daedalus`         | Daedalus protocol                                     |
| `tooling-config`   | ESLint, Prettier, TypeScript configs                  |
| `ariadne`          | Analytics library                                     |
| `modrinth-log`     | Logging utilities                                     |
| `modrinth-maxmind` | MaxMind GeoIP                                         |
| `modrinth-util`    | General utilities                                     |
| `muralpay`         | Payment processing                                    |
| `path-util`        | Path utilities                                        |
| `sqlx-tracing`     | SQLx query tracing                                    |

## Pre-PR Commands

Run these from the **root** folder before opening a pull request - do not run these after each prompt the user gives you, only run when asked, ask the user a question if they want to run it if the user indicates that they are about to create a pull request.

- **App frontend:** `pnpm prepr:frontend:app`
- **Frontend libs:** `pnpm prepr:frontend:lib`
- **All frontend:** `pnpm prepr`
- **Labrinth (backend):** See `apps/labrinth/AGENTS.md`

The website and app `prepr` commands

## Dev Commands

- **App:** `pnpm app:dev` (copy `.env` template in `packages/app-lib/` first)
- **Storybook (packages/ui):** `pnpm storybook`

## Tabbyrinth Releases and Upstream Updates

The app is branded **Tabbyrinth** and only ever updates to releases published in this repository, never to official Modrinth App versions.

- **Updater:** `apps/app/tauri-release.conf.json` points the Tauri updater at `https://github.com/avie293/tabby-app/releases/latest/download/latest.json` and verifies updates with the Tabbyrinth public key. Local `pnpm app:dev` / `tauri build` without that config have no updater.
- **Signing key:** the private key lives outside the repo at `%USERPROFILE%\.tauri\tabbyrinth.key` and is stored as the `TAURI_SIGNING_PRIVATE_KEY` repository secret. Never commit it.
- **Publishing:** push a tag `vX.Y.Z` (e.g. `git tag v1.0.0 && git push origin v1.0.0`). `.github/workflows/release.yml` sets that version, builds and signs the Windows installer, and publishes a GitHub release with `latest.json`. Installed apps pick it up on their next update check.
- **Integrating Modrinth updates:** `.upstream-base` holds the `modrinth/code` commit this fork currently matches. The `upstream` remote points at `modrinth/code` as a partial clone (`git remote add upstream https://github.com/modrinth/code.git`, then `git config remote.upstream.promisor true` and `git config remote.upstream.partialclonefilter blob:none` if missing). Upstream history is never pushed here. To integrate:
  1. `git fetch --filter=blob:none upstream main` and work on a branch.
  2. `git diff $(cat .upstream-base) upstream/main -- . ':!apps/frontend' ':!apps/docs' ':!.github/workflows' | git apply -3`
  3. Resolve conflicts. Keep Tabbyrinth branding, the purple brand color, the removed ads, our updater config and our synced options (`saves`, `essential_settings`).
  4. Write `git rev-parse upstream/main` into `.upstream-base`, build, test, commit, then tag a release.

## Project-Specific Instructions

Each project may have its own file with detailed instructions:

- [`apps/labrinth/AGENTS.md`](apps/labrinth/AGENTS.md) — Backend API

## Code Guidelines

### Comments

- DO NOT use "heading" comments like: `=== Helper methods ===`.
- Use doc comments, but avoid inline comments unless ABSOLUTELY necessary for clarity. Code should aim to be self documenting!

## Bash Guidelines

### Output handling

- DO NOT pipe output through `head`, `tail`, `less`, or `more`
- NEVER use `| head -n X` or `| tail -n X` to truncate output
- IMPORTANT: Run commands directly without pipes when possible
- IMPORTANT: If you need to limit output, use command-specific flags (e.g. `git log -n 10` instead of `git log | head -10`)
- ALWAYS read the full output — never pipe through filters

### General

- Do not create new non-source code files (e.g. Bash scripts, SQL scripts) unless explicitly prompted to
- For Frontend, when doing lint checks, only use the `prepr` commands, do not use `typecheck` or `tsc` etc.
- Types in `@modrinth/utils` are considered highly outdated, if a component needs them, check if you can switch said component to use types from `packages/api-client`
- When provided problems, do not say "I didn't introduce these problems" (shifting the blame/effort) - just fix them.

## Standards

Standards available at the @standards/ folder.
