#!/usr/bin/env bash
set -euo pipefail

# === Config ===
FRONTEND_DIR="$HOME/2025-CODERED-Hackathon/frontend"
BUILD_DIR="$FRONTEND_DIR/out"          # Next static export output
WEB_ROOT="/var/www/codered"

# === Pick a package manager ===
PKG="npm"
if command -v pnpm >/dev/null 2>&1; then PKG="pnpm"
elif command -v yarn >/dev/null 2>&1; then PKG="yarn"
fi

echo "👉 Using package manager: $PKG"
echo "👉 Frontend: $FRONTEND_DIR"

# === Build (static export via next.config.js output:'export') ===
cd "$FRONTEND_DIR"
if [[ "$PKG" == "npm" ]]; then
  npm ci || npm install
  npm run build
elif [[ "$PKG" == "yarn" ]]; then
  yarn install --frozen-lockfile || yarn install
  yarn build
else
  pnpm install --frozen-lockfile || pnpm install
  pnpm build
fi

# Sanity check
if [[ ! -d "$BUILD_DIR" ]]; then
  echo "❌ Build directory $BUILD_DIR not found (did Next.js export run?)."
  exit 1
fi

# === Deploy to nginx web root ===
sudo mkdir -p "$WEB_ROOT"
echo "👉 Syncing $BUILD_DIR → $WEB_ROOT"
sudo rsync -a --delete "$BUILD_DIR"/ "$WEB_ROOT"/

# === Permissions & SELinux labels ===
sudo chown -R root:root "$WEB_ROOT"
sudo find "$WEB_ROOT" -type d -exec chmod 755 {} \;
sudo find "$WEB_ROOT" -type f -exec chmod 644 {} \;
if command -v chcon >/dev/null 2>&1; then
  sudo chcon -R -t httpd_sys_content_t "$WEB_ROOT" || true
fi

# === Nginx check & reload ===
echo "👉 Checking nginx config"
sudo nginx -t
echo "👉 Reloading nginx"
sudo systemctl reload nginx

echo "✅ Deployed Next.js static export to $WEB_ROOT"