use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Mod manifest defining the mod's metadata, entrypoints, and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    pub id: String,              // "com.yourid.packetalchemy"
    pub name: String,
    pub version: String,         // semver
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub entrypoints: Entrypoints,
    pub capabilities: Capabilities,
    pub signature: Option<String>, // base64, optional unsigned for dev
    pub requires: Option<Vec<String>>, // mod dependencies
}

/// Entrypoints defining where the mod's code and content can be found
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Entrypoints {
    pub wasm_ops: Vec<String>,      // e.g., ["Op_AdaptiveFft", "Op_Anomaly"]
    pub lua_events: Vec<String>,    // e.g., ["on_tick.lua", "on_fault.lua"]
    pub pipelines: Option<String>,  // path to pipelines.toml
    pub blackswans: Option<String>, // path to events.toml
    pub tech: Option<String>,       // path to tech.toml
    pub scenarios: Option<String>,  // path to scenarios.toml
}

/// Capabilities defining what the mod is allowed to do
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Capabilities {
    pub sim_time: bool,     // read-only access to simulation time
    pub rng: bool,          // deterministic RNG handle
    pub metrics_read: bool, // read KPIs and metrics
    pub enqueue_job: bool,  // push jobs into queues
    pub log_debug: bool,    // write debug logs
    pub modify_tunables: bool, // modify system tunables
    pub trigger_events: bool, // trigger Black Swan events
}

/// Specification for a WASM operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmOpSpec {
    pub name: String,
    pub version: String,
    pub cost_hint_ms: u32,
    pub work_units_hint: f32,
    pub vram_hint_mb: f32,
    pub bandwidth_hint_mb: f32,
    pub description: Option<String>,
}

/// Lua event hook specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuaEventSpec {
    pub name: String,
    pub file: String,
    pub description: Option<String>,
    pub instruction_budget: Option<u32>, // override default
}

/// Mod registry entry containing loaded mod information
#[derive(Debug, Clone)]
pub struct ModRegistryEntry {
    pub manifest: ModManifest,
    pub enabled: bool,
    pub wasm_ops: HashMap<String, WasmOpHandle>,
    pub lua_scripts: HashMap<String, LuaScriptHandle>,
    pub content_hashes: ContentHashes,
    pub load_time: std::time::SystemTime,
}

/// Handle to a loaded WASM operation
#[derive(Debug, Clone)]
pub struct WasmOpHandle {
    pub spec: WasmOpSpec,
    pub module_hash: String,
    pub fuel_limit: u64,
    pub memory_limit: usize,
}

/// Handle to a loaded Lua script
#[derive(Debug, Clone)]
pub struct LuaScriptHandle {
    pub spec: LuaEventSpec,
    pub script_hash: String,
    pub instruction_budget: u32,
}

/// Content hashes for hot reload validation
#[derive(Debug, Clone, Default)]
pub struct ContentHashes {
    pub pipelines: Option<String>,
    pub blackswans: Option<String>,
    pub tech: Option<String>,
    pub scenarios: Option<String>,
}

/// Mod validation result
#[derive(Debug, Clone)]
pub struct ModValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub fuel_estimate: u64,
    pub memory_estimate: usize,
}

/// Hot reload transaction for atomic mod updates
#[derive(Debug, Clone)]
pub struct HotReloadTransaction {
    pub mod_id: String,
    pub old_entry: Option<ModRegistryEntry>,
    pub new_entry: ModRegistryEntry,
    pub shadow_world_result: Option<ShadowWorldResult>,
    pub status: HotReloadStatus,
}

/// Result of shadow world validation
#[derive(Debug, Clone)]
pub struct ShadowWorldResult {
    pub success: bool,
    pub kpi_deltas: KpiDeltas,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub ticks_simulated: u32,
}

/// KPI deltas from shadow world simulation
#[derive(Debug, Clone, Default)]
pub struct KpiDeltas {
    pub deadline_hit_rate_change: f32,
    pub power_draw_change: f32,
    pub bandwidth_util_change: f32,
    pub corruption_field_change: f32,
    pub heat_levels_change: Vec<f32>,
}

/// Hot reload status
#[derive(Debug, Clone, PartialEq)]
pub enum HotReloadStatus {
    Pending,
    Validating,
    ShadowWorld,
    Ready,
    Applied,
    Failed,
    Reverted,
}

/// Mod console log entry
#[derive(Debug, Clone)]
pub struct ModLogEntry {
    pub timestamp: std::time::SystemTime,
    pub mod_id: String,
    pub level: LogLevel,
    pub message: String,
    pub context: Option<HashMap<String, String>>,
}

/// Log levels for mod console
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Mod API documentation
#[derive(Debug, Clone)]
pub struct ModApiDocs {
    pub sdk_version: String,
    pub wasm_abi: WasmAbiDocs,
    pub lua_api: LuaApiDocs,
    pub capabilities: Vec<CapabilityDocs>,
    pub examples: Vec<ExampleDocs>,
}

/// WASM ABI documentation
#[derive(Debug, Clone)]
pub struct WasmAbiDocs {
    pub functions: Vec<WasmFunctionDocs>,
    pub memory_layout: String,
    pub calling_convention: String,
}

/// WASM function documentation
#[derive(Debug, Clone)]
pub struct WasmFunctionDocs {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub return_codes: Vec<ReturnCodeDocs>,
}

/// Return code documentation
#[derive(Debug, Clone)]
pub struct ReturnCodeDocs {
    pub code: i32,
    pub meaning: String,
    pub description: String,
}

/// Lua API documentation
#[derive(Debug, Clone)]
pub struct LuaApiDocs {
    pub global_functions: Vec<LuaFunctionDocs>,
    pub event_hooks: Vec<LuaEventDocs>,
    pub sandbox_limits: SandboxLimitsDocs,
}

/// Lua function documentation
#[derive(Debug, Clone)]
pub struct LuaFunctionDocs {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub requires_capability: Option<String>,
}

/// Lua event documentation
#[derive(Debug, Clone)]
pub struct LuaEventDocs {
    pub name: String,
    pub description: String,
    pub parameters: Vec<EventParameterDocs>,
    pub example: Option<String>,
}

/// Event parameter documentation
#[derive(Debug, Clone)]
pub struct EventParameterDocs {
    pub name: String,
    pub type_: String,
    pub description: String,
}

/// Capability documentation
#[derive(Debug, Clone)]
pub struct CapabilityDocs {
    pub name: String,
    pub description: String,
    pub security_implications: String,
    pub examples: Vec<String>,
}

/// Example documentation
#[derive(Debug, Clone)]
pub struct ExampleDocs {
    pub title: String,
    pub description: String,
    pub code: String,
    pub language: String,
}

/// Sandbox limits documentation
#[derive(Debug, Clone)]
pub struct SandboxLimitsDocs {
    pub wasm_fuel_limit: u64,
    pub wasm_memory_limit: usize,
    pub lua_instruction_budget: u32,
    pub lua_table_size_limit: usize,
    pub max_output_size: usize,
}

impl Default for ModManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: "0.1.0".to_string(),
            authors: Vec::new(),
            description: None,
            entrypoints: Entrypoints::default(),
            capabilities: Capabilities::default(),
            signature: None,
            requires: None,
        }
    }
}

impl ModManifest {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            ..Default::default()
        }
    }

    pub fn validate(&self) -> ModValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate ID format
        if self.id.is_empty() {
            errors.push("Mod ID cannot be empty".to_string());
        } else if !self.id.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_') {
            errors.push("Mod ID contains invalid characters".to_string());
        }

        // Validate version format (basic semver check)
        if self.version.is_empty() {
            errors.push("Version cannot be empty".to_string());
        }

        // Validate authors
        if self.authors.is_empty() {
            warnings.push("No authors specified".to_string());
        }

        // Validate entrypoints
        if self.entrypoints.wasm_ops.is_empty() && self.entrypoints.lua_events.is_empty() {
            warnings.push("No entrypoints specified".to_string());
        }

        // Estimate resource usage
        let fuel_estimate = self.entrypoints.wasm_ops.len() as u64 * 5_000_000; // 5M fuel per op
        let memory_estimate = self.entrypoints.wasm_ops.len() * 64 * 1024 * 1024; // 64MB per op

        ModValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
            fuel_estimate,
            memory_estimate,
        }
    }
}

impl ModRegistryEntry {
    pub fn new(manifest: ModManifest) -> Self {
        Self {
            manifest,
            enabled: true,
            wasm_ops: HashMap::new(),
            lua_scripts: HashMap::new(),
            content_hashes: ContentHashes::default(),
            load_time: std::time::SystemTime::now(),
        }
    }

    pub fn is_compatible_with(&self, other: &ModRegistryEntry) -> bool {
        // Check if mods are compatible (no conflicting entrypoints, etc.)
        // This is a simplified check - in reality, you'd want more sophisticated compatibility checking
        self.manifest.id != other.manifest.id
    }
}

impl ModLogEntry {
    pub fn new(mod_id: String, level: LogLevel, message: String) -> Self {
        Self {
            timestamp: std::time::SystemTime::now(),
            mod_id,
            level,
            message,
            context: None,
        }
    }

    pub fn with_context(mut self, context: HashMap<String, String>) -> Self {
        self.context = Some(context);
        self
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_manifest_creation() {
        let manifest = ModManifest::new(
            "com.test.mymod".to_string(),
            "My Test Mod".to_string(),
        );
        
        assert_eq!(manifest.id, "com.test.mymod");
        assert_eq!(manifest.name, "My Test Mod");
        assert_eq!(manifest.version, "0.1.0");
    }

    #[test]
    fn test_mod_manifest_validation() {
        let mut manifest = ModManifest::new(
            "com.test.mymod".to_string(),
            "My Test Mod".to_string(),
        );
        
        let result = manifest.validate();
        assert!(result.valid);
        assert!(result.warnings.contains(&"No entrypoints specified".to_string()));
        
        // Test invalid ID
        manifest.id = "invalid id with spaces".to_string();
        let result = manifest.validate();
        assert!(!result.valid);
        assert!(result.errors.contains(&"Mod ID contains invalid characters".to_string()));
    }

    #[test]
    fn test_mod_registry_entry() {
        let manifest = ModManifest::new(
            "com.test.mymod".to_string(),
            "My Test Mod".to_string(),
        );
        
        let entry = ModRegistryEntry::new(manifest);
        assert!(entry.enabled);
        assert_eq!(entry.manifest.id, "com.test.mymod");
    }

    #[test]
    fn test_mod_log_entry() {
        let log = ModLogEntry::new(
            "com.test.mymod".to_string(),
            LogLevel::Info,
            "Test message".to_string(),
        );
        
        assert_eq!(log.mod_id, "com.test.mymod");
        assert_eq!(log.level, LogLevel::Info);
        assert_eq!(log.message, "Test message");
    }
}
