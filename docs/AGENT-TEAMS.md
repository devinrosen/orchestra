# Agent Teams Research

Research notes on Claude Code agent teams for orchestrating parallel work.

Sources:
- [Agent Teams docs](https://code.claude.com/docs/en/agent-teams)
- [Use case examples](https://code.claude.com/docs/en/agent-teams#use-case-examples)
- [Agent team token costs](https://code.claude.com/docs/en/costs#agent-team-token-costs)

## What Are Agent Teams?

Multiple Claude Code instances coordinated as a team — one **lead** session plus multiple **teammates**, each with their own context window. Unlike subagents (which only report results back to the caller), teammates can message each other directly and share a task list.

### Agent Teams vs Subagents

| | Subagents | Agent Teams |
|---|---|---|
| **Context** | Own context window; results return to caller | Own context window; fully independent |
| **Communication** | Report back to main agent only | Teammates message each other directly |
| **Coordination** | Main agent manages all work | Shared task list with self-coordination |
| **Best for** | Focused tasks where only the result matters | Complex work requiring discussion and collaboration |
| **Token cost** | Lower: results summarized back to main context | Higher: each teammate is a separate Claude instance |

Use **subagents** when you need quick, focused workers that report back. Use **agent teams** when teammates need to share findings, challenge each other, and coordinate on their own.

## When to Use Agent Teams

Best use cases:
- **Research and review**: multiple teammates investigate different aspects simultaneously, then share and challenge each other's findings
- **New modules or features**: teammates each own a separate piece without stepping on each other
- **Debugging with competing hypotheses**: teammates test different theories in parallel and converge faster
- **Cross-layer coordination**: changes spanning frontend, backend, and tests, each owned by a different teammate

Avoid agent teams for: sequential tasks, same-file edits, or work with many dependencies — use a single session or subagents instead.

## Setup

### Enable

Agent teams are experimental and disabled by default. Enable via settings.json or environment:

```json
{
  "env": {
    "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"
  }
}
```

### Display Modes

- **In-process** (default): all teammates run inside the main terminal. Use Shift+Up/Down to select a teammate and type to message them. Press Ctrl+T to toggle the task list.
- **Split panes**: each teammate gets its own pane (requires tmux or iTerm2). Set `"teammateMode": "tmux"` in settings.json or pass `--teammate-mode in-process` as a flag.

## Key Mechanics

- Teammates load CLAUDE.md, MCP servers, and skills automatically but do **not** inherit the lead's conversation history
- **Delegate mode** (Shift+Tab): restricts the lead to coordination-only tools (spawning, messaging, task management) — prevents the lead from implementing tasks itself
- **Plan approval**: teammates can be required to plan before implementing; the lead reviews and approves/rejects plans
- **Task claiming**: file-locking prevents race conditions when multiple teammates try to claim the same task
- **Hooks**: `TeammateIdle` and `TaskCompleted` hooks enforce quality gates

### Teammate Lifecycle

1. Lead creates team and spawns teammates with specific prompts
2. Teammates claim or are assigned tasks from the shared task list
3. Teammates work independently, messaging each other as needed
4. When done, lead sends shutdown requests; teammates approve/reject
5. Lead cleans up the team (removes shared resources)

## Use Case Examples

### Parallel Code Review

Split review criteria into independent domains so security, performance, and test coverage all get thorough attention simultaneously:

```
Create an agent team to review PR #142. Spawn three reviewers:
- One focused on security implications
- One checking performance impact
- One validating test coverage
Have them each review and report findings.
```

### Competing Hypotheses Debugging

Make teammates explicitly adversarial — each investigates its own theory and challenges the others:

```
Users report the app exits after one message instead of staying connected.
Spawn 5 agent teammates to investigate different hypotheses. Have them talk to
each other to try to disprove each other's theories, like a scientific
debate. Update the findings doc with whatever consensus emerges.
```

## Token Costs

Agent teams use **~7x more tokens** than standard sessions when teammates run in plan mode. Each teammate maintains its own context window and runs as a separate Claude instance.

### Cost Optimization

- **Use Sonnet for teammates** — balances capability and cost for coordination tasks
- **Keep teams small** — token usage is roughly proportional to team size
- **Keep spawn prompts focused** — everything in the prompt adds to context from the start
- **Clean up teams when done** — active teammates continue consuming tokens even if idle

## Best Practices

- **Give teammates enough context**: include task-specific details in the spawn prompt (they don't inherit the lead's conversation history)
- **Size tasks appropriately**: self-contained units that produce a clear deliverable (a function, a test file, a review). Aim for 5-6 tasks per teammate.
- **Avoid file conflicts**: break work so each teammate owns different files — two teammates editing the same file leads to overwrites
- **Monitor and steer**: check in on progress, redirect approaches that aren't working, synthesize findings as they come in
- **Wait for teammates**: tell the lead "Wait for your teammates to complete their tasks before proceeding" if it starts implementing itself
- **Start with research/review**: if new to agent teams, begin with non-code tasks (PR review, library research, bug investigation) before parallel implementation

## Limitations

- No session resumption with in-process teammates (`/resume` and `/rewind` don't restore them)
- Task status can lag — teammates sometimes fail to mark tasks completed
- One team per session; no nested teams
- Lead is fixed for the team's lifetime
- All teammates start with the lead's permission mode (changeable after spawn, not at spawn time)
- Split panes require tmux or iTerm2 (not supported in VS Code terminal, Windows Terminal, or Ghostty)
