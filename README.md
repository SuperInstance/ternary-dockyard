# ternary-dockyard: Maintenance and repair of ternary agents

Dry dock, diagnostics, refitting, and salvage for agents operating in {-1, 0, +1} state spaces.

## Why This Exists

Ternary agents degrade over time — states stagnate, health drops, components fail. Without a maintenance system, you'd need to throw away agents and start fresh every time something goes wrong. This crate provides the infrastructure to take agents offline, diagnose what's wrong, plan repairs, and even salvage useful parts from dead agents. Think of it as a shipyard for your ternary fleet.

## Core Concepts

- **Ternary**: A value in {-1, 0, +1}. Negative, Zero, or Positive.
- **Agent**: An entity with a ternary state vector, health score, and operational status.
- **Dockyard**: The central facility managing active and docked agents.
- **Dry Dock**: The process of taking an agent offline for repairs and returning it to service.
- **RepairBill**: A diagnostic report listing issues found in an agent, with severity and priority.
- **RefitPlan**: A plan to upgrade an agent's state vector (e.g., rebalancing an imbalanced agent).
- **MaintenanceSchedule**: Preventive maintenance intervals — agents get serviced before they break.
- **SalvageYard**: Recovers ternary components from decommissioned agents for reuse.

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-dockyard = "0.1"
```

```rust
use ternary_dockyard::*;

// Set up a dockyard
let mut dock = Dockyard::new();

// Register an agent with a ternary state vector
let id = dock.register(vec![Ternary::Positive, Ternary::Positive, Ternary::Positive]);
println!("Agent {} registered, active: {}", id, dock.active_count());

// Take it offline for maintenance
DryDock::dock(&mut dock, id, 100).unwrap();

// Diagnose issues
let agent = dock.get_agent(id).unwrap();
let bill = RepairBillDiagnostic::diagnose(agent);
println!("Found {} issues, priority {:?}", bill.issues.len(), bill.priority);

// Apply a refit plan to rebalance
let plan = RefitPlanner::plan_balance(agent);
RefitPlanner::apply(&mut dock, &plan).unwrap();

// Return to service
DryDock::release(&mut dock, id, 200).unwrap();
```

## API Overview

| Type | Description |
|------|-------------|
| `Dockyard` | Central facility managing active and docked agents |
| `Agent` | An entity with ternary state vector, health, and status |
| `DryDock` | Static API for docking/releasing agents |
| `RepairBill` | Diagnostic report with issues, effort estimate, priority |
| `RefitPlan` | Upgrade plan with specific state modifications |
| `MaintenanceSchedule` | Tracks preventive maintenance intervals per agent |
| `SalvageYard` | Recovers components from decommissioned agents |

## How It Works

The dockyard maintains two rosters: active agents (deployed) and docked agents (offline). `DryDock::dock()` moves an agent from active to docked, changing its status. `DryDock::release()` reverses this.

Diagnostics scan for five failure modes: low health, state stagnation (all identical values), oscillation (alternating -1/+1), excessive passivity (too many zeros), and imbalance. Each issue gets a severity (Low to Critical) and the aggregate produces a priority (Routine to Emergency).

Refit plans currently support rebalancing — converting excess positive or negative states to zero to achieve a more balanced ternary vector. The plan records each modification with a reason.

The salvage yard decommissions agents and recovers their state components with quality scores based on agent health and position in the state vector.

## Known Limitations

- Refit planning only supports rebalancing; custom refit strategies are not yet implemented.
- Diagnostics are rule-based, not statistical — they won't catch subtle degradation patterns.
- No persistence layer — all dockyard state is in-memory and lost on drop.
- The maintenance schedule is tick-based, not wall-clock-based.
- Salvage quality scoring is a simple heuristic; it doesn't track actual reuse success.
- No support for partial agent repair (all-or-nothing docking).

## Use Cases

- **Agent lifecycle management**: Track agents from deployment through maintenance to decommissioning.
- **Automated health monitoring**: Periodically diagnose agents and trigger repairs when health drops.
- **Preventive maintenance**: Schedule regular maintenance cycles to prevent agent degradation.
- **Component reuse**: Salvage ternary components from retired agents for new ones.
- **Fleet management**: Manage a pool of ternary agents with centralized status tracking.

## Ecosystem Context

Part of the SuperInstance ternary ecosystem. Works alongside `ternary-compass` for state navigation and `ternary-chronicle` for recording maintenance history. Agents maintained here can be deployed into systems managed by other ternary crates.

## License

MIT
