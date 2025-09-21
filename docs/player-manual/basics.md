# Player Manual: Basics

Welcome to the Colony Simulator! This guide will teach you the fundamentals of managing a computational colony.

## 🎯 What is a Colony?

A **colony** is a distributed system of workers that process data through complex pipelines. Your job is to manage resources, handle faults, and keep your colony running efficiently.

### Core Concepts

- **Workers**: The heart of your colony, processing data through various operations
- **Workyards**: Facilities where workers operate (CPU arrays, GPU farms, signal hubs)
- **Pipelines**: Sequences of operations that transform incoming data
- **Resources**: Power, heat, bandwidth, and corruption that affect performance
- **Events**: Black Swans, research breakthroughs, and system failures

## 🏭 Colony Components

### Workers

Workers are the backbone of your colony. Each worker has:

- **Class**: CPU, GPU, or I/O specialization
- **Skills**: Efficiency ratings for different operation types
- **State**: Idle, running, or faulted
- **Discipline**: Resistance to corruption
- **Focus**: Concentration level affecting performance

### Workyards

Workyards are facilities that house and manage workers:

- **CPU Array**: General-purpose processing with moderate heat generation
- **GPU Farm**: High-performance parallel processing with significant heat
- **Signal Hub**: I/O operations with low heat but bandwidth requirements

### Pipelines

Pipelines define how data flows through your colony:

- **UDP Demux**: Network packet processing
- **HTTP Parse**: Web request handling
- **Decode**: Data decompression
- **Kalman**: Signal processing
- **FFT**: Frequency analysis
- **Yolo**: AI inference (GPU only)

## ⚡ Resource Management

### Power System

Your colony consumes power to operate:

- **Power Cap**: Maximum power your colony can draw
- **Power Draw**: Current power consumption
- **Power Deficit**: When draw exceeds cap, causing system degradation

**Tips:**
- Monitor power consumption closely
- Upgrade power capacity before adding more workers
- Power deficits cause immediate performance loss

### Heat Management

Workers generate heat during operation:

- **Heat Generation**: Based on operation complexity and worker efficiency
- **Heat Capacity**: Maximum heat a workyard can handle
- **Thermal Throttling**: Automatic performance reduction when heat is high

**Tips:**
- Keep heat below 80% of capacity for optimal performance
- GPU operations generate more heat than CPU operations
- High heat increases fault rates

### Bandwidth Utilization

Data flows through your colony's network:

- **Bandwidth Cap**: Maximum data throughput
- **Utilization**: Current bandwidth usage
- **Latency**: Delay caused by high utilization

**Tips:**
- Monitor bandwidth during peak processing
- High utilization increases processing delays
- I/O operations are most affected by bandwidth limits

### Corruption Field

Corruption represents system degradation:

- **Corruption Rate**: How quickly corruption increases
- **Decay Rate**: Natural corruption reduction
- **Effects**: Higher corruption increases fault rates

**Tips:**
- Keep corruption below 10% for stable operation
- Research can reduce corruption rates
- Faults increase corruption significantly

## 🎮 Basic Gameplay

### Starting a Session

1. **Select Scenario**: Choose from available scenarios
2. **Configure Colony**: Set initial parameters
3. **Start Simulation**: Begin processing data
4. **Monitor Dashboard**: Watch key metrics
5. **Respond to Events**: Handle faults and opportunities

### Dashboard Overview

The dashboard shows critical information:

- **Power Meter**: Current vs. maximum power
- **Heat Gauges**: Temperature for each workyard
- **Bandwidth Graph**: Network utilization over time
- **Corruption Bar**: System degradation level
- **Fault Counter**: Number of active faults
- **Research Progress**: Available technologies

### Basic Controls

- **Pause/Resume**: Stop or start the simulation
- **Fast Forward**: Speed up time (useful for long scenarios)
- **Save/Load**: Preserve your progress
- **Settings**: Adjust simulation parameters

## 📊 Understanding Metrics

### Key Performance Indicators (KPIs)

- **Throughput**: Data processed per second
- **Latency**: Average processing delay
- **Fault Rate**: Percentage of failed operations
- **Efficiency**: Resource utilization effectiveness
- **Uptime**: Percentage of time system is operational

### Monitoring Tips

- **Green**: Everything is running smoothly
- **Yellow**: Warning conditions, monitor closely
- **Red**: Critical issues, immediate action required

## 🚨 Common Scenarios

### Power Crisis

**Symptoms:**
- Power draw exceeds capacity
- System performance degrades
- Workers may shut down

**Solutions:**
- Reduce worker count temporarily
- Upgrade power capacity
- Optimize pipeline efficiency

### Heat Emergency

**Symptoms:**
- Workyard temperatures near maximum
- Thermal throttling active
- Increased fault rates

**Solutions:**
- Pause high-heat operations
- Improve worker efficiency
- Add cooling systems (research)

### Bandwidth Saturation

**Symptoms:**
- High network utilization
- Increased processing delays
- I/O operations failing

**Solutions:**
- Reduce concurrent I/O operations
- Optimize data flow
- Upgrade network capacity

### Corruption Spiral

**Symptoms:**
- Corruption field growing rapidly
- Increasing fault rates
- System instability

**Solutions:**
- Research corruption reduction
- Fix underlying fault causes
- Consider system restart

## 🎯 Early Game Strategy

### First 30 Minutes

1. **Start Small**: Begin with 2-4 workers
2. **Monitor Resources**: Watch power, heat, and bandwidth
3. **Learn the Interface**: Familiarize yourself with the dashboard
4. **Handle First Faults**: Practice fault recovery
5. **Research Basics**: Unlock fundamental technologies

### Building Your Colony

1. **Stable Foundation**: Ensure power and heat are under control
2. **Gradual Expansion**: Add workers incrementally
3. **Diversify Operations**: Mix CPU, GPU, and I/O work
4. **Research Priority**: Focus on efficiency improvements
5. **Fault Preparedness**: Always have recovery plans

### Research Priorities

- **Efficiency Improvements**: Better worker performance
- **Resource Management**: Power and heat optimization
- **Fault Tolerance**: Reduced corruption and better recovery
- **Advanced Operations**: New pipeline capabilities

## 🔧 Troubleshooting

### Performance Issues

- **Check Power**: Ensure adequate power capacity
- **Monitor Heat**: Keep temperatures reasonable
- **Review Bandwidth**: Avoid network saturation
- **Assess Corruption**: Keep degradation low

### Fault Recovery

- **Identify Root Cause**: What triggered the fault?
- **Apply Fix**: Address the underlying issue
- **Monitor Recovery**: Ensure system stabilizes
- **Learn**: Prevent similar issues in the future

### System Optimization

- **Worker Allocation**: Balance CPU, GPU, and I/O workers
- **Pipeline Efficiency**: Optimize operation sequences
- **Resource Planning**: Anticipate future needs
- **Research Investment**: Focus on high-impact technologies

## 📚 Next Steps

Now that you understand the basics:

1. **Practice**: Try the "First Light Chill" scenario
2. **Experiment**: Test different worker configurations
3. **Learn**: Read the [Survival Guide](survival.md) for advanced strategies
4. **Master**: Study [Victory & Loss](victory-loss.md) conditions

Remember: The Colony Simulator is about learning through experimentation. Don't be afraid to fail - each failure teaches you something new about system management!

---

**Ready to build your first colony?** 🏭✨
