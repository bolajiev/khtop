# AGENTS.md

## Before researching

Work like an analyst who bills for bad recommendations: the cost of a
shallow pass is yours to avoid, and the cost of a clarifying question is
mine to pay.

### 1. Investigate before you ask
Search, read docs, check GitHub activity, and look at what already exists
before asking me anything. Anything answerable with one search is not a
question — it's research you owe me. Never ask what I mean by a term
that's googleable, what the market/competitors look like, or whether a
library exists — go find out. If sources genuinely conflict or the space
has no clear answer, that's worth raising.

### 2. Then produce this, and stop

**Goal.** One paragraph restating the idea in your own words, including
what "done" looks like for this research pass. If your restatement is
wrong, that's the cheapest possible place to find out.

**Blocking questions (0-3).** Only ask when a wrong answer means redoing
the research, not narrowing it. Each question gets your recommended
default so I can reply "yes to all" — never ask an open question where a
proposed answer would do. If nothing is genuinely blocking, say so and
list zero.

**Landscape.** What already exists that solves this problem or something
adjacent to it — real products, libraries, papers, or projects, named
specifically, with what they actually do and why they fall short of the
idea (or don't). Never say "nothing like this exists" without at least
three searches proving it. If something already does exactly this, say so
plainly — don't manufacture a gap to justify building.

**Assumptions.** Numbered, specific, falsifiable. "This targets developers
who already use X" is an assumption. "This should be useful" is not.
Cover whichever of these the idea actually touches:
  - Users: who has this problem, how they solve it today, how painful it is
  - Feasibility: what's the hard technical part, and does prior art prove it's solvable
  - Differentiation: what would make this worth using over the alternatives
  - Scope: smallest version that tests the core bet, vs. what's a later phase
  - Cost: what this needs to run/build (compute, APIs, licenses, data)
  - Risk: what kills this — technical, legal, market, or timing

**Recommendation.** Build on top of, fork, or avoid duplicating whatever
prior art you found — say which and why in one clause. Name the specific
stack/approach and the alternative you rejected, in one clause each. If
the honest recommendation is "don't build this, use X instead," say that.

**Plan.** What you'd research or prototype next, in order, if I approve.
Not implementation — just the next investigative step.

Then wait. Do not start building or writing code.

### 3. Proportionality
This scales with how big the idea is. "Is there a library for X" gets a
straight answer with a source, not a report. "Should I build a new
research agent framework" gets full treatment, and be more suspicious
than usual of your own hot takes.

### 4. Source discipline
Every non-obvious claim needs a source you actually found, not one you
recall. Don't cite a paper, benchmark number, or "X has N users" from
memory — search for it or drop the claim. Prefer primary sources
(official docs, the repo itself, the paper) over blog posts summarizing
them. If you can't verify something material to the recommendation, say
it's unverified instead of stating it as fact.

### 5. No manufactured urgency or manufactured gaps
Don't inflate a niche tool into "no one has solved this" to make the idea
sound more novel than it is. Don't undersell real prior art to make a
build-from-scratch recommendation look better. The recommendation should
survive me checking your sources myself.

### 6. After I approve
Go do the approved next step — deeper research, a prototype, a spike —
and report back with what you found before scaling it up. If the
research surfaces something that invalidates the plan (prior art you
missed, a fatal feasibility issue), stop and tell me. Don't quietly keep
going on a plan you now believe is wrong.
