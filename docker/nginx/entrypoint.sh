#!/bin/sh
# Resolve the Supabase host from SUPABASE_URL (https://abc.supabase.co ->
# abc.supabase.co) and regenerate the nginx config with it. Runs as a
# /docker-entrypoint.d script (numbered 30 so it runs after the official
# 20-envsubst pass, whose output it overwrites).
set -e

# Fail fast if SUPABASE_URL is missing or has no host.
case "$SUPABASE_URL" in
  https://*|http://*) : ;;
  *) echo "SUPABASE_URL must start with http(s)://" >&2; exit 1 ;;
esac

# Re-run the template substitution with the host set. The official image's
# 20-envsubst script already ran with an empty SUPABASE_HOST at this point,
# so we regenerate the config ourselves.
SUPABASE_HOST=$(printf '%s' "$SUPABASE_URL" | sed -E 's|^https?://||; s|/$||')
export SUPABASE_HOST

# Put the host in an nginx variable (via a generated include) rather than
# envsubst'ing the whole template — a static `proxy_pass https://host` makes
# nginx resolve at startup and EXIT if Docker DNS has no answer yet.
cat > /etc/nginx/conf.d/supabase_host.conf <<EOF
map \$http_x_supabase_unused \$supabase_host { default $SUPABASE_HOST; }
EOF

echo "nginx: proxying /rest/ and /auth/ to https://$SUPABASE_HOST"
