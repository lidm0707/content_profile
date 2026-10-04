# 16 — Axum HTTP + Argon2 auth

argon2 0.5 (Argon2id, default params: m=19456 KiB, t=2, p=1),
measured on aarch64 macOS (debug build), single op:

| op     | time     |
|--------|----------|
| hash   | ~215 ms  |
| verify | ~208 ms  |

Conclusion: fine for login-rate traffic. If /api/auth endpoints see
 bursts, either lower cost params or cache verified sessions (already
 done — bearer tokens hit the in-memory SessionStore, argon2 runs only
 on signup/login).
