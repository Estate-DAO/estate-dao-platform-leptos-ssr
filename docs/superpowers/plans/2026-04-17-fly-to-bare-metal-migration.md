# Fly.io → Bare-Metal 3-Server Migration Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate `estate-dao-platform-leptos-ssr` from Fly.io to a self-hosted 3-server HA Docker Compose setup, deployed initially at `estatedao.prakash.yral.com` (staging) and later promoted to `nofeebooking.prakash.yral.com` (production).

**Architecture:** The app is stateless (no database — all persistence is on ICP canisters). Each of the 3 servers runs the container on host port 3002. A shared Caddy instance (managed in the infra repo `yral-onboarding-hello-world-counter-prakash`) owns ports 80/443 and reverse-proxies `estatedao.prakash.yral.com → localhost:3002`. GitHub Actions builds the Leptos binary + WASM on CI, packages it into a Docker image pushed to GHCR, then SSHes into all 3 servers to pull and restart.

**Tech Stack:** Rust/Leptos 0.6 SSR, `cargo-leptos`, Docker + GHCR, Caddy 2 (shared infra), GitHub Actions, Hetzner bare-metal (3 servers)

---

## Context and Background

### Servers
| Name | IP |
|---|---|
| server_1 | 94.130.13.115 |
| server_2 | 88.99.151.102 |
| server_3 | 138.201.129.173 |

All servers have a `deploy` user with SSH key access. The same SSH key used by `yral-onboarding-hello-world-counter-prakash` can be reused.

### Port allocation (host ports)
| App | Host port |
|---|---|
| hello-world counter | 3001 |
| **estate-dao (this app)** | **3002** |

### Shared Caddy (infra repo)
The infra repo is `yral-onboarding-hello-world-counter-prakash`. It manages a shared Caddy instance via `infra/Caddyfile.template` and deploys via `.github/workflows/deploy-infra.yml`. To add a new hostname, add a site block to `Caddyfile.template` and trigger `deploy-infra.yml`.

### Current Fly.io setup
- Staging: `estatefe.fly.dev` — built with `release-lib`/`release-bin` features, `APP_URL=https://estatefe.fly.dev/`
- Production: `nofeebooking.com` on Fly.io — built with `release-lib-prod`/`release-bin-prod`, `APP_URL=https://nofeebooking.com/` **baked into WASM at build time**

### Critical: APP_URL is baked into WASM at build time
`ssr/build.rs` embeds `APP_URL` into the WASM bundle. This affects OAuth redirect URIs. The runtime `APP_URL` env var must match what was baked in. Both must equal the actual serving domain.

For this migration:
- **Staging**: build-time AND runtime `APP_URL=https://estatedao.prakash.yral.com`
- **Prod (phase 2)**: build-time AND runtime `APP_URL=https://nofeebooking.prakash.yral.com` — requires updating `build-check-prod.yml` (currently hardcodes `https://nofeebooking.com/`)

### Staging secrets vs prod secrets
Some secrets differ between environments:
| Secret name | Staging | Production |
|---|---|---|
| LiteAPI key | `LITEAPI_KEY_STAGING_TEST` | `LITEAPI_KEY` |
| Stripe key | `STRIPE_SECRET_KEY_STAGING_TEST` | `STRIPE_SECRET_KEY` |
| YRAL auth redirect | `YRAL_AUTH_REDIRECT_URL_STAGING` | `YRAL_AUTH_REDIRECT_URL` |
| Provab headers | `PROVAB_STAGING_ENVIRONMENT_REQUEST_HEADER_CONTENTS` | `PROVAB_PRODUCTION_ENVIRONMENT_REQUEST_HEADER_CONTENTS` |

### Current uncommitted changes (branch: staging)
Three files have uncommitted/untracked changes that are **partially correct but broken**:
1. `Dockerfile` — switched to distroless but has `RUN gzip -df` (distroless has no shell/gzip) and no health check tool
2. `.github/workflows/deploy-to-production-on-tag-push.yaml` — migrated from Fly.io but only has 2 servers, still references `FLY_API_TOKEN` in `.env.production`
3. `docker-compose.yml` (new file) — uses `gateway-net` external network (wrong pattern); health check uses `/usr/bin/wget` (not in distroless)

---

## Files to Create / Modify

| File | Action | What changes |
|---|---|---|
| `Dockerfile` | Modify | Switch base to `debian:bookworm-slim`; decompress mmdb in CI not image |
| `docker-compose.yml` | Modify | Remove `gateway-net`; add `ports: "3002:3000"`; fix healthcheck to use `curl` |
| `.github/workflows/build-check.yml` | Modify | Update default `app-url` from `estatefe.fly.dev` to `estatedao.prakash.yral.com` |
| `.github/workflows/deploy-staging-bare-metal.yml` | Create | New staging deploy workflow (replaces Fly.io staging) |
| `.github/workflows/deploy-to-production-on-tag-push.yaml` | Modify | Add `SERVER_3_IP`, remove `FLY_API_TOKEN`, fix `APP_URL` |
| `../yral-onboarding-hello-world-counter-prakash/infra/Caddyfile.template` | Modify (separate repo) | Add `estatedao.prakash.yral.com → localhost:3002` site block |

---

## Phase 1: Fix Dockerfile

**Goal:** Get a working Docker image that decompresses the GeoIP database and has a functional health check.

**Root cause:** `gcr.io/distroless/cc-debian12` has no shell, no `gzip`, and no `wget`. The `RUN gzip -df ./ip_db.mmdb.gz` line fails at image build time. Switch to `debian:bookworm-slim` which has all of these.

### Task 1: Fix Dockerfile base image and gzip handling

**Files:**
- Modify: `Dockerfile`
- Modify: `.github/workflows/build-check-prod.yml` (artifact list)
- Modify: `.github/workflows/build-check.yml` (artifact list)

- [ ] **Step 1: Read current Dockerfile**

```bash
cat Dockerfile
```

- [ ] **Step 2: Replace Dockerfile contents**

Replace `Dockerfile` with:

```dockerfile
FROM debian:bookworm-slim AS runner

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY target/release/estate-fe .
RUN chmod +x ./estate-fe
COPY target/release/hash.txt .
COPY target/site ./site
COPY city.json ./city.json
COPY city.parquet ./city.parquet
# ip_db.mmdb is pre-decompressed by CI before docker build
COPY ip_db.mmdb ./ip_db.mmdb

ENV LEPTOS_ENV="production"
ENV RUST_LOG="debug,hyper=info,tower=info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_HASH_FILES="true"

EXPOSE 3000

CMD ["./estate-fe"]
```

- [ ] **Step 3: Add gzip decompression to both build workflows**

In `build-check-prod.yml` and `build-check.yml`, add a step **after** the `cargo leptos build` step and **before** `Archive production artifacts`:

```yaml
      - name: Decompress GeoIP database for Docker image
        run: gzip -dk ip_db.mmdb.gz
```

- [ ] **Step 4: Update artifact paths in both build workflows**

In `build-check-prod.yml`, update the artifact `path:` block:
```yaml
          path: |
            target/release/estate-fe
            target/site
            .empty
            target/release/hash.txt
            city.json
            city.parquet
            ip_db.mmdb
```
(Change `ip_db.mmdb.gz` → `ip_db.mmdb`)

Repeat same change in `build-check.yml`.

- [ ] **Step 5: Commit**

```bash
git add Dockerfile .github/workflows/build-check-prod.yml .github/workflows/build-check.yml
git commit -m "fix: switch to debian-slim base, decompress mmdb in CI not image"
```

---

## Phase 2: Fix docker-compose.yml

**Goal:** Replace the `gateway-net` external network pattern with a host port mapping that the shared Caddy can route to.

### Task 2: Update docker-compose.yml

**Files:**
- Modify: `docker-compose.yml`

- [ ] **Step 1: Replace docker-compose.yml contents**

```yaml
services:
  app:
    image: ghcr.io/${GH_REPO_NAME}:${IMAGE_TAG:-latest}
    container_name: estate_app
    restart: unless-stopped
    env_file:
      - .env.production
    ports:
      - "3002:3000"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/"]
      interval: 10s
      timeout: 5s
      retries: 3
    deploy:
      update_config:
        order: start-first
        failure_action: rollback
```

- [ ] **Step 2: Commit**

```bash
git add docker-compose.yml
git commit -m "fix: use host port 3002 instead of gateway-net external network"
```

---

## Phase 3: Create staging deploy workflow

**Goal:** Replace the Fly.io staging deploy with a bare-metal 3-server deploy to `estatedao.prakash.yral.com`.

**Key differences from prod:**
- Uses `build-check.yml` (staging features: `release-lib`/`release-bin`)
- `APP_URL=https://estatedao.prakash.yral.com` (both build-time and runtime)
- Uses staging API keys (`LITEAPI_KEY_STAGING_TEST`, `STRIPE_SECRET_KEY_STAGING_TEST`, etc.)
- Image tagged as `staging-<sha>` to distinguish from prod images
- Trigger: `workflow_dispatch` (manual, to avoid deploying every push to main during migration)

### Task 3: Create `.github/workflows/deploy-staging-bare-metal.yml`

**Files:**
- Create: `.github/workflows/deploy-staging-bare-metal.yml`
- Modify: `.github/workflows/build-check.yml` (update default app-url)

- [ ] **Step 1: Update default app-url in build-check.yml**

In `.github/workflows/build-check.yml`, change:
```yaml
        default: "https://estatefe.fly.dev/"
```
to:
```yaml
        default: "https://estatedao.prakash.yral.com/"
```

- [ ] **Step 2: Create staging deploy workflow**

Create `.github/workflows/deploy-staging-bare-metal.yml`:

```yaml
name: Deploy to bare-metal staging (estatedao.prakash.yral.com)

on:
  workflow_dispatch:
    inputs:
      branch:
        description: 'Branch to deploy from'
        required: true
        default: 'main'
        type: string

concurrency:
  group: staging-bare-metal-deployment
  cancel-in-progress: false

permissions:
  contents: read
  packages: write

jobs:
  build:
    uses: ./.github/workflows/build-check.yml
    with:
      publish-artifact: true
      app-url: "https://estatedao.prakash.yral.com/"
    secrets: inherit

  deploy:
    name: Deploy to staging servers
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.event.inputs.branch }}

      - name: Download build
        uses: actions/download-artifact@v4
        with:
          name: build-debian

      - run: chmod +x target/release/estate-fe

      - name: Remove .dockerignore
        run: rm -f .dockerignore

      - name: Log in to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Set up QEMU
        uses: docker/setup-qemu-action@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build and push Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ./Dockerfile
          push: true
          tags: |
            ghcr.io/${{ github.repository_owner }}/estate-dao-platform-leptos-ssr:staging-${{ github.sha }}
            ghcr.io/${{ github.repository_owner }}/estate-dao-platform-leptos-ssr:staging-latest

      - name: Write .env.production file
        run: |
          cat << 'EOF' > .env.production
          PROVAB_HEADERS=${{ secrets.PROVAB_STAGING_ENVIRONMENT_REQUEST_HEADER_CONTENTS }}
          NOW_PAYMENTS_USDC_ETHEREUM_API_KEY=${{ secrets.NOW_PAYMENTS_USDC_ETHEREUM_API_KEY }}
          ESTATE_DAO_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY=${{ secrets.ESTATE_DAO_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY }}
          BASIC_AUTH_PASSWORD_FOR_LEPTOS_ROUTE=${{ secrets.BASIC_AUTH_PASSWORD_FOR_LEPTOS_ROUTE }}
          BASIC_AUTH_USERNAME_FOR_LEPTOS_ROUTE=${{ secrets.BASIC_AUTH_USERNAME_FOR_LEPTOS_ROUTE }}
          COOKIE_KEY=${{ secrets.COOKIE_KEY }}
          EMAIL_ACCESS_TOKEN=${{ secrets.EMAIL_ACCESS_TOKEN }}
          EMAIL_CLIENT_ID=${{ secrets.EMAIL_CLIENT_ID }}
          EMAIL_CLIENT_SECRET=${{ secrets.EMAIL_CLIENT_SECRET }}
          EMAIL_REFRESH_TOKEN=${{ secrets.EMAIL_REFRESH_TOKEN }}
          EMAIL_TOKEN_EXPIRY=${{ secrets.EMAIL_TOKEN_EXPIRY }}
          LITEAPI_KEY=${{ secrets.LITEAPI_KEY_STAGING_TEST }}
          LITEAPI_PREBOOK_BASE_URL=${{ secrets.LITEAPI_PREBOOK_BASE_URL }}
          NOWPAYMENTS_IPN_SECRET=${{ secrets.NOWPAYMENTS_IPN_SECRET }}
          STRIPE_SECRET_KEY=${{ secrets.STRIPE_SECRET_KEY_STAGING_TEST }}
          YRAL_AUTH_CLIENT_ID=${{ secrets.YRAL_AUTH_CLIENT_ID }}
          YRAL_AUTH_CLIENT_SECRET=${{ secrets.YRAL_AUTH_CLIENT_SECRET }}
          YRAL_AUTH_REDIRECT_URL=${{ secrets.YRAL_AUTH_REDIRECT_URL_STAGING }}
          GOOGLE_CLIENT_ID=${{ secrets.GOOGLE_CLIENT_ID }}
          GOOGLE_CLIENT_SECRET=${{ secrets.GOOGLE_CLIENT_SECRET }}
          GA4_API_SECRET=${{ secrets.GA4_API_SECRET }}
          APP_URL=https://estatedao.prakash.yral.com/
          LEPTOS_ENV=production
          LEPTOS_SITE_ADDR=0.0.0.0:3000
          LEPTOS_SITE_ROOT=site
          LEPTOS_HASH_FILES=true
          EOF
          chmod 600 .env.production

      - name: Deploy to all 3 bare-metal servers
        env:
          SERVER_1_IP: ${{ secrets.SERVER_1_IP }}
          SERVER_2_IP: ${{ secrets.SERVER_2_IP }}
          SERVER_3_IP: ${{ secrets.SERVER_3_IP }}
          SSH_KEY: ${{ secrets.DEPLOY_SSH_PRIVATE_KEY }}
          GH_REPO_NAME: ${{ github.repository_owner }}/estate-dao-platform-leptos-ssr
          IMAGE_TAG: staging-${{ github.sha }}
        run: |
          mkdir -p ~/.ssh
          echo "$SSH_KEY" > ~/.ssh/deploy_key
          chmod 600 ~/.ssh/deploy_key

          for IP in "$SERVER_1_IP" "$SERVER_2_IP" "$SERVER_3_IP"; do
            if [ -z "$IP" ]; then continue; fi
            echo "--> Deploying to $IP"
            ssh -i ~/.ssh/deploy_key -o StrictHostKeyChecking=no deploy@$IP \
              "mkdir -p /home/deploy/estate-dao-platform-leptos-ssr"
            scp -i ~/.ssh/deploy_key -o StrictHostKeyChecking=no \
              docker-compose.yml .env.production \
              deploy@$IP:/home/deploy/estate-dao-platform-leptos-ssr/
            ssh -i ~/.ssh/deploy_key -o StrictHostKeyChecking=no deploy@$IP \
              "cd /home/deploy/estate-dao-platform-leptos-ssr && \
               docker login ghcr.io -u ${{ github.actor }} -p ${{ secrets.GITHUB_TOKEN }} && \
               export GH_REPO_NAME=$GH_REPO_NAME && \
               export IMAGE_TAG=$IMAGE_TAG && \
               docker compose pull && \
               docker compose up -d --remove-orphans"
          done
```

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/deploy-staging-bare-metal.yml .github/workflows/build-check.yml
git commit -m "ci: add bare-metal staging deploy workflow for estatedao.prakash.yral.com"
```

---

## Phase 4: Fix prod deploy workflow

**Goal:** Update the existing prod deploy workflow to target all 3 servers, remove Fly.io remnants, and set the correct `APP_URL` for the intermediate prod subdomain.

**Note on APP_URL for prod phase:** The prod build currently bakes `https://nofeebooking.com/` at build time (in `build-check-prod.yml`). If deploying to `nofeebooking.prakash.yral.com` first, OAuth callbacks registered for `nofeebooking.com` won't work. **Decision required:** Either (a) update `build-check-prod.yml` to use `https://nofeebooking.prakash.yral.com/` for this migration phase, or (b) add `nofeebooking.prakash.yral.com` to all OAuth provider allow-lists. Option (a) is cleaner.

### Task 4: Update prod deploy workflow

**Files:**
- Modify: `.github/workflows/deploy-to-production-on-tag-push.yaml`
- Modify: `.github/workflows/build-check-prod.yml`

- [ ] **Step 1: Update build-check-prod.yml APP_URL**

`build-check-prod.yml` has no `app-url` input — `APP_URL` is a hardcoded value in the `env:` block at line 83. Edit it directly:

In `.github/workflows/build-check-prod.yml` line 83, change:
```yaml
          APP_URL: https://nofeebooking.com/
```
to:
```yaml
          APP_URL: https://nofeebooking.prakash.yral.com/
```

Do NOT attempt to pass it as a `with:` input from the calling workflow — that input does not exist in `build-check-prod.yml`.

- [ ] **Step 2: Update deploy-to-production-on-tag-push.yaml**

Replace the `Write .env.production` and `Deploy to Bare Metal` steps:

In the `.env.production` block, change:
```
APP_URL=https://estate.prakash.yral.com/
GOOGLE_REDIRECT_URL=https://estate.prakash.yral.com/auth/google/callback
```
to:
```
APP_URL=https://nofeebooking.prakash.yral.com/
```

Remove `GOOGLE_REDIRECT_URL` entirely — the app constructs the Google callback URL dynamically from `APP_URL` at runtime (`format!("{}/auth/google/callback", app_url)`). Setting this variable has no effect.

Also remove the `FLY_API_TOKEN` line from the `.env.production` block (not needed at runtime).

Update the deploy step to add `SERVER_3_IP` and use all 3 servers:
```yaml
      - name: Deploy to Bare Metal
        env:
          SERVER_1_IP: ${{ secrets.SERVER_1_IP }}
          SERVER_2_IP: ${{ secrets.SERVER_2_IP }}
          SERVER_3_IP: ${{ secrets.SERVER_3_IP }}
          SSH_KEY: ${{ secrets.DEPLOY_SSH_PRIVATE_KEY }}
          GH_REPO_NAME: ${{ github.repository_owner }}/estate-dao-platform-leptos-ssr
          IMAGE_TAG: ${{ github.sha }}
        run: |
          mkdir -p ~/.ssh
          echo "$SSH_KEY" > ~/.ssh/deploy_key
          chmod 600 ~/.ssh/deploy_key

          for IP in "$SERVER_1_IP" "$SERVER_2_IP" "$SERVER_3_IP"; do
            if [ -z "$IP" ]; then continue; fi
            echo "--> Deploying to $IP"
            ssh -i ~/.ssh/deploy_key -o StrictHostKeyChecking=no deploy@$IP \
              "mkdir -p /home/deploy/estate-dao-platform-leptos-ssr"
            scp -i ~/.ssh/deploy_key -o StrictHostKeyChecking=no \
              docker-compose.yml .env.production \
              deploy@$IP:/home/deploy/estate-dao-platform-leptos-ssr/
            ssh -i ~/.ssh/deploy_key -o StrictHostKeyChecking=no deploy@$IP \
              "cd /home/deploy/estate-dao-platform-leptos-ssr && \
               docker login ghcr.io -u ${{ github.actor }} -p ${{ secrets.GITHUB_TOKEN }} && \
               export GH_REPO_NAME=$GH_REPO_NAME && \
               export IMAGE_TAG=$IMAGE_TAG && \
               docker compose pull && \
               docker compose up -d --remove-orphans"
          done
```

- [ ] **Step 3: Change `cancel-in-progress` to `false` in prod workflow**

In `.github/workflows/deploy-to-production-on-tag-push.yaml`, change:
```yaml
  cancel-in-progress: true
```
to:
```yaml
  cancel-in-progress: false
```

This prevents a second tag push from cancelling an in-progress deploy mid-loop, which would leave some servers on the old image and some on the new.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/deploy-to-production-on-tag-push.yaml .github/workflows/build-check-prod.yml
git commit -m "ci: add server_3, fix APP_URL, remove fly remnants from prod deploy workflow"
```

---

## Phase 5: Add GitHub Secrets to estate repo

**Goal:** Add the 4 secrets that the new workflows need. These use the same values as the infra repo.

### Task 5: Configure repository secrets

- [ ] **Step 1: Go to GitHub repo Settings → Secrets and variables → Actions**

URL: `https://github.com/<org>/estate-dao-platform-leptos-ssr/settings/secrets/actions`

- [ ] **Step 2: Add the following secrets** (same values as in `yral-onboarding-hello-world-counter-prakash`)

| Secret name | Value |
|---|---|
| `SERVER_1_IP` | `94.130.13.115` |
| `SERVER_2_IP` | `88.99.151.102` |
| `SERVER_3_IP` | `138.201.129.173` |
| `DEPLOY_SSH_PRIVATE_KEY` | same private key used by the infra repo |

- [ ] **Step 3: Verify all existing app secrets are present**

The following must already exist (carried over from Fly.io era). Confirm each is set:
- `PROVAB_STAGING_ENVIRONMENT_REQUEST_HEADER_CONTENTS`
- `PROVAB_PRODUCTION_ENVIRONMENT_REQUEST_HEADER_CONTENTS`
- `NOW_PAYMENTS_USDC_ETHEREUM_API_KEY`
- `ESTATE_DAO_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY`
- `BASIC_AUTH_PASSWORD_FOR_LEPTOS_ROUTE`
- `BASIC_AUTH_USERNAME_FOR_LEPTOS_ROUTE`
- `COOKIE_KEY`
- `EMAIL_ACCESS_TOKEN`, `EMAIL_CLIENT_ID`, `EMAIL_CLIENT_SECRET`, `EMAIL_REFRESH_TOKEN`, `EMAIL_TOKEN_EXPIRY`
- `LITEAPI_KEY`, `LITEAPI_KEY_STAGING_TEST`, `LITEAPI_PREBOOK_BASE_URL`
- `NOWPAYMENTS_IPN_SECRET`
- `STRIPE_SECRET_KEY`, `STRIPE_SECRET_KEY_STAGING_TEST`
- `YRAL_AUTH_CLIENT_ID`, `YRAL_AUTH_CLIENT_SECRET`, `YRAL_AUTH_REDIRECT_URL` (prod), `YRAL_AUTH_REDIRECT_URL_STAGING`
- `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`
- `GA4_API_SECRET`

---

## Phase 6: Add Caddy site block (infra repo)

**Goal:** Tell the shared Caddy instance to route `estatedao.prakash.yral.com` to `localhost:3002`.

**This is done in the OTHER repo:** `yral-onboarding-hello-world-counter-prakash`

### Task 6: Update infra Caddyfile.template

**Files (in infra repo):**
- Modify: `infra/Caddyfile.template`

- [ ] **Step 1: Open infra repo**

```bash
cd ~/path/to/yral-onboarding-hello-world-counter-prakash
```

- [ ] **Step 2: Add site block to `infra/Caddyfile.template`**

Add after the `hello-world.prakash.yral.com` block:

```
estatedao.prakash.yral.com {
__TLS_DIRECTIVE__
    encode zstd gzip
    reverse_proxy localhost:3002
}
```

- [ ] **Step 3: Commit and push**

```bash
git add infra/Caddyfile.template
git commit -m "feat: add estatedao.prakash.yral.com caddy site block (port 3002)"
git push
```

- [ ] **Step 4: Verify `deploy-infra.yml` triggers and completes**

`deploy-infra.yml` only auto-triggers on pushes to **`main`** branch with changes in `infra/**`. If you're working on a feature branch, merge the PR to `main` first — the workflow will not fire from a branch push. Alternatively trigger it manually via `workflow_dispatch`.

Go to GitHub Actions → deploy-infra.yml → confirm green. Caddy will reload with the new config. At this point `estatedao.prakash.yral.com` will return a 502 (no app yet) — that's expected.

---

## Phase 7: DNS Setup

**Goal:** Point `estatedao.prakash.yral.com` at all 3 servers.

### Task 7: Configure DNS

- [ ] **Step 1: Log into Cloudflare (or your DNS provider)**

- [ ] **Step 2: Add A records for `estatedao.prakash.yral.com`**

Add 3 A records (one per server), all with the same hostname. Cloudflare will round-robin DNS:

```
estatedao.prakash.yral.com  A  94.130.13.115   proxied: yes (or DNS-only)
estatedao.prakash.yral.com  A  88.99.151.102   proxied: yes
estatedao.prakash.yral.com  A  138.201.129.173  proxied: yes
```

If using **Cloudflare proxy (orange cloud)**: Set Cloudflare SSL/TLS mode to **Full (strict)**. TLS terminates at Cloudflare edge and re-originates to the server using the cert from `CADDY_TLS_CERT_PEM_B64`/`KEY` (the infra deploy workflow). Without "Full (strict)", Cloudflare will connect to the origin over plain HTTP and the Caddy TLS directive will be ignored.

If using **DNS-only (grey cloud)**: TLS is handled entirely by Caddy. Simpler for initial rollout — recommended if you're unsure which cert is installed on the servers.

- [ ] **Step 3: For prod (later) — repeat for `nofeebooking.prakash.yral.com`**

Same 3 A records but with `nofeebooking.prakash.yral.com`.

---

## Phase 8: OAuth Provider Updates

**Goal:** Register the new callback URLs with Google and YRAL auth so logins work on the new domain.

### Task 8: Update OAuth allowed redirect URIs

- [ ] **Step 1: Google Cloud Console**

Go to [console.cloud.google.com](https://console.cloud.google.com) → APIs & Services → Credentials → your OAuth 2.0 Client ID.

Add to **Authorised redirect URIs**:
```
https://estatedao.prakash.yral.com/auth/google/callback
```

- [ ] **Step 2: YRAL auth provider**

Update the YRAL auth application's allowed redirect URLs to include:
```
https://estatedao.prakash.yral.com/auth/callback
```

Also update the `YRAL_AUTH_REDIRECT_URL_STAGING` GitHub secret if it was previously set to the Fly.io URL.

- [ ] **Step 3: For prod phase — repeat for `nofeebooking.prakash.yral.com`**

Add `https://nofeebooking.prakash.yral.com/auth/google/callback` and `/auth/callback` to both providers.

---

## Phase 9: First Staging Deploy

**Goal:** Trigger the first deploy and verify `estatedao.prakash.yral.com` serves traffic.

### Task 9: Deploy and verify

- [ ] **Step 1: Merge all changes to main (or staging branch)**

Ensure Phases 1–5 are committed and merged.

- [ ] **Step 2: Trigger staging deploy**

Go to GitHub Actions → `Deploy to bare-metal staging` → Run workflow → select branch `main`.

- [ ] **Step 3: Monitor Actions log**

Watch for:
1. `Build the Leptos project` — completes (~10–20 min, long Rust compile)
2. `Build and push Docker image` — image pushed to GHCR
3. `Deploy to all 3 bare-metal servers` — each server: pull + up

- [ ] **Step 4: Verify each server responds**

```bash
curl -sI --resolve estatedao.prakash.yral.com:443:94.130.13.115 https://estatedao.prakash.yral.com/ | grep -E "HTTP|x-served-by"
curl -sI --resolve estatedao.prakash.yral.com:443:88.99.151.102 https://estatedao.prakash.yral.com/ | grep -E "HTTP|x-served-by"
curl -sI --resolve estatedao.prakash.yral.com:443:138.201.129.173 https://estatedao.prakash.yral.com/ | grep -E "HTTP|x-served-by"
```

Expected: `HTTP/2 200` from each server.

- [ ] **Step 5: Verify container health on each server**

SSH into any server and run:
```bash
docker ps --filter "name=estate_app" --format "table {{.Names}}\t{{.Status}}"
```

Expected: `estate_app   Up X minutes (healthy)`

- [ ] **Step 6: Smoke test core flows**

1. Load homepage — search UI renders
2. Search for a city — results appear (LiteAPI staging)
3. Google login — OAuth redirect works, returns to correct callback URL
4. Booking flow start (don't need to complete payment in staging)

---

## Phase 10: Prod Promotion (later)

> Run this phase only after staging is verified stable.

**Goal:** Deploy to `nofeebooking.prakash.yral.com` and eventually cut over from Fly.io.

### Task 10: Prod deploy

- [ ] **Step 1: Add `nofeebooking.prakash.yral.com` to Caddy**

In infra repo, add to `Caddyfile.template`:
```
nofeebooking.prakash.yral.com {
__TLS_DIRECTIVE__
    encode zstd gzip
    reverse_proxy localhost:3002
}
```

Commit, push, let `deploy-infra.yml` run.

- [ ] **Step 2: Add DNS records for `nofeebooking.prakash.yral.com`**

Same 3 A records as step 7, but with `nofeebooking.prakash.yral.com`.

- [ ] **Step 3: Update OAuth providers**

Add `https://nofeebooking.prakash.yral.com/auth/google/callback` to Google Console.
Add `https://nofeebooking.prakash.yral.com/auth/callback` to YRAL auth provider.

- [ ] **Step 4: Push a `v*` tag to trigger prod deploy**

```bash
git tag v1.0.0
git push origin v1.0.0
```

This triggers `deploy-to-production-on-tag-push.yaml`.

- [ ] **Step 5: Verify prod**

```bash
curl -sI --resolve nofeebooking.prakash.yral.com:443:94.130.13.115 https://nofeebooking.prakash.yral.com/ | grep "HTTP"
```

- [ ] **Step 6: Decommission Fly.io (when ready)**

Once `nofeebooking.prakash.yral.com` is stable, scale down Fly.io apps to 0 machines. Keep the apps for 1–2 weeks before deleting (easy rollback if issues arise).

```bash
flyctl scale count 0 --app nofeebooking
flyctl scale count 0 --app estatefe
```

---

## Troubleshooting Reference

### Container won't start
```bash
# On a server
docker logs estate_app --tail 50
```

Common issues:
- Missing env var → app panics at startup with the var name
- `COOKIE_KEY` must be base64-encoded 64-byte value
- `ESTATE_DAO_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY` must be a valid PEM secp256k1 key

### 502 Bad Gateway
1. Check container is healthy: `docker ps -a | grep estate`
2. Check Caddy is routing to port 3002: `docker logs caddy --tail 20`
3. Test directly: `curl http://localhost:3002/` from the server

### WASM loads but OAuth redirect fails
- The baked-in `APP_URL` in the WASM doesn't match the serving domain
- Verify: `build-check.yml` was called with correct `app-url` input
- Verify: runtime `APP_URL` in `.env.production` matches

### Image pull fails on server
The GHCR login is per-deploy. If pulling manually:
```bash
docker login ghcr.io -u <github_username> -p <personal_access_token>
docker pull ghcr.io/<owner>/estate-dao-platform-leptos-ssr:staging-latest
```
