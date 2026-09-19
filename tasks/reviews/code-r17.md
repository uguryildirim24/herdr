+++
verdict = "MERGE"
round = "r17"
candidate = "085b069a2e365da776f55268014223661b0cfccd"
manifest_hash = "6b5a5c6d232197c44c52ba320cb87dbb609c2ebbd220cbacbcb28ede52446513"
policy_hash = "7cf667dea9748fe2844267e971e613ff74b94a3ef0af9df5ee974d759cb7e9da"
gates = []
+++

# Round r17 review

## t-0038 — the project screen plan and pictures

MERGE. The plan gives the project screen explicit sources for its goal, result, progress, active work, landed work, questions and recorded choices while retaining the conversation contract.

The review fixes one lifecycle gap. The original text removed a lane from `Running now` as soon as its thread was resolved, but the standing workflow resolves and closes a lane when its report arrives, before review and merge. The screen now keeps pinned work visible as `checking` until its carrying round lands, regardless of pane lifetime, and the acceptance rows cover that transition. It also maps every unresolved thread group to one of the three visible state words and removes one human-authority reference phrased as “owner.”

The lane copied the supplied spec, prompts and decision row exactly, with the required two-byte trim on the wide prompt. Both PNGs exist at their requested dimensions and visibly follow the one-column overview, wide chat panel and narrow pinned-card layouts. All five `plain:` lines pass the deterministic checker and remain below 25 words.

The round lists no project gates, so `gates = []`. No code was built for this document round.
