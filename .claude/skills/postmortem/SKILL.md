---
name: postmortem
description: Conduct a structured session retrospective (what went well, wrong, improvements)
disable-model-invocation: false
context: fork
---

# Session Postmortem

Conduct a structured retrospective after completing a workflow session.

## Instructions

1. **Gather session context:**

   Review the current conversation to identify:
   - What was the goal of this session (feature, bug fix, refactor, etc.)?
   - What was planned vs. what was actually delivered?
   - Which steps were completed?

2. **Summarize the timeline:**

   Build a brief chronological summary of the session:
   - When did each phase start/end (approximate)?
   - Were there any significant pauses, pivots, or restarts?
   - How many implementation cycles were needed before the work was accepted?

3. **Present the postmortem in three sections:**

   ### What Went Well
   Identify things that worked effectively:
   - Phases that went smoothly or faster than expected
   - Good planning decisions that paid off
   - Effective use of parallelization or tooling
   - Clean implementations that passed validation on the first try
   - Successful agent coordination

   ### What Went Wrong
   Identify problems, friction, or failures:
   - Bugs or regressions introduced during implementation
   - Misunderstandings or incorrect assumptions
   - Phases that took longer than expected and why
   - Failed builds, test failures, or validation issues
   - Missing context that led to rework
   - Agent errors or low-confidence guesses that caused problems

   ### What Can Be Improved
   Actionable takeaways for future sessions:
   - Process improvements (e.g., "add X check to planning phase")
   - New context or documentation that should be written
   - Patterns or anti-patterns discovered
   - Tooling gaps or workflow friction points
   - Knowledge that should be recorded

4. **Record lessons learned:**

   If any actionable improvements are identified:
   - Update related files with new insights
   - Propose updates to workflow docs, if process changes are warranted
   - Ask the user before making any doc changes

5. **Ask the user:**
   - "Is there anything else you noticed that I should record?"
   - "Would you like me to update any workflow docs or memory files based on these findings?"
