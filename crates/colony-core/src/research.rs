use bevy::prelude::*;
use serde::{Serialize, Deserialize};
// HashMap import removed - not used in this file

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechNode {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub cost_pts: u32,
    pub requires: Vec<String>,
    pub grants: Vec<TechGrant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TechGrant {
    Tunable { key: String, mult: f32 },                // e.g., "thermal_min_throttle", 1.05
    UnlockOp { op: String },                           // e.g., "Adjudicator"
    UnlockRitual { ritual_id: String },
    SchedulerBias { policy: String, weight: f32 },     // EDF tweak
    Sensor { metric: String },                         // makes metric visible; required for dispelling illusions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RitualDef {
    pub id: String,
    pub name: String,
    pub time_ms: u64,
    pub parts: u32,            // consume inventory
    pub effects: Vec<String>,  // e.g., "clear:DebtPowerMult", "reduce:corruption=0.1", "reimage:domain=1"
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct ResearchState {
    pub pts: u32,
    pub acquired: Vec<String>,       // tech ids
    pub rituals: Vec<RitualDef>,     // loaded & unlocked
}

impl ResearchState {
    pub fn new() -> Self {
        Self {
            pts: 0,
            acquired: Vec::new(),
            rituals: Vec::new(),
        }
    }

    pub fn can_afford(&self, cost: u32) -> bool {
        self.pts >= cost
    }

    pub fn has_tech(&self, tech_id: &str) -> bool {
        self.acquired.contains(&tech_id.to_string())
    }

    pub fn can_research(&self, tech: &TechNode) -> bool {
        if self.has_tech(&tech.id) {
            return false;
        }
        
        if !self.can_afford(tech.cost_pts) {
            return false;
        }

        // Check prerequisites
        for req in &tech.requires {
            if !self.has_tech(req) {
                return false;
            }
        }

        true
    }

    pub fn research_tech(&mut self, tech: &TechNode) -> bool {
        if !self.can_research(tech) {
            return false;
        }

        self.pts -= tech.cost_pts;
        self.acquired.push(tech.id.clone());

        // Add unlocked rituals
        for grant in &tech.grants {
            if let TechGrant::UnlockRitual { ritual_id } = grant {
                // TODO: Load ritual definition from content
                let ritual = RitualDef {
                    id: ritual_id.clone(),
                    name: format!("Ritual: {}", ritual_id),
                    time_ms: 30000, // 30 seconds default
                    parts: 1,
                    effects: vec!["clear:DebtPowerMult".to_string()],
                };
                self.rituals.push(ritual);
            }
        }

        true
    }

    pub fn get_available_rituals(&self) -> Vec<&RitualDef> {
        self.rituals.iter().collect()
    }
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct TechTree {
    pub nodes: Vec<TechNode>,
}

impl TechTree {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }

    pub fn add_tech(&mut self, tech: TechNode) {
        self.nodes.push(tech);
    }

    pub fn get_tech(&self, id: &str) -> Option<&TechNode> {
        self.nodes.iter().find(|tech| tech.id == id)
    }

    pub fn get_available_techs(&self, research_state: &ResearchState) -> Vec<&TechNode> {
        self.nodes
            .iter()
            .filter(|tech| research_state.can_research(tech))
            .collect()
    }

    pub fn get_researched_techs(&self, research_state: &ResearchState) -> Vec<&TechNode> {
        self.nodes
            .iter()
            .filter(|tech| research_state.has_tech(&tech.id))
            .collect()
    }
}

// Default tech tree
pub fn create_default_tech_tree() -> TechTree {
    let mut tech_tree = TechTree::new();

    // Truth Beacon - reveals real metrics
    tech_tree.add_tech(TechNode {
        id: "truth_beacon".to_string(),
        name: "Truth Beacon".to_string(),
        desc: "Reveals real metrics, dispelling UI illusions".to_string(),
        cost_pts: 10,
        requires: vec![],
        grants: vec![
            TechGrant::Sensor { metric: "bandwidth_util".to_string() },
            TechGrant::Sensor { metric: "corruption_field".to_string() },
            TechGrant::Sensor { metric: "power_draw".to_string() },
        ],
    });

    // Dual-Run Adjudicator - enables dual-run mutations
    tech_tree.add_tech(TechNode {
        id: "dual_run_adjudicator".to_string(),
        name: "Dual-Run Adjudicator".to_string(),
        desc: "Enables dual-run pipeline mutations for fault tolerance".to_string(),
        cost_pts: 15,
        requires: vec!["truth_beacon".to_string()],
        grants: vec![
            TechGrant::UnlockOp { op: "Adjudicator".to_string() },
        ],
    });

    // NUMA Isolation - improves thermal management
    tech_tree.add_tech(TechNode {
        id: "numa_isolation".to_string(),
        name: "NUMA Isolation".to_string(),
        desc: "Improves thermal management and reduces bandwidth tax".to_string(),
        cost_pts: 20,
        requires: vec![],
        grants: vec![
            TechGrant::Tunable { key: "thermal_min_throttle".to_string(), mult: 1.05 },
            TechGrant::Tunable { key: "bandwidth_tail_exp".to_string(), mult: 0.95 },
        ],
    });

    // ECC Scrub - enables memory error correction
    tech_tree.add_tech(TechNode {
        id: "ecc_scrub".to_string(),
        name: "ECC Scrub".to_string(),
        desc: "Enables memory error correction and VRAM maintenance".to_string(),
        cost_pts: 25,
        requires: vec!["truth_beacon".to_string()],
        grants: vec![
            TechGrant::UnlockRitual { ritual_id: "ecc_scrub".to_string() },
        ],
    });

    // PCIe Lanes - improves bandwidth
    tech_tree.add_tech(TechNode {
        id: "pcie_lanes".to_string(),
        name: "PCIe Lanes".to_string(),
        desc: "Increases PCIe bandwidth and reduces latency".to_string(),
        cost_pts: 18,
        requires: vec!["numa_isolation".to_string()],
        grants: vec![
            TechGrant::Tunable { key: "bandwidth_total_gbps".to_string(), mult: 1.2 },
        ],
    });

    // VRAM Pager - reduces VRAM pressure
    tech_tree.add_tech(TechNode {
        id: "vram_pager".to_string(),
        name: "VRAM Pager".to_string(),
        desc: "Reduces VRAM pressure and StickyConfig fault bias".to_string(),
        cost_pts: 22,
        requires: vec!["ecc_scrub".to_string()],
        grants: vec![
            TechGrant::Tunable { key: "vram_gb".to_string(), mult: 1.15 },
            TechGrant::Tunable { key: "sticky_fault_bias".to_string(), mult: 0.8 },
        ],
    });

    tech_tree
}

pub fn research_progress_system(
    mut research_state: ResMut<ResearchState>,
    // TODO: Add event reader for research requests
) {
    // This system will handle research progress and tech unlocks
    // For now, it's a placeholder
}

pub fn apply_tech_grants(
    research_state: &ResearchState,
    tech_tree: &TechTree,
    // TODO: Add resources to modify based on tech grants
) {
    for tech_id in &research_state.acquired {
        if let Some(tech) = tech_tree.get_tech(tech_id) {
            for grant in &tech.grants {
                match grant {
                    TechGrant::Tunable { key, mult } => {
                        // TODO: Apply tunable modifications
                        println!("Applied tech grant: {} * {}", key, mult);
                    }
                    TechGrant::UnlockOp { op } => {
                        // TODO: Unlock operation
                        println!("Unlocked operation: {}", op);
                    }
                    TechGrant::UnlockRitual { ritual_id } => {
                        // TODO: Unlock ritual
                        println!("Unlocked ritual: {}", ritual_id);
                    }
                    TechGrant::SchedulerBias { policy, weight } => {
                        // TODO: Apply scheduler bias
                        println!("Applied scheduler bias: {} * {}", policy, weight);
                    }
                    TechGrant::Sensor { metric } => {
                        // TODO: Enable sensor
                        println!("Enabled sensor: {}", metric);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_state() {
        let mut state = ResearchState::new();
        assert_eq!(state.pts, 0);
        assert!(!state.has_tech("test_tech"));

        state.pts = 100;
        assert!(state.can_afford(50));
        assert!(!state.can_afford(150));
    }

    #[test]
    fn test_tech_tree() {
        let mut tech_tree = TechTree::new();
        let tech = TechNode {
            id: "test_tech".to_string(),
            name: "Test Tech".to_string(),
            desc: "Test description".to_string(),
            cost_pts: 10,
            requires: vec![],
            grants: vec![],
        };
        tech_tree.add_tech(tech);

        assert!(tech_tree.get_tech("test_tech").is_some());
        assert!(tech_tree.get_tech("nonexistent").is_none());
    }

    #[test]
    fn test_tech_research() {
        let mut research_state = ResearchState::new();
        research_state.pts = 100;

        let tech = TechNode {
            id: "test_tech".to_string(),
            name: "Test Tech".to_string(),
            desc: "Test description".to_string(),
            cost_pts: 50,
            requires: vec![],
            grants: vec![],
        };

        assert!(research_state.can_research(&tech));
        assert!(research_state.research_tech(&tech));
        assert!(research_state.has_tech("test_tech"));
        assert_eq!(research_state.pts, 50);
    }

    #[test]
    fn test_prerequisites() {
        let mut research_state = ResearchState::new();
        research_state.pts = 100;

        let tech = TechNode {
            id: "advanced_tech".to_string(),
            name: "Advanced Tech".to_string(),
            desc: "Requires basic tech".to_string(),
            cost_pts: 50,
            requires: vec!["basic_tech".to_string()],
            grants: vec![],
        };

        assert!(!research_state.can_research(&tech));

        research_state.acquired.push("basic_tech".to_string());
        assert!(research_state.can_research(&tech));
    }
}
