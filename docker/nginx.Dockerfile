# Nginx reverse proxy replacing the old Pingora content_proxy.
# Routes /rest/ + /auth/ to Supabase, everything else to content_ui.
FROM nginx:alpine

COPY nginx/nginx.conf.template /etc/nginx/templates/default.conf.template

# nginx's official image runs envsubst on /etc/nginx/templates/*.template
# using the environment variables listed here.
ENV SUPABASE_HOST=""
