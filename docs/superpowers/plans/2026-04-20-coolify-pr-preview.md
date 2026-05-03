# Coolify PR Preview Deployments — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a new GitHub Actions workflow that deploys a Coolify-hosted preview environment for every PR opened against `main`, and tears it down on close.

**Architecture:** A single new workflow file `.github/workflows/deploy-preview-coolify.yml` with three jobs: `build_check` (reuses existing `build-check.yml`), `preview` (builds Docker image, creates/updates Coolify app, pushes env vars, deploys), and `destroy-preview` (deletes Coolify app on PR close). Secrets come from GitHub secrets. The existing Fly.io `deploy-preview.yml` is left completely untouched.

**Tech Stack:** GitHub Actions, Coolify REST API (`coolify.yral.com`), GHCR (GitHub Container Registry), Docker Buildx, existing `build-check.yml` reusable workflow.

---

## Pre-requisites (verify before starting)

- [ ] GitHub secrets set on estate-dao-platform-leptos-ssr repo:
  - `COOLIFY_TOKEN`
  - `COOLIFY_PROJECT_UUID` = `ns8scggs0cs8s0ooc88g8o8o`
  - `COOLIFY_SERVER_UUID` = `j4o88k0cg00kow044co4s040`
  - `COOLIFY_ENV_UUID` = `vokswss04k80ggosg0s8os80`
- [ ] Confirm whether GHCR image for this repo is public or private:
  - Go to `https://github.com/orgs/prakash-bhatt-yral/packages` (or the org that owns the repo)
  - If private: Coolify's server needs pre-configured GHCR credentials (ask Naitik to configure `ghcr.io` registry credentials in Coolify settings pointing to a PAT with `read:packages`)
  - If public: no extra step needed

---

## File Structure

| Action | File |
|--------|------|
| Create | `.github/workflows/deploy-preview-coolify.yml` |

No other files are modified.

---

## Reference Files

- Spec: `docs/superpowers/specs/2026-04-20-coolify-pr-preview-design.md`
- yral-metadata reference (provided in conversation — the working Coolify workflow to port from)
- `estate-dao-platform-leptos-ssr/.github/workflows/deploy-staging-bare-metal.yml` — source of truth for build steps and secrets list
- `estate-dao-platform-leptos-ssr/.github/workflows/deploy-preview.yml` — source of truth for Fly.io secrets list (same secrets, different destination)

---

### Task 1: Workflow skeleton — trigger, permissions, job stubs

**Files:**
- Create: `.github/workflows/deploy-preview-coolify.yml`

- [ ] **Step 1: Create the file with trigger, permissions, and empty job stubs**

```yaml
name: Deploy Preview (Coolify)

on:
  pull_request:
    branches: [main]
    types: [opened, reopened, synchronize, closed]

permissions:
  contents: read
  packages: write
  deployments: write
  pull-requests: write

jobs:
  build_check:
    if: ${{ github.event.action != 'closed' }}
    uses: ./.github/workflows/build-check.yml
    with:
      publish-artifact: true
      app-url: https://pr-${{ github.event.number }}-estate.preview.yral.com/
    secrets: inherit

  preview:
    if: ${{ github.event.pull_request.state == 'open' }}
    needs: build_check
    runs-on: ubuntu-latest
    concurrency:
      group: pr-${{ github.event.number }}-estate-coolify
      cancel-in-progress: true
    environment:
      name: pr-${{ github.event.number }}
      url: ${{ steps.meta.outputs.preview_url }}
    steps:
      - run: echo "TODO"

  destroy-preview:
    if: ${{ github.event.action == 'closed' }}
    runs-on: ubuntu-latest
    steps:
      - run: echo "TODO"
```

- [ ] **Step 2: Validate YAML syntax**

```bash
cd /Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/deploy-preview-coolify.yml'))" && echo "YAML OK"
```

Expected: `YAML OK`

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/deploy-preview-coolify.yml
git commit -m "ci: add Coolify PR preview workflow skeleton"
```

---

### Task 2: `preview` job — metadata, checkout, build artifact

**Files:**
- Modify: `.github/workflows/deploy-preview-coolify.yml` (preview job steps)

- [ ] **Step 1: Replace the `preview` job `steps:` with metadata, checkout, artifact download, and Docker setup**

Replace the `steps:` section of the `preview` job with:

```yaml
    steps:
      - name: Compute Preview Metadata
        id: meta
        run: |
          PR_NUMBER=${{ github.event.number }}
          APP_NAME="pr-${PR_NUMBER}-estate"
          IMAGE="ghcr.io/$(echo '${{ github.repository_owner }}' | tr '[:upper:]' '[:lower:]')/estate-dao-platform-leptos-ssr"
          IMAGE_TAG="pr-${PR_NUMBER}-estate"
          PREVIEW_DOMAIN="pr-${PR_NUMBER}-estate.preview.yral.com"
          PREVIEW_URL="https://${PREVIEW_DOMAIN}"

          echo "pr_number=${PR_NUMBER}"           >> $GITHUB_OUTPUT
          echo "app_name=${APP_NAME}"             >> $GITHUB_OUTPUT
          echo "image=${IMAGE}"                   >> $GITHUB_OUTPUT
          echo "image_tag=${IMAGE_TAG}"           >> $GITHUB_OUTPUT
          echo "preview_domain=${PREVIEW_DOMAIN}" >> $GITHUB_OUTPUT
          echo "preview_url=${PREVIEW_URL}"       >> $GITHUB_OUTPUT

      - uses: actions/checkout@v4

      - name: Download build artifact
        uses: actions/download-artifact@v4
        with:
          name: build-debian

      - name: Fix binary permissions
        run: chmod +x target/release/estate-fe

      - name: Remove .dockerignore
        run: rm -f .dockerignore

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to GHCR
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ./Dockerfile
          push: true
          tags: ${{ steps.meta.outputs.image }}:${{ steps.meta.outputs.image_tag }}
          cache-from: type=registry,ref=${{ steps.meta.outputs.image }}:cache-${{ steps.meta.outputs.image_tag }}
          cache-to: type=registry,ref=${{ steps.meta.outputs.image }}:cache-${{ steps.meta.outputs.image_tag }},mode=max
```

- [ ] **Step 2: Validate YAML syntax**

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/deploy-preview-coolify.yml'))" && echo "YAML OK"
```

Expected: `YAML OK`

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/deploy-preview-coolify.yml
git commit -m "ci: add preview job build and image push steps"
```

---

### Task 3: `preview` job — Coolify app create/reuse

**Files:**
- Modify: `.github/workflows/deploy-preview-coolify.yml`

- [ ] **Step 1: Add Coolify app creation step after the Docker push step**

```yaml
      - name: Create or Get Coolify Application
        id: coolify_app
        env:
          COOLIFY_TOKEN: ${{ secrets.COOLIFY_TOKEN }}
          COOLIFY_PROJECT_UUID: ${{ secrets.COOLIFY_PROJECT_UUID }}
          COOLIFY_SERVER_UUID: ${{ secrets.COOLIFY_SERVER_UUID }}
          COOLIFY_ENV_UUID: ${{ secrets.COOLIFY_ENV_UUID }}
        run: |
          APP_NAME="${{ steps.meta.outputs.app_name }}"
          IMAGE="${{ steps.meta.outputs.image }}"
          IMAGE_TAG="${{ steps.meta.outputs.image_tag }}"
          PREVIEW_DOMAIN="${{ steps.meta.outputs.preview_domain }}"

          # Reuse existing app if this PR already has one (e.g. new commit on same PR)
          EXISTING=$(curl -s \
            "https://coolify.yral.com/api/v1/applications" \
            -H "Authorization: Bearer ${COOLIFY_TOKEN}" \
            | jq -r --arg name "$APP_NAME" '.[] | select(.name == $name) | .uuid // empty')

          if [ -n "$EXISTING" ]; then
            echo "Reusing existing Coolify app: $EXISTING"
            echo "app_uuid=${EXISTING}" >> $GITHUB_OUTPUT
          else
            echo "Creating new Coolify app: $APP_NAME"

            RESPONSE=$(curl -s -X POST \
              "https://coolify.yral.com/api/v1/applications/dockerimage" \
              -H "Authorization: Bearer ${COOLIFY_TOKEN}" \
              -H "Content-Type: application/json" \
              -d "{
                \"name\": \"${APP_NAME}\",
                \"project_uuid\": \"${COOLIFY_PROJECT_UUID}\",
                \"server_uuid\": \"${COOLIFY_SERVER_UUID}\",
                \"environment_name\": \"preview\",
                \"environment_uuid\": \"${COOLIFY_ENV_UUID}\",
                \"docker_registry_image_name\": \"${IMAGE}\",
                \"docker_registry_image_tag\": \"${IMAGE_TAG}\",
                \"domains\": \"https://${PREVIEW_DOMAIN}\",
                \"ports_exposes\": \"3000\",
                \"is_static\": false
              }")

            echo "Create response: $RESPONSE"

            APP_UUID=$(echo "$RESPONSE" | jq -r '.uuid // empty')
            if [ -z "$APP_UUID" ]; then
              echo "Failed to create Coolify application"
              echo "$RESPONSE"
              exit 1
            fi

            echo "app_uuid=${APP_UUID}" >> $GITHUB_OUTPUT
          fi
```

- [ ] **Step 2: Validate YAML**

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/deploy-preview-coolify.yml'))" && echo "YAML OK"
```

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/deploy-preview-coolify.yml
git commit -m "ci: add Coolify app create/reuse step"
```

---

### Task 4: `preview` job — push environment variables

**Files:**
- Modify: `.github/workflows/deploy-preview-coolify.yml`

- [ ] **Step 1: Add the env vars push step after the Coolify app step**

```yaml
      - name: Push Environment Variables
        env:
          COOLIFY_TOKEN: ${{ secrets.COOLIFY_TOKEN }}
          PROVAB_HEADERS: ${{ secrets.PROVAB_PRODUCTION_ENVIRONMENT_REQUEST_HEADER_CONTENTS }}
          NOW_PAYMENTS_USDC_ETHEREUM_API_KEY: ${{ secrets.NOW_PAYMENTS_USDC_ETHEREUM_API_KEY }}
          ESTATE_DAO_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY: ${{ secrets.ESTATE_DAO_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY }}
          BASIC_AUTH_PASSWORD_FOR_LEPTOS_ROUTE: ${{ secrets.BASIC_AUTH_PASSWORD_FOR_LEPTOS_ROUTE }}
          BASIC_AUTH_USERNAME_FOR_LEPTOS_ROUTE: ${{ secrets.BASIC_AUTH_USERNAME_FOR_LEPTOS_ROUTE }}
          COOKIE_KEY: ${{ secrets.COOKIE_KEY }}
          EMAIL_ACCESS_TOKEN: ${{ secrets.EMAIL_ACCESS_TOKEN }}
          EMAIL_CLIENT_ID: ${{ secrets.EMAIL_CLIENT_ID }}
          EMAIL_CLIENT_SECRET: ${{ secrets.EMAIL_CLIENT_SECRET }}
          EMAIL_REFRESH_TOKEN: ${{ secrets.EMAIL_REFRESH_TOKEN }}
          EMAIL_TOKEN_EXPIRY: ${{ secrets.EMAIL_TOKEN_EXPIRY }}
          LITEAPI_KEY: ${{ secrets.LITEAPI_KEY_STAGING_TEST }}
          LITEAPI_PREBOOK_BASE_URL: ${{ secrets.LITEAPI_PREBOOK_BASE_URL }}
          NOWPAYMENTS_IPN_SECRET: ${{ secrets.NOWPAYMENTS_IPN_SECRET }}
          STRIPE_SECRET_KEY: ${{ secrets.STRIPE_SECRET_KEY_STAGING_TEST }}
          YRAL_AUTH_CLIENT_ID: ${{ secrets.YRAL_AUTH_CLIENT_ID }}
          YRAL_AUTH_CLIENT_SECRET: ${{ secrets.YRAL_AUTH_CLIENT_SECRET }}
          GOOGLE_CLIENT_ID: ${{ secrets.GOOGLE_CLIENT_ID }}
          GOOGLE_CLIENT_SECRET: ${{ secrets.GOOGLE_CLIENT_SECRET }}
          GA4_API_SECRET: ${{ secrets.GA4_API_SECRET }}
          SENTRY_DSN: ${{ secrets.SENTRY_DSN }}
        run: |
          APP_UUID="${{ steps.coolify_app.outputs.app_uuid }}"
          PREVIEW_URL="${{ steps.meta.outputs.preview_url }}"

          ENV_JSON=$(jq -n \
            --arg provab_headers "$PROVAB_HEADERS" \
            --arg now_payments_usdc_ethereum_api_key "$NOW_PAYMENTS_USDC_ETHEREUM_API_KEY" \
            --arg estate_dao_sns_proposal_submission_identity_private_key "$ESTATE_DAO_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY" \
            --arg basic_auth_password_for_leptos_route "$BASIC_AUTH_PASSWORD_FOR_LEPTOS_ROUTE" \
            --arg basic_auth_username_for_leptos_route "$BASIC_AUTH_USERNAME_FOR_LEPTOS_ROUTE" \
            --arg cookie_key "$COOKIE_KEY" \
            --arg email_access_token "$EMAIL_ACCESS_TOKEN" \
            --arg email_client_id "$EMAIL_CLIENT_ID" \
            --arg email_client_secret "$EMAIL_CLIENT_SECRET" \
            --arg email_refresh_token "$EMAIL_REFRESH_TOKEN" \
            --arg email_token_expiry "$EMAIL_TOKEN_EXPIRY" \
            --arg liteapi_key "$LITEAPI_KEY" \
            --arg liteapi_prebook_base_url "$LITEAPI_PREBOOK_BASE_URL" \
            --arg nowpayments_ipn_secret "$NOWPAYMENTS_IPN_SECRET" \
            --arg stripe_secret_key "$STRIPE_SECRET_KEY" \
            --arg yral_auth_client_id "$YRAL_AUTH_CLIENT_ID" \
            --arg yral_auth_client_secret "$YRAL_AUTH_CLIENT_SECRET" \
            --arg google_client_id "$GOOGLE_CLIENT_ID" \
            --arg google_client_secret "$GOOGLE_CLIENT_SECRET" \
            --arg ga4_api_secret "$GA4_API_SECRET" \
            --arg sentry_dsn "$SENTRY_DSN" \
            --arg app_url "${PREVIEW_URL}" \
            --arg yral_auth_redirect_url "${PREVIEW_URL%/}/auth/callback" \
            --arg google_redirect_url "${PREVIEW_URL%/}/auth/google/callback" \
            '{
              "data": [
                {"key": "PROVAB_HEADERS", "value": $provab_headers, "is_literal": true},
                {"key": "NOW_PAYMENTS_USDC_ETHEREUM_API_KEY", "value": $now_payments_usdc_ethereum_api_key, "is_literal": true},
                {"key": "ESTATE_DAO_SNS_PROPOSAL_SUBMISSION_IDENTITY_PRIVATE_KEY", "value": $estate_dao_sns_proposal_submission_identity_private_key, "is_multiline": true},
                {"key": "BASIC_AUTH_PASSWORD_FOR_LEPTOS_ROUTE", "value": $basic_auth_password_for_leptos_route, "is_literal": true},
                {"key": "BASIC_AUTH_USERNAME_FOR_LEPTOS_ROUTE", "value": $basic_auth_username_for_leptos_route, "is_literal": true},
                {"key": "COOKIE_KEY", "value": $cookie_key, "is_literal": true},
                {"key": "EMAIL_ACCESS_TOKEN", "value": $email_access_token, "is_literal": true},
                {"key": "EMAIL_CLIENT_ID", "value": $email_client_id, "is_literal": true},
                {"key": "EMAIL_CLIENT_SECRET", "value": $email_client_secret, "is_literal": true},
                {"key": "EMAIL_REFRESH_TOKEN", "value": $email_refresh_token, "is_literal": true},
                {"key": "EMAIL_TOKEN_EXPIRY", "value": $email_token_expiry, "is_literal": true},
                {"key": "LITEAPI_KEY", "value": $liteapi_key, "is_literal": true},
                {"key": "LITEAPI_PREBOOK_BASE_URL", "value": $liteapi_prebook_base_url, "is_literal": true},
                {"key": "NOWPAYMENTS_IPN_SECRET", "value": $nowpayments_ipn_secret, "is_literal": true},
                {"key": "STRIPE_SECRET_KEY", "value": $stripe_secret_key, "is_literal": true},
                {"key": "YRAL_AUTH_CLIENT_ID", "value": $yral_auth_client_id, "is_literal": true},
                {"key": "YRAL_AUTH_CLIENT_SECRET", "value": $yral_auth_client_secret, "is_literal": true},
                {"key": "GOOGLE_CLIENT_ID", "value": $google_client_id, "is_literal": true},
                {"key": "GOOGLE_CLIENT_SECRET", "value": $google_client_secret, "is_literal": true},
                {"key": "GA4_API_SECRET", "value": $ga4_api_secret, "is_literal": true},
                {"key": "SENTRY_DSN", "value": $sentry_dsn, "is_literal": true},
                {"key": "APP_URL", "value": $app_url, "is_literal": true},
                {"key": "YRAL_AUTH_REDIRECT_URL", "value": $yral_auth_redirect_url, "is_literal": true},
                {"key": "GOOGLE_REDIRECT_URL", "value": $google_redirect_url, "is_literal": true}
              ]
            }')

          HTTP_STATUS=$(curl -s -o /tmp/curl_response.txt -w "%{http_code}" -X PATCH \
            "https://coolify.yral.com/api/v1/applications/${APP_UUID}/envs/bulk" \
            -H "Authorization: Bearer ${COOLIFY_TOKEN}" \
            -H "Content-Type: application/json" \
            -d "${ENV_JSON}")

          if [ "$HTTP_STATUS" -lt 200 ] || [ "$HTTP_STATUS" -ge 300 ]; then
            echo "Failed to push env vars (HTTP $HTTP_STATUS)"
            cat /tmp/curl_response.txt
            exit 1
          fi

          echo "Env vars pushed successfully"
```

- [ ] **Step 2: Validate YAML**

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/deploy-preview-coolify.yml'))" && echo "YAML OK"
```

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/deploy-preview-coolify.yml
git commit -m "ci: add environment variables push to Coolify"
```

---

### Task 5: `preview` job — deploy and wait

**Files:**
- Modify: `.github/workflows/deploy-preview-coolify.yml`

- [ ] **Step 1: Add the deploy trigger and wait steps**

```yaml
      - name: Update Image Tag and Deploy
        id: deploy
        env:
          COOLIFY_TOKEN: ${{ secrets.COOLIFY_TOKEN }}
        run: |
          APP_UUID="${{ steps.coolify_app.outputs.app_uuid }}"
          IMAGE_TAG="${{ steps.meta.outputs.image_tag }}"

          # Update image tag (important on re-pushes to the same PR)
          curl -s -X PATCH \
            "https://coolify.yral.com/api/v1/applications/${APP_UUID}" \
            -H "Authorization: Bearer ${COOLIFY_TOKEN}" \
            -H "Content-Type: application/json" \
            -d "{\"docker_registry_image_tag\": \"${IMAGE_TAG}\"}"

          # Trigger deployment
          RESPONSE=$(curl -s -X POST \
            "https://coolify.yral.com/api/v1/applications/${APP_UUID}/start" \
            -H "Authorization: Bearer ${COOLIFY_TOKEN}" \
            -H "Content-Type: application/json" \
            -d "{\"uuid\": \"${APP_UUID}\", \"force\": true}")

          echo "Deploy response: $RESPONSE"

          DEPLOY_UUID=$(echo "$RESPONSE" | jq -r '.deployment_uuid // empty')
          if [ -z "$DEPLOY_UUID" ]; then
            echo "Failed to trigger deployment"
            echo "$RESPONSE"
            exit 1
          fi

          echo "deploy_uuid=${DEPLOY_UUID}" >> $GITHUB_OUTPUT

      - name: Wait for Deployment
        env:
          COOLIFY_TOKEN: ${{ secrets.COOLIFY_TOKEN }}
        run: |
          DEPLOY_UUID="${{ steps.deploy.outputs.deploy_uuid }}"
          MAX_WAIT=120
          WAITED=0

          while [ $WAITED -lt $MAX_WAIT ]; do
            STATUS=$(curl -s \
              "https://coolify.yral.com/api/v1/deployments/${DEPLOY_UUID}" \
              -H "Authorization: Bearer ${COOLIFY_TOKEN}" \
              | jq -r '.status // empty')

            echo "Status: ${STATUS} (${WAITED}s elapsed)"

            if [ "$STATUS" = "finished" ]; then
              echo "Deployment succeeded"
              exit 0
            elif [ "$STATUS" = "failed" ] || [ "$STATUS" = "error" ]; then
              echo "Deployment failed"
              exit 1
            fi

            sleep 10
            WAITED=$((WAITED + 10))
          done

          echo "Timed out waiting for deployment"
          exit 1
```

- [ ] **Step 2: Validate YAML**

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/deploy-preview-coolify.yml'))" && echo "YAML OK"
```

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/deploy-preview-coolify.yml
git commit -m "ci: add Coolify deploy trigger and wait steps"
```

---

### Task 6: `destroy-preview` job

**Files:**
- Modify: `.github/workflows/deploy-preview-coolify.yml`

- [ ] **Step 1: Replace the `destroy-preview` job stub with the real implementation**

```yaml
  destroy-preview:
    if: ${{ github.event.action == 'closed' }}
    runs-on: ubuntu-latest
    steps:
      - name: Compute app name
        id: meta
        run: |
          echo "app_name=pr-${{ github.event.number }}-estate" >> $GITHUB_OUTPUT

      - name: Find and Delete Coolify Application
        env:
          COOLIFY_TOKEN: ${{ secrets.COOLIFY_TOKEN }}
        run: |
          APP_NAME="${{ steps.meta.outputs.app_name }}"

          APP_UUID=$(curl -s \
            "https://coolify.yral.com/api/v1/applications" \
            -H "Authorization: Bearer ${COOLIFY_TOKEN}" \
            | jq -r --arg name "$APP_NAME" '.[] | select(.name == $name) | .uuid // empty')

          if [ -z "$APP_UUID" ]; then
            echo "No app found for $APP_NAME — skipping"
            exit 0
          fi

          echo "Deleting Coolify app $APP_NAME ($APP_UUID)"

          curl -s -X DELETE \
            "https://coolify.yral.com/api/v1/applications/${APP_UUID}" \
            -H "Authorization: Bearer ${COOLIFY_TOKEN}"

          echo "Deleted"
```

- [ ] **Step 2: Validate YAML**

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/deploy-preview-coolify.yml'))" && echo "YAML OK"
```

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/deploy-preview-coolify.yml
git commit -m "ci: add destroy-preview job for PR close cleanup"
```

---

### Task 7: End-to-end test

- [ ] **Step 1: Open a test PR against `main` in the estatedao repo**

Create a throwaway branch with a trivial change (e.g. add a comment to `Cargo.toml`), push it, and open a PR against `main`.

- [ ] **Step 2: Watch the workflow run in GitHub Actions**

Go to: `Actions → Deploy Preview (Coolify)` and confirm:
- `build_check` job passes
- `preview` job runs all steps without error
- `Wait for Deployment` step exits with `Deployment succeeded`

- [ ] **Step 3: Verify the preview URL**

Open `https://pr-{N}-estate.preview.yral.com/` in a browser. Expected: estatedao app loads correctly.

- [ ] **Step 4: Check GitHub environment**

On the PR page, confirm the environment `pr-{N}` appears with a link to the preview URL.

- [ ] **Step 5: Close the PR and verify cleanup**

Close (or merge) the test PR. Confirm:
- `destroy-preview` job runs
- The Coolify app `pr-{N}-estate` no longer appears in `coolify.yral.com`

- [ ] **Step 6: Commit spec doc and final workflow file together**

```bash
cd /Users/prk-jr/Desktop/work/estate/estate-dao-platform-leptos-ssr
git add docs/superpowers/specs/2026-04-20-coolify-pr-preview-design.md
git add docs/superpowers/plans/2026-04-20-coolify-pr-preview.md
git commit -m "docs: add Coolify PR preview spec and implementation plan"
git push origin main
```

---

## Troubleshooting

**`Failed to create Coolify application` in step 3:**
- Check that all 4 `COOLIFY_*` secrets are set correctly in the repo
- Verify the UUIDs are correct by checking `coolify.yral.com` directly

**`docker push` 403 error:**
- The `packages: write` permission + `GITHUB_TOKEN` should be sufficient for GHCR push from Actions
- If still failing, check if the package visibility needs to be set to public or if Coolify needs pre-configured registry credentials (ask Naitik)

**Coolify can't pull the image (`image pull failed`):**
- GHCR image is private and Coolify has no credentials
- Solution: Ask Naitik to add GHCR registry credentials in Coolify settings (`coolify.yral.com → Settings → Registries → Add ghcr.io with a PAT that has read:packages`)

**Preview URL 502/404 after deployment succeeds:**
- DNS for `*.preview.yral.com` may not be configured to point to the Coolify server
- Ask Saikat to confirm `*.preview.yral.com` has a wildcard DNS record pointing to the Coolify server
