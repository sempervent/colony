# Reference: Lua API

This document provides comprehensive reference for the Lua scripting API available to mods in the Asynchronous Colony Simulator.

## Overview

The Lua API provides a high-level, easy-to-use interface for modding the colony simulator. Lua scripts run in a sandboxed environment with strict capability gating to ensure security and stability.

## Global Objects

### `colony`

The main global object providing access to all colony simulator functionality.

```lua
-- Access the colony API
local current_tick = colony.get_current_tick()
colony.log_info("Current tick: " .. current_tick)
```

## Core API Functions

### Time and Simulation

#### `colony.get_current_tick() -> number`

Returns the current simulation tick number.

**Capability Required:** `sim_time`

**Returns:**
- `number`: Current tick number (0-based)

**Example:**
```lua
local tick = colony.get_current_tick()
if tick % 100 == 0 then
    colony.log_info("Reached tick " .. tick)
end
```

#### `colony.get_simulation_time() -> number`

Returns the current simulation time in seconds.

**Capability Required:** `sim_time`

**Returns:**
- `number`: Simulation time in seconds

### Logging

#### `colony.log_debug(message)`
#### `colony.log_info(message)`
#### `colony.log_warn(message)`
#### `colony.log_error(message)`

Log messages to the game console with different severity levels.

**Capability Required:** `log_message`

**Parameters:**
- `message` (string): The message to log

**Example:**
```lua
colony.log_info("Mod initialized successfully")
colony.log_warn("Resource levels are low")
colony.log_error("Critical system failure detected")
```

### Job Management

#### `colony.enqueue_job(pipeline_id, options?) -> number`

Enqueue a new job for processing.

**Capability Required:** `enqueue_job`

**Parameters:**
- `pipeline_id` (string): ID of the pipeline to execute
- `options` (table, optional): Job options
  - `priority` (string): Job priority ("low", "normal", "high", "critical")
  - `deadline_ticks` (number): Custom deadline in ticks
  - `metadata` (table): Custom metadata for the job

**Returns:**
- `number`: Job ID (0 if failed)

**Example:**
```lua
local job_id = colony.enqueue_job("video_process", {
    priority = "high",
    deadline_ticks = 100,
    metadata = { source = "camera_1" }
})

if job_id > 0 then
    colony.log_info("Enqueued job " .. job_id)
else
    colony.log_error("Failed to enqueue job")
end
```

#### `colony.get_job_status(job_id) -> table`

Get the current status of a job.

**Capability Required:** `read_job_status`

**Parameters:**
- `job_id` (number): The job ID

**Returns:**
- `table`: Job status information
  - `status` (string): "pending", "running", "completed", "failed"
  - `progress` (number): Completion percentage (0-100)
  - `current_op` (string): Current operation being executed
  - `worker_id` (number): ID of assigned worker

**Example:**
```lua
local status = colony.get_job_status(job_id)
colony.log_info("Job " .. job_id .. " is " .. status.status .. 
                " (" .. status.progress .. "% complete)")
```

### Key Performance Indicators (KPIs)

#### `colony.get_kpi(kpi_name) -> number`

Get the current value of a Key Performance Indicator.

**Capability Required:** `read_kpis`

**Parameters:**
- `kpi_name` (string): Name of the KPI

**Returns:**
- `number`: Current KPI value

**Common KPIs:**
- `"Power"`: Current power level (kW)
- `"Heat"`: Current heat level (°C)
- `"Bandwidth"`: Current bandwidth usage (Gbps)
- `"CorruptionField"`: Current corruption field value
- `"DeadlineHitRate"`: Percentage of deadlines met
- `"WorkerUtilization"`: Percentage of workers active
- `"JobQueueLength"`: Number of jobs in queue

**Example:**
```lua
local power = colony.get_kpi("Power")
local heat = colony.get_kpi("Heat")

if power < 1000 then
    colony.log_warn("Power critically low: " .. power .. " kW")
end

if heat > 80 then
    colony.log_warn("Heat critically high: " .. heat .. "°C")
end
```

#### `colony.get_all_kpis() -> table`

Get all available KPIs as a table.

**Capability Required:** `read_kpis`

**Returns:**
- `table`: Dictionary of KPI names to values

**Example:**
```lua
local kpis = colony.get_all_kpis()
for name, value in pairs(kpis) do
    colony.log_info(name .. ": " .. value)
end
```

### Random Number Generation

#### `colony.random() -> number`

Generate a deterministic random number between 0 and 1.

**Capability Required:** `deterministic_rng`

**Returns:**
- `number`: Random value in range [0, 1)

**Example:**
```lua
local chance = colony.random()
if chance < 0.1 then
    colony.log_info("Lucky event occurred!")
end
```

#### `colony.random_int(min, max) -> number`

Generate a deterministic random integer in the specified range.

**Capability Required:** `deterministic_rng`

**Parameters:**
- `min` (number): Minimum value (inclusive)
- `max` (number): Maximum value (inclusive)

**Returns:**
- `number`: Random integer in range [min, max]

**Example:**
```lua
local worker_count = colony.random_int(1, 10)
colony.log_info("Selected " .. worker_count .. " workers")
```

### Event System

#### `colony.on_event(event_type, callback)`

Register a callback for a specific event type.

**Capability Required:** `listen_events`

**Parameters:**
- `event_type` (string): Type of event to listen for
- `callback` (function): Function to call when event occurs

**Event Types:**
- `"job_completed"`: Job finished processing
- `"job_failed"`: Job failed with error
- `"black_swan_triggered"`: Black Swan event occurred
- `"tech_unlocked"`: Technology was unlocked
- `"resource_critical"`: Resource level became critical
- `"worker_stuck"`: Worker became stuck due to fault

**Example:**
```lua
colony.on_event("job_completed", function(job_id, pipeline_id, status)
    colony.log_info("Job " .. job_id .. " (" .. pipeline_id .. ") completed: " .. status)
    
    -- Trigger follow-up action
    if status == "success" then
        colony.enqueue_job("cleanup_pipeline")
    end
end)

colony.on_event("black_swan_triggered", function(swan_type, severity)
    colony.log_error("Black Swan triggered: " .. swan_type .. " (severity: " .. severity .. ")")
    
    -- Emergency response
    colony.enqueue_job("emergency_shutdown", { priority = "critical" })
end)
```

### Configuration Access

#### `colony.get_config(key) -> any`

Get a configuration value.

**Capability Required:** `read_config`

**Parameters:**
- `key` (string): Configuration key (dot notation supported)

**Returns:**
- `any`: Configuration value

**Example:**
```lua
local max_workers = colony.get_config("workyard.cpu.max_workers")
local tick_scale = colony.get_config("simulation.tick_scale")

colony.log_info("Max CPU workers: " .. max_workers)
colony.log_info("Tick scale: " .. tick_scale)
```

### Mod State Management

#### `colony.save_state(key, value)`

Save a value to persistent mod state.

**Capability Required:** `persistent_state`

**Parameters:**
- `key` (string): State key
- `value` (any): Value to save (must be serializable)

**Example:**
```lua
colony.save_state("jobs_processed", 42)
colony.save_state("last_cleanup_tick", colony.get_current_tick())
```

#### `colony.load_state(key, default?) -> any`

Load a value from persistent mod state.

**Capability Required:** `persistent_state`

**Parameters:**
- `key` (string): State key
- `default` (any, optional): Default value if key not found

**Returns:**
- `any`: Saved value or default

**Example:**
```lua
local jobs_processed = colony.load_state("jobs_processed", 0)
local last_cleanup = colony.load_state("last_cleanup_tick", 0)

colony.log_info("Processed " .. jobs_processed .. " jobs so far")
```

## Utility Functions

### `colony.format_number(number, decimals?) -> string`

Format a number with specified decimal places.

**Parameters:**
- `number` (number): Number to format
- `decimals` (number, optional): Number of decimal places (default: 2)

**Returns:**
- `string`: Formatted number string

**Example:**
```lua
local power = colony.get_kpi("Power")
local formatted = colony.format_number(power, 1)
colony.log_info("Power: " .. formatted .. " kW")
```

### `colony.format_percentage(value, decimals?) -> string`

Format a number as a percentage.

**Parameters:**
- `value` (number): Value to format (0-1 range)
- `decimals` (number, optional): Number of decimal places (default: 1)

**Returns:**
- `string`: Formatted percentage string

**Example:**
```lua
local hit_rate = colony.get_kpi("DeadlineHitRate") / 100
local formatted = colony.format_percentage(hit_rate, 1)
colony.log_info("Deadline hit rate: " .. formatted)
```

## Error Handling

### Exception Handling

Lua scripts should use `pcall` for robust error handling:

```lua
local success, result = pcall(function()
    local job_id = colony.enqueue_job("invalid_pipeline")
    return job_id
end)

if not success then
    colony.log_error("Failed to enqueue job: " .. result)
else
    colony.log_info("Job enqueued successfully: " .. result)
end
```

### Validation

Always validate inputs and handle edge cases:

```lua
function safe_enqueue_job(pipeline_id, options)
    if type(pipeline_id) ~= "string" or pipeline_id == "" then
        colony.log_error("Invalid pipeline ID")
        return 0
    end
    
    if options and type(options) ~= "table" then
        colony.log_error("Options must be a table")
        return 0
    end
    
    return colony.enqueue_job(pipeline_id, options)
end
```

## Best Practices

### Performance

- Cache frequently accessed values
- Avoid complex computations in event callbacks
- Use appropriate data structures
- Minimize string concatenation in loops

```lua
-- Good: Cache KPI values
local power_threshold = 1000
local last_power_check = 0

function check_power()
    local current_tick = colony.get_current_tick()
    if current_tick - last_power_check > 10 then
        local power = colony.get_kpi("Power")
        if power < power_threshold then
            colony.log_warn("Power low: " .. power)
        end
        last_power_check = current_tick
    end
end
```

### Modularity

- Split large scripts into multiple files
- Use descriptive function names
- Document complex logic
- Implement proper error handling

```lua
-- mod_utils.lua
local mod_utils = {}

function mod_utils.format_kpi(name, value, unit)
    return name .. ": " .. colony.format_number(value, 1) .. " " .. unit
end

function mod_utils.is_resource_critical(resource_name, threshold)
    local value = colony.get_kpi(resource_name)
    return value < threshold
end

return mod_utils
```

### State Management

- Use persistent state for important data
- Implement proper initialization
- Handle hot reload gracefully

```lua
-- Initialize mod state
local mod_state = {
    initialized = false,
    jobs_processed = 0,
    last_cleanup = 0
}

function initialize_mod()
    if not mod_state.initialized then
        mod_state.jobs_processed = colony.load_state("jobs_processed", 0)
        mod_state.last_cleanup = colony.load_state("last_cleanup", 0)
        mod_state.initialized = true
        colony.log_info("Mod initialized")
    end
end

-- Save state periodically
function save_mod_state()
    colony.save_state("jobs_processed", mod_state.jobs_processed)
    colony.save_state("last_cleanup", mod_state.last_cleanup)
end
```

This Lua API provides a powerful yet safe interface for creating dynamic, event-driven mods that can significantly enhance the colony simulator experience.
