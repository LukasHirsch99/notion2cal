

## Building

### Creating secrets

There need to be the following secrets present:
- `notion_key_v1`
- `database_url_v1`
- `notion_client_secret_v1`
- `db_password_v1`

```sh
docker --context server secret create notion_key_v1 <(openssl rand -base64 32)

DBPW=$(openssl rand -base64 24 | tr -d '\n=+/')
printf '%s' "$DBPW" | docker --context server secret create db_password_v1 -
printf 'postgres://notion2cal:%s@db:5432/notion2cal' "$DBPW" | docker --context server secret create database_url_v1 -
unset DBPW

# Paste the client secret when prompted, with no echo and no shell history
read -rs NCS && printf '%s' "$NCS" | docker --context server secret create notion_client_secret_v1 - ; unset NCS
```

### Deployment

Build the image locally:
```sh
docker build -t notion2cal:<v> .
```

Modify the version tag `<v>` in `stack.yml`.

```sh
docker --context server stack deploy -c stack.yml --resolve-image never notion2cal
docker --context server stack services notion2cal
docker --context server service logs notion2cal_app
```

```sh
docker --context server stack deploy -c stack.yml --resolve-image never notion2cal
```
does a rolling update.

### Development

**1. Start local postgres**
```sh
docker compose -f compose.dev.yml up -d
```

**2. Create dev encryption key**
```sh
mkdir -p dev-secrets
openssl rand -base64 32 > dev-secrets/notion_key_v1
echo "dev-secrets/" >> .gitignore
echo ".env" >> .gitignore
```
**Never reuse the production key locally!!**

**3. Local `.env` file**
```sh
DATABASE_URL=postgres://notion2cal:devpassword@localhost:5432/notion2cal
NOTION_API_VERSION=2026-03-11
NOTION_CLIENT_ID=your-client-id
NOTION_CLIENT_SECRET=your-client-secret
NOTION_AUTH_URL=your-auth-uri
NOTION_REDIRECT_URI=http://localhost:3001/auth/notion/callback
NOTION_KEY_DIR=./dev-secrets
NOTION_KEY_CURRENT=1
```

**4. Migrations and Run**
```sh
cargo install sqlx-cli --no-default-features --features postgres,rustls   # once
sqlx migrate run
topcoat dev
```
