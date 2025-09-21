# Knowledge Base: Debugging

This article provides comprehensive guidance on debugging the Colony Simulator, covering debugging techniques, tools, and best practices for troubleshooting issues.

## Overview

Debugging in the Colony Simulator involves:

- **System Debugging**: Debugging core simulation systems
- **Mod Debugging**: Debugging WASM operations and Lua scripts
- **Performance Debugging**: Debugging performance issues
- **Network Debugging**: Debugging network and communication issues
- **State Debugging**: Debugging simulation state and data

## Debugging Tools

### Built-in Debugging Tools

The Colony Simulator provides several built-in debugging tools:

1. **Debug Console**: Interactive debugging console
2. **State Inspector**: Inspect simulation state
3. **Event Logger**: Log and analyze events
4. **Performance Profiler**: Profile performance bottlenecks
5. **Memory Analyzer**: Analyze memory usage and leaks

### External Debugging Tools

Use these external tools for debugging:

1. **Rust Debugger**: `rust-gdb` or `rust-lldb`
2. **WASM Debugger**: `wasmtime` debugger
3. **Lua Debugger**: `luadebug` or `mobdebug`
4. **Network Debugger**: `wireshark` or `tcpdump`
5. **Memory Debugger**: `valgrind` or `AddressSanitizer`

## System Debugging

### Core System Debugging

Debug core simulation systems:

```rust
// Debug system execution
pub struct SystemDebugger {
    pub breakpoints: Vec<Breakpoint>,
    pub watchpoints: Vec<Watchpoint>,
    pub trace_log: TraceLog,
}

impl SystemDebugger {
    pub fn debug_system(&mut self, system_id: SystemId) {
        // Set breakpoint
        self.set_breakpoint(system_id, BreakpointType::Entry);
        
        // Set watchpoint
        self.set_watchpoint(system_id, WatchpointType::StateChange);
        
        // Enable tracing
        self.enable_tracing(system_id);
    }
    
    fn set_breakpoint(&mut self, system_id: SystemId, breakpoint_type: BreakpointType) {
        let breakpoint = Breakpoint {
            system_id,
            breakpoint_type,
            condition: None,
            action: BreakpointAction::Pause,
        };
        
        self.breakpoints.push(breakpoint);
    }
    
    fn set_watchpoint(&mut self, system_id: SystemId, watchpoint_type: WatchpointType) {
        let watchpoint = Watchpoint {
            system_id,
            watchpoint_type,
            condition: None,
            action: WatchpointAction::Log,
        };
        
        self.watchpoints.push(watchpoint);
    }
}
```

### State Debugging

Debug simulation state:

```rust
// Debug simulation state
pub struct StateDebugger {
    pub state_snapshots: Vec<StateSnapshot>,
    pub state_diff: StateDiff,
    pub state_validator: StateValidator,
}

impl StateDebugger {
    pub fn debug_state(&mut self, state: &GameState) {
        // Create state snapshot
        let snapshot = self.create_snapshot(state);
        self.state_snapshots.push(snapshot);
        
        // Compare with previous state
        if let Some(previous) = self.state_snapshots.get(self.state_snapshots.len() - 2) {
            let diff = self.compare_states(previous, &snapshot);
            self.state_diff = diff;
        }
        
        // Validate state
        self.state_validator.validate(state);
    }
    
    fn create_snapshot(&self, state: &GameState) -> StateSnapshot {
        StateSnapshot {
            tick: state.current_tick,
            entities: state.entities.clone(),
            components: state.components.clone(),
            resources: state.resources.clone(),
            timestamp: std::time::SystemTime::now(),
        }
    }
    
    fn compare_states(&self, previous: &StateSnapshot, current: &StateSnapshot) -> StateDiff {
        StateDiff {
            tick_diff: current.tick - previous.tick,
            entity_changes: self.compare_entities(&previous.entities, &current.entities),
            component_changes: self.compare_components(&previous.components, &current.components),
            resource_changes: self.compare_resources(&previous.resources, &current.resources),
        }
    }
}
```

### Event Debugging

Debug event system:

```rust
// Debug event system
pub struct EventDebugger {
    pub event_log: EventLog,
    pub event_filter: EventFilter,
    pub event_analyzer: EventAnalyzer,
}

impl EventDebugger {
    pub fn debug_event(&mut self, event: &Event) {
        // Log event
        self.event_log.log(event);
        
        // Filter events
        if self.event_filter.should_log(event) {
            self.log_event(event);
        }
        
        // Analyze event patterns
        self.event_analyzer.analyze(event);
    }
    
    fn log_event(&self, event: &Event) {
        println!("Event: {:?} at tick {}", event.event_type, event.tick);
        println!("  Data: {:?}", event.data);
        println!("  Source: {:?}", event.source);
        println!("  Target: {:?}", event.target);
    }
}
```

## Mod Debugging

### WASM Debugging

Debug WASM operations:

```rust
// Debug WASM operations
pub struct WasmDebugger {
    pub module_debugger: ModuleDebugger,
    pub execution_tracer: ExecutionTracer,
    pub memory_inspector: MemoryInspector,
}

impl WasmDebugger {
    pub fn debug_wasm_operation(&mut self, operation: &WasmOperation) {
        // Debug module
        self.module_debugger.debug_module(&operation.module);
        
        // Trace execution
        self.execution_tracer.trace_execution(operation);
        
        // Inspect memory
        self.memory_inspector.inspect_memory(&operation.module);
    }
    
    fn debug_module(&self, module: &WasmModule) {
        // Check module validity
        if !module.is_valid() {
            println!("Invalid WASM module: {:?}", module.errors);
        }
        
        // Check module imports
        for import in &module.imports {
            if !self.is_import_available(import) {
                println!("Missing import: {:?}", import);
            }
        }
        
        // Check module exports
        for export in &module.exports {
            println!("Export: {:?}", export);
        }
    }
}
```

### Lua Debugging

Debug Lua scripts:

```lua
-- Debug Lua scripts
local function debug_lua_script(script_name)
    print("Debugging Lua script: " .. script_name)
    
    -- Enable debug mode
    debug.sethook(function(event, line)
        print("Debug event: " .. event .. " at line " .. line)
    end, "l")
    
    -- Set breakpoints
    debug.sethook(function(event, line)
        if line == 42 then -- Breakpoint at line 42
            print("Breakpoint hit at line " .. line)
            debug.debug() -- Enter debugger
        end
    end, "l")
    
    -- Monitor variables
    local function monitor_variable(name, value)
        print("Variable " .. name .. " = " .. tostring(value))
    end
    
    -- Monitor function calls
    local function monitor_function_call(func_name, args)
        print("Function call: " .. func_name .. " with args: " .. tostring(args))
    end
end

-- Debug event handlers
local function debug_event_handler(event_type, handler)
    print("Debugging event handler: " .. event_type)
    
    -- Wrap handler with debugging
    local function debug_wrapper(event_data)
        print("Event handler called: " .. event_type)
        print("Event data: " .. tostring(event_data))
        
        local success, result = pcall(handler, event_data)
        if not success then
            print("Event handler error: " .. result)
        end
        
        return result
    end
    
    return debug_wrapper
end
```

## Performance Debugging

### Performance Bottleneck Debugging

Debug performance bottlenecks:

```rust
// Debug performance bottlenecks
pub struct PerformanceDebugger {
    pub profiler: Profiler,
    pub bottleneck_analyzer: BottleneckAnalyzer,
    pub optimization_suggester: OptimizationSuggester,
}

impl PerformanceDebugger {
    pub fn debug_performance(&mut self) {
        // Profile performance
        let profile = self.profiler.profile();
        
        // Analyze bottlenecks
        let bottlenecks = self.bottleneck_analyzer.analyze(&profile);
        
        // Suggest optimizations
        let suggestions = self.optimization_suggester.suggest(&bottlenecks);
        
        // Generate debug report
        self.generate_debug_report(&bottlenecks, &suggestions);
    }
    
    fn generate_debug_report(&self, bottlenecks: &[Bottleneck], suggestions: &[OptimizationSuggestion]) {
        println!("Performance Debug Report");
        println!("========================");
        
        for bottleneck in bottlenecks {
            println!("Bottleneck: {:?}", bottleneck.name);
            println!("  Impact: {:?}", bottleneck.impact);
            println!("  Location: {:?}", bottleneck.location);
            println!("  Metrics: {:?}", bottleneck.metrics);
        }
        
        for suggestion in suggestions {
            println!("Suggestion: {:?}", suggestion.description);
            println!("  Priority: {:?}", suggestion.priority);
            println!("  Expected Improvement: {:?}", suggestion.expected_improvement);
        }
    }
}
```

### Memory Debugging

Debug memory issues:

```rust
// Debug memory issues
pub struct MemoryDebugger {
    pub memory_tracker: MemoryTracker,
    pub leak_detector: LeakDetector,
    pub allocation_analyzer: AllocationAnalyzer,
}

impl MemoryDebugger {
    pub fn debug_memory(&mut self) {
        // Track memory usage
        let memory_usage = self.memory_tracker.track();
        
        // Detect memory leaks
        let leaks = self.leak_detector.detect();
        
        // Analyze allocations
        let allocation_analysis = self.allocation_analyzer.analyze();
        
        // Generate memory report
        self.generate_memory_report(&memory_usage, &leaks, &allocation_analysis);
    }
    
    fn generate_memory_report(&self, usage: &MemoryUsage, leaks: &[MemoryLeak], analysis: &AllocationAnalysis) {
        println!("Memory Debug Report");
        println!("===================");
        
        println!("Memory Usage:");
        println!("  Total: {} bytes", usage.total);
        println!("  Allocated: {} bytes", usage.allocated);
        println!("  Free: {} bytes", usage.free);
        println!("  Fragmentation: {:.2}%", usage.fragmentation);
        
        if !leaks.is_empty() {
            println!("Memory Leaks:");
            for leak in leaks {
                println!("  Leak: {} bytes at {:?}", leak.size, leak.location);
            }
        }
        
        println!("Allocation Analysis:");
        println!("  Total Allocations: {}", analysis.total_allocations);
        println!("  Average Size: {} bytes", analysis.average_size);
        println!("  Peak Usage: {} bytes", analysis.peak_usage);
    }
}
```

## Network Debugging

### Network Communication Debugging

Debug network communication:

```rust
// Debug network communication
pub struct NetworkDebugger {
    pub packet_logger: PacketLogger,
    pub connection_monitor: ConnectionMonitor,
    pub protocol_analyzer: ProtocolAnalyzer,
}

impl NetworkDebugger {
    pub fn debug_network(&mut self) {
        // Log network packets
        self.packet_logger.log_packets();
        
        // Monitor connections
        self.connection_monitor.monitor_connections();
        
        // Analyze protocol
        self.protocol_analyzer.analyze_protocol();
    }
    
    fn log_packets(&self) {
        // Log incoming packets
        self.packet_logger.log_incoming_packets();
        
        // Log outgoing packets
        self.packet_logger.log_outgoing_packets();
        
        // Log packet errors
        self.packet_logger.log_packet_errors();
    }
}
```

### Network Performance Debugging

Debug network performance:

```rust
// Debug network performance
pub struct NetworkPerformanceDebugger {
    pub latency_monitor: LatencyMonitor,
    pub throughput_monitor: ThroughputMonitor,
    pub bandwidth_analyzer: BandwidthAnalyzer,
}

impl NetworkPerformanceDebugger {
    pub fn debug_network_performance(&mut self) {
        // Monitor latency
        let latency_stats = self.latency_monitor.monitor();
        
        // Monitor throughput
        let throughput_stats = self.throughput_monitor.monitor();
        
        // Analyze bandwidth
        let bandwidth_analysis = self.bandwidth_analyzer.analyze();
        
        // Generate network performance report
        self.generate_network_performance_report(&latency_stats, &throughput_stats, &bandwidth_analysis);
    }
}
```

## Debugging Techniques

### Logging and Tracing

Use logging and tracing for debugging:

```rust
// Logging and tracing
pub struct DebugLogger {
    pub log_level: LogLevel,
    pub log_output: LogOutput,
    pub log_filter: LogFilter,
}

impl DebugLogger {
    pub fn log_debug(&self, message: &str, context: &DebugContext) {
        if self.should_log(LogLevel::Debug) {
            self.log(LogLevel::Debug, message, context);
        }
    }
    
    pub fn log_info(&self, message: &str, context: &DebugContext) {
        if self.should_log(LogLevel::Info) {
            self.log(LogLevel::Info, message, context);
        }
    }
    
    pub fn log_warning(&self, message: &str, context: &DebugContext) {
        if self.should_log(LogLevel::Warning) {
            self.log(LogLevel::Warning, message, context);
        }
    }
    
    pub fn log_error(&self, message: &str, context: &DebugContext) {
        if self.should_log(LogLevel::Error) {
            self.log(LogLevel::Error, message, context);
        }
    }
}
```

### Breakpoints and Watchpoints

Use breakpoints and watchpoints:

```rust
// Breakpoints and watchpoints
pub struct DebugBreakpoint {
    pub location: DebugLocation,
    pub condition: Option<DebugCondition>,
    pub action: BreakpointAction,
}

pub struct DebugWatchpoint {
    pub variable: String,
    pub condition: Option<DebugCondition>,
    pub action: WatchpointAction,
}

impl DebugBreakpoint {
    pub fn set_breakpoint(&mut self, location: DebugLocation, condition: Option<DebugCondition>) {
        self.location = location;
        self.condition = condition;
        self.action = BreakpointAction::Pause;
    }
    
    pub fn check_breakpoint(&self, context: &DebugContext) -> bool {
        if self.location.matches(context) {
            if let Some(condition) = &self.condition {
                return condition.evaluate(context);
            }
            return true;
        }
        false
    }
}
```

### State Inspection

Inspect simulation state:

```rust
// State inspection
pub struct StateInspector {
    pub state_snapshots: Vec<StateSnapshot>,
    pub state_comparator: StateComparator,
    pub state_validator: StateValidator,
}

impl StateInspector {
    pub fn inspect_state(&mut self, state: &GameState) {
        // Create snapshot
        let snapshot = self.create_snapshot(state);
        self.state_snapshots.push(snapshot);
        
        // Compare with previous state
        if let Some(previous) = self.state_snapshots.get(self.state_snapshots.len() - 2) {
            let diff = self.state_comparator.compare(previous, &snapshot);
            self.analyze_state_diff(&diff);
        }
        
        // Validate state
        self.state_validator.validate(state);
    }
    
    fn analyze_state_diff(&self, diff: &StateDiff) {
        if !diff.entity_changes.is_empty() {
            println!("Entity changes: {:?}", diff.entity_changes);
        }
        
        if !diff.component_changes.is_empty() {
            println!("Component changes: {:?}", diff.component_changes);
        }
        
        if !diff.resource_changes.is_empty() {
            println!("Resource changes: {:?}", diff.resource_changes);
        }
    }
}
```

## Debugging Best Practices

### General Guidelines

1. **Reproduce Issues**: Always reproduce issues before debugging
2. **Isolate Problems**: Isolate problems to specific components
3. **Use Logging**: Use comprehensive logging for debugging
4. **Test Incrementally**: Test changes incrementally
5. **Document Issues**: Document issues and solutions

### Specific Recommendations

1. **Use Debug Builds**: Use debug builds for debugging
2. **Enable Assertions**: Enable assertions for debugging
3. **Use Debug Tools**: Use appropriate debug tools
4. **Monitor Performance**: Monitor performance during debugging
5. **Clean Up**: Clean up debug code after debugging

### Debugging Anti-patterns

Avoid these debugging anti-patterns:

1. **Debugging in Production**: Don't debug in production
2. **Excessive Logging**: Don't log everything
3. **Ignoring Warnings**: Don't ignore compiler warnings
4. **Poor Error Handling**: Don't ignore error conditions
5. **Incomplete Testing**: Don't skip testing after debugging

## Troubleshooting

### Common Debugging Issues

1. **State Corruption**: Simulation state becomes corrupted
2. **Memory Leaks**: Memory usage increases over time
3. **Performance Degradation**: Performance decreases over time
4. **Network Issues**: Network communication fails
5. **Mod Failures**: Mods fail to load or execute

### Debug Techniques

1. **Binary Search**: Use binary search to isolate issues
2. **Log Analysis**: Analyze logs to find patterns
3. **State Comparison**: Compare states to find differences
4. **Performance Profiling**: Profile performance to find bottlenecks
5. **Memory Analysis**: Analyze memory usage to find leaks

## Conclusion

Debugging is an essential skill for developing and maintaining the Colony Simulator. By using the right tools and techniques, you can effectively identify and resolve issues.

Key points to remember:

- **Use Appropriate Tools**: Choose the right debugging tools for the job
- **Follow Best Practices**: Follow debugging best practices
- **Document Issues**: Document issues and solutions
- **Test Thoroughly**: Test thoroughly after debugging
- **Learn Continuously**: Continuously learn new debugging techniques

---

**Effective debugging is key to maintaining a reliable and performant simulation.** 🏭🐛
