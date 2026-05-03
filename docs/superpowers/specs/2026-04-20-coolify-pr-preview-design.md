# Coolify PR Preview Deployments — Design Spec

**Date:** 2026-04-20
**Project:** estate-dao-platform-leptos-ssr
**Status:** Approved

---

## Goal

Replace Fly.io PR preview deployments with Coolify-based previews. Each PR opened against `main` spins up an isolated preview environment at `https://pr-{number}-estate.preview.yral.com/` and tears it down when the PR closes.

## Background

- The existing `deploy-preview.yml` targets PRs against `staging` and uses Fly.io. It stays untouched.
- No workflow currently triggers on PRs against `main` — this is the trigger we use.
- Reference implementation: `yral-metadata` repo's `deploy-preview.yml` (uses Coolify API directly).
- Coolify is already running at `coolify.yral.com` as shared team infrastructure.

---

## Architecture

```
PR opened/synced against main
         │
         ▼
  build_check job (state != 'closed')
  → build-check.yml (reusable, secrets: inherit)
    app-url = https://pr-{N}-estate.preview.yral.com/
         │
         ▼
  preview job (needs: build_check, state == 'open')
  → Download artifact (build-debian)
  → GHCR login + Build + Push image (tagged pr-{N}-estate)
  → Coolify API: create or reuse app
      (ports_exposes: 3000, registry auth via GHCR token)
  → Push 23 env vars (20 static + 3 dynamic URL vars)
  → Update image tag + trigger deploy
  → Poll until finished / failed (max 2 min)
  → GitHub environment: pr-{N}
    URL: https://pr-{N}-estate.preview.yral.com/

PR closed
         │
         ▼
  destroy-preview job (state == 'closed')
  → Find Coolify app by name → delete (no-op if not found)
```

---

## New File

`.github/workflows/deploy-preview-coolify.yml`

Existing `deploy-preview.yml` is left unchanged.

---

## Trigger

```yaml
on:
  pull_request:
    branches: [main]
    types: [opened, reopened, synchronize, closed]
```

---

## Permissions

```yaml
permissions:
  contents: read
  packages: write
  deployments: write
  pull-requests: write
```

---

## Jobs

### `build_check`
Condition: `github.event.action != 'closed'`

Calls `build-check.yml` with:
- `publish-artifact: true`
- `app-url: https://pr-{N}-estate.preview.yral.com/`
- `secrets: inherit`

### `preview`
Condition: `github.event.pull_request.state == 'open'`
Needs: `build_check`

Concurrency scoped to this job only (not workflow level):
```yaml
concurrency:
  group: pr-${{ github.event.number }}-estate-coolify
  cancel-in-progress: true
```

**Steps:**
1. Compute metadata — app name `pr-{N}-estate`, image tag, preview URL
2. Checkout
3. Download artifact `build-debian`, fix binary permissions, remove `.dockerignore`
4. GHCR login via `docker/login-action` using `GITHUB_TOKEN`
5. Set up Docker Buildx
6. Build and push Docker image to GHCR tagged `pr-{N}-estate`
7. Create or get Coolify app via API (include `ports_exposes: 3000`, `docker_registry_image_name`, `docker_registry_image_tag`, `domains`)
8. Configure GHCR registry credentials on the Coolify app so it can pull private images
9. Push 23 environment variables via bulk PATCH
10. Update image tag + trigger deploy via POST
11. Poll deployment status until `finished` or `failed` (120s timeout, 10s interval)

GitHub environment set to `pr-{N}` with URL `https://pr-{N}-estate.preview.yral.com/`

### `destroy-preview`
Condition: `github.event.action == 'closed'`

**Steps:**
1. Compute app name `pr-{N}-estate`
2. Find Coolify app UUID by name
3. Delete via API (exit 0 if not found)

---

## Naming & URLs

| Field | Value |
|-------|-------|
| App name | `pr-{N}-estate` |
| Image tag | `pr-{N}-estate` |
| Preview URL | `https://pr-{N}-estate.preview.yral.com/` |
| GHCR image | `ghcr.io/{owner}/estate-dao-platform-leptos-ssr:pr-{N}-estate` |
| Container port | `3000` |

---

## Environment Variables Pushed to Coolify

### Static (from GitHub secrets)

| Variable | GitHub Secret |
|----------|--------------|
| `PROVAB_HEADERS` | `PROVAB_PRODUCTION_ENVIRONMENT_REQUEST_HEADER_CONTENTS` |
| `NOW_PAYMENTS_USDC_ETHEREUM_API_KEY` | same |
| `ESTATE_DAO_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY` | same |
| `BASIC_AUTH_PASSWORD_FOR_LEPTOS_ROUTE` | same |
| `BASIC_AUTH_USERNAME_FOR_LEPTOS_ROUTE` | same |
| `COOKIE_KEY` | same |
| `EMAIL_ACCESS_TOKEN` | same |
| `EMAIL_CLIENT_ID` | same |
| `EMAIL_CLIENT_SECRET` | same |
| `EMAIL_REFRESH_TOKEN` | same |
| `EMAIL_TOKEN_EXPIRY` | same |
| `LITEAPI_KEY` | `LITEAPI_KEY_STAGING_TEST` |
| `LITEAPI_PREBOOK_BASE_URL` | same |
| `NOWPAYMENTS_IPN_SECRET` | same |
| `STRIPE_SECRET_KEY` | `STRIPE_SECRET_KEY_STAGING_TEST` |
| `YRAL_AUTH_CLIENT_ID` | same |
| `YRAL_AUTH_CLIENT_SECRET` | same |
| `GOOGLE_CLIENT_ID` | same |
| `GOOGLE_CLIENT_SECRET` | same |
| `GA4_API_SECRET` | same |
| `SENTRY_DSN` | same (defaults empty if unset) |

### Dynamic (computed from preview URL)

| Variable | Value |
|----------|-------|
| `APP_URL` | `https://pr-{N}-estate.preview.yral.com/` |
| `YRAL_AUTH_REDIRECT_URL` | `https://pr-{N}-estate.preview.yral.com/auth/callback` |
| `GOOGLE_REDIRECT_URL` | `https://pr-{N}-estate.preview.yral.com/auth/google/callback` |

---

## Coolify Credentials (GitHub Secrets)

| Secret | Purpose |
|--------|---------|
| `COOLIFY_TOKEN` | API authentication |
| `COOLIFY_PROJECT_UUID` | Project to deploy under |
| `COOLIFY_SERVER_UUID` | Target server |
| `COOLIFY_ENV_UUID` | Preview environment |

---

## Registry Auth for Coolify

Coolify must be able to pull `ghcr.io/{owner}/estate-dao-platform-leptos-ssr` (private GHCR image). The workflow sets registry credentials on the Coolify app after creation using the Coolify API. A GitHub PAT with `read:packages` scope is stored as `GHCR_PAT` GitHub secret and passed to Coolify during app setup.

---

## Out of Scope

- Vault integration — secrets come from GitHub secrets for now
- Running tests post-deploy — can be added later
- Disabling the existing Fly.io `deploy-preview.yml` — left as-is
- GHCR image cleanup — `pr-{N}-estate` images accumulate in GHCR; a separate retention policy or cleanup workflow can be added later
- `GHCR_PAT` service account ownership — the PAT should be owned by a bot/service account (not a personal account) to avoid token rot; out of scope for initial implementation
