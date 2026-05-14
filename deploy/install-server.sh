#!/usr/bin/env bash
set -euo pipefail

SERVER_IMAGE="${SIMPRINT_SERVER_IMAGE:-ghcr.io/simprint/simprint-server:latest}"
UPDATE_IMAGE="${SIMPRINT_UPDATE_IMAGE:-ghcr.io/simprint/simprint-update-server:latest}"
CONSOLE_IMAGE="${SIMPRINT_CONSOLE_IMAGE:-ghcr.io/simprint/simprint-console-server:latest}"
EXTENSION_SYNC_IMAGE="${SIMPRINT_EXTENSION_SYNC_IMAGE:-ghcr.io/simprint/simprint-extension-sync-server:latest}"

TARGET_DIR="$PWD/simprint-self-hosted"
CONFIG_DIR="$TARGET_DIR/configs"
COMPOSE_FILE="$TARGET_DIR/docker-compose.yml"
SERVER_CONFIG_FILE="$CONFIG_DIR/server.toml"
UPDATE_CONFIG_FILE="$CONFIG_DIR/update.toml"
CONSOLE_CONFIG_FILE="$CONFIG_DIR/console.toml"
EXTENSION_SYNC_CONFIG_FILE="$CONFIG_DIR/config.prod.yaml"
EXTENSION_SYNC_PRESETS_FILE="$CONFIG_DIR/extensions.yaml"

DEFAULT_APP_SECRET="Nuexz9Y2hRc5Z6HK7Atb"
DEFAULT_POSTGRES_PASSWORD="simprint-postgres-password"
DEFAULT_PUBLIC_BASE_URL="https://pub-39307a5e69c74324855a762027cbf9bf.r2.dev"
DEFAULT_REFERRAL_LINK_PREFIX="https://www.simprint.app/download"

PUBLIC_BASE_URL="${PUBLIC_BASE_URL:-$DEFAULT_PUBLIC_BASE_URL}"
STORAGE_ENDPOINT="${STORAGE_ENDPOINT:-}"
STORAGE_ACCESS_KEY="${STORAGE_ACCESS_KEY:-}"
STORAGE_SECRET_ACCESS_KEY="${STORAGE_SECRET_ACCESS_KEY:-}"
REFERRAL_LINK_PREFIX="${REFERRAL_LINK_PREFIX:-$DEFAULT_REFERRAL_LINK_PREFIX}"
SMTP_SERVER="${SMTP_SERVER:-}"
SMTP_USERNAME="${SMTP_USERNAME:-}"
SMTP_PASSWORD="${SMTP_PASSWORD:-}"
POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-}"
TTY_DEVICE="/dev/tty"

check_command() {
  command -v "$1" >/dev/null 2>&1
}

detect_compose() {
  if docker compose version >/dev/null 2>&1; then
    COMPOSE_CMD=("docker" "compose")
    return 0
  fi

  if check_command docker-compose; then
    COMPOSE_CMD=("docker-compose")
    return 0
  fi

  return 1
}

require_value() {
  local var_name="$1"
  local prompt_text="$2"
  local secret="${3:-false}"
  local current_value="${!var_name:-}"

  if [ -n "$current_value" ]; then
    return 0
  fi

  if [ ! -r "$TTY_DEVICE" ]; then
    echo "Error: missing required value for $var_name. No interactive terminal available." >&2
    exit 1
  fi

  if [ "$secret" = "true" ]; then
    read -r -s -p "$prompt_text: " current_value <"$TTY_DEVICE"
    echo
  else
    read -r -p "$prompt_text: " current_value <"$TTY_DEVICE"
  fi

  if [ -z "$current_value" ]; then
    echo "Error: $var_name cannot be empty." >&2
    exit 1
  fi

  printf -v "$var_name" '%s' "$current_value"
}

require_secret_with_default() {
  local var_name="$1"
  local prompt_text="$2"
  local default_value="$3"
  local current_value="${!var_name:-}"
  local use_default=""

  if [ -n "$current_value" ]; then
    return 0
  fi

  if [ ! -r "$TTY_DEVICE" ]; then
    printf -v "$var_name" '%s' "$default_value"
    return 0
  fi

  read -r -p "[$prompt_text, default:$default_value][Y/n]? " use_default <"$TTY_DEVICE"
  case "${use_default:-Y}" in
    Y|y|"")
      printf -v "$var_name" '%s' "$default_value"
      return 0
      ;;
  esac

  read -r -s -p "$prompt_text: " current_value <"$TTY_DEVICE"
  echo

  if [ -z "$current_value" ]; then
    echo "Error: $var_name cannot be empty." >&2
    exit 1
  fi

  printf -v "$var_name" '%s' "$current_value"
}

create_console_api_key() {
  local output=""
  local api_key_line=""
  local attempt=1
  local max_attempts=20

  while [ "$attempt" -le "$max_attempts" ]; do
    if output=$(
      docker exec simprint-console-gateway /app/app \
        --config /app/configs/console.toml apikey create --name "extension-sync" 2>&1
    ); then
      api_key_line=$(printf '%s\n' "$output" | sed -n 's/^[[:space:]]*X-API-KEY: //p' | tail -n 1)

      if [ -n "$api_key_line" ] && [ "${api_key_line#*:}" != "$api_key_line" ]; then
        EXTENSION_SYNC_API_KEY_ID="${api_key_line%%:*}"
        EXTENSION_SYNC_API_KEY_SECRET="${api_key_line#*:}"
        return 0
      fi

      echo "Error: Console API key command succeeded but returned an unexpected payload." >&2
      echo "$output" >&2
      exit 1
    fi

    echo "Waiting for simprint-console-gateway to become ready (attempt $attempt/$max_attempts)..."
    sleep 3
    attempt=$((attempt + 1))
  done

  echo "Error: failed to create Console API key for extension sync after $max_attempts attempts." >&2
  echo "$output" >&2
  exit 1
}

if ! check_command docker; then
  echo "Error: docker is not installed. Install Docker first." >&2
  exit 1
fi

if ! docker info >/dev/null 2>&1; then
  echo "Error: docker is installed but the daemon is not available." >&2
  exit 1
fi

if ! detect_compose; then
  echo "Error: neither 'docker compose' nor 'docker-compose' is available." >&2
  exit 1
fi

mkdir -p "$CONFIG_DIR"

require_secret_with_default POSTGRES_PASSWORD "PostgreSQL password" "$DEFAULT_POSTGRES_PASSWORD"
require_value STORAGE_ENDPOINT "Object storage endpoint"
require_value STORAGE_ACCESS_KEY "Object storage access key"
require_value STORAGE_SECRET_ACCESS_KEY "Object storage secret access key" true
require_value SMTP_SERVER "SMTP server"
require_value SMTP_USERNAME "SMTP username"
require_value SMTP_PASSWORD "SMTP password" true

cat >"$SERVER_CONFIG_FILE" <<EOF
[app]
name = "simprint-server"
port = 40041
secret = "$DEFAULT_APP_SECRET"
prefix = "/api/v1"
encrypt_secret_location = "./assets/secret"
route_whitelists = [
    "POST+/api/v1/users/login",
    "POST+/api/v1/users/register",
    "POST+/api/v1/users/register-send-code",
    "POST+/api/v1/users/reset-password",
    "POST+/api/v1/users/reset-password-send-code",
    "POST+/api/v1/users/refresh-credentials",
    "GET+/api/v1/secret/public/key",
    "GET+/api/v1/time/now",
    "POST+/api/v1/versions/check"
]
referral_link_prefix = "$REFERRAL_LINK_PREFIX"

[database]
url = "postgres://simprint:$POSTGRES_PASSWORD@postgres:5432/simprintdb"
max_connections = 25
min_connections = 5
max_lifetime = 3000
acquire_timeout = 30
idle_timeout = 600

[redis]
url = "redis://redis:6379?protocol=resp3"

[storage]
endpoint = "$STORAGE_ENDPOINT"
public_base_url = "$PUBLIC_BASE_URL"
access_key = "$STORAGE_ACCESS_KEY"
secret_access_key = "$STORAGE_SECRET_ACCESS_KEY"
bucket = "simprint-client"
avatar_root = "avatars"
extension_root = "extensions"
version_root = "versions"

[smtp]
smtp_server = "$SMTP_SERVER"
smtp_username = "$SMTP_USERNAME"
smtp_password = "$SMTP_PASSWORD"

[workspace_quota]
[workspace_quota.default]
max_environments = 99999
max_team_members = 99999
max_proxies = 99999
max_rpa_tasks = 99999
EOF

cat >"$UPDATE_CONFIG_FILE" <<EOF
[app]
name = "simprint-update-server"
port = 40042
secret = "$DEFAULT_APP_SECRET"
prefix = "/api/v1"
route_whitelists = [
    "POST+/api/v1/versions/check",
    "POST+/api/v1/maintenances/active",
    "GET+/api/v1/health"
]

[database]
url = "postgres://simprint:$POSTGRES_PASSWORD@postgres:5432/simprintdb"
max_connections = 25
min_connections = 5
max_lifetime = 3000
acquire_timeout = 30
idle_timeout = 600

[redis]
url = "redis://redis:6379?protocol=resp3"

[storage]
public_base_url = "$PUBLIC_BASE_URL"
bucket = "simprint-client"
version_root = "versions"
EOF

cat >"$CONSOLE_CONFIG_FILE" <<EOF
[app]
name = "simprint-console-server"
port = 40043
secret = "$DEFAULT_APP_SECRET"
prefix = "/api/v1"
route_whitelists = [
    "POST+/api/v1/users/login",
    "POST+/api/v1/users/reset-password",
    "POST+/api/v1/users/reset-password-send-code",
    "POST+/api/v1/users/refresh-credentials",
    "GET+/api/v1/secret/public/key",
    "GET+/api/v1/time/now",
    "GET+/api/v1/health",
]

[database]
url = "postgres://simprint:$POSTGRES_PASSWORD@postgres:5432/simprintdb"
max_connections = 25
min_connections = 5
max_lifetime = 3000
acquire_timeout = 30
idle_timeout = 600

[redis]
url = "redis://redis:6379?protocol=resp3"

[storage]
endpoint = "$STORAGE_ENDPOINT"
public_base_url = "$PUBLIC_BASE_URL"
access_key = "$STORAGE_ACCESS_KEY"
secret_access_key = "$STORAGE_SECRET_ACCESS_KEY"
bucket = "simprint-client"
avatar_root = "avatars"
extension_root = "extensions"
version_root = "versions"

[smtp]
smtp_server = "$SMTP_SERVER"
smtp_username = "$SMTP_USERNAME"
smtp_password = "$SMTP_PASSWORD"

[workspace_quota]
[workspace_quota.default]
max_environments = 8
max_team_members = 1
max_proxies = 99999
max_rpa_tasks = 99999
EOF

cat >"$EXTENSION_SYNC_PRESETS_FILE" <<'EOF'
# Simprint Extension Sync Server preset extensions

preset_extensions: []

categories:
  - id: "security"
    name: "安全隐私"
    name_en: "Security & Privacy"

  - id: "development"
    name: "开发工具"
    name_en: "Developer Tools"

  - id: "productivity"
    name: "效率工具"
    name_en: "Productivity"

  - id: "automation"
    name: "自动化"
    name_en: "Automation"

  - id: "other"
    name: "其他"
    name_en: "Other"

high_risk_permissions:
  - "debugger"
  - "cookies"
  - "webRequest"
  - "webRequestBlocking"
  - "<all_urls>"
  - "nativeMessaging"
  - "proxy"
  - "history"
  - "downloads"
  - "management"
  - "privacy"
  - "browsingData"
EOF

cat >"$COMPOSE_FILE" <<EOF
services:
  postgres:
    image: postgres:16-alpine
    container_name: simprint-postgres
    environment:
      POSTGRES_DB: simprintdb
      POSTGRES_USER: simprint
      POSTGRES_PASSWORD: $POSTGRES_PASSWORD
    volumes:
      - simprint-postgres-data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U simprint -d simprintdb"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped
    networks:
      - simprint-network

  redis:
    image: redis:7-alpine
    container_name: simprint-redis
    command: ["redis-server", "--appendonly", "yes"]
    volumes:
      - simprint-redis-data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped
    networks:
      - simprint-network

  simprint-server:
    image: $SERVER_IMAGE
    container_name: simprint-server
    command: ["-f=/app/configs/server.toml"]
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    ports:
      - "40041:40041"
    volumes:
      - ./configs:/app/configs:ro
      - simprint-secret-data:/app/assets/secret
    environment:
      - RUST_LOG=info
    restart: unless-stopped
    networks:
      - simprint-network

  simprint-update-gateway:
    image: $UPDATE_IMAGE
    container_name: simprint-update-gateway
    command: ["-f=/app/configs/update.toml"]
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    ports:
      - "40042:40042"
    volumes:
      - ./configs:/app/configs:ro
    environment:
      - RUST_LOG=info
    restart: unless-stopped
    networks:
      - simprint-network

  simprint-console-gateway:
    image: $CONSOLE_IMAGE
    container_name: simprint-console-gateway
    command: ["--config", "/app/configs/console.toml", "serve"]
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    ports:
      - "40043:40043"
    volumes:
      - ./configs:/app/configs:ro
    environment:
      - RUST_LOG=info
    restart: unless-stopped
    networks:
      - simprint-network

  simprint-extension-sync:
    image: $EXTENSION_SYNC_IMAGE
    container_name: simprint-extension-sync
    depends_on:
      - simprint-console-gateway
    ports:
      - "18080:8080"
    volumes:
      - ./configs:/app/configs:ro
      - simprint-extension-sync-data:/app/storage
    restart: unless-stopped
    networks:
      - simprint-network

networks:
  simprint-network:
    name: simprint-network

volumes:
  simprint-postgres-data:
  simprint-redis-data:
  simprint-secret-data:
  simprint-extension-sync-data:
EOF

echo "Using compose command: ${COMPOSE_CMD[*]}"
echo "Working directory: $TARGET_DIR"
echo "Server image: $SERVER_IMAGE"
echo "Update image: $UPDATE_IMAGE"
echo "Console image: $CONSOLE_IMAGE"
echo "Extension sync image: $EXTENSION_SYNC_IMAGE"
echo "Generated files:"
echo "  - $COMPOSE_FILE"
echo "  - $SERVER_CONFIG_FILE"
echo "  - $UPDATE_CONFIG_FILE"
echo "  - $CONSOLE_CONFIG_FILE"
echo "  - $EXTENSION_SYNC_CONFIG_FILE"
echo "  - $EXTENSION_SYNC_PRESETS_FILE"

(
  cd "$TARGET_DIR"
  "${COMPOSE_CMD[@]}" up -d postgres redis simprint-server simprint-update-gateway simprint-console-gateway
)

create_console_api_key

cat >"$EXTENSION_SYNC_CONFIG_FILE" <<EOF
# Simprint Extension Sync Server configuration

backend:
  api_url: "http://simprint-console-gateway:40043/api/v1"
  api_key_id: "$EXTENSION_SYNC_API_KEY_ID"
  api_key_secret: "$EXTENSION_SYNC_API_KEY_SECRET"

sync:
  full_interval_hours: 24
  update_check_interval_minutes: 60
  concurrent_downloads: 5
  max_retries: 3

logging:
  level: "INFO"
  format: "console"

http:
  host: "0.0.0.0"
  port: 8080

storage:
  sqlite_path: "storage/extensions.db"
EOF

(
  cd "$TARGET_DIR"
  "${COMPOSE_CMD[@]}" up -d simprint-extension-sync
  "${COMPOSE_CMD[@]}" ps
)
