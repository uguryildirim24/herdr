# SPEC-pro-bridge v1 → v2 (fold of checks a and b)

1. Safe mode stays, but Pro has no hands, so the plugin pastes the brief in and writes the answer out.
2. The desktop app is the default host (tabs stay off the screen); a Chrome window per turn is the fallback; the plugin never runs the bridge.
3. v1 uses your normal Codex home with per-process overrides; a separate shared Pro home waits until herdr remembers a pane's folder after a full restart.
4. One Codex folder per lane is out: each would need its own login, and the crash only happens on a brand-new folder.
5. DONE is typed by the collector after it reads the result, never because the pane went idle.
6. Two Pro turns at once by default, four as the hard max, a weekly counter at 150, and a stop switch so retries cannot storm.
7. The 09-09 and 09-13 events were per-call refusals of file reads, not "detection."
8. Coordinators call the plugin start and turn commands; old Pro tabs stay until that chat is done; never restart the live herdr server for this.
9. Keep the old Pro-tab type until the last old pane is gone; ADE must later accept Pro as Codex.
10. The first live proof is one signed-in turn you approve; the model name is read from the desktop-app view, not from the model.

## Questions that need Rolf

Each item is a picturable choice. Pick one sentence.

11. Bridge host
    - A: Use the desktop app. ChatGPT tabs stay off the screen.
    - B: Use a real Chrome window that opens for every Pro turn.
    - C: Desktop app now; Chrome window only if the app cannot be installed.

12. Weekly send budget
    - A: Cap at about 150 Pro sends a week (slack under 200).
    - B: Cap at another number you name.
    - C: Do not add a cap. Rely only on ChatGPT's own limit.

13. New conversation or growing thread
    - A: Each Pro attack starts a new conversation with only the brief and files.
    - B: Keep one conversation per topic that grows.
    - C: New conversation for long topics; keep one thread for short ones.

14. After a full herdr restart
    - A: The plugin starts the Pro lane again itself (v1).
    - B: herdr remembers enough to bring the lane back (later fork work).
    - C: Do not resume. Start a new Pro lane after a restart.

15. The one live proof
    - A: You run it by hand, once, after you sign in.
    - B: A lane runs it once, after you have signed in and said go.
    - C: You run the first turn; a lane may run a second lane in parallel after that.

16. Retries
    - A: Only the plugin stop switch (no new turns after a limit or crash).
    - B: Stop switch, and also try to tell Codex to retry less.
    - C: No extra brake. Accept that one limit can become several sends.

17. Pro chats in your normal Codex history
    - A: Yes. It is OK that they show up in history and memories.
    - B: No. Wait for a separate Pro home so they stay out.
    - C: History is OK. Memories must stay off if we can turn them off.

18. Extra message that names the chat
    - A: Accept one extra Pro message to name a new chat.
    - B: Skip the title if we can find a way.
    - C: Block new sessions until we know the title request does not spend Pro.

19. Your normal Codex rules
    - A: Pro may see your normal rules (v1 shares that home).
    - B: Hide those rules from Pro if we can.
    - C: Attach a short Pro-only rules file and ignore the rest.

20. Who packs the files for Pro
    - A: The coordinator names each file every turn.
    - B: The plugin always packs a standard set (brief, spec, last answer).
    - C: The coordinator names files, and may also attach a copy of another lane's screen.

21. After a failed turn
    - A: Do nothing until you say so.
    - B: Allow one retry after a health check.
    - C: The coordinator may retry freely.

22. After you install or update the desktop app
    - A: You run one disconnect command so daily Codex does not go through the bridge.
    - B: We need a setup that never writes that route, even if that delays the app.
    - C: Disconnect after the first install only; check on each update.

23. The hook that tells herdr the Codex conversation id
    - A: Leave it off. You turned it off on purpose.
    - B: Turn it on so herdr can resume Codex panes after a restart.
    - C: Leave it off for daily Codex; we will not rely on it for Pro.

24. Remember a pane's folder after a full restart
    - A: Yes, later. Needed for a separate Pro home.
    - B: Not needed. Keep v1's shared normal home and plugin re-start.
    - C: Yes, but only as part of a larger herdr restore change, not for Pro alone.

25. Chrome-window fallback check
    - A: If we use that mode, check once after setup that the background service finds its settings, before any real turn.
    - B: Do not use the Chrome-window mode, so skip this check.
    - C: Check it only if the desktop app cannot be installed.
