# Modding: Capabilities

The capabilities system provides fine-grained access control for mods, ensuring security while allowing necessary functionality. This guide explains how capabilities work, how to request them, and how to use them safely.

## Overview

The capabilities system provides:

- **Fine-Grained Access Control**: Precise control over what mods can access
- **Security**: Prevents unauthorized access to sensitive systems
- **Transparency**: Clear declaration of required capabilities
- **Validation**: Automatic validation of capability requests
- **Auditing**: Track capability usage for security analysis
- **Flexibility**: Easy to add new capabilities as needed

## Capability Types

### Core Capabilities

```rust
pub enum Capability {
    // Time and simulation
    SimTime,                     // Access to simulation time
    SimState,                    // Access to simulation state
    SimControl,                  // Control simulation execution
    
    // Job management
    EnqueueJob,                  // Enqueue new jobs
    CancelJob,                   // Cancel existing jobs
    ModifyJob,                   // Modify job properties
    JobStatus,                   // Access job status information
    
    // Worker management
    CreateWorker,                // Create new workers
    DestroyWorker,               // Destroy workers
    ModifyWorker,                // Modify worker properties
    WorkerStatus,                // Access worker status information
    
    // Resource management
    ReadResources,               // Read resource information
    ModifyResources,             // Modify resource values
    ResourceHistory,             // Access resource history
    
    // System control
    SystemControl,               // Control system behavior
    SystemConfig,                // Modify system configuration
    SystemStatus,                // Access system status
    
    // Network access
    NetworkRead,                 // Read network data
    NetworkWrite,                // Write network data
    NetworkControl,              // Control network behavior
    
    // File system access
    FileRead,                    // Read files
    FileWrite,                   // Write files
    FileDelete,                  // Delete files
    FileList,                    // List files
    
    // Event system
    EventRegister,               // Register event handlers
    EventEmit,                   // Emit custom events
    EventAccess,                 // Access event system
    
    // Research and rituals
    ResearchAccess,              // Access research system
    RitualPerform,               // Perform rituals
    MutationApply,               // Apply mutations
    
    // Black Swan events
    BlackSwanAccess,             // Access Black Swan system
    BlackSwanTrigger,            // Trigger Black Swan events
    
    // Victory and loss
    VictoryAccess,               // Access victory conditions
    LossAccess,                  // Access loss conditions
    
    // Modding system
    ModLoad,                     // Load other mods
    ModUnload,                   // Unload other mods
    ModInfo,                     // Access mod information
    
    // Custom capabilities
    Custom(String),              // Custom capability
}
```

### Capability Groups

```rust
pub enum CapabilityGroup {
    ReadOnly,                    // Read-only access
    WriteAccess,                 // Write access
    SystemControl,               // System control access
    NetworkAccess,               // Network access
    FileAccess,                  // File system access
    EventAccess,                 // Event system access
    ResearchAccess,              // Research system access
    ModdingAccess,               // Modding system access
    Custom(String),              // Custom capability group
}

impl CapabilityGroup {
    pub fn get_capabilities(&self) -> Vec<Capability> {
        match self {
            CapabilityGroup::ReadOnly => vec![
                Capability::SimTime,
                Capability::SimState,
                Capability::JobStatus,
                Capability::WorkerStatus,
                Capability::ReadResources,
                Capability::SystemStatus,
                Capability::NetworkRead,
                Capability::FileRead,
                Capability::EventAccess,
                Capability::ResearchAccess,
                Capability::VictoryAccess,
                Capability::LossAccess,
                Capability::ModInfo,
            ],
            CapabilityGroup::WriteAccess => vec![
                Capability::EnqueueJob,
                Capability::CancelJob,
                Capability::ModifyJob,
                Capability::CreateWorker,
                Capability::DestroyWorker,
                Capability::ModifyWorker,
                Capability::ModifyResources,
                Capability::NetworkWrite,
                Capability::FileWrite,
                Capability::FileDelete,
            ],
            CapabilityGroup::SystemControl => vec![
                Capability::SimControl,
                Capability::SystemControl,
                Capability::SystemConfig,
                Capability::NetworkControl,
            ],
            CapabilityGroup::NetworkAccess => vec![
                Capability::NetworkRead,
                Capability::NetworkWrite,
                Capability::NetworkControl,
            ],
            CapabilityGroup::FileAccess => vec![
                Capability::FileRead,
                Capability::FileWrite,
                Capability::FileDelete,
                Capability::FileList,
            ],
            CapabilityGroup::EventAccess => vec![
                Capability::EventRegister,
                Capability::EventEmit,
                Capability::EventAccess,
            ],
            CapabilityGroup::ResearchAccess => vec![
                Capability::ResearchAccess,
                Capability::RitualPerform,
                Capability::MutationApply,
            ],
            CapabilityGroup::ModdingAccess => vec![
                Capability::ModLoad,
                Capability::ModUnload,
                Capability::ModInfo,
            ],
            CapabilityGroup::Custom(_) => vec![],
        }
    }
}
```

## Capability Declaration

### Mod Manifest

Capabilities are declared in the mod manifest:

```toml
# In mod.toml
[mod]
name = "My Custom Mod"
version = "1.0.0"
description = "A custom mod with specific capabilities"
author = "Your Name"

[mod.capabilities]
# Individual capabilities
capabilities = [
    "sim_time",
    "sim_state",
    "enqueue_job",
    "job_status",
    "read_resources",
    "event_register",
    "event_emit"
]

# Capability groups
groups = [
    "read_only",
    "write_access",
    "event_access"
]

# Custom capabilities
custom = [
    "my_custom_capability"
]

# Capability constraints
constraints = [
    { capability = "enqueue_job", limit = 100, period = "1m" },
    { capability = "modify_resources", limit = 10, period = "1s" },
    { capability = "file_write", limit = 5, period = "1m" }
]
```

### Capability Validation

```rust
pub struct CapabilityValidator {
    pub mod_capabilities: HashMap<ModId, Vec<Capability>>,
    pub capability_limits: HashMap<Capability, CapabilityLimit>,
    pub usage_tracker: UsageTracker,
}

pub struct CapabilityLimit {
    pub max_usage: u32,
    pub time_period: Duration,
    pub current_usage: u32,
    pub last_reset: u64,
}

impl CapabilityValidator {
    pub fn validate_capability_request(
        &mut self,
        mod_id: &ModId,
        capability: &Capability,
    ) -> Result<(), CapabilityError> {
        // Check if mod has the capability
        if !self.mod_has_capability(mod_id, capability) {
            return Err(CapabilityError::CapabilityNotGranted);
        }
        
        // Check usage limits
        if let Some(limit) = self.capability_limits.get(capability) {
            if !self.check_usage_limit(limit) {
                return Err(CapabilityError::UsageLimitExceeded);
            }
        }
        
        // Track usage
        self.usage_tracker.track_usage(mod_id, capability);
        
        Ok(())
    }
    
    fn mod_has_capability(&self, mod_id: &ModId, capability: &Capability) -> bool {
        if let Some(capabilities) = self.mod_capabilities.get(mod_id) {
            capabilities.contains(capability)
        } else {
            false
        }
    }
    
    fn check_usage_limit(&mut self, limit: &mut CapabilityLimit) -> bool {
        let current_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        
        // Reset counter if period has passed
        if current_time - limit.last_reset >= limit.time_period.as_secs() {
            limit.current_usage = 0;
            limit.last_reset = current_time;
        }
        
        // Check if limit is exceeded
        if limit.current_usage >= limit.max_usage {
            return false;
        }
        
        // Increment usage
        limit.current_usage += 1;
        true
    }
}
```

## Capability Usage

### WASM Operations

```rust
use colony_modsdk::capabilities::Capability;

pub struct MyWasmOp {
    pub name: String,
    pub description: String,
    pub resource_cost: OpResourceCost,
    pub required_capabilities: Vec<Capability>,
}

impl WasmOpSpec for MyWasmOp {
    fn get_name(&self) -> String {
        self.name.clone()
    }
    
    fn get_description(&self) -> String {
        self.description.clone()
    }
    
    fn get_resource_cost(&self) -> OpResourceCost {
        self.resource_cost
    }
    
    fn execute(&self, context: &mut WasmOpContext) -> OpResult {
        // Check capabilities before execution
        for capability in &self.required_capabilities {
            context.validate_capability(capability)?;
        }
        
        // Execute operation with capabilities
        self.execute_with_capabilities(context)
    }
}

impl MyWasmOp {
    fn execute_with_capabilities(&self, context: &mut WasmOpContext) -> OpResult {
        // Use capabilities to access simulation state
        if self.required_capabilities.contains(&Capability::SimState) {
            let sim_state = context.get_simulation_state()?;
            // Use simulation state
        }
        
        // Use capabilities to enqueue jobs
        if self.required_capabilities.contains(&Capability::EnqueueJob) {
            let new_job = self.create_job()?;
            context.enqueue_job(new_job)?;
        }
        
        // Use capabilities to modify resources
        if self.required_capabilities.contains(&Capability::ModifyResources) {
            context.modify_resource("power", 100.0)?;
        }
        
        OpResult::Success
    }
}
```

### Lua Scripts

```lua
-- In your Lua mod
local function on_mod_loaded(event_data)
    print("Mod loaded, checking capabilities")
    
    -- Check if mod has required capabilities
    if colony.capabilities.has("sim_time") then
        print("Mod has sim_time capability")
    else
        print("Mod does not have sim_time capability")
        return
    end
    
    if colony.capabilities.has("enqueue_job") then
        print("Mod has enqueue_job capability")
    else
        print("Mod does not have enqueue_job capability")
        return
    end
    
    -- Register event handlers
    colony.events.register("tick_start", on_tick_start)
    colony.events.register("job_created", on_job_created)
end

local function on_tick_start(event_data)
    -- Use sim_time capability
    local current_time = colony.time.get_current_tick()
    print("Current tick: " .. current_time)
    
    -- Use enqueue_job capability
    if current_time % 100 == 0 then
        local job = {
            id = "periodic_job_" .. current_time,
            pipeline = "data_processing",
            priority = 5,
            deadline = current_time + 1000
        }
        
        colony.jobs.enqueue(job)
        print("Enqueued periodic job: " .. job.id)
    end
end

local function on_job_created(event_data)
    -- Use job_status capability
    local job_status = colony.jobs.get_status(event_data.job.id)
    print("Job status: " .. job_status)
    
    -- Use read_resources capability
    local power_level = colony.resources.get("power")
    print("Current power level: " .. power_level)
    
    -- Use modify_resources capability (if available)
    if colony.capabilities.has("modify_resources") then
        if power_level < 100 then
            colony.resources.set("power", power_level + 10)
            print("Boosted power level")
        end
    end
end

-- Register mod loaded handler
colony.events.register("mod_loaded", on_mod_loaded)
```

## Capability Management

### Capability Registry

```rust
pub struct CapabilityRegistry {
    pub mod_capabilities: HashMap<ModId, ModCapabilities>,
    pub capability_definitions: HashMap<Capability, CapabilityDefinition>,
    pub usage_tracker: UsageTracker,
    pub audit_log: AuditLog,
}

pub struct ModCapabilities {
    pub mod_id: ModId,
    pub capabilities: Vec<Capability>,
    pub groups: Vec<CapabilityGroup>,
    pub custom_capabilities: Vec<String>,
    pub constraints: Vec<CapabilityConstraint>,
    pub granted_at: u64,
    pub expires_at: Option<u64>,
}

pub struct CapabilityDefinition {
    pub name: String,
    pub description: String,
    pub group: CapabilityGroup,
    pub default_limit: Option<CapabilityLimit>,
    pub security_level: SecurityLevel,
    pub dependencies: Vec<Capability>,
}

pub enum SecurityLevel {
    Low,                         // Low security risk
    Medium,                      // Medium security risk
    High,                        // High security risk
    Critical,                    // Critical security risk
}
```

### Capability Granting

```rust
impl CapabilityRegistry {
    pub fn grant_capabilities(&mut self, mod_id: &ModId, capabilities: ModCapabilities) -> Result<(), CapabilityError> {
        // Validate capability request
        self.validate_capability_request(&capabilities)?;
        
        // Check security constraints
        self.check_security_constraints(&capabilities)?;
        
        // Grant capabilities
        self.mod_capabilities.insert(mod_id.clone(), capabilities.clone());
        
        // Log capability grant
        self.audit_log.log_capability_grant(mod_id, &capabilities);
        
        Ok(())
    }
    
    fn validate_capability_request(&self, capabilities: &ModCapabilities) -> Result<(), CapabilityError> {
        // Check if all requested capabilities exist
        for capability in &capabilities.capabilities {
            if !self.capability_definitions.contains_key(capability) {
                return Err(CapabilityError::UnknownCapability);
            }
        }
        
        // Check capability dependencies
        for capability in &capabilities.capabilities {
            let definition = &self.capability_definitions[capability];
            for dependency in &definition.dependencies {
                if !capabilities.capabilities.contains(dependency) {
                    return Err(CapabilityError::MissingDependency);
                }
            }
        }
        
        Ok(())
    }
    
    fn check_security_constraints(&self, capabilities: &ModCapabilities) -> Result<(), CapabilityError> {
        // Check for high-risk capabilities
        for capability in &capabilities.capabilities {
            let definition = &self.capability_definitions[capability];
            if definition.security_level == SecurityLevel::Critical {
                // Require additional validation for critical capabilities
                return Err(CapabilityError::CriticalCapabilityRequiresValidation);
            }
        }
        
        Ok(())
    }
}
```

## Usage Tracking and Auditing

### Usage Tracker

```rust
pub struct UsageTracker {
    pub usage_log: Vec<UsageEntry>,
    pub mod_usage: HashMap<ModId, ModUsageStats>,
    pub capability_usage: HashMap<Capability, CapabilityUsageStats>,
}

pub struct UsageEntry {
    pub mod_id: ModId,
    pub capability: Capability,
    pub timestamp: u64,
    pub success: bool,
    pub error: Option<String>,
    pub context: Option<String>,
}

pub struct ModUsageStats {
    pub mod_id: ModId,
    pub total_usage: u64,
    pub successful_usage: u64,
    pub failed_usage: u64,
    pub capability_usage: HashMap<Capability, u64>,
    pub last_usage: Option<u64>,
}

pub struct CapabilityUsageStats {
    pub capability: Capability,
    pub total_usage: u64,
    pub successful_usage: u64,
    pub failed_usage: u64,
    pub mod_usage: HashMap<ModId, u64>,
    pub last_usage: Option<u64>,
}

impl UsageTracker {
    pub fn track_usage(&mut self, mod_id: &ModId, capability: &Capability, success: bool, error: Option<String>) {
        let entry = UsageEntry {
            mod_id: mod_id.clone(),
            capability: capability.clone(),
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            success,
            error,
            context: None,
        };
        
        self.usage_log.push(entry);
        
        // Update mod usage stats
        let mod_stats = self.mod_usage.entry(mod_id.clone()).or_insert(ModUsageStats {
            mod_id: mod_id.clone(),
            total_usage: 0,
            successful_usage: 0,
            failed_usage: 0,
            capability_usage: HashMap::new(),
            last_usage: None,
        });
        
        mod_stats.total_usage += 1;
        if success {
            mod_stats.successful_usage += 1;
        } else {
            mod_stats.failed_usage += 1;
        }
        mod_stats.capability_usage.entry(capability.clone()).and_modify(|count| *count += 1).or_insert(1);
        mod_stats.last_usage = Some(entry.timestamp);
        
        // Update capability usage stats
        let cap_stats = self.capability_usage.entry(capability.clone()).or_insert(CapabilityUsageStats {
            capability: capability.clone(),
            total_usage: 0,
            successful_usage: 0,
            failed_usage: 0,
            mod_usage: HashMap::new(),
            last_usage: None,
        });
        
        cap_stats.total_usage += 1;
        if success {
            cap_stats.successful_usage += 1;
        } else {
            cap_stats.failed_usage += 1;
        }
        cap_stats.mod_usage.entry(mod_id.clone()).and_modify(|count| *count += 1).or_insert(1);
        cap_stats.last_usage = Some(entry.timestamp);
    }
}
```

### Audit Log

```rust
pub struct AuditLog {
    pub entries: Vec<AuditEntry>,
    pub max_entries: usize,
}

pub struct AuditEntry {
    pub timestamp: u64,
    pub event_type: AuditEventType,
    pub mod_id: Option<ModId>,
    pub capability: Option<Capability>,
    pub details: String,
    pub success: bool,
}

pub enum AuditEventType {
    CapabilityGranted,
    CapabilityRevoked,
    CapabilityUsed,
    CapabilityDenied,
    ModLoaded,
    ModUnloaded,
    SecurityViolation,
    UsageLimitExceeded,
}

impl AuditLog {
    pub fn log_capability_grant(&mut self, mod_id: &ModId, capabilities: &ModCapabilities) {
        let entry = AuditEntry {
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            event_type: AuditEventType::CapabilityGranted,
            mod_id: Some(mod_id.clone()),
            capability: None,
            details: format!("Granted capabilities: {:?}", capabilities.capabilities),
            success: true,
        };
        
        self.add_entry(entry);
    }
    
    pub fn log_capability_use(&mut self, mod_id: &ModId, capability: &Capability, success: bool, error: Option<String>) {
        let entry = AuditEntry {
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            event_type: AuditEventType::CapabilityUsed,
            mod_id: Some(mod_id.clone()),
            capability: Some(capability.clone()),
            details: if let Some(error) = error {
                format!("Capability use failed: {}", error)
            } else {
                "Capability used successfully".to_string()
            },
            success,
        };
        
        self.add_entry(entry);
    }
    
    fn add_entry(&mut self, entry: AuditEntry) {
        self.entries.push(entry);
        
        // Limit log size
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }
}
```

## Security Considerations

### Security Validation

```rust
pub struct SecurityValidator {
    pub security_policies: Vec<SecurityPolicy>,
    pub risk_assessor: RiskAssessor,
}

pub struct SecurityPolicy {
    pub name: String,
    pub description: String,
    pub rules: Vec<SecurityRule>,
    pub enforcement_level: EnforcementLevel,
}

pub enum SecurityRule {
    CapabilityLimit(Capability, u32, Duration),
    ModRestriction(ModId, Vec<Capability>),
    TimeRestriction(Duration, Vec<Capability>),
    ResourceRestriction(ResourceType, f32),
    NetworkRestriction(NetworkPolicy),
    FileRestriction(FilePolicy),
}

pub enum EnforcementLevel {
    Advisory,                    // Advisory only
    Mandatory,                   // Must be enforced
    Critical,                    // Critical security requirement
}

impl SecurityValidator {
    pub fn validate_capability_use(&self, mod_id: &ModId, capability: &Capability) -> Result<(), SecurityError> {
        // Check security policies
        for policy in &self.security_policies {
            for rule in &policy.rules {
                if let Err(error) = self.validate_rule(rule, mod_id, capability) {
                    match policy.enforcement_level {
                        EnforcementLevel::Advisory => {
                            // Log warning but allow
                            println!("Security warning: {}", error);
                        },
                        EnforcementLevel::Mandatory => {
                            // Deny access
                            return Err(error);
                        },
                        EnforcementLevel::Critical => {
                            // Deny access and log critical violation
                            self.log_critical_violation(mod_id, capability, &error);
                            return Err(error);
                        },
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn validate_rule(&self, rule: &SecurityRule, mod_id: &ModId, capability: &Capability) -> Result<(), SecurityError> {
        match rule {
            SecurityRule::CapabilityLimit(cap, limit, period) => {
                if cap == capability {
                    // Check usage limit
                    // Implementation depends on usage tracking
                }
            },
            SecurityRule::ModRestriction(restricted_mod, allowed_capabilities) => {
                if mod_id == restricted_mod && !allowed_capabilities.contains(capability) {
                    return Err(SecurityError::ModRestricted);
                }
            },
            SecurityRule::TimeRestriction(duration, restricted_capabilities) => {
                if restricted_capabilities.contains(capability) {
                    // Check time restrictions
                    // Implementation depends on time tracking
                }
            },
            SecurityRule::ResourceRestriction(resource_type, limit) => {
                // Check resource restrictions
                // Implementation depends on resource tracking
            },
            SecurityRule::NetworkRestriction(policy) => {
                // Check network restrictions
                // Implementation depends on network policy
            },
            SecurityRule::FileRestriction(policy) => {
                // Check file restrictions
                // Implementation depends on file policy
            },
        }
        
        Ok(())
    }
}
```

## Best Practices

### Capability Design

1. **Principle of Least Privilege**: Grant only necessary capabilities
2. **Explicit Declaration**: Always declare required capabilities
3. **Capability Groups**: Use capability groups for common patterns
4. **Custom Capabilities**: Create custom capabilities for specific needs
5. **Documentation**: Document capability requirements clearly

### Security Guidelines

1. **Input Validation**: Validate all inputs when using capabilities
2. **Error Handling**: Handle capability errors gracefully
3. **Usage Limits**: Respect usage limits and constraints
4. **Audit Logging**: Enable audit logging for security analysis
5. **Regular Review**: Regularly review capability usage

### Performance Considerations

1. **Capability Caching**: Cache capability checks when possible
2. **Efficient Validation**: Optimize capability validation
3. **Usage Tracking**: Minimize overhead of usage tracking
4. **Audit Logging**: Balance audit logging with performance
5. **Resource Management**: Manage resources efficiently

---

**The capabilities system provides essential security and access control for the Colony Simulator. Understanding these concepts is key to creating secure and effective mods.** 🏭🔒
