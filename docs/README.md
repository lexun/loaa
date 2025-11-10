# Loa'a Documentation

This directory contains comprehensive design documentation for the Loa'a project.

## Documentation Index

### Core Documents

**[vision.md](vision.md)** - Project vision and philosophy
- The problem we're solving
- Evolution from previous attempt
- Core concepts and features
- Market dynamics philosophy
- Success criteria and uncertainties

**[roadmap.md](roadmap.md)** - Development roadmap and phases
- Phase 1 (MVP): Single-user task tracking
- Phase 2: Multi-user with approval workflow
- Phase 3: Prerequisites and unlocks
- Phase 4: Dynamic bounties
- Phase 5: Rewards and redemption
- Timeline estimates and pivot points

**[architecture.md](architecture.md)** - Technical architecture
- Tech stack decisions (Leptos + SurrealDB)
- Workspace structure
- Data models
- Deployment strategy
- Security considerations

**[tech-stack.md](tech-stack.md)** - Tech stack decision summary
- Quick reference for chosen technologies
- Rationale and alternatives
- Future considerations

## Quick Reference

### Current Status

- **Phase**: Phase 1 (MVP)
- **Goal**: Single-user local application for parent
- **Timeline**: 2-3 weeks to usable MVP

### Tech Stack

- **Language**: Rust
- **Web Framework**: Leptos (full-stack SSR)
- **Database**: SurrealDB (embedded mode)
- **Architecture**: Workspace (core/web/mcp)
- **Deployment**: Local (parent's laptop)

### Key Principles

1. **Start simple** - Validate core mechanics before adding complexity
2. **Local-first** - No deployment, no external dependencies
3. **Phase-driven** - Build incrementally, learn and iterate
4. **Type-safe** - Rust everywhere for safety and productivity

## For AI Agents

When working on this project:

1. **Start with vision.md** - Understand the problem and goals
2. **Check roadmap.md** - Know what phase we're in
3. **Review architecture.md** - Understand technical decisions
4. **Follow AGENTS.md** - Project-specific development guidelines

See `../AGENTS.md` for complete development workflow.
