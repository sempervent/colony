# Developer Docs: Research & Rituals

The research and rituals system is a core progression mechanic that allows players to unlock new technologies, abilities, and game mechanics. This document explains how the research system works, how to create new research items, and how rituals can modify the simulation.

## Overview

The research and rituals system provides:

- **Technology Progression**: Unlock new technologies and abilities
- **Research Points**: Currency for purchasing research
- **Tech Tree**: Hierarchical structure of research items
- **Rituals**: Special actions that can modify the simulation
- **Mutations**: Dynamic changes to game mechanics
- **Research Bonuses**: Temporary or permanent benefits

## Research System

### Research Items

Research items represent technologies that can be unlocked:

```rust
pub struct ResearchItem {
    pub id: ResearchId,
    pub name: String,
    pub description: String,
    pub cost: ResearchCost,
    pub prerequisites: Vec<ResearchId>,
    pub category: ResearchCategory,
    pub effects: Vec<ResearchEffect>,
    pub unlock_conditions: Vec<UnlockCondition>,
    pub research_time: u64,      // Time to complete research
    pub is_repeatable: bool,     // Can be researched multiple times
    pub max_level: u32,          // Maximum research level
}

pub enum ResearchCategory {
    Infrastructure,              // Infrastructure improvements
    Operations,                  // Operation enhancements
    Faults,                      // Fault handling improvements
    Resources,                   // Resource management
    Workers,                     // Worker improvements
    Systems,                     // System enhancements
    Modding,                     // Modding capabilities
    Rituals,                     // Ritual unlocks
}
```

### Research Cost

```rust
pub struct ResearchCost {
    pub research_points: u64,    // Primary currency
    pub resources: ResourceCost, // Resource requirements
    pub time: u64,              // Time to complete
    pub prerequisites: Vec<ResearchId>, // Required research
    pub special_requirements: Vec<SpecialRequirement>,
}

pub struct ResourceCost {
    pub power: f32,              // Power requirement
    pub bandwidth: f32,          // Bandwidth requirement
    pub workers: u32,            // Worker requirement
    pub materials: HashMap<String, u64>, // Material requirements
}
```

### Research Effects

Research items can have various effects:

```rust
pub enum ResearchEffect {
    UnlockOperation(OpType),     // Unlock new operation
    UnlockPipeline(PipelineType), // Unlock new pipeline
    UnlockRitual(RitualType),    // Unlock new ritual
    ImproveEfficiency(f32),      // Improve efficiency
    ReduceCosts(ResourceCost),   // Reduce costs
    IncreaseCapacity(u32),       // Increase capacity
    ReduceFaultRate(f32),        // Reduce fault rate
    ImproveRecovery(f32),        // Improve recovery
    UnlockModding(ModdingCapability), // Unlock modding features
    Mutation(MutationType),      // Apply mutation
}
```

## Tech Tree

### Tech Tree Structure

The tech tree is organized hierarchically:

```rust
pub struct TechTree {
    pub nodes: HashMap<ResearchId, ResearchNode>,
    pub connections: Vec<TechTreeConnection>,
    pub categories: HashMap<ResearchCategory, Vec<ResearchId>>,
    pub root_nodes: Vec<ResearchId>, // Starting research items
}

pub struct ResearchNode {
    pub research_item: ResearchItem,
    pub position: (f32, f32),    // Position in tech tree
    pub connections: Vec<ResearchId>, // Connected nodes
    pub is_unlocked: bool,
    pub is_researched: bool,
    pub research_progress: f32,  // Research progress (0.0-1.0)
}
```

### Tech Tree Navigation

```rust
impl TechTree {
    pub fn get_available_research(&self, player_state: &PlayerState) -> Vec<ResearchId> {
        let mut available = Vec::new();
        
        for (id, node) in &self.nodes {
            if !node.is_researched && self.can_research(id, player_state) {
                available.push(*id);
            }
        }
        
        available
    }
    
    pub fn can_research(&self, research_id: &ResearchId, player_state: &PlayerState) -> bool {
        let node = &self.nodes[research_id];
        
        // Check prerequisites
        for prereq in &node.research_item.prerequisites {
            if !self.nodes[prereq].is_researched {
                return false;
            }
        }
        
        // Check resources
        if !player_state.can_afford(&node.research_item.cost) {
            return false;
        }
        
        // Check unlock conditions
        for condition in &node.research_item.unlock_conditions {
            if !condition.is_met(player_state) {
                return false;
            }
        }
        
        true
    }
}
```

## Research Points

### Research Point Generation

Research points are generated through various means:

```rust
pub struct ResearchPointGenerator {
    pub base_rate: f32,          // Base generation rate
    pub worker_bonus: f32,       // Bonus from workers
    pub efficiency_bonus: f32,   // Bonus from efficiency
    pub research_bonus: f32,     // Bonus from research
    pub ritual_bonus: f32,       // Bonus from rituals
    pub mutation_bonus: f32,     // Bonus from mutations
}

impl ResearchPointGenerator {
    pub fn calculate_rate(&self, context: &SimulationContext) -> f32 {
        let worker_count = context.get_worker_count();
        let efficiency = context.get_efficiency();
        let research_level = context.get_research_level();
        let ritual_effects = context.get_ritual_effects();
        let mutation_effects = context.get_mutation_effects();
        
        self.base_rate
            + (worker_count as f32 * self.worker_bonus)
            + (efficiency * self.efficiency_bonus)
            + (research_level * self.research_bonus)
            + (ritual_effects * self.ritual_bonus)
            + (mutation_effects * self.mutation_bonus)
    }
}
```

### Research Point Sources

```rust
pub enum ResearchPointSource {
    WorkerProductivity,          // Generated by workers
    SystemEfficiency,            // Generated by efficiency
    ResearchCompletion,          // Generated by completing research
    RitualPerformance,           // Generated by performing rituals
    MutationEffects,             // Generated by mutations
    SpecialEvents,               // Generated by special events
    BlackSwanSurvival,           // Generated by surviving Black Swans
    Achievement,                 // Generated by achievements
}
```

## Rituals

### Ritual System

Rituals are special actions that can modify the simulation:

```rust
pub struct Ritual {
    pub id: RitualId,
    pub name: String,
    pub description: String,
    pub cost: RitualCost,
    pub effects: Vec<RitualEffect>,
    pub duration: u64,           // Duration of ritual effects
    pub cooldown: u64,           // Cooldown between uses
    pub requirements: Vec<RitualRequirement>,
    pub category: RitualCategory,
}

pub enum RitualCategory {
    Resource,                    // Resource-related rituals
    Fault,                       // Fault-related rituals
    Performance,                 // Performance rituals
    Research,                    // Research rituals
    Mutation,                    // Mutation rituals
    Emergency,                   // Emergency rituals
    Maintenance,                 // Maintenance rituals
    Celebration,                 // Celebration rituals
}
```

### Ritual Effects

Rituals can have various effects:

```rust
pub enum RitualEffect {
    BoostResourceGeneration(f32), // Boost resource generation
    ReduceFaultRate(f32),        // Reduce fault rate
    ImproveEfficiency(f32),      // Improve efficiency
    AccelerateResearch(f32),     // Accelerate research
    ApplyMutation(MutationType), // Apply mutation
    TriggerEvent(EventType),     // Trigger special event
    ModifyCorruption(f32),       // Modify corruption level
    HealWorkers(u32),            // Heal workers
    RestoreResources(ResourceCost), // Restore resources
    UnlockTemporary(UnlockType), // Unlock temporary features
}
```

### Ritual Requirements

```rust
pub enum RitualRequirement {
    ResearchLevel(ResearchId, u32), // Required research level
    ResourceCost(ResourceCost),     // Resource cost
    WorkerCount(u32),               // Minimum worker count
    SystemState(SystemState),       // Required system state
    TimeOfDay(TimeOfDay),           // Time of day requirement
    SpecialCondition(SpecialCondition), // Special condition
}
```

## Mutations

### Mutation System

Mutations are dynamic changes to game mechanics:

```rust
pub struct Mutation {
    pub id: MutationId,
    pub name: String,
    pub description: String,
    pub mutation_type: MutationType,
    pub effects: Vec<MutationEffect>,
    pub duration: Option<u64>,   // None for permanent mutations
    pub trigger_conditions: Vec<MutationTrigger>,
    pub severity: MutationSeverity,
    pub is_beneficial: bool,     // Whether mutation is beneficial
}

pub enum MutationType {
    Operation,                   // Operation mutations
    Pipeline,                    // Pipeline mutations
    Resource,                    // Resource mutations
    Fault,                       // Fault mutations
    Worker,                      // Worker mutations
    System,                      // System mutations
    Research,                    // Research mutations
    Ritual,                      // Ritual mutations
}
```

### Mutation Effects

```rust
pub enum MutationEffect {
    ModifyOperationCost(OpType, ResourceCost), // Modify operation cost
    ChangeOperationBehavior(OpType, BehaviorChange), // Change behavior
    AlterResourceGeneration(ResourceType, f32), // Alter resource generation
    ModifyFaultProbability(FaultType, f32), // Modify fault probability
    ChangeWorkerBehavior(WorkerType, BehaviorChange), // Change worker behavior
    AlterSystemProperties(SystemProperty, f32), // Alter system properties
    ModifyResearchCost(ResearchId, ResearchCost), // Modify research cost
    ChangeRitualEffects(RitualId, Vec<RitualEffect>), // Change ritual effects
}
```

### Mutation Triggers

```rust
pub enum MutationTrigger {
    ResearchCompletion(ResearchId), // Triggered by research completion
    RitualPerformance(RitualId),    // Triggered by ritual performance
    FaultOccurrence(FaultType),     // Triggered by fault occurrence
    ResourceDeficit(ResourceType),  // Triggered by resource deficit
    TimeBased(u64),                 // Triggered by time
    Random(f32),                    // Random trigger
    BlackSwanEvent(BlackSwanType),  // Triggered by Black Swan
    SpecialEvent(EventType),        // Triggered by special event
}
```

## Research State Management

### Research State

```rust
pub struct ResearchState {
    pub completed_research: HashSet<ResearchId>,
    pub current_research: Option<ActiveResearch>,
    pub research_points: u64,
    pub research_rate: f32,
    pub available_rituals: HashSet<RitualId>,
    pub active_mutations: HashMap<MutationId, Mutation>,
    pub research_bonuses: Vec<ResearchBonus>,
    pub ritual_cooldowns: HashMap<RitualId, u64>,
}

pub struct ActiveResearch {
    pub research_id: ResearchId,
    pub start_time: u64,
    pub progress: f32,
    pub assigned_workers: u32,
    pub estimated_completion: u64,
}
```

### Research Progress

```rust
impl ResearchState {
    pub fn update_research_progress(&mut self, context: &SimulationContext) {
        if let Some(ref mut active) = self.current_research {
            let research_item = &context.tech_tree.nodes[&active.research_id].research_item;
            let progress_rate = self.calculate_progress_rate(active, context);
            
            active.progress += progress_rate;
            
            if active.progress >= 1.0 {
                self.complete_research(active.research_id, context);
            }
        }
    }
    
    fn calculate_progress_rate(&self, active: &ActiveResearch, context: &SimulationContext) -> f32 {
        let base_rate = 1.0 / active.research_item.research_time as f32;
        let worker_bonus = active.assigned_workers as f32 * 0.1;
        let efficiency_bonus = context.get_efficiency() * 0.2;
        let research_bonus = self.get_research_bonus();
        
        base_rate * (1.0 + worker_bonus + efficiency_bonus + research_bonus)
    }
}
```

## Configuration

### Research Configuration

```toml
# In game configuration
[research]
base_generation_rate = 1.0       # Base research point generation
worker_bonus = 0.1               # Bonus per worker
efficiency_bonus = 0.2           # Bonus from efficiency
research_bonus = 0.15            # Bonus from research
ritual_bonus = 0.3               # Bonus from rituals
mutation_bonus = 0.25            # Bonus from mutations

[research.categories]
infrastructure = ["power_efficiency", "bandwidth_optimization", "thermal_management"]
operations = ["operation_optimization", "pipeline_efficiency", "fault_handling"]
resources = ["resource_management", "storage_optimization", "allocation_strategies"]
workers = ["worker_training", "skill_development", "fault_recovery"]
systems = ["system_integration", "monitoring", "automation"]
modding = ["modding_tools", "api_access", "hot_reload"]
rituals = ["ritual_mastery", "effect_amplification", "cooldown_reduction"]
```

### Ritual Configuration

```toml
[rituals.power_boost]
name = "Power Boost Ritual"
description = "Temporarily increases power generation"
cost = { research_points = 100, power = 50.0, time = 10 }
effects = [
    { type = "BoostResourceGeneration", resource = "power", multiplier = 1.5 }
]
duration = 300                    # 300 ticks
cooldown = 600                    # 600 ticks
requirements = [
    { type = "ResearchLevel", research = "power_efficiency", level = 2 }
]

[rituals.fault_ward]
name = "Fault Ward Ritual"
description = "Reduces fault probability"
cost = { research_points = 150, bandwidth = 25.0, time = 15 }
effects = [
    { type = "ReduceFaultRate", reduction = 0.3 }
]
duration = 600                    # 600 ticks
cooldown = 1200                   # 1200 ticks
requirements = [
    { type = "ResearchLevel", research = "fault_handling", level = 3 }
]
```

### Mutation Configuration

```toml
[mutations.operation_efficiency]
name = "Operation Efficiency Mutation"
description = "Operations consume fewer resources"
mutation_type = "Operation"
effects = [
    { type = "ModifyOperationCost", operation = "all", cost_reduction = 0.2 }
]
duration = 1000                   # 1000 ticks
trigger_conditions = [
    { type = "ResearchCompletion", research = "operation_optimization" }
]
severity = "moderate"
is_beneficial = true

[mutations.fault_cascade]
name = "Fault Cascade Mutation"
description = "Faults spread more easily"
mutation_type = "Fault"
effects = [
    { type = "ModifyFaultProbability", fault_type = "cascading", multiplier = 1.5 }
]
duration = 500                    # 500 ticks
trigger_conditions = [
    { type = "FaultOccurrence", fault_type = "sticky" }
]
severity = "major"
is_beneficial = false
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_research_completion() {
        let mut research_state = ResearchState::new();
        let research_item = create_test_research_item();
        
        research_state.start_research(research_item.id);
        research_state.complete_research(research_item.id);
        
        assert!(research_state.is_research_completed(research_item.id));
    }
    
    #[test]
    fn test_ritual_performance() {
        let mut research_state = ResearchState::new();
        let ritual = create_test_ritual();
        
        let result = research_state.perform_ritual(ritual.id);
        assert!(result.is_success());
        
        assert!(research_state.is_ritual_on_cooldown(ritual.id));
    }
    
    #[test]
    fn test_mutation_application() {
        let mut research_state = ResearchState::new();
        let mutation = create_test_mutation();
        
        research_state.apply_mutation(mutation.clone());
        
        assert!(research_state.has_active_mutation(mutation.id));
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_research_progression() {
        let mut simulation = create_test_simulation();
        let tech_tree = create_test_tech_tree();
        
        simulation.set_tech_tree(tech_tree);
        
        // Start research
        let research_id = simulation.get_available_research()[0];
        simulation.start_research(research_id);
        
        // Run simulation until research completes
        for _ in 0..1000 {
            simulation.tick();
            if simulation.is_research_completed(research_id) {
                break;
            }
        }
        
        assert!(simulation.is_research_completed(research_id));
    }
    
    #[test]
    fn test_ritual_effects() {
        let mut simulation = create_test_simulation();
        let ritual = create_test_ritual();
        
        let initial_power = simulation.get_power_generation();
        simulation.perform_ritual(ritual.id);
        
        // Check that power generation increased
        let new_power = simulation.get_power_generation();
        assert!(new_power > initial_power);
    }
}
```

## Best Practices

### Design Guidelines

1. **Balanced Progression**: Ensure research provides meaningful progression
2. **Meaningful Choices**: Make research decisions impactful
3. **Clear Benefits**: Make research effects clear and understandable
4. **Strategic Depth**: Provide strategic depth in research choices
5. **Player Agency**: Give players control over their progression

### Performance Considerations

1. **Efficient Updates**: Optimize research state updates
2. **Caching**: Cache frequently accessed research data
3. **Lazy Loading**: Load research data as needed
4. **Memory Management**: Manage memory usage efficiently
5. **Update Frequency**: Balance update frequency with performance

## Troubleshooting

### Common Issues

1. **Research Stalling**: Research not progressing
2. **Ritual Failures**: Rituals not working as expected
3. **Mutation Conflicts**: Conflicting mutations
4. **Performance Issues**: Slow research updates
5. **Balance Problems**: Research too easy or hard

### Debug Tools

- **Research Tracker**: Track research progress and completion
- **Ritual Monitor**: Monitor ritual performance and effects
- **Mutation Analyzer**: Analyze mutation effects and conflicts
- **Progress Visualizer**: Visualize research progression
- **Effect Calculator**: Calculate research and ritual effects

---

**The research and rituals system provides deep progression mechanics and strategic choices. Understanding these systems is key to creating engaging gameplay experiences.** 🏭🔬
