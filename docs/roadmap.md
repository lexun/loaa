# Loa'a Development Roadmap

## Guiding Principles

1. **Start simple, add complexity gradually** - Validate core mechanics before adding features
2. **Local-first** - Run on parent's laptop, no deployment complexity early on
3. **Working software beats perfect design** - Get it usable quickly, iterate based on real usage
4. **Learn as we go** - This is partly experimentation with new tech (Leptos, SurrealDB)

## Phase 1: MVP - Single User Task Tracking (2-3 weeks)

**Goal**: Parent can define tasks, track completions, and maintain ledgers for each kid

### Scope

**Core Features:**
- Task definitions (name, description, value, cadence)
- Task completion workflow (creates ledger entries, resets recurring tasks)
- Ledger per kid showing balance owed
- Transaction history (earned, adjusted)
- **Read-only web UI** for visibility (dashboard view)
- **MCP server** for all mutations via Claude Code/AI assistant

**What's NOT included:**
- Kids cannot log in (no authentication yet)
- No prerequisites or dependencies
- No dynamic bounties
- No rewards redemption
- No screen time tracking
- No approval workflow (parent just marks things done)

### Success Criteria

- [x] Core data models implemented (Task, Kid, LedgerEntry)
- [x] Task CRUD operations working
- [x] Task completion workflow implemented
- [ ] MCP server with tools for task management
- [ ] Read-only web UI showing tasks, kids, balances
- [ ] Parent can use Claude Code to manage everything via natural language
- [ ] Data persists between restarts
- [ ] System runs locally without external dependencies

### Technical Milestones

1. **Foundation** ✅
   - SurrealDB embedded mode connected
   - Core data models defined (Task, Kid, LedgerEntry)
   - Repository layer with proper ID serialization

2. **Business Logic** ✅
   - Task CRUD operations
   - Kid CRUD operations
   - Ledger entry creation
   - Task completion workflow (with reset logic)

3. **MCP Server** 🔄
   - Implement using `rmcp` (official Rust SDK)
   - Expose tools: create/list/update tasks, create kids, complete tasks, get balances
   - Test with Claude Code integration

4. **Read-Only Web UI** 🔄
   - Leptos web app with SSR
   - Dashboard showing all tasks
   - Kid balance cards
   - Recent transaction history
   - Auto-refresh when data changes

5. **Polish**
   - Basic UI styling
   - Error handling
   - Data persistence validation

### Why This Scope?

Starting with single-user proves:
- The core loop works (task → completion → ledger)
- The tech stack works together (Leptos + SurrealDB)
- The concept resonates with the family
- Data model is sound

We can do all of this without dealing with authentication, permissions, or multi-user complexity.

### Uncertainties to Validate

- Does tracking completions this way reduce nagging?
- Are the monetary values motivating?
- How often do tasks need to reset?
- Is the UI intuitive enough?

## Phase 2: Multi-User & Approval Workflow (1-2 weeks)

**Goal**: Kids can log in, see their tasks, mark things complete; parent approves

### Scope

**New Features:**
- User accounts (parent role + kid role)
- Authentication (username/password)
- Kids can view their own task list
- Kids can mark tasks complete (status: pending)
- Parent sees pending approvals
- Parent approves/rejects completions
- Each kid sees only their own ledger

**Architecture Changes:**
- Session management
- Role-based permissions
- Completion workflow state machine

### Success Criteria

- [ ] Kids can log in and see their dashboard
- [ ] Kids can mark tasks complete
- [ ] Parent sees all pending approvals
- [ ] Parent can approve → money added to ledger
- [ ] Parent can reject → task returns to available
- [ ] Kids can only see their own data

### Why Now?

By this point we've validated:
- The basic mechanics work
- The family uses it
- The tech stack is stable

Adding multi-user makes it self-service for kids, which is where the real magic happens - they gain agency and the parent gets less friction.

## Phase 3: Prerequisites & Unlocks (1 week)

**Goal**: Certain tasks require others to be completed first

### Scope

**New Features:**
- Tasks can have prerequisites (must do X before Y unlocks)
- UI shows locked vs available tasks
- Visual indication of what's blocking what

**Example Use Cases:**
- "Complete homework" unlocks "Request screen time"
- "Clean room" unlocks "Have friend over"
- Ensures essentials get done before rewards

### Success Criteria

- [ ] Can define prerequisites when creating tasks
- [ ] Locked tasks show in UI but can't be claimed
- [ ] Completing prerequisite unlocks dependent tasks
- [ ] Clear visual feedback about why something is locked

### Technical Considerations

- Graph relationships in SurrealDB (if we're still using it)
- Cycle detection (can't have circular dependencies)
- Caching/performance if dependency chains get deep

## Phase 4: Dynamic Bounties (1 week)

**Goal**: Task values increase over time to create urgency and competition

### Scope

**New Features:**
- Tasks can have bounty rules (start value, increment, cap)
- Background job increases bounties on schedule
- UI shows current bounty and next increase time
- Sibling competition for high-value tasks

**Example:**
- "Take out trash" starts at $1
- Increases $0.25 every day not done
- Caps at $3
- Creates urgency and market dynamics

### Success Criteria

- [ ] Can configure bounty rules per task
- [ ] Bounties increase on schedule
- [ ] UI shows bounty progression
- [ ] First kid to complete claims the current bounty

### Uncertainties

- Will this cause chaos or healthy competition?
- What increment amounts feel right?
- Do we need cooldowns between claims?
- Should bounties reset after someone claims them?

We'll need to experiment and tune based on real usage.

## Phase 5: Rewards & Redemption (1-2 weeks)

**Goal**: Kids can spend their balance on defined rewards

### Scope

**New Features:**
- Reward definitions (name, cost)
- Kids can redeem balance for rewards
- Redemption creates debit transaction
- Track cash payouts vs virtual rewards

**Reward Types:**
- Screen time (30 min gaming = $2)
- Physical cash (payout from ledger)
- Special privileges (stay up late = $5)

### Success Criteria

- [ ] Parent can define rewards with costs
- [ ] Kids can see available rewards
- [ ] Kids can redeem if balance sufficient
- [ ] Ledger debits correctly
- [ ] Parent gets notification of redemptions
- [ ] Track cash owed separately

## Phase 6: Voice-First UI with Claude API Integration (1-2 weeks)

**Goal**: In-app voice interface for hands-free management

### Scope

**New Features:**
- Microphone button in web UI
- Speech-to-text capture
- Direct Claude API integration from backend
- MCP client implementation (handle tool calling protocol)
- Voice confirmation and feedback
- Error handling for speech recognition

### Technical Architecture

```
User speaks → Web UI (mic button)
                  ↓
            Speech-to-Text
                  ↓
          Backend Server (Rust)
                  ↓
    Claude API (with tool definitions)
                  ↓
    MCP Client Protocol Handler
                  ↓
         Local MCP Server
                  ↓
         Execute actions
                  ↓
    Update UI with confirmation
```

### Success Criteria

- [ ] Microphone button captures audio
- [ ] Speech-to-text conversion works reliably
- [ ] Claude API integration with tool calling
- [ ] MCP client protocol implemented in Rust
- [ ] Voice commands execute correct actions
- [ ] UI shows confirmation of what was done
- [ ] Error states handled gracefully

### Why Later?

This is deferred because:
- Phase 1 MCP server works with Claude Code today (zero extra work)
- Voice UI requires implementing MCP client protocol ourselves
- Want to validate the core concept first
- Can use Claude Code as interim solution

Once we validate the workflow with Claude Code, this provides a better UX by eliminating the need for a separate app.

## Future Possibilities (Backlog)

**Ideas we might explore:**

- **Google Home integration** - Voice commands via smart speaker
- **Recurring tasks** - Auto-create daily/weekly tasks
- **Chore templates** - Library of common tasks
- **Reports** - Weekly earnings summary
- **Sibling accounts** - Transfer money between kids
- **Gamification** - Badges, streaks, achievements
- **Mobile app** - Native iOS/Android
- **Notifications** - Remind kids of high-value bounties
- **Parent controls** - Spending limits, reward approval
- **Integration** - Screen time enforcement on devices
- **Export** - CSV of all transactions for taxes/records

## Timeline Estimates

| Phase | Duration | Cumulative |
|-------|----------|------------|
| Phase 1: MVP (MCP + Read-Only UI) | 2-3 weeks | 3 weeks |
| Phase 2: Multi-user | 1-2 weeks | 5 weeks |
| Phase 3: Prerequisites | 1 week | 6 weeks |
| Phase 4: Bounties | 1 week | 7 weeks |
| Phase 5: Rewards | 1-2 weeks | 9 weeks |
| Phase 6: Voice UI | 1-2 weeks | 11 weeks |

**Realistic timeline: ~2.5 months to full feature set with voice UI**

But Phase 1 gets us something usable in 2-3 weeks.

## Pivot Points

Places where we might change direction based on learnings:

**After Phase 1:**
- Is the tech stack working? Or should we swap SurrealDB for SQLite?
- Is Leptos productive? Or should we simplify to Axum + HTMX?
- Do the kids engage? Or is this solving the wrong problem?

**After Phase 2:**
- Does the approval workflow feel good? Or is it too much friction?
- Are kids using it? Or do they forget?

**After Phase 3:**
- Do prerequisites help? Or do they cause more frustration?

**After Phase 4:**
- Are bounties motivating? Or causing chaos?
- Do siblings compete or resent each other?

The phased approach gives us natural checkpoints to evaluate and adjust.

## Success Metrics

We'll know this is working if:

**Qualitative:**
- Less parental nagging
- Kids take more initiative
- Fewer arguments about chores
- Kids understand value of work

**Quantitative:**
- Tasks completed per week increases
- Parent approval time decreases
- Kids check the system daily
- Balance redemptions are regular

## Development Notes

- Work on one phase at a time
- Each phase should be mergeable to main
- Write docs as we go (capture learnings)
- Update beads issues when scope changes
- Commit frequently with clear messages
- No feature branches until multi-user (Phase 2)

## Questions & Uncertainties

- How much money per week is sustainable? ($50? $100?)
- What task values feel right?
- Daily vs weekly task reset?
- Should older kids get higher rates?
- How to handle task quality (not just completion)?
- What if kids game the system?

We'll discover answers through iteration and real-world use.
