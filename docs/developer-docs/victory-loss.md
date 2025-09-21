# Developer Docs: Victory & Loss Conditions

The victory and loss system defines the end conditions for the Colony Simulator. This document explains how victory and loss conditions work, how to create custom conditions, and how the scoring system operates.

## Overview

The victory and loss system provides:

- **Victory Conditions**: Criteria for achieving victory
- **Loss Conditions**: Criteria for game over
- **Scoring System**: Dynamic scoring based on performance
- **Condition Evaluation**: Real-time evaluation of conditions
- **Condition Configuration**: Configurable conditions for different scenarios
- **Achievement Tracking**: Track progress toward conditions

## Victory Conditions

### Victory Condition Types

Victory conditions can be based on various criteria:

```rust
pub enum VictoryCondition {
    Uptime,                       // Maintain uptime for specified duration
    DeadlineHitRate,              // Achieve minimum deadline hit rate
    CorruptionThreshold,          // Keep corruption below threshold
    ResearchMilestone,            // Complete research milestones
    ScoreThreshold,               // Achieve minimum score
    ResourceEfficiency,           // Achieve resource efficiency
    FaultRecovery,                // Recover from specified number of faults
    BlackSwanSurvival,            // Survive Black Swan events
    WorkerProductivity,           // Achieve worker productivity
    SystemStability,              // Maintain system stability
    Custom(CustomVictoryCondition), // Custom victory condition
}

pub struct VictoryConditionSpec {
    pub condition_type: VictoryCondition,
    pub target_value: f32,
    pub duration: u64,            // Duration to maintain condition
    pub observation_window: u64,  // Window for evaluation
    pub weight: f32,              // Weight in overall victory
    pub description: String,
}
```

### Victory Condition Evaluation

```rust
pub struct VictoryEvaluator {
    pub conditions: Vec<VictoryConditionSpec>,
    pub current_progress: HashMap<VictoryCondition, f32>,
    pub observation_windows: HashMap<VictoryCondition, ObservationWindow>,
    pub victory_threshold: f32,   // Overall victory threshold
}

impl VictoryEvaluator {
    pub fn evaluate(&mut self, context: &SimulationContext) -> VictoryResult {
        let mut total_progress = 0.0;
        let mut total_weight = 0.0;
        
        for condition_spec in &self.conditions {
            let progress = self.evaluate_condition(condition_spec, context);
            let weight = condition_spec.weight;
            
            total_progress += progress * weight;
            total_weight += weight;
            
            self.current_progress.insert(condition_spec.condition_type.clone(), progress);
        }
        
        let overall_progress = if total_weight > 0.0 {
            total_progress / total_weight
        } else {
            0.0
        };
        
        if overall_progress >= self.victory_threshold {
            VictoryResult::Achieved
        } else {
            VictoryResult::InProgress(overall_progress)
        }
    }
    
    fn evaluate_condition(&self, condition_spec: &VictoryConditionSpec, context: &SimulationContext) -> f32 {
        match &condition_spec.condition_type {
            VictoryCondition::Uptime => {
                let uptime = context.get_uptime();
                (uptime / condition_spec.target_value).min(1.0)
            },
            VictoryCondition::DeadlineHitRate => {
                let hit_rate = context.get_deadline_hit_rate();
                (hit_rate / condition_spec.target_value).min(1.0)
            },
            VictoryCondition::CorruptionThreshold => {
                let corruption = context.get_corruption_level();
                if corruption <= condition_spec.target_value {
                    1.0
                } else {
                    0.0
                }
            },
            VictoryCondition::ResearchMilestone => {
                let research_progress = context.get_research_progress();
                (research_progress / condition_spec.target_value).min(1.0)
            },
            VictoryCondition::ScoreThreshold => {
                let score = context.get_current_score();
                (score / condition_spec.target_value).min(1.0)
            },
            // ... other condition types
        }
    }
}
```

## Loss Conditions

### Loss Condition Types

Loss conditions define when the game ends in failure:

```rust
pub enum LossCondition {
    PowerDeficit,                 // Sustained power deficit
    DeadlineMissRate,             // High deadline miss rate
    StickyWorkerLimit,            // Too many sticky workers
    BlackSwanChain,               // Black Swan event chain
    TimeLimit,                    // Time limit exceeded
    ResourceDepletion,            // Critical resource depletion
    SystemFailure,                // Complete system failure
    CorruptionOverflow,           // Corruption exceeds critical level
    FaultCascade,                 // Uncontrolled fault cascade
    WorkerExhaustion,             // All workers exhausted
    Custom(CustomLossCondition),  // Custom loss condition
}

pub struct LossConditionSpec {
    pub condition_type: LossCondition,
    pub threshold: f32,
    pub duration: u64,            // Duration to maintain condition
    pub observation_window: u64,  // Window for evaluation
    pub severity: LossSeverity,
    pub description: String,
}

pub enum LossSeverity {
    Minor,                        // Minor failure, recoverable
    Major,                        // Major failure, difficult recovery
    Critical,                     // Critical failure, system-threatening
    Catastrophic,                 // Catastrophic failure, game over
}
```

### Loss Condition Evaluation

```rust
pub struct LossEvaluator {
    pub conditions: Vec<LossConditionSpec>,
    pub current_status: HashMap<LossCondition, LossStatus>,
    pub observation_windows: HashMap<LossCondition, ObservationWindow>,
    pub loss_threshold: f32,      // Overall loss threshold
}

impl LossEvaluator {
    pub fn evaluate(&mut self, context: &SimulationContext) -> LossResult {
        let mut total_severity = 0.0;
        let mut critical_conditions = 0;
        
        for condition_spec in &self.conditions {
            let status = self.evaluate_condition(condition_spec, context);
            
            match status {
                LossStatus::Triggered(severity) => {
                    total_severity += severity as f32;
                    if severity == LossSeverity::Critical || severity == LossSeverity::Catastrophic {
                        critical_conditions += 1;
                    }
                },
                LossStatus::Warning(severity) => {
                    total_severity += severity as f32 * 0.5;
                },
                LossStatus::Safe => {
                    // No contribution to loss
                },
            }
            
            self.current_status.insert(condition_spec.condition_type.clone(), status);
        }
        
        if critical_conditions > 0 || total_severity >= self.loss_threshold {
            LossResult::GameOver(total_severity)
        } else if total_severity > self.loss_threshold * 0.7 {
            LossResult::Warning(total_severity)
        } else {
            LossResult::Safe
        }
    }
}
```

## Scoring System

### Score Components

The scoring system evaluates performance across multiple dimensions:

```rust
pub struct ScoreComponents {
    pub uptime_score: f32,        // Uptime contribution
    pub efficiency_score: f32,    // Efficiency contribution
    pub fault_score: f32,         // Fault handling contribution
    pub research_score: f32,      // Research contribution
    pub resource_score: f32,      // Resource management contribution
    pub stability_score: f32,     // Stability contribution
    pub bonus_score: f32,         // Bonus achievements
    pub penalty_score: f32,       // Penalties
}

pub struct ScoreCalculator {
    pub weights: ScoreWeights,
    pub bonus_multipliers: HashMap<AchievementType, f32>,
    pub penalty_multipliers: HashMap<PenaltyType, f32>,
    pub historical_data: Vec<ScoreSnapshot>,
}

pub struct ScoreWeights {
    pub uptime: f32,
    pub efficiency: f32,
    pub fault_handling: f32,
    pub research: f32,
    pub resource_management: f32,
    pub stability: f32,
}
```

### Score Calculation

```rust
impl ScoreCalculator {
    pub fn calculate_score(&self, context: &SimulationContext) -> f32 {
        let components = self.calculate_components(context);
        let weights = &self.weights;
        
        let base_score = components.uptime_score * weights.uptime
            + components.efficiency_score * weights.efficiency
            + components.fault_score * weights.fault_handling
            + components.research_score * weights.research
            + components.resource_score * weights.resource_management
            + components.stability_score * weights.stability;
        
        let bonus_score = self.calculate_bonus_score(context);
        let penalty_score = self.calculate_penalty_score(context);
        
        (base_score + bonus_score - penalty_score).max(0.0)
    }
    
    fn calculate_components(&self, context: &SimulationContext) -> ScoreComponents {
        ScoreComponents {
            uptime_score: self.calculate_uptime_score(context),
            efficiency_score: self.calculate_efficiency_score(context),
            fault_score: self.calculate_fault_score(context),
            research_score: self.calculate_research_score(context),
            resource_score: self.calculate_resource_score(context),
            stability_score: self.calculate_stability_score(context),
            bonus_score: 0.0, // Calculated separately
            penalty_score: 0.0, // Calculated separately
        }
    }
}
```

## Condition Monitoring

### Real-time Monitoring

```rust
pub struct ConditionMonitor {
    pub victory_evaluator: VictoryEvaluator,
    pub loss_evaluator: LossEvaluator,
    pub score_calculator: ScoreCalculator,
    pub monitoring_interval: u64,
    pub last_evaluation: u64,
    pub condition_history: Vec<ConditionSnapshot>,
}

impl ConditionMonitor {
    pub fn update(&mut self, context: &SimulationContext) -> ConditionUpdate {
        let current_tick = context.get_current_tick();
        
        if current_tick - self.last_evaluation >= self.monitoring_interval {
            let victory_result = self.victory_evaluator.evaluate(context);
            let loss_result = self.loss_evaluator.evaluate(context);
            let current_score = self.score_calculator.calculate_score(context);
            
            let snapshot = ConditionSnapshot {
                tick: current_tick,
                victory_result: victory_result.clone(),
                loss_result: loss_result.clone(),
                score: current_score,
                context_snapshot: context.create_snapshot(),
            };
            
            self.condition_history.push(snapshot);
            self.last_evaluation = current_tick;
            
            ConditionUpdate {
                victory_result,
                loss_result,
                score: current_score,
                snapshot,
            }
        } else {
            ConditionUpdate::NoUpdate
        }
    }
}
```

### Condition Alerts

```rust
pub struct ConditionAlert {
    pub alert_type: AlertType,
    pub condition: ConditionType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: u64,
    pub threshold: f32,
    pub current_value: f32,
}

pub enum AlertType {
    VictoryProgress,              // Progress toward victory
    VictoryAchieved,              // Victory achieved
    LossWarning,                  // Loss condition warning
    LossTriggered,                // Loss condition triggered
    ScoreMilestone,               // Score milestone reached
    ConditionChange,              // Condition status changed
}

pub enum AlertSeverity {
    Info,                         // Informational
    Warning,                      // Warning
    Critical,                     // Critical
    Emergency,                    // Emergency
}
```

## Configuration

### Victory Condition Configuration

```toml
# In game configuration
[victory_conditions]
victory_threshold = 0.8          # Overall victory threshold
observation_window = 1000        # Default observation window

[victory_conditions.uptime]
condition_type = "Uptime"
target_value = 0.95              # 95% uptime
duration = 5000                  # 5000 ticks
observation_window = 1000        # 1000 tick window
weight = 0.3                     # 30% weight
description = "Maintain 95% uptime for 5000 ticks"

[victory_conditions.deadline_hit_rate]
condition_type = "DeadlineHitRate"
target_value = 0.9               # 90% hit rate
duration = 3000                  # 3000 ticks
observation_window = 500         # 500 tick window
weight = 0.25                    # 25% weight
description = "Achieve 90% deadline hit rate for 3000 ticks"

[victory_conditions.corruption_threshold]
condition_type = "CorruptionThreshold"
target_value = 0.3               # 30% corruption
duration = 2000                  # 2000 ticks
observation_window = 200         # 200 tick window
weight = 0.2                     # 20% weight
description = "Keep corruption below 30% for 2000 ticks"
```

### Loss Condition Configuration

```toml
[loss_conditions]
loss_threshold = 0.7             # Overall loss threshold
observation_window = 500         # Default observation window

[loss_conditions.power_deficit]
condition_type = "PowerDeficit"
threshold = 0.0                  # No power
duration = 100                   # 100 ticks
observation_window = 50          # 50 tick window
severity = "Critical"
description = "Sustained power deficit for 100 ticks"

[loss_conditions.sticky_workers]
condition_type = "StickyWorkerLimit"
threshold = 0.8                  # 80% sticky workers
duration = 200                   # 200 ticks
observation_window = 100         # 100 tick window
severity = "Major"
description = "More than 80% workers stuck for 200 ticks"

[loss_conditions.black_swan_chain]
condition_type = "BlackSwanChain"
threshold = 3                    # 3 Black Swans
duration = 1000                  # 1000 ticks
observation_window = 200         # 200 tick window
severity = "Catastrophic"
description = "3 Black Swan events within 1000 ticks"
```

### Scoring Configuration

```toml
[scoring]
base_score = 1000.0              # Base score
max_score = 10000.0              # Maximum possible score

[scoring.weights]
uptime = 0.25                    # 25% weight
efficiency = 0.2                 # 20% weight
fault_handling = 0.15            # 15% weight
research = 0.15                  # 15% weight
resource_management = 0.15       # 15% weight
stability = 0.1                  # 10% weight

[scoring.bonuses]
perfect_uptime = 2.0             # 2x multiplier for perfect uptime
no_faults = 1.5                  # 1.5x multiplier for no faults
research_master = 1.3            # 1.3x multiplier for research mastery
efficiency_expert = 1.2          # 1.2x multiplier for efficiency

[scoring.penalties]
fault_cascade = 0.5              # 0.5x multiplier for fault cascades
resource_waste = 0.8             # 0.8x multiplier for resource waste
instability = 0.7                # 0.7x multiplier for instability
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_victory_condition_evaluation() {
        let mut evaluator = VictoryEvaluator::new();
        let condition = VictoryConditionSpec {
            condition_type: VictoryCondition::Uptime,
            target_value: 0.95,
            duration: 1000,
            observation_window: 100,
            weight: 1.0,
            description: "Test uptime condition".to_string(),
        };
        
        evaluator.conditions.push(condition);
        
        let context = create_test_context_with_uptime(0.98);
        let result = evaluator.evaluate(&context);
        
        assert!(matches!(result, VictoryResult::Achieved));
    }
    
    #[test]
    fn test_loss_condition_evaluation() {
        let mut evaluator = LossEvaluator::new();
        let condition = LossConditionSpec {
            condition_type: LossCondition::PowerDeficit,
            threshold: 0.0,
            duration: 100,
            observation_window: 50,
            severity: LossSeverity::Critical,
            description: "Test power deficit condition".to_string(),
        };
        
        evaluator.conditions.push(condition);
        
        let context = create_test_context_with_power_deficit();
        let result = evaluator.evaluate(&context);
        
        assert!(matches!(result, LossResult::GameOver(_)));
    }
    
    #[test]
    fn test_score_calculation() {
        let calculator = ScoreCalculator::new();
        let context = create_test_context();
        
        let score = calculator.calculate_score(&context);
        
        assert!(score >= 0.0);
        assert!(score <= calculator.max_score);
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_victory_achievement() {
        let mut simulation = create_test_simulation();
        let victory_conditions = create_test_victory_conditions();
        
        simulation.set_victory_conditions(victory_conditions);
        
        // Run simulation until victory is achieved
        for _ in 0..10000 {
            simulation.tick();
            if simulation.is_victory_achieved() {
                break;
            }
        }
        
        assert!(simulation.is_victory_achieved());
    }
    
    #[test]
    fn test_loss_condition() {
        let mut simulation = create_test_simulation();
        let loss_conditions = create_test_loss_conditions();
        
        simulation.set_loss_conditions(loss_conditions);
        
        // Trigger loss condition
        simulation.trigger_power_deficit();
        
        // Run simulation until loss is triggered
        for _ in 0..1000 {
            simulation.tick();
            if simulation.is_game_over() {
                break;
            }
        }
        
        assert!(simulation.is_game_over());
    }
}
```

## Best Practices

### Design Guidelines

1. **Balanced Conditions**: Ensure victory and loss conditions are balanced
2. **Clear Objectives**: Make conditions clear and understandable
3. **Meaningful Choices**: Provide meaningful strategic choices
4. **Progressive Difficulty**: Increase difficulty over time
5. **Player Agency**: Give players control over their fate

### Performance Considerations

1. **Efficient Evaluation**: Optimize condition evaluation
2. **Caching**: Cache frequently calculated values
3. **Update Frequency**: Balance update frequency with performance
4. **Memory Management**: Manage condition history efficiently
5. **Scalability**: Ensure system scales with complexity

## Troubleshooting

### Common Issues

1. **Condition Stalling**: Conditions not progressing
2. **Imbalanced Scoring**: Scoring too easy or hard
3. **Performance Issues**: Slow condition evaluation
4. **False Positives**: Conditions triggering incorrectly
5. **Missing Conditions**: Important conditions not covered

### Debug Tools

- **Condition Tracker**: Track condition progress and status
- **Score Analyzer**: Analyze score components and trends
- **Alert Monitor**: Monitor condition alerts and warnings
- **History Viewer**: View condition history and trends
- **Performance Profiler**: Profile condition evaluation performance

---

**The victory and loss system provides clear objectives and meaningful end conditions. Understanding these systems is key to creating engaging and balanced gameplay experiences.** 🏭🏆
