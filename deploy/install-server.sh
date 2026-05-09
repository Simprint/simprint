#!/usr/bin/env bash
set -euo pipefail

IMAGE="${SIMPRINT_SERVER_IMAGE:-ghcr.io/simprint/simprint-server:latest}"
TARGET_DIR="$PWD/simprint-self-hosted"
CONFIG_DIR="$TARGET_DIR/configs"
COMPOSE_FILE="$TARGET_DIR/docker-compose.yml"
CONFIG_FILE="$CONFIG_DIR/config.toml"

DEFAULT_SERVER_SECRET="Nuexz9Y2hRc5Z6HK7Atb"
DEFAULT_POSTGRES_PASSWORD="simprint-postgres-password"
PUBLIC_BASE_URL="https://pub-39307a5e69c74324855a762027cbf9bf.r2.dev"
STORAGE_ENDPOINT="https://example.invalid"
REFERRAL_LINK_PREFIX="https://www.simprint.app/download"
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
require_value SMTP_SERVER "SMTP server"
require_value SMTP_USERNAME "SMTP username"
require_value SMTP_PASSWORD "SMTP password" true

cat >"$CONFIG_FILE" <<EOF
[app]
name = "simprint-server"
port = 40041
secret = "$DEFAULT_SERVER_SECRET"
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
access_key = "disabled"
secret_access_key = "disabled"
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

  simprint-server:
    image: $IMAGE
    container_name: simprint-server
    command: ["-f=/app/configs/config.toml"]
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

volumes:
  simprint-postgres-data:
  simprint-redis-data:
  simprint-secret-data:
EOF

echo "Using compose command: ${COMPOSE_CMD[*]}"
echo "Working directory: $TARGET_DIR"
echo "Server image: $IMAGE"
echo "Generated files:"
echo "  - $COMPOSE_FILE"
echo "  - $CONFIG_FILE"

(
  cd "$TARGET_DIR"
  "${COMPOSE_CMD[@]}" up -d
  "${COMPOSE_CMD[@]}" ps
)
