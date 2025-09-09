# Authentik GitLab OAuth Proxy

This proxy lets apps expecting a GitLab OAuth interface (e.g. Plane) authenticate users via Authentik using GitLab-compatible endpoints.

## Features

- Stateless, lightweight Rust implementation
- Minimal Docker image (scratch)
- Multi-arch builds (AMD64, ARM64) via GHCR
- Passes through Authentik tokens/userinfo as-is

## Endpoints

- `/oauth/authorize`: Redirects to Authentik's authorize endpoint.
- `/oauth/token`: Exchanges code for Authentik tokens.
- `/api/v4/user`: Returns user info in GitLab schema (from Authentik).

## Configuration

Set the following environment variables:

```env
AUTHENTIK_URL=https://auth.example.com/application/o
AUTHENTIK_CLIENT_ID=your-client-id
AUTHENTIK_CLIENT_SECRET=your-client-secret
AUTHENTIK_REDIRECT_URI=https://your.plane.instance/auth/gitlab/callback/
```

## Authentik Setup

1. **Add an OAuth2 application**:
   - Redirect URIs: `https://your.plane.instance/auth/gitlab/callback/`
   - Scopes: `openid email profile`
   - Response type: `code`
   - Grant type: `authorization_code`
   - Client type: confidential

2. **Attributes**:
   - Ensure these are included in the userinfo response (can be mapped in Authentik):
     - `sub` (unique user id)
     - `email`
     - `name`
     - `avatar_url` (custom attribute, if desired)
     - `family_name` (optional)

3. **Customize userinfo response**:
   - In Authentik, go to your application > "User info attributes"
   - Add mappings as needed:
     ```yaml
     sub: user.id
     email: user.email
     name: user.name
     avatar_url: user.avatar_url # If available
     family_name: user.family_name # If available
     ```
   - Save and test the userinfo endpoint

## Build and Run

```sh
docker build -t authentik-gitlab-proxy .
docker run -e AUTHENTIK_URL=... -e AUTHENTIK_CLIENT_ID=... -e AUTHENTIK_CLIENT_SECRET=... -e AUTHENTIK_REDIRECT_URI=... -p 8080:8080 authentik-gitlab-proxy
```

Or pull from GHCR (after first pipeline run):

```sh
docker pull ghcr.io/<your-gh-username>/authentik-gitlab-proxy:latest
```

## Expose via Traefik

Add a Traefik router/service for `/oauth/*` and `/api/v4/user`.

## Notes

- This proxy does minimal transformation, relying on Authentik for token/userinfo logic.
- For advanced claim mapping, refer to Authentik’s documentation.