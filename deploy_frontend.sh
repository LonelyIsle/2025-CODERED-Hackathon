#!/usr/bin/env bash
set -euo pipefail

# === Config ===
FRONTEND_DIR="$HOME/2025-CODERED-Hackathon/frontend"   # change if needed
BUILD_DIR="$FRONTEND_DIR/dist"
WEB_ROOT="/var/www/codered"

# === Detect package manager ===
PKG="npm"
if command -v pnpm >/dev/null 2>&1; then PKG="pnpm"
elif command -v yarn >/dev/null 2>&1; then PKG="yarn"
fi

echo "👉 Using package manager: $PKG"
echo "👉 Building frontend in:  $FRONTEND_DIR"

# === Build ===
cd "$FRONTEND_DIR"
if [[ "$PKG" == "npm" ]]; then
  npm ci
  npm run build
elif [[ "$PKG" == "yarn" ]]; then
  yarn install --frozen-lockfile
  yarn build
else
  pnpm install --frozen-lockfile
  pnpm build
fi

# === Install to nginx web root ===
sudo mkdir -p "$WEB_ROOT"
echo "👉 Syncing $BUILD_DIR → $WEB_ROOT"
sudo rsync -a --delete "$BUILD_DIR"/ "$WEB_ROOT"/

# === Permissions & SELinux ===
# Let nginx read the files even with SELinux enforcing
sudo chown -R root:root "$WEB_ROOT"
sudo find "$WEB_ROOT" -type d -exec chmod 755 {} \;
sudo find "$WEB_ROOT" -type f -exec chmod 644 {} \;
if command -v chcon >/dev/null 2>&1; then
  sudo chcon -R -t httpd_sys_content_t "$WEB_ROOT" || true
fi

# === Nginx sanity & reload ===
echo "👉 Checking nginx config"
sudo nginx -t
echo "👉 Reloading nginx"
sudo systemctl reload nginx

echo "✅ Deployed frontend to $WEB_ROOT"