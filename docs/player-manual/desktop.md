# Desktop Application - Player Guide

## Overview

Compute Colony Desktop is an interactive colony management simulation where you control a computational infrastructure. Your goal is to manage power, bandwidth, workers, and simulators to maintain a stable and efficient colony.

## Getting Started

### Running the Game

```bash
cd /Users/joshuagrant/dev/colony
cargo run --bin colony-desktop
```

The game will start in **Main Menu** mode, displaying your colony's current status and available controls.

## Game Controls

### Main Menu Controls
- **SPACE** - Start the simulation
- **S** - Stop all simulators
- **M** - Run maintenance
- **1** - Switch to FCFS (First-Come-First-Served) Scheduler
- **2** - Switch to SJF (Shortest Job First) Scheduler
- **3** - Switch to EDF (Earliest Deadline First) Scheduler

### In-Game Controls
- **P** - Pause the simulation
- **S** - Stop all simulators
- **M** - Run maintenance
- **1/2/3** - Switch between schedulers
- **U** - Toggle UDP Simulator on/off
- **H** - Toggle HTTP Simulator on/off

### Paused Game Controls
- **R** - Resume the simulation
- **S** - Stop all simulators
- **M** - Run maintenance
- **1/2/3** - Switch between schedulers

## Understanding Your Colony

### Key Metrics

The UI displays real-time information about your colony:

#### Power Management
- **Power Usage**: Current power draw vs. maximum capacity (in kW)
- **Power Capacity**: Total available power for your colony
- **Power Efficiency**: Lower power usage means better efficiency

#### Bandwidth Utilization
- **Bandwidth Usage**: Percentage of available bandwidth being used
- **Bandwidth Total**: Total available bandwidth (in Gbps)
- **Network Health**: High utilization can cause delays

#### Corruption Levels
- **Corruption Field**: Percentage of system corruption
- **Corruption Impact**: High corruption affects system reliability
- **Maintenance**: Run maintenance (M key) to reduce corruption

#### Time Management
- **Simulation Time**: Current tick/time in the simulation
- **Uptime Target**: Your colony's target uptime in days

## Simulator Management

### UDP Simulator (U Key)
- **Purpose**: Handles UDP network traffic simulation
- **Impact**: Affects bandwidth utilization and network performance
- **Strategy**: Toggle on/off based on network load requirements

### HTTP Simulator (H Key)
- **Purpose**: Handles HTTP web traffic simulation
- **Impact**: Affects bandwidth utilization and web service performance
- **Strategy**: Toggle on/off based on web service requirements

### Simulator Strategy
- **Start Simulators**: When you need to test network performance
- **Stop Simulators**: When you want to reduce bandwidth usage
- **Monitor Impact**: Watch how simulators affect your colony metrics

## Scheduler Management

### FCFS (First-Come-First-Served) - Key 1
- **Best For**: Simple, predictable job processing
- **Advantages**: Fair, easy to understand
- **Disadvantages**: May not be optimal for time-sensitive tasks

### SJF (Shortest Job First) - Key 2
- **Best For**: Maximizing throughput with short jobs
- **Advantages**: Efficient for mixed job sizes
- **Disadvantages**: Long jobs may wait indefinitely

### EDF (Earliest Deadline First) - Key 3
- **Best For**: Time-critical applications
- **Advantages**: Meets deadlines effectively
- **Disadvantages**: Complex scheduling, may cause starvation

### Scheduler Strategy
- **Switch Schedulers**: Based on your current workload
- **Monitor Performance**: Watch how different schedulers affect efficiency
- **Adapt to Conditions**: Change schedulers as colony needs evolve

## Maintenance and Health

### Running Maintenance (M Key)
- **Purpose**: Reduces corruption and improves system health
- **Frequency**: Run when corruption levels are high
- **Impact**: Temporarily reduces performance but improves long-term stability

### System Health Monitoring
- **Power**: Keep power usage below capacity
- **Bandwidth**: Monitor utilization to avoid bottlenecks
- **Corruption**: Keep corruption levels low through maintenance

## Game Strategies

### Early Game
1. **Start the simulation** with SPACE
2. **Monitor baseline metrics** to understand your colony
3. **Run initial maintenance** to ensure clean start
4. **Choose appropriate scheduler** based on expected workload

### Mid Game
1. **Activate simulators** (U, H) to test system under load
2. **Monitor performance** as simulators affect metrics
3. **Switch schedulers** to optimize for current conditions
4. **Run maintenance** when corruption builds up

### Advanced Play
1. **Balance simulators** - use them strategically to test limits
2. **Optimize scheduling** - switch between schedulers for different scenarios
3. **Manage power efficiency** - keep usage optimal
4. **Maintain system health** - prevent corruption buildup

## Troubleshooting

### Common Issues

#### High Power Usage
- **Solution**: Stop unnecessary simulators
- **Prevention**: Monitor power capacity vs. usage

#### High Bandwidth Utilization
- **Solution**: Toggle off simulators (U, H)
- **Prevention**: Balance simulator usage

#### High Corruption
- **Solution**: Run maintenance (M key)
- **Prevention**: Regular maintenance schedule

#### Poor Performance
- **Solution**: Switch to appropriate scheduler
- **Prevention**: Match scheduler to workload type

### Performance Tips

1. **Monitor Metrics**: Keep an eye on all colony metrics
2. **Proactive Maintenance**: Don't wait for high corruption
3. **Strategic Simulator Use**: Use simulators to test system limits
4. **Scheduler Optimization**: Match scheduler to your needs
5. **Power Management**: Keep power usage efficient

## Game States

### Main Menu
- **Purpose**: Initial setup and colony overview
- **Actions**: Start game, configure initial settings
- **Status**: Colony metrics displayed but simulation not running

### In Game
- **Purpose**: Active simulation with full controls
- **Actions**: All controls available, real-time monitoring
- **Status**: Colony actively processing and responding to changes

### Paused
- **Purpose**: Temporary halt for planning or analysis
- **Actions**: Limited controls, can resume or make adjustments
- **Status**: Simulation paused, metrics frozen at pause time

## Advanced Features

### Real-Time Monitoring
- All metrics update in real-time
- Immediate feedback on control actions
- Visual indicators for system health

### Interactive Control
- Immediate response to key presses
- Toggle simulators on/off instantly
- Switch schedulers without restart

### System Integration
- All controls affect actual simulation systems
- Changes propagate through the entire colony
- Realistic system behavior and responses

## Getting Help

### In-Game Help
- All controls are displayed in the UI
- Current status always visible
- Real-time feedback on actions

### Key Reminders
- **SPACE** to start, **P** to pause, **R** to resume
- **U** and **H** for simulator control
- **1, 2, 3** for scheduler switching
- **M** for maintenance
- **S** to stop simulators

### Best Practices
1. Start with a clean system (run maintenance)
2. Monitor metrics before making changes
3. Test system limits with simulators
4. Maintain regular maintenance schedule
5. Adapt strategy based on colony performance

---

**Happy Colony Managing!** 🏭⚡

Remember: The key to success is balancing efficiency, reliability, and performance while keeping your colony healthy and productive.
