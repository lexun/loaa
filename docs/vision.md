# Loa'a Vision

## The Problem

Kids need motivation to do chores and study, but pure top-down pressure creates overwhelm and resistance. They need:
- Clear visibility into what's expected of them
- Tangible rewards that feel earned, not given
- Agency to choose their path while meeting essentials
- Understanding of value and work/reward relationship

Traditional allowances don't create this connection. Verbal reminders create friction. We need a system that makes expectations clear, rewards feel earned, and gives kids control over their priorities.

## Core Concept

**Loa'a** (Hawaiian: "to earn" or "receive") is a chore and rewards tracking system that gamifies task completion with real monetary rewards. It treats household responsibilities like a marketplace where kids can:
- See all available tasks with their values
- Choose what to work on (within constraints)
- Track their earnings in a personal ledger
- Redeem rewards (screen time, cash, privileges)

## Evolution from Previous Attempt

An earlier project (`gp`) implemented a basic points system with:
- Children with point balances
- Manual point awards by parent
- Device management for screen time tracking
- Next.js + TypeScript + Supabase stack

**What was missing:**
- No task/chore definitions - just manual point awards
- No self-service for kids - parent had to award everything
- No prerequisites or dependencies
- No dynamic incentives (bounties)
- No completion workflow (kids mark done → parent approves)
- TypeScript was not enjoyable to work with

**Key improvements in Loa'a:**
- Task definitions with values and refresh cadence
- Kids can mark tasks complete (pending approval)
- Ledger system with transaction history
- Future: prerequisites, bounties, market dynamics
- Built in Rust for better developer experience

## Key Features

### MVP Features (Phase 1)

**Task Management**
- Tasks have names, descriptions, monetary values ($)
- Refresh cadence: daily, weekly, one-time
- Parent creates and manages task definitions

**Completion Workflow**
- Kids mark tasks as complete
- Parent reviews and approves/rejects
- Approved tasks add value to kid's ledger

**Ledger System**
- Each kid has a balance (how much they're owed)
- Transaction history (earned, redeemed, adjusted)
- Parent can make manual adjustments

**Single-User Start**
- Initially runs locally on parent's laptop
- Parent manages everything through web UI
- Kids tracked as data records (no login yet)

### Future Phases

**Phase 2: Multi-User Accounts**
- Kids can log in and see their own dashboard
- Self-service task completion
- Balance visibility

**Phase 3: Prerequisites & Unlocks**
- Some tasks require others to be completed first
- Creates incentive to do less-desirable essentials
- Example: "Complete homework" unlocks "Screen time request"

**Phase 4: Dynamic Bounties**
- Task values increase over time if not claimed
- Creates market dynamics and urgency
- Sibling competition for high-value tasks
- Example: "Take out trash" starts at $1, increases $0.25/day

**Phase 5: Rewards Redemption**
- Defined rewards with costs (e.g., "30 min screen time" = $2)
- Kids can redeem ledger balance for rewards
- Tracks cash payouts vs virtual redemptions

## Market Dynamics Philosophy

The bounty system creates interesting game mechanics:
- **Choice**: Kids decide what's worth doing at what price
- **Urgency**: Waiting too long means someone else claims it
- **Value discovery**: Kids learn what work is worth to them
- **Competition**: Siblings compete for high-value tasks
- **Natural incentives**: Important tasks get done without nagging

This teaches real-world economic concepts while getting chores done.

## Integration with AI Assistant (MCP)

A key feature is MCP server integration, allowing the parent to:
- Tell their AI assistant "Kid X finished chore Y"
- Query: "What tasks are pending approval?"
- Check: "How much do I owe each kid?"
- Create tasks via natural language

This reduces friction - parent can update the system without context switching.

## Success Criteria

**For Parents:**
- Less nagging, clearer expectations
- Easy to track what's done and what's owed
- Simple to adjust values and add new tasks
- Kids take more initiative

**For Kids:**
- Clear visibility into what's expected
- Sense of control over their earnings
- Tangible rewards feel earned
- Fun/game-like rather than punitive

## Uncertainties & Experiments

We don't know yet:
- What cadence feels right (daily refresh vs weekly)
- Whether bounties will cause chaos or motivation
- How to balance sibling competition vs cooperation
- What reward values feel appropriate
- Whether kids will engage with the system long-term

The phased approach lets us learn and iterate. Start simple, validate the core loop, then add complexity.

## Why Rust + Leptos + SurrealDB?

**Developer preference**: After working in TypeScript, Rust feels more productive
**Learning opportunity**: Experiment with Leptos (modern Rust web framework)
**Curiosity**: Try SurrealDB's embedded mode and graph features
**Full-stack Rust**: Type safety from database to UI
**Local-first**: Embedded database, no external dependencies

We're optimizing for developer joy and learning, not production scale. This is a family tool, not a SaaS product.
