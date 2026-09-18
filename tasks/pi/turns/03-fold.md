# SPEC-pi v2 fold (turn 03)

Lane `pi-fold`, 2026-09-18. v1 → v2 from checks a and b and Rolf's 17:50 / 18:05 / 18:30 rows.
18:30 supersedes brief item 1 and the 17:50 / 18:05 Cursor rows: Cursor stays outside pi.

## What changed (v1 → v2)

Workers share one pi folder and one login file, not a copy per worker.
The "approve" switch is gone: it trusted project add-on code, and it approved no tools.
A helper inside pi reports a usage limit, a dead login, or a dead endpoint as stuck / waiting, never as finished.
A small `pi` launcher on your path brings a worker back with that folder after a herdr restart; the unmerged restart branch is not required for pi.
Cursor stays in the Cursor app while this port runs, then that recipe is retired; it never runs under pi. Grok inside pi, if ever, is only an xAI sign-in or key. Pro stays a Codex worker the bridge plugin drives.
Pi is a library inside the main harness add-on, plus a small setup / login / doctor program.
Workers never trust a project's own pi add-ons, and they do not load your personal skill collection.
Login is pi's own on-screen login, once per provider; there is no shell login command.
Done is still the typed harness line from inside the worker.
Direct DeepSeek and Kimi stay; OpenCode is only for what only OpenCode has, unless you say otherwise.

## Decided (do not ask again)

- Cursor stays outside pi. Native Cursor lanes only, then that recipe is retired. No SDK connector, no community add-on (Rolf, 2026-09-18 18:30). This replaces the 17:50 and 18:05 Cursor rows.
- Unofficial Cursor add-ons that reuse the Cursor app login are out (both checkers; kept as a note).
- OpenCode still runs under pi (Rolf, 2026-09-18 17:50 / 18:05, not withdrawn).
- Pro stays a separate Codex worker that the bridge plugin drives (Rolf, 2026-09-18 18:05).
- Pro is a general worker, not spec-only, and has no weekly send cap from us (Rolf, 2026-09-18 17:50).
- The model picker is built in this plugin round (Rolf, 2026-09-18 17:50).
- Turn on the Codex conversation-id hook (Rolf, 2026-09-18 17:50).
- Workers never run add-on code that a project stores in its own folder (fold default).
- Pi workers see only the skills the harness gives them, not your personal collection (fold default).
- Pi lives inside the main harness add-on as a library, with a small setup program (fold default).
- One shared settings folder and one login for every pi worker (fold default).
- A herdr restart does not wait on the unmerged restart branch for pi workers (fold default).

## Questions that still need you

Each item is a picturable choice. Pick one option.

1. Daily ChatGPT coding models (Astra and Sol):
   - Use ChatGPT's own coding login, the same subscription, no extra web bridge.
   - Send them through the same local web bridge as Pro.
   - Leave them on Codex until you say to move them.

2. When you type `pi` in a normal terminal:
   - It uses the harness logins (one pi on this Mac).
   - It stays your personal pi; only workers use the harness copy.
   - There is no `pi` in a normal terminal; only workers have it.

3. DeepSeek and OpenCode keys:
   - Sit in the harness login file, like today's Codex keys.
   - Sit in the Mac keychain; pi asks the keychain each run.
   - You paste them each time a worker starts (not recommended).

4. DeepSeek and Kimi bills:
   - Pay each one directly, as today.
   - Run both through your OpenCode account so there is one bill.
   - Direct for now, and look at one bill later.

5. When a worker hits a usage limit:
   - It tells its coordinator and waits for you.
   - It tries again by itself after the wait time the service gives.
   - It is marked stuck until you look at that pane.

6. After Cursor lanes go away, Grok:
   - Workers use a SuperGrok or X Premium sign-in.
   - Workers skip Grok.
   - Workers pay per token for Grok through OpenCode.

7. If a worker's folder was deleted before a restart:
   - The harness leaves that worker stopped for you.
   - The harness starts it fresh somewhere else.
   - The harness asks you with Continue or Cancel and waits.

8. Look once at the Pro slider on the ChatGPT page (when you next open it):
   - The label is GPT-6 Pro, same as the page you already use.
   - The label is a different Pro, and you will say which.
   - Skip this until Pro is in use as a worker.
