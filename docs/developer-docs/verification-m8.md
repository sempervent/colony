# Developer Docs: Verification (M8)

The M8 verification system provides comprehensive end-to-end testing, validation, and release candidate generation. This document explains how the verification system works, how to run verification tests, and how to generate release candidates.

## Overview

The M8 verification system provides:

- **End-to-End Testing**: Comprehensive testing of all systems
- **Determinism Verification**: Ensure deterministic behavior
- **Performance Validation**: Validate performance requirements
- **Security Testing**: Test security and sandboxing
- **Release Candidate Generation**: Generate release candidates
- **Automated Verification**: Automated verification pipeline
- **Test Reporting**: Comprehensive test reports

## Verification Matrix

### Test Categories

The verification system covers multiple test categories:

```rust
pub enum TestCategory {
    Unit,                        // Unit tests
    Integration,                 // Integration tests
    EndToEnd,                    // End-to-end tests
    Performance,                 // Performance tests
    Security,                    // Security tests
    Determinism,                 // Determinism tests
    Compatibility,               // Compatibility tests
    Stress,                      // Stress tests
    Regression,                  // Regression tests
    Smoke,                       // Smoke tests
}

pub struct VerificationMatrix {
    pub categories: HashMap<TestCategory, TestSuite>,
    pub requirements: Vec<VerificationRequirement>,
    pub success_criteria: SuccessCriteria,
    pub reporting: ReportingConfiguration,
}
```

### Verification Requirements

```rust
pub struct VerificationRequirement {
    pub id: RequirementId,
    pub name: String,
    pub description: String,
    pub category: TestCategory,
    pub priority: Priority,
    pub success_criteria: SuccessCriteria,
    pub test_suite: TestSuite,
    pub dependencies: Vec<RequirementId>,
}

pub enum Priority {
    Critical,                    // Must pass for release
    High,                        // Should pass for release
    Medium,                      // Nice to have
    Low,                         // Optional
}

pub struct SuccessCriteria {
    pub pass_rate: f32,          // Minimum pass rate (0.0-1.0)
    pub performance_threshold: PerformanceThreshold,
    pub security_requirements: SecurityRequirements,
    pub determinism_requirements: DeterminismRequirements,
}
```

## Test Suites

### Unit Tests

```rust
pub struct UnitTestSuite {
    pub tests: Vec<UnitTest>,
    pub coverage_requirements: CoverageRequirements,
    pub timeout: Duration,
    pub parallel: bool,
}

pub struct UnitTest {
    pub name: String,
    pub module: String,
    pub function: String,
    pub timeout: Duration,
    pub expected_result: TestResult,
    pub dependencies: Vec<TestDependency>,
}

impl UnitTestSuite {
    pub fn run(&self) -> TestSuiteResult {
        let mut results = Vec::new();
        
        for test in &self.tests {
            let result = self.run_unit_test(test);
            results.push(result);
        }
        
        TestSuiteResult {
            suite_name: "Unit Tests".to_string(),
            total_tests: self.tests.len(),
            passed: results.iter().filter(|r| r.is_success()).count(),
            failed: results.iter().filter(|r| r.is_failure()).count(),
            skipped: results.iter().filter(|r| r.is_skipped()).count(),
            results,
            coverage: self.calculate_coverage(&results),
        }
    }
}
```

### Integration Tests

```rust
pub struct IntegrationTestSuite {
    pub tests: Vec<IntegrationTest>,
    pub test_environment: TestEnvironment,
    pub setup_scripts: Vec<SetupScript>,
    pub teardown_scripts: Vec<TeardownScript>,
}

pub struct IntegrationTest {
    pub name: String,
    pub description: String,
    pub test_scenario: TestScenario,
    pub expected_outcome: ExpectedOutcome,
    pub timeout: Duration,
    pub retry_count: u32,
}

pub enum TestScenario {
    FullSimulation,              // Full simulation test
    ComponentInteraction,        // Component interaction test
    ApiIntegration,              // API integration test
    DatabaseIntegration,         // Database integration test
    NetworkIntegration,          // Network integration test
    ModIntegration,              // Mod integration test
}
```

### End-to-End Tests

```rust
pub struct EndToEndTestSuite {
    pub scenarios: Vec<E2EScenario>,
    pub test_environment: E2ETestEnvironment,
    pub data_sets: Vec<TestDataSet>,
    pub validation_rules: Vec<ValidationRule>,
}

pub struct E2EScenario {
    pub name: String,
    pub description: String,
    pub steps: Vec<E2EStep>,
    pub expected_results: Vec<ExpectedResult>,
    pub timeout: Duration,
    pub retry_count: u32,
}

pub enum E2EStep {
    StartSimulation,             // Start simulation
    ConfigureScenario,           // Configure scenario
    RunSimulation,               // Run simulation
    PerformActions,              // Perform user actions
    WaitForCondition,            // Wait for condition
    ValidateState,               // Validate state
    StopSimulation,              // Stop simulation
}
```

## Performance Testing

### Performance Test Suite

```rust
pub struct PerformanceTestSuite {
    pub benchmarks: Vec<Benchmark>,
    pub performance_requirements: PerformanceRequirements,
    pub measurement_config: MeasurementConfig,
}

pub struct Benchmark {
    pub name: String,
    pub description: String,
    pub benchmark_type: BenchmarkType,
    pub measurement: Measurement,
    pub threshold: PerformanceThreshold,
    pub iterations: u32,
    pub warmup_iterations: u32,
}

pub enum BenchmarkType {
    Throughput,                  // Throughput benchmark
    Latency,                     // Latency benchmark
    Memory,                      // Memory usage benchmark
    CPU,                         // CPU usage benchmark
    Disk,                        // Disk I/O benchmark
    Network,                     // Network I/O benchmark
    Startup,                     // Startup time benchmark
    Shutdown,                    // Shutdown time benchmark
}

pub struct PerformanceThreshold {
    pub min_throughput: f32,     // Minimum throughput
    pub max_latency: Duration,   // Maximum latency
    pub max_memory: u64,         // Maximum memory usage
    pub max_cpu: f32,            // Maximum CPU usage
    pub max_disk_io: u64,        // Maximum disk I/O
    pub max_network_io: u64,     // Maximum network I/O
    pub max_startup_time: Duration, // Maximum startup time
    pub max_shutdown_time: Duration, // Maximum shutdown time
}
```

### Performance Measurement

```rust
pub struct PerformanceMeasurer {
    pub metrics: PerformanceMetrics,
    pub profiler: Profiler,
    pub monitor: SystemMonitor,
}

pub struct PerformanceMetrics {
    pub throughput: f32,         // Operations per second
    pub latency: Duration,       // Average latency
    pub memory_usage: u64,       // Memory usage in bytes
    pub cpu_usage: f32,          // CPU usage percentage
    pub disk_io: u64,            // Disk I/O in bytes
    pub network_io: u64,         // Network I/O in bytes
    pub startup_time: Duration,  // Startup time
    pub shutdown_time: Duration, // Shutdown time
}

impl PerformanceMeasurer {
    pub fn measure_benchmark(&mut self, benchmark: &Benchmark) -> BenchmarkResult {
        let mut results = Vec::new();
        
        // Warmup iterations
        for _ in 0..benchmark.warmup_iterations {
            self.run_benchmark_iteration(benchmark);
        }
        
        // Actual measurements
        for _ in 0..benchmark.iterations {
            let result = self.run_benchmark_iteration(benchmark);
            results.push(result);
        }
        
        let aggregated = self.aggregate_results(&results);
        let passed = self.validate_performance(&aggregated, &benchmark.threshold);
        
        BenchmarkResult {
            benchmark_name: benchmark.name.clone(),
            results,
            aggregated,
            passed,
            threshold: benchmark.threshold.clone(),
        }
    }
}
```

## Security Testing

### Security Test Suite

```rust
pub struct SecurityTestSuite {
    pub tests: Vec<SecurityTest>,
    pub security_requirements: SecurityRequirements,
    pub vulnerability_scanner: VulnerabilityScanner,
}

pub struct SecurityTest {
    pub name: String,
    pub description: String,
    pub test_type: SecurityTestType,
    pub attack_vector: AttackVector,
    pub expected_result: SecurityTestResult,
    pub severity: SecuritySeverity,
}

pub enum SecurityTestType {
    SandboxEscape,               // Sandbox escape test
    PrivilegeEscalation,         // Privilege escalation test
    DataExfiltration,            // Data exfiltration test
    DenialOfService,             // Denial of service test
    Injection,                   // Injection attack test
    BufferOverflow,              // Buffer overflow test
    RaceCondition,               // Race condition test
    MemoryCorruption,            // Memory corruption test
}

pub enum AttackVector {
    ModExecution,                // Attack through mod execution
    NetworkInput,                // Attack through network input
    FileSystem,                  // Attack through file system
    Memory,                      // Attack through memory
    Process,                     // Attack through process
    System,                      // Attack through system
}
```

### Sandbox Testing

```rust
pub struct SandboxTester {
    pub wasm_sandbox: WasmSandboxTester,
    pub lua_sandbox: LuaSandboxTester,
    pub capability_tester: CapabilityTester,
}

impl SandboxTester {
    pub fn test_wasm_sandbox(&self) -> SandboxTestResult {
        let mut results = Vec::new();
        
        // Test WASM sandbox isolation
        results.push(self.test_wasm_isolation());
        
        // Test WASM resource limits
        results.push(self.test_wasm_resource_limits());
        
        // Test WASM capability gating
        results.push(self.test_wasm_capability_gating());
        
        // Test WASM fault injection
        results.push(self.test_wasm_fault_injection());
        
        SandboxTestResult {
            sandbox_type: "WASM".to_string(),
            results,
            overall_result: self.aggregate_results(&results),
        }
    }
    
    pub fn test_lua_sandbox(&self) -> SandboxTestResult {
        let mut results = Vec::new();
        
        // Test Lua sandbox isolation
        results.push(self.test_lua_isolation());
        
        // Test Lua resource limits
        results.push(self.test_lua_resource_limits());
        
        // Test Lua capability gating
        results.push(self.test_lua_capability_gating());
        
        // Test Lua fault injection
        results.push(self.test_lua_fault_injection());
        
        SandboxTestResult {
            sandbox_type: "Lua".to_string(),
            results,
            overall_result: self.aggregate_results(&results),
        }
    }
}
```

## Determinism Testing

### Determinism Test Suite

```rust
pub struct DeterminismTestSuite {
    pub tests: Vec<DeterminismTest>,
    pub seed_generator: SeedGenerator,
    pub state_comparator: StateComparator,
}

pub struct DeterminismTest {
    pub name: String,
    pub description: String,
    pub test_scenario: DeterminismScenario,
    pub seed: u64,
    pub iterations: u32,
    pub expected_determinism: bool,
}

pub enum DeterminismScenario {
    FullSimulation,              // Full simulation determinism
    ModExecution,                // Mod execution determinism
    RandomGeneration,            // Random generation determinism
    StateTransitions,            // State transition determinism
    EventProcessing,             // Event processing determinism
    ReplayPlayback,              // Replay playback determinism
}

impl DeterminismTestSuite {
    pub fn run_determinism_test(&self, test: &DeterminismTest) -> DeterminismTestResult {
        let mut results = Vec::new();
        
        for i in 0..test.iterations {
            let result = self.run_single_iteration(test, i);
            results.push(result);
        }
        
        let is_deterministic = self.check_determinism(&results);
        
        DeterminismTestResult {
            test_name: test.name.clone(),
            iterations: test.iterations,
            results,
            is_deterministic,
            expected_determinism: test.expected_determinism,
            passed: is_deterministic == test.expected_determinism,
        }
    }
}
```

## Release Candidate Generation

### RC Builder

```rust
pub struct ReleaseCandidateBuilder {
    pub version: Version,
    pub build_config: BuildConfig,
    pub packaging_config: PackagingConfig,
    pub signing_config: SigningConfig,
}

pub struct BuildConfig {
    pub target_platforms: Vec<TargetPlatform>,
    pub build_features: Vec<BuildFeature>,
    pub optimization_level: OptimizationLevel,
    pub debug_info: bool,
    pub strip_symbols: bool,
}

pub struct PackagingConfig {
    pub include_docs: bool,
    pub include_examples: bool,
    pub include_mods: bool,
    pub include_tests: bool,
    pub compression: CompressionType,
    pub checksums: Vec<ChecksumType>,
}

impl ReleaseCandidateBuilder {
    pub fn build_rc(&mut self) -> Result<ReleaseCandidate, BuildError> {
        // Build all targets
        let mut artifacts = Vec::new();
        
        for platform in &self.build_config.target_platforms {
            let artifact = self.build_target(platform)?;
            artifacts.push(artifact);
        }
        
        // Package artifacts
        let package = self.package_artifacts(&artifacts)?;
        
        // Generate checksums
        let checksums = self.generate_checksums(&package)?;
        
        // Sign package
        let signature = self.sign_package(&package)?;
        
        // Create release candidate
        let rc = ReleaseCandidate {
            version: self.version.clone(),
            build_date: chrono::Utc::now(),
            artifacts,
            package,
            checksums,
            signature,
            test_report: self.generate_test_report()?,
        };
        
        Ok(rc)
    }
}
```

### Test Report Generation

```rust
pub struct TestReportGenerator {
    pub test_results: Vec<TestSuiteResult>,
    pub performance_results: Vec<BenchmarkResult>,
    pub security_results: Vec<SecurityTestResult>,
    pub determinism_results: Vec<DeterminismTestResult>,
}

impl TestReportGenerator {
    pub fn generate_report(&self) -> TestReport {
        let summary = self.generate_summary();
        let detailed_results = self.generate_detailed_results();
        let recommendations = self.generate_recommendations();
        
        TestReport {
            summary,
            detailed_results,
            recommendations,
            generated_at: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    
    fn generate_summary(&self) -> TestSummary {
        let total_tests = self.test_results.iter().map(|r| r.total_tests).sum();
        let passed_tests = self.test_results.iter().map(|r| r.passed).sum();
        let failed_tests = self.test_results.iter().map(|r| r.failed).sum();
        let skipped_tests = self.test_results.iter().map(|r| r.skipped).sum();
        
        let pass_rate = if total_tests > 0 {
            passed_tests as f32 / total_tests as f32
        } else {
            0.0
        };
        
        TestSummary {
            total_tests,
            passed_tests,
            failed_tests,
            skipped_tests,
            pass_rate,
            overall_status: if failed_tests == 0 { "PASSED" } else { "FAILED" }.to_string(),
        }
    }
}
```

## Configuration

### Verification Configuration

```toml
# In verification configuration
[verification]
timeout = 3600                    # 1 hour timeout
parallel = true                   # Run tests in parallel
retry_count = 3                   # Retry failed tests
report_format = "json"            # Report format
output_directory = "verification_results"

[verification.unit_tests]
enabled = true
coverage_threshold = 0.95         # 95% coverage
timeout = 300                     # 5 minutes
parallel = true

[verification.integration_tests]
enabled = true
test_environment = "docker"
setup_timeout = 600               # 10 minutes
test_timeout = 1800               # 30 minutes

[verification.performance_tests]
enabled = true
benchmark_iterations = 100
warmup_iterations = 10
measurement_duration = 60         # 60 seconds
performance_threshold = "strict"

[verification.security_tests]
enabled = true
vulnerability_scan = true
sandbox_testing = true
penetration_testing = false
security_threshold = "high"

[verification.determinism_tests]
enabled = true
test_iterations = 10
seed_range = [1, 10000]
determinism_threshold = 1.0       # 100% deterministic
```

### Build Configuration

```toml
[build]
target_platforms = ["x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc", "x86_64-apple-darwin"]
build_features = ["all"]
optimization_level = "release"
debug_info = false
strip_symbols = true

[build.packaging]
include_docs = true
include_examples = true
include_mods = true
include_tests = false
compression = "gzip"
checksums = ["sha256", "sha512"]

[build.signing]
enabled = true
key_file = "release.key"
certificate_file = "release.crt"
algorithm = "rsa"
```

## Usage

### Running Verification

```bash
# Run all verification tests
cargo xtask verify

# Run specific test category
cargo xtask verify --category unit
cargo xtask verify --category integration
cargo xtask verify --category performance
cargo xtask verify --category security
cargo xtask verify --category determinism

# Run with specific configuration
cargo xtask verify --config verification.toml

# Generate release candidate
cargo xtask verify --rc
```

### Verification Results

```bash
# View test results
cat verification_results/test_report.json

# View performance results
cat verification_results/performance_report.json

# View security results
cat verification_results/security_report.json

# View determinism results
cat verification_results/determinism_report.json
```

## Best Practices

### Test Design

1. **Comprehensive Coverage**: Cover all critical paths
2. **Deterministic Tests**: Ensure tests are deterministic
3. **Performance Awareness**: Consider performance impact
4. **Security Focus**: Include security testing
5. **Maintainability**: Keep tests maintainable

### Verification Process

1. **Automated Pipeline**: Use automated verification
2. **Regular Testing**: Run tests regularly
3. **Failure Analysis**: Analyze test failures
4. **Continuous Improvement**: Improve tests continuously
5. **Documentation**: Document test procedures

## Troubleshooting

### Common Issues

1. **Test Failures**: Tests failing unexpectedly
2. **Performance Issues**: Tests running too slowly
3. **Determinism Issues**: Non-deterministic behavior
4. **Security Issues**: Security vulnerabilities
5. **Build Issues**: Build failures

### Debug Tools

- **Test Runner**: Run individual tests
- **Performance Profiler**: Profile test performance
- **Debug Logger**: Enable debug logging
- **State Inspector**: Inspect test state
- **Failure Analyzer**: Analyze test failures

---

**The M8 verification system provides comprehensive testing and validation. Understanding and using these systems is key to ensuring quality and reliability.** 🏭✅
