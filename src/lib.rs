#![forbid(unsafe_code)]

//! Maintenance and repair of ternary agents.
//!
//! Provides a dockyard metaphor for taking ternary agents offline,
//! diagnosing issues, planning repairs, scheduling preventive maintenance,
//! and salvaging components from decommissioned agents.

use std::collections::HashMap;

/// A ternary value: Negative (-1), Zero (0), or Positive (+1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ternary {
    Negative,
    Zero,
    Positive,
}

impl Ternary {
    pub fn value(self) -> i8 {
        match self {
            Ternary::Negative => -1,
            Ternary::Zero => 0,
            Ternary::Positive => 1,
        }
    }

    pub fn from_value(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Ternary::Negative),
            0 => Some(Ternary::Zero),
            1 => Some(Ternary::Positive),
            _ => None,
        }
    }
}

/// Operational status of an agent in the dockyard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Active,
    InDryDock,
    UnderRepair,
    Refitting,
    Decommissioned,
}

/// A unique agent identifier.
pub type AgentId = u64;

/// An agent with a ternary state vector.
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: AgentId,
    pub state: Vec<Ternary>,
    pub health: f64,
    pub status: AgentStatus,
    pub generation: u32,
}

impl Agent {
    pub fn new(id: AgentId, state: Vec<Ternary>) -> Self {
        let health = 1.0;
        Agent {
            id,
            state,
            health,
            status: AgentStatus::Active,
            generation: 1,
        }
    }

    pub fn with_health(mut self, health: f64) -> Self {
        self.health = health.clamp(0.0, 1.0);
        self
    }

    pub fn with_generation(mut self, gen: u32) -> Self {
        self.generation = gen;
        self
    }

    /// Check if the agent is operational (not in dockyard).
    pub fn is_operational(&self) -> bool {
        self.status == AgentStatus::Active
    }

    /// Compute a simple integrity score based on state balance.
    pub fn integrity(&self) -> f64 {
        if self.state.is_empty() {
            return self.health;
        }
        let sum: i64 = self.state.iter().map(|t| t.value() as i64).sum();
        let len = self.state.len() as f64;
        // Perfect balance = sum 0, max imbalance = len
        let imbalance = (sum.abs() as f64) / len;
        self.health * (1.0 - imbalance)
    }
}

// ─── Dockyard ────────────────────────────────────────────────────────

/// The main dockyard managing agent maintenance operations.
#[derive(Debug, Clone)]
pub struct Dockyard {
    /// Agents currently in the dockyard (offline).
    docked: HashMap<AgentId, Agent>,
    /// Agents currently active (online).
    active: HashMap<AgentId, Agent>,
    /// Maintenance log: agent_id -> list of (timestamp, description).
    log: HashMap<AgentId, Vec<(u64, String)>>,
    /// Next available agent ID.
    next_id: AgentId,
}

impl Dockyard {
    pub fn new() -> Self {
        Dockyard {
            docked: HashMap::new(),
            active: HashMap::new(),
            log: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register a new agent into active service.
    pub fn register(&mut self, state: Vec<Ternary>) -> AgentId {
        let id = self.next_id;
        self.next_id += 1;
        let agent = Agent::new(id, state);
        self.active.insert(id, agent);
        id
    }

    /// Register a pre-built agent.
    pub fn register_agent(&mut self, agent: Agent) -> AgentId {
        let id = agent.id;
        self.active.insert(id, agent);
        id
    }

    /// Get an agent by ID (docked or active).
    pub fn get_agent(&self, id: AgentId) -> Option<&Agent> {
        self.docked.get(&id).or_else(|| self.active.get(&id))
    }

    /// Get a mutable reference to an agent by ID.
    pub fn get_agent_mut(&mut self, id: AgentId) -> Option<&mut Agent> {
        if self.docked.contains_key(&id) {
            self.docked.get_mut(&id)
        } else {
            self.active.get_mut(&id)
        }
    }

    /// Number of agents in the dockyard.
    pub fn docked_count(&self) -> usize {
        self.docked.len()
    }

    /// Number of active agents.
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    /// Add a log entry for an agent.
    fn log_entry(&mut self, id: AgentId, timestamp: u64, desc: &str) {
        self.log
            .entry(id)
            .or_default()
            .push((timestamp, desc.to_string()));
    }

    /// Get maintenance log for an agent.
    pub fn get_log(&self, id: AgentId) -> &[Vec<(u64, String)>] {
        static EMPTY: Vec<(u64, String)> = Vec::new();
        match self.log.get(&id) {
            Some(v) => std::slice::from_ref(v),
            None => std::slice::from_ref(&EMPTY),
        }
    }
}

impl Default for Dockyard {
    fn default() -> Self {
        Self::new()
    }
}

// ─── DryDock ─────────────────────────────────────────────────────────

/// Takes agents offline for repairs and returns them to service.
pub struct DryDock;

impl DryDock {
    /// Take an agent offline into dry dock.
    pub fn dock(dockyard: &mut Dockyard, agent_id: AgentId, timestamp: u64) -> Result<(), &'static str> {
        let agent = dockyard
            .active
            .remove(&agent_id)
            .ok_or("Agent not found in active roster")?;
        let mut agent = agent;
        agent.status = AgentStatus::InDryDock;
        dockyard.docked.insert(agent_id, agent);
        dockyard.log_entry(agent_id, timestamp, "Entered dry dock");
        Ok(())
    }

    /// Release an agent from dry dock back to active service.
    pub fn release(dockyard: &mut Dockyard, agent_id: AgentId, timestamp: u64) -> Result<(), &'static str> {
        let agent = dockyard
            .docked
            .remove(&agent_id)
            .ok_or("Agent not found in dry dock")?;
        let mut agent = agent;
        agent.status = AgentStatus::Active;
        dockyard.active.insert(agent_id, agent);
        dockyard.log_entry(agent_id, timestamp, "Released from dry dock");
        Ok(())
    }
}

// ─── RepairBill ──────────────────────────────────────────────────────

/// Diagnosis of issues found in a ternary agent.
#[derive(Debug, Clone)]
pub struct RepairBill {
    pub agent_id: AgentId,
    pub issues: Vec<Issue>,
    pub estimated_effort: f64,
    pub priority: Priority,
}

/// A specific issue found in an agent.
#[derive(Debug, Clone)]
pub struct Issue {
    pub description: String,
    pub severity: Severity,
    pub affected_indices: Vec<usize>,
}

/// Issue severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Repair priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Routine,
    Scheduled,
    Urgent,
    Emergency,
}

/// Diagnoses issues in ternary agents.
pub struct RepairBillDiagnostic;

impl RepairBillDiagnostic {
    /// Diagnose an agent and produce a repair bill.
    pub fn diagnose(agent: &Agent) -> RepairBill {
        let mut issues = Vec::new();
        let mut critical_count = 0usize;
        let mut high_count = 0usize;

        // Check health
        if agent.health < 0.2 {
            issues.push(Issue {
                description: "Agent health critically low".to_string(),
                severity: Severity::Critical,
                affected_indices: Vec::new(),
            });
            critical_count += 1;
        } else if agent.health < 0.5 {
            issues.push(Issue {
                description: "Agent health degraded".to_string(),
                severity: Severity::High,
                affected_indices: Vec::new(),
            });
            high_count += 1;
        } else if agent.health < 0.8 {
            issues.push(Issue {
                description: "Agent health below optimal".to_string(),
                severity: Severity::Medium,
                affected_indices: Vec::new(),
            });
        }

        // Check for state stuck at same value (stagnation)
        if agent.state.len() > 2 {
            let all_same = agent.state.windows(2).all(|w| w[0] == w[1]);
            if all_same {
                let indices: Vec<usize> = (0..agent.state.len()).collect();
                issues.push(Issue {
                    description: format!("State stagnation: all {} values identical", agent.state.len()),
                    severity: Severity::High,
                    affected_indices: indices,
                });
                high_count += 1;
            }
        }

        // Check for oscillation (alternating between -1 and +1)
        if agent.state.len() > 2 {
            let oscillating = agent.state.windows(2).all(|w| {
                (w[0] == Ternary::Negative && w[1] == Ternary::Positive)
                    || (w[0] == Ternary::Positive && w[1] == Ternary::Negative)
            });
            if oscillating {
                let indices: Vec<usize> = (0..agent.state.len()).collect();
                issues.push(Issue {
                    description: "State oscillation detected".to_string(),
                    severity: Severity::Medium,
                    affected_indices: indices,
                });
            }
        }

        // Check for excessive zeros (passivity)
        let zero_count = agent.state.iter().filter(|t| **t == Ternary::Zero).count();
        let zero_ratio = zero_count as f64 / agent.state.len().max(1) as f64;
        if zero_ratio > 0.8 && agent.state.len() > 3 {
            issues.push(Issue {
                description: format!("Excessive passivity: {:.0}% zero states", zero_ratio * 100.0),
                severity: Severity::Low,
                affected_indices: (0..agent.state.len())
                    .filter(|&i| agent.state[i] == Ternary::Zero)
                    .collect(),
            });
        }

        // Compute effort and priority
        let estimated_effort = issues.iter().map(|i| match i.severity {
            Severity::Low => 1.0,
            Severity::Medium => 3.0,
            Severity::High => 5.0,
            Severity::Critical => 10.0,
        }).sum();

        let priority = if critical_count > 0 {
            Priority::Emergency
        } else if high_count > 1 {
            Priority::Urgent
        } else if high_count > 0 {
            Priority::Scheduled
        } else {
            Priority::Routine
        };

        RepairBill {
            agent_id: agent.id,
            issues,
            estimated_effort,
            priority,
        }
    }
}

// ─── RefitPlan ───────────────────────────────────────────────────────

/// A plan to upgrade agent capabilities.
#[derive(Debug, Clone)]
pub struct RefitPlan {
    pub agent_id: AgentId,
    pub new_state: Vec<Ternary>,
    pub target_generation: u32,
    pub modifications: Vec<Modification>,
}

/// A single modification in a refit plan.
#[derive(Debug, Clone)]
pub struct Modification {
    pub index: usize,
    pub from: Ternary,
    pub to: Ternary,
    pub reason: String,
}

/// Creates refit plans for upgrading agents.
pub struct RefitPlanner;

impl RefitPlanner {
    /// Create a refit plan that balances an agent's state.
    pub fn plan_balance(agent: &Agent) -> RefitPlan {
        let neg_count = agent.state.iter().filter(|t| **t == Ternary::Negative).count();
        let pos_count = agent.state.iter().filter(|t| **t == Ternary::Positive).count();
        let zero_count = agent.state.iter().filter(|t| **t == Ternary::Zero).count();

        let mut modifications = Vec::new();
        let mut new_state = agent.state.clone();

        // If too many positives, convert some to zero
        if pos_count > neg_count + zero_count {
            let excess = pos_count - neg_count - zero_count;
            let mut converted = 0;
            for (i, t) in agent.state.iter().enumerate() {
                if converted >= excess { break; }
                if *t == Ternary::Positive && (i % 2 == 0) {
                    modifications.push(Modification {
                        index: i,
                        from: *t,
                        to: Ternary::Zero,
                        reason: "Rebalance excess positive".to_string(),
                    });
                    new_state[i] = Ternary::Zero;
                    converted += 1;
                }
            }
        }

        // If too many negatives, convert some to zero
        if neg_count > pos_count + zero_count {
            let excess = neg_count - pos_count - zero_count;
            let mut converted = 0;
            for (i, t) in agent.state.iter().enumerate() {
                if converted >= excess { break; }
                if *t == Ternary::Negative && (i % 2 == 0) {
                    modifications.push(Modification {
                        index: i,
                        from: *t,
                        to: Ternary::Zero,
                        reason: "Rebalance excess negative".to_string(),
                    });
                    new_state[i] = Ternary::Zero;
                    converted += 1;
                }
            }
        }

        RefitPlan {
            agent_id: agent.id,
            new_state,
            target_generation: agent.generation + 1,
            modifications,
        }
    }

    /// Apply a refit plan to an agent in the dockyard.
    pub fn apply(dockyard: &mut Dockyard, plan: &RefitPlan) -> Result<(), &'static str> {
        let agent = dockyard
            .docked
            .get_mut(&plan.agent_id)
            .ok_or("Agent must be in dry dock to refit")?;
        agent.state = plan.new_state.clone();
        agent.generation = plan.target_generation;
        agent.status = AgentStatus::Refitting;
        Ok(())
    }
}

// ─── MaintenanceSchedule ─────────────────────────────────────────────

/// A preventive maintenance schedule for agents.
#[derive(Debug, Clone)]
pub struct MaintenanceSchedule {
    /// agent_id -> interval in ticks between maintenance.
    intervals: HashMap<AgentId, u64>,
    /// agent_id -> last maintenance tick.
    last_maintenance: HashMap<AgentId, u64>,
    /// Default interval for agents without specific schedules.
    default_interval: u64,
}

impl MaintenanceSchedule {
    pub fn new(default_interval: u64) -> Self {
        MaintenanceSchedule {
            intervals: HashMap::new(),
            last_maintenance: HashMap::new(),
            default_interval,
        }
    }

    /// Set a specific maintenance interval for an agent.
    pub fn set_interval(&mut self, agent_id: AgentId, interval: u64) {
        self.intervals.insert(agent_id, interval.max(1));
    }

    /// Record that maintenance was performed on an agent.
    pub fn record_maintenance(&mut self, agent_id: AgentId, tick: u64) {
        self.last_maintenance.insert(agent_id, tick);
    }

    /// Check if an agent is due for maintenance.
    pub fn is_due(&self, agent_id: AgentId, current_tick: u64) -> bool {
        let interval = self.intervals.get(&agent_id).copied().unwrap_or(self.default_interval);
        let last = self.last_maintenance.get(&agent_id).copied().unwrap_or(0);
        current_tick >= last + interval
    }

    /// Get all agents due for maintenance at the given tick.
    pub fn due_agents(&self, current_tick: u64) -> Vec<AgentId> {
        let mut due = Vec::new();
        for &agent_id in self.intervals.keys() {
            if self.is_due(agent_id, current_tick) {
                due.push(agent_id);
            }
        }
        due
    }

    /// Get the default maintenance interval.
    pub fn default_interval(&self) -> u64 {
        self.default_interval
    }
}

// ─── SalvageYard ─────────────────────────────────────────────────────

/// A recovered component from a decommissioned agent.
#[derive(Debug, Clone)]
pub struct SalvagedPart {
    pub source_id: AgentId,
    pub index: usize,
    pub value: Ternary,
    pub quality: f64,
}

/// Recovers usable components from decommissioned agents.
pub struct SalvageYard {
    parts: Vec<SalvagedPart>,
    total_recovered: usize,
}

impl SalvageYard {
    pub fn new() -> Self {
        SalvageYard {
            parts: Vec::new(),
            total_recovered: 0,
        }
    }

    /// Decommission an agent and salvage its components.
    pub fn salvage(&mut self, agent: &Agent) -> Vec<SalvagedPart> {
        let mut recovered = Vec::new();
        for (i, &val) in agent.state.iter().enumerate() {
            // Quality is based on agent health and position
            let quality = agent.health * (1.0 - (i as f64 / agent.state.len().max(1) as f64) * 0.3);
            let part = SalvagedPart {
                source_id: agent.id,
                index: i,
                value: val,
                quality,
            };
            recovered.push(part.clone());
        }
        self.total_recovered += recovered.len();
        self.parts.extend(recovered.clone());
        recovered
    }

    /// Find parts matching a specific ternary value, sorted by quality.
    pub fn find_parts(&self, value: Ternary) -> Vec<&SalvagedPart> {
        let mut matching: Vec<_> = self.parts.iter().filter(|p| p.value == value).collect();
        matching.sort_by(|a, b| b.quality.partial_cmp(&a.quality).unwrap_or(std::cmp::Ordering::Equal));
        matching
    }

    /// Total number of parts in the yard.
    pub fn total_parts(&self) -> usize {
        self.parts.len()
    }

    /// Total number of parts ever recovered.
    pub fn total_recovered(&self) -> usize {
        self.total_recovered
    }

    /// Remove and return the best part matching a value.
    pub fn take_best(&mut self, value: Ternary) -> Option<SalvagedPart> {
        let best_idx = self.parts.iter().enumerate()
            .filter(|(_, p)| p.value == value)
            .max_by(|(_, a), (_, b)| a.quality.partial_cmp(&b.quality).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)?;
        Some(self.parts.remove(best_idx))
    }
}

impl Default for SalvageYard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_agent(id: AgentId, state: Vec<Ternary>) -> Agent {
        Agent::new(id, state)
    }

    fn t() -> Vec<Ternary> {
        vec![Ternary::Positive, Ternary::Negative, Ternary::Zero]
    }

    #[test]
    fn test_ternary_values() {
        assert_eq!(Ternary::Negative.value(), -1);
        assert_eq!(Ternary::Zero.value(), 0);
        assert_eq!(Ternary::Positive.value(), 1);
    }

    #[test]
    fn test_ternary_from_value() {
        assert_eq!(Ternary::from_value(-1), Some(Ternary::Negative));
        assert_eq!(Ternary::from_value(0), Some(Ternary::Zero));
        assert_eq!(Ternary::from_value(1), Some(Ternary::Positive));
        assert_eq!(Ternary::from_value(2), None);
    }

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new(1, t());
        assert_eq!(agent.id, 1);
        assert_eq!(agent.state.len(), 3);
        assert_eq!(agent.health, 1.0);
        assert!(agent.is_operational());
    }

    #[test]
    fn test_agent_health_clamp() {
        let agent = Agent::new(1, t()).with_health(1.5);
        assert!((agent.health - 1.0).abs() < f64::EPSILON);
        let agent = Agent::new(2, t()).with_health(-0.5);
        assert!((agent.health - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_agent_integrity_balanced() {
        let agent = Agent::new(1, vec![Ternary::Positive, Ternary::Negative]);
        let integrity = agent.integrity();
        assert!((integrity - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_agent_integrity_imbalanced() {
        let agent = Agent::new(1, vec![Ternary::Positive, Ternary::Positive]);
        let integrity = agent.integrity();
        assert!(integrity < 1.0);
    }

    #[test]
    fn test_dockyard_register() {
        let mut dock = Dockyard::new();
        let id = dock.register(t());
        assert_eq!(id, 1);
        assert_eq!(dock.active_count(), 1);
        assert_eq!(dock.docked_count(), 0);
    }

    #[test]
    fn test_dockyard_register_multiple() {
        let mut dock = Dockyard::new();
        dock.register(t());
        dock.register(t());
        assert_eq!(dock.active_count(), 2);
    }

    #[test]
    fn test_drydock_dock_and_release() {
        let mut dock = Dockyard::new();
        let id = dock.register(t());
        assert!(DryDock::dock(&mut dock, id, 10).is_ok());
        assert_eq!(dock.docked_count(), 1);
        assert_eq!(dock.active_count(), 0);
        assert!(DryDock::release(&mut dock, id, 20).is_ok());
        assert_eq!(dock.active_count(), 1);
        assert_eq!(dock.docked_count(), 0);
    }

    #[test]
    fn test_drydock_dock_nonexistent() {
        let mut dock = Dockyard::new();
        assert!(DryDock::dock(&mut dock, 999, 10).is_err());
    }

    #[test]
    fn test_drydock_release_not_docked() {
        let mut dock = Dockyard::new();
        let id = dock.register(t());
        assert!(DryDock::release(&mut dock, id, 10).is_err());
    }

    #[test]
    fn test_diagnose_healthy_agent() {
        let agent = Agent::new(1, t());
        let bill = RepairBillDiagnostic::diagnose(&agent);
        assert!(bill.issues.is_empty());
        assert_eq!(bill.priority, Priority::Routine);
    }

    #[test]
    fn test_diagnose_low_health() {
        let agent = Agent::new(1, t()).with_health(0.1);
        let bill = RepairBillDiagnostic::diagnose(&agent);
        assert!(!bill.issues.is_empty());
        assert_eq!(bill.priority, Priority::Emergency);
        assert!(bill.issues.iter().any(|i| i.severity == Severity::Critical));
    }

    #[test]
    fn test_diagnose_stagnation() {
        let agent = Agent::new(1, vec![Ternary::Positive; 5]);
        let bill = RepairBillDiagnostic::diagnose(&agent);
        assert!(bill.issues.iter().any(|i| i.description.contains("stagnation")));
    }

    #[test]
    fn test_diagnose_oscillation() {
        let agent = Agent::new(1, vec![Ternary::Negative, Ternary::Positive, Ternary::Negative, Ternary::Positive]);
        let bill = RepairBillDiagnostic::diagnose(&agent);
        assert!(bill.issues.iter().any(|i| i.description.contains("oscillation")));
    }

    #[test]
    fn test_diagnose_passivity() {
        let agent = Agent::new(1, vec![Ternary::Zero; 10]);
        let bill = RepairBillDiagnostic::diagnose(&agent);
        assert!(bill.issues.iter().any(|i| i.description.contains("passivity")));
    }

    #[test]
    fn test_refit_plan_balance() {
        let agent = Agent::new(1, vec![Ternary::Positive; 6]);
        let plan = RefitPlanner::plan_balance(&agent);
        assert!(!plan.modifications.is_empty());
        assert_eq!(plan.target_generation, 2);
    }

    #[test]
    fn test_refit_apply() {
        let mut dock = Dockyard::new();
        let agent = Agent::new(1, vec![Ternary::Positive; 6]);
        dock.register_agent(agent);
        DryDock::dock(&mut dock, 1, 10).unwrap();
        let plan = RefitPlanner::plan_balance(dock.get_agent(1).unwrap());
        assert!(RefitPlanner::apply(&mut dock, &plan).is_ok());
        let agent = dock.get_agent(1).unwrap();
        assert_eq!(agent.status, AgentStatus::Refitting);
    }

    #[test]
    fn test_refit_apply_not_docked() {
        let mut dock = Dockyard::new();
        dock.register(vec![Ternary::Positive; 4]);
        let agent = dock.get_agent(1).unwrap().clone();
        let plan = RefitPlanner::plan_balance(&agent);
        assert!(RefitPlanner::apply(&mut dock, &plan).is_err());
    }

    #[test]
    fn test_maintenance_schedule() {
        let mut sched = MaintenanceSchedule::new(100);
        sched.set_interval(1, 50);
        sched.record_maintenance(1, 0);
        assert!(!sched.is_due(1, 49));
        assert!(sched.is_due(1, 50));
    }

    #[test]
    fn test_maintenance_schedule_default_interval() {
        let sched = MaintenanceSchedule::new(100);
        assert!(sched.is_due(99, 100));
    }

    #[test]
    fn test_salvage_yard() {
        let mut yard = SalvageYard::new();
        let agent = Agent::new(1, t());
        let parts = yard.salvage(&agent);
        assert_eq!(parts.len(), 3);
        assert_eq!(yard.total_parts(), 3);
        assert_eq!(yard.total_recovered(), 3);
    }

    #[test]
    fn test_salvage_find_parts() {
        let mut yard = SalvageYard::new();
        let agent = Agent::new(1, vec![Ternary::Positive, Ternary::Positive, Ternary::Negative]);
        yard.salvage(&agent);
        let pos_parts = yard.find_parts(Ternary::Positive);
        assert_eq!(pos_parts.len(), 2);
    }

    #[test]
    fn test_salvage_take_best() {
        let mut yard = SalvageYard::new();
        let agent = Agent::new(1, t());
        yard.salvage(&agent);
        let best = yard.take_best(Ternary::Positive);
        assert!(best.is_some());
        assert_eq!(yard.total_parts(), 2);
    }

    #[test]
    fn test_salvage_take_best_empty() {
        let mut yard = SalvageYard::new();
        assert!(yard.take_best(Ternary::Positive).is_none());
    }

    #[test]
    fn test_maintenance_due_agents() {
        let mut sched = MaintenanceSchedule::new(10);
        sched.set_interval(1, 5);
        sched.set_interval(2, 20);
        sched.record_maintenance(1, 0);
        sched.record_maintenance(2, 0);
        let due = sched.due_agents(5);
        assert_eq!(due, vec![1]);
    }
}
