# Modding: Lua Events

Lua events provide flexible, event-driven scripting capabilities for the Colony Simulator. This guide explains how to create, implement, and integrate Lua event handlers into your mods.

## Overview

Lua events offer:

- **Event-Driven Programming**: Respond to simulation events
- **Flexible Scripting**: Easy to write and modify
- **Sandboxed Execution**: Secure execution environment
- **Hot Reloading**: Update scripts without restarting
- **Rich API**: Access to simulation state and systems
- **Easy Integration**: Simple integration with existing mods

## Lua Event System

### Event Types

The Colony Simulator supports various event types:

```lua
-- Event types available in Lua
local EventTypes = {
    SIMULATION_START = "simulation_start",
    SIMULATION_END = "simulation_end",
    TICK_START = "tick_start",
    TICK_END = "tick_end",
    JOB_CREATED = "job_created",
    JOB_COMPLETED = "job_completed",
    JOB_FAILED = "job_failed",
    WORKER_CREATED = "worker_created",
    WORKER_DESTROYED = "worker_destroyed",
    FAULT_OCCURRED = "fault_occurred",
    FAULT_RECOVERED = "fault_recovered",
    RESOURCE_CHANGED = "resource_changed",
    RESEARCH_COMPLETED = "research_completed",
    RITUAL_PERFORMED = "ritual_performed",
    MUTATION_APPLIED = "mutation_applied",
    BLACK_SWAN_TRIGGERED = "black_swan_triggered",
    VICTORY_CONDITION_MET = "victory_condition_met",
    LOSS_CONDITION_MET = "loss_condition_met",
    MOD_LOADED = "mod_loaded",
    MOD_UNLOADED = "mod_unloaded",
}
```

### Event Handler Structure

```lua
-- Basic event handler structure
local function on_event(event_type, event_data)
    -- Event handling logic here
    print("Event received: " .. event_type)
    
    -- Access event data
    if event_data then
        print("Event data: " .. tostring(event_data))
    end
end

-- Register event handler
colony.events.register("tick_start", on_event)
```

## Creating Lua Event Handlers

### Basic Event Handler

```lua
-- Basic event handler example
local function on_tick_start(event_data)
    -- This function is called at the start of each tick
    local current_tick = event_data.tick
    local simulation_time = event_data.simulation_time
    
    -- Log tick information
    print("Tick started: " .. current_tick .. " at time " .. simulation_time)
    
    -- Perform tick-based operations
    if current_tick % 100 == 0 then
        print("Reached tick milestone: " .. current_tick)
    end
end

-- Register the event handler
colony.events.register("tick_start", on_tick_start)
```

### Job Event Handlers

```lua
-- Job event handlers
local function on_job_created(event_data)
    local job = event_data.job
    local pipeline = event_data.pipeline
    
    print("New job created: " .. job.id)
    print("Pipeline: " .. pipeline.name)
    print("Priority: " .. job.priority)
    
    -- Track job statistics
    local stats = colony.stats.get_job_stats()
    stats.total_jobs = stats.total_jobs + 1
    
    -- Apply custom job modifications
    if pipeline.name == "video_stream" then
        -- Boost priority for video streams
        job.priority = math.min(job.priority + 1, 10)
        print("Boosted priority for video stream job")
    end
end

local function on_job_completed(event_data)
    local job = event_data.job
    local completion_time = event_data.completion_time
    local success = event_data.success
    
    print("Job completed: " .. job.id)
    print("Success: " .. tostring(success))
    print("Completion time: " .. completion_time)
    
    -- Update statistics
    local stats = colony.stats.get_job_stats()
    if success then
        stats.completed_jobs = stats.completed_jobs + 1
    else
        stats.failed_jobs = stats.failed_jobs + 1
    end
    
    -- Calculate performance metrics
    local deadline_hit = completion_time <= job.deadline
    if deadline_hit then
        stats.deadline_hits = stats.deadline_hits + 1
    else
        stats.deadline_misses = stats.deadline_misses + 1
    end
end

-- Register job event handlers
colony.events.register("job_created", on_job_created)
colony.events.register("job_completed", on_job_completed)
```

### Worker Event Handlers

```lua
-- Worker event handlers
local function on_worker_created(event_data)
    local worker = event_data.worker
    local workyard = event_data.workyard
    
    print("New worker created: " .. worker.id)
    print("Workyard: " .. workyard.name)
    print("Skills: CPU=" .. worker.cpu_skill .. ", GPU=" .. worker.gpu_skill .. ", IO=" .. worker.io_skill)
    
    -- Initialize worker tracking
    local worker_stats = colony.stats.get_worker_stats(worker.id)
    worker_stats.created_at = colony.time.get_current_tick()
    worker_stats.total_operations = 0
    worker_stats.faults = 0
end

local function on_worker_destroyed(event_data)
    local worker = event_data.worker
    local reason = event_data.reason
    
    print("Worker destroyed: " .. worker.id)
    print("Reason: " .. reason)
    
    -- Clean up worker statistics
    colony.stats.remove_worker_stats(worker.id)
end

-- Register worker event handlers
colony.events.register("worker_created", on_worker_created)
colony.events.register("worker_destroyed", on_worker_destroyed)
```

### Fault Event Handlers

```lua
-- Fault event handlers
local function on_fault_occurred(event_data)
    local fault = event_data.fault
    local component = event_data.component
    local severity = event_data.severity
    
    print("Fault occurred: " .. fault.id)
    print("Component: " .. component.id)
    print("Severity: " .. severity)
    print("Type: " .. fault.type)
    
    -- Track fault statistics
    local stats = colony.stats.get_fault_stats()
    stats.total_faults = stats.total_faults + 1
    
    -- Categorize fault by severity
    if severity == "critical" then
        stats.critical_faults = stats.critical_faults + 1
    elseif severity == "major" then
        stats.major_faults = stats.major_faults + 1
    elseif severity == "minor" then
        stats.minor_faults = stats.minor_faults + 1
    end
    
    -- Apply fault mitigation strategies
    if fault.type == "soft" then
        -- Soft faults can be retried
        print("Soft fault detected, will retry")
    elseif fault.type == "sticky" then
        -- Sticky faults require intervention
        print("Sticky fault detected, intervention required")
        
        -- Attempt automatic recovery
        local recovery_success = colony.faults.attempt_recovery(fault.id)
        if recovery_success then
            print("Automatic recovery successful")
        else
            print("Automatic recovery failed, manual intervention needed")
        end
    end
end

local function on_fault_recovered(event_data)
    local fault = event_data.fault
    local recovery_time = event_data.recovery_time
    local recovery_method = event_data.recovery_method
    
    print("Fault recovered: " .. fault.id)
    print("Recovery time: " .. recovery_time)
    print("Recovery method: " .. recovery_method)
    
    -- Update fault statistics
    local stats = colony.stats.get_fault_stats()
    stats.recovered_faults = stats.recovered_faults + 1
    stats.total_recovery_time = stats.total_recovery_time + recovery_time
end

-- Register fault event handlers
colony.events.register("fault_occurred", on_fault_occurred)
colony.events.register("fault_recovered", on_fault_recovered)
```

## Resource Management Events

### Resource Change Handlers

```lua
-- Resource change event handlers
local function on_resource_changed(event_data)
    local resource_type = event_data.resource_type
    local old_value = event_data.old_value
    local new_value = event_data.new_value
    local change_amount = new_value - old_value
    
    print("Resource changed: " .. resource_type)
    print("Old value: " .. old_value)
    print("New value: " .. new_value)
    print("Change: " .. change_amount)
    
    -- Track resource trends
    local trends = colony.stats.get_resource_trends()
    if not trends[resource_type] then
        trends[resource_type] = {}
    end
    
    table.insert(trends[resource_type], {
        tick = colony.time.get_current_tick(),
        value = new_value,
        change = change_amount
    })
    
    -- Keep only recent data (last 1000 ticks)
    if #trends[resource_type] > 1000 then
        table.remove(trends[resource_type], 1)
    end
    
    -- Apply resource-based logic
    if resource_type == "power" and new_value < 100 then
        print("Power critically low, activating emergency protocols")
        colony.resources.activate_emergency_power()
    elseif resource_type == "heat" and new_value > 80 then
        print("Heat critically high, activating cooling systems")
        colony.resources.activate_emergency_cooling()
    end
end

-- Register resource change handler
colony.events.register("resource_changed", on_resource_changed)
```

## Research and Ritual Events

### Research Event Handlers

```lua
-- Research event handlers
local function on_research_completed(event_data)
    local research = event_data.research
    local research_time = event_data.research_time
    local effects = event_data.effects
    
    print("Research completed: " .. research.name)
    print("Research time: " .. research_time)
    print("Effects: " .. tostring(effects))
    
    -- Track research progress
    local stats = colony.stats.get_research_stats()
    stats.completed_research = stats.completed_research + 1
    stats.total_research_time = stats.total_research_time + research_time
    
    -- Apply research effects
    for _, effect in ipairs(effects) do
        if effect.type == "unlock_operation" then
            print("Unlocked new operation: " .. effect.operation)
        elseif effect.type == "improve_efficiency" then
            print("Improved efficiency by: " .. effect.amount)
        elseif effect.type == "reduce_costs" then
            print("Reduced costs by: " .. effect.amount)
        end
    end
end

-- Register research event handler
colony.events.register("research_completed", on_research_completed)
```

### Ritual Event Handlers

```lua
-- Ritual event handlers
local function on_ritual_performed(event_data)
    local ritual = event_data.ritual
    local effects = event_data.effects
    local duration = event_data.duration
    
    print("Ritual performed: " .. ritual.name)
    print("Duration: " .. duration)
    print("Effects: " .. tostring(effects))
    
    -- Track ritual usage
    local stats = colony.stats.get_ritual_stats()
    stats.performed_rituals = stats.performed_rituals + 1
    
    -- Apply ritual effects
    for _, effect in ipairs(effects) do
        if effect.type == "boost_resource_generation" then
            print("Boosted resource generation by: " .. effect.multiplier)
        elseif effect.type == "reduce_fault_rate" then
            print("Reduced fault rate by: " .. effect.reduction)
        elseif effect.type == "improve_efficiency" then
            print("Improved efficiency by: " .. effect.amount)
        end
    end
end

-- Register ritual event handler
colony.events.register("ritual_performed", on_ritual_performed)
```

## Black Swan Event Handlers

### Black Swan Event Handling

```lua
-- Black Swan event handlers
local function on_black_swan_triggered(event_data)
    local black_swan = event_data.black_swan
    local severity = event_data.severity
    local effects = event_data.effects
    
    print("Black Swan triggered: " .. black_swan.name)
    print("Severity: " .. severity)
    print("Effects: " .. tostring(effects))
    
    -- Track Black Swan events
    local stats = colony.stats.get_black_swan_stats()
    stats.triggered_swans = stats.triggered_swans + 1
    
    -- Apply Black Swan effects
    for _, effect in ipairs(effects) do
        if effect.type == "resource_drain" then
            print("Resource drained: " .. effect.resource .. " by " .. effect.amount)
        elseif effect.type == "fault_cascade" then
            print("Fault cascade triggered with " .. effect.fault_count .. " faults")
        elseif effect.type == "system_disruption" then
            print("System disruption: " .. effect.disruption_type)
        end
    end
    
    -- Implement Black Swan response strategies
    if severity == "catastrophic" then
        print("Catastrophic Black Swan detected, activating emergency protocols")
        colony.emergency.activate_emergency_protocols()
    elseif severity == "major" then
        print("Major Black Swan detected, implementing mitigation strategies")
        colony.mitigation.implement_mitigation_strategies()
    end
end

-- Register Black Swan event handler
colony.events.register("black_swan_triggered", on_black_swan_triggered)
```

## Victory and Loss Event Handlers

### Victory and Loss Events

```lua
-- Victory and loss event handlers
local function on_victory_condition_met(event_data)
    local condition = event_data.condition
    local progress = event_data.progress
    
    print("Victory condition met: " .. condition.name)
    print("Progress: " .. progress)
    
    -- Track victory progress
    local stats = colony.stats.get_victory_stats()
    stats.conditions_met = stats.conditions_met + 1
    
    -- Check if all victory conditions are met
    if stats.conditions_met >= stats.total_conditions then
        print("All victory conditions met! Victory achieved!")
        colony.victory.achieve_victory()
    end
end

local function on_loss_condition_met(event_data)
    local condition = event_data.condition
    local severity = event_data.severity
    
    print("Loss condition met: " .. condition.name)
    print("Severity: " .. severity)
    
    -- Track loss conditions
    local stats = colony.stats.get_loss_stats()
    stats.conditions_met = stats.conditions_met + 1
    
    -- Check if loss is inevitable
    if severity == "critical" or severity == "catastrophic" then
        print("Critical loss condition met! Game over!")
        colony.loss.trigger_game_over()
    end
end

-- Register victory and loss event handlers
colony.events.register("victory_condition_met", on_victory_condition_met)
colony.events.register("loss_condition_met", on_loss_condition_met)
```

## Mod Lifecycle Events

### Mod Loading and Unloading

```lua
-- Mod lifecycle event handlers
local function on_mod_loaded(event_data)
    local mod = event_data.mod
    local version = event_data.version
    
    print("Mod loaded: " .. mod.name)
    print("Version: " .. version)
    
    -- Initialize mod-specific data
    local mod_data = colony.mods.get_mod_data(mod.id)
    mod_data.loaded_at = colony.time.get_current_tick()
    mod_data.version = version
    
    -- Register mod-specific event handlers
    if mod.name == "My Custom Mod" then
        colony.events.register("tick_start", my_custom_tick_handler)
    end
end

local function on_mod_unloaded(event_data)
    local mod = event_data.mod
    local reason = event_data.reason
    
    print("Mod unloaded: " .. mod.name)
    print("Reason: " .. reason)
    
    -- Clean up mod-specific data
    colony.mods.remove_mod_data(mod.id)
    
    -- Unregister mod-specific event handlers
    if mod.name == "My Custom Mod" then
        colony.events.unregister("tick_start", my_custom_tick_handler)
    end
end

-- Register mod lifecycle event handlers
colony.events.register("mod_loaded", on_mod_loaded)
colony.events.register("mod_unloaded", on_mod_unloaded)
```

## Advanced Event Handling

### Event Filtering and Prioritization

```lua
-- Event filtering and prioritization
local function create_event_filter(event_type, filter_func)
    return function(event_data)
        if filter_func(event_data) then
            -- Process the event
            print("Filtered event processed: " .. event_type)
        else
            -- Skip the event
            print("Event filtered out: " .. event_type)
        end
    end
end

-- Example: Only process high-priority jobs
local function is_high_priority_job(event_data)
    return event_data.job and event_data.job.priority >= 8
end

local high_priority_job_handler = create_event_filter("job_created", is_high_priority_job)
colony.events.register("job_created", high_priority_job_handler)

-- Example: Only process critical faults
local function is_critical_fault(event_data)
    return event_data.fault and event_data.fault.severity == "critical"
end

local critical_fault_handler = create_event_filter("fault_occurred", is_critical_fault)
colony.events.register("fault_occurred", critical_fault_handler)
```

### Event Chaining and Composition

```lua
-- Event chaining and composition
local function create_event_chain(handlers)
    return function(event_data)
        for _, handler in ipairs(handlers) do
            local success, result = pcall(handler, event_data)
            if not success then
                print("Error in event handler: " .. result)
                break
            end
        end
    end
end

-- Example: Chain multiple job event handlers
local job_handlers = {
    function(event_data)
        print("Job created: " .. event_data.job.id)
    end,
    function(event_data)
        -- Update statistics
        local stats = colony.stats.get_job_stats()
        stats.total_jobs = stats.total_jobs + 1
    end,
    function(event_data)
        -- Apply custom logic
        if event_data.job.priority >= 8 then
            print("High priority job detected")
        end
    end
}

local chained_job_handler = create_event_chain(job_handlers)
colony.events.register("job_created", chained_job_handler)
```

### Event State Management

```lua
-- Event state management
local EventState = {
    handlers = {},
    filters = {},
    priorities = {},
    enabled = true
}

function EventState.register_handler(event_type, handler, priority)
    priority = priority or 0
    
    if not EventState.handlers[event_type] then
        EventState.handlers[event_type] = {}
    end
    
    table.insert(EventState.handlers[event_type], {
        handler = handler,
        priority = priority
    })
    
    -- Sort by priority (higher priority first)
    table.sort(EventState.handlers[event_type], function(a, b)
        return a.priority > b.priority
    end)
end

function EventState.process_event(event_type, event_data)
    if not EventState.enabled then
        return
    end
    
    local handlers = EventState.handlers[event_type]
    if not handlers then
        return
    end
    
    for _, handler_info in ipairs(handlers) do
        local success, result = pcall(handler_info.handler, event_data)
        if not success then
            print("Error in event handler: " .. result)
        end
    end
end

-- Usage example
EventState.register_handler("tick_start", function(event_data)
    print("High priority tick handler")
end, 10)

EventState.register_handler("tick_start", function(event_data)
    print("Low priority tick handler")
end, 1)
```

## Error Handling and Debugging

### Error Handling in Event Handlers

```lua
-- Error handling in event handlers
local function safe_event_handler(handler)
    return function(event_data)
        local success, result = pcall(handler, event_data)
        if not success then
            print("Error in event handler: " .. result)
            print("Event data: " .. tostring(event_data))
            
            -- Log error for debugging
            colony.debug.log_error("Event handler error", {
                error = result,
                event_data = event_data,
                handler = tostring(handler)
            })
        end
    end
end

-- Usage example
local function my_event_handler(event_data)
    -- This might throw an error
    local result = event_data.some_field.some_nested_field
    print("Result: " .. result)
end

-- Wrap with error handling
local safe_handler = safe_event_handler(my_event_handler)
colony.events.register("some_event", safe_handler)
```

### Debugging Event Handlers

```lua
-- Debugging utilities for event handlers
local function debug_event_handler(event_type, handler)
    return function(event_data)
        print("DEBUG: Event " .. event_type .. " triggered")
        print("DEBUG: Event data: " .. tostring(event_data))
        
        local start_time = os.clock()
        local success, result = pcall(handler, event_data)
        local end_time = os.clock()
        
        if success then
            print("DEBUG: Handler completed in " .. (end_time - start_time) .. " seconds")
        else
            print("DEBUG: Handler failed with error: " .. result)
        end
    end
end

-- Usage example
local function my_debug_handler(event_data)
    print("Processing event: " .. event_data.type)
end

local debug_handler = debug_event_handler("tick_start", my_debug_handler)
colony.events.register("tick_start", debug_handler)
```

## Best Practices

### Design Guidelines

1. **Single Responsibility**: Each event handler should have a single responsibility
2. **Error Handling**: Always handle errors gracefully
3. **Performance**: Keep event handlers efficient
4. **State Management**: Manage state carefully in event handlers
5. **Documentation**: Document your event handlers clearly

### Performance Considerations

1. **Efficient Processing**: Keep event handlers fast
2. **Memory Management**: Avoid memory leaks in event handlers
3. **Event Filtering**: Use event filtering to reduce unnecessary processing
4. **Batch Processing**: Process multiple events together when possible
5. **Caching**: Cache frequently used data in event handlers

### Security Considerations

1. **Input Validation**: Validate event data before processing
2. **Sandboxing**: Work within the Lua sandbox constraints
3. **Resource Limits**: Respect resource limits in event handlers
4. **Error Information**: Don't leak sensitive information in errors
5. **Access Control**: Control access to sensitive operations

---

**Lua events provide powerful, flexible scripting capabilities for the Colony Simulator. Understanding these concepts is key to creating effective mods.** 🏭📜
