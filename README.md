# creative-engine-rust

> Rust port of the Creative Dynamics Engine — Lorenz-based creative systems with regime detection, adaptive ε, quality metrics, and coupling.

This is the Rust implementation of [creative-engine-c](https://github.com/SuperInstance/creative-engine-c), providing the same dynamical systems primitives with Rust's safety guarantees and zero-cost abstractions.

## Features

- **Lorenz attractor** — RK4 integration with configurable σ, ρ, β parameters
- **Regime detection** — Fixed-point (ρ < 5), periodic (5 ≤ ρ ≤ 24.74), chaotic (ρ > 24.74)
- **Soft snap** — `C(x, ε) = (1−ε)·round(x) + ε·x` continuous interpolation
- **Quality metrics** — Novelty (std dev), coherence (spectral concentration), and combined quality
- **Synchronization** — Kuramoto order parameter for coupled oscillator sync
- **Creative network** — Multiple coupled Lorenz agents with expertise-based coupling
- **Creative thermostat** — Adaptive ε controller that tracks a target diversity

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
creative-engine = { git = "https://github.com/SuperInstance/creative-engine-rust" }
```

Or clone and use locally:

```bash
git clone https://github.com/SuperInstance/creative-engine-rust
cd creative-engine-rust

# Run tests
cargo test

# Build
cargo build --release
```

## Quick Start

### Single Lorenz system

```rust
use creative_engine::CreativeSystem;

let mut sys = CreativeSystem::new(28.0);  // chaotic regime
sys.run(5000, 1000);  // 5000 steps, discard 1000 transient

let quality = sys.quality();
println!("novelty: {:.3}, coherence: {:.3}, quality: {:.3}",
    quality.novelty, quality.coherence, quality.quality);

let diversity = sys.diversity();
println!("diversity: {:.3}", diversity);
```

### Regime-aware epsilon

```rust
use creative_engine::Regime;

let regime = Regime::from_rho(28.0);  // Chaotic
let eps = regime.optimal_epsilon();    // 0.2 (chaotic needs less exploration noise)
println!("{:?}: ε* = {}, {}", regime, eps, regime.description());
// Chaotic: ε* = 0.2, Creative — strange attractor, needs focus
```

### Soft snap

```rust
use creative_engine::CreativeSystem;

let snapped = CreativeSystem::soft_snap(2.7, 0.0);  // 3.0 (full snap)
let raw = CreativeSystem::soft_snap(2.7, 1.0);       // 2.7 (no snap)
let blended = CreativeSystem::soft_snap(2.7, 0.5);   // 2.85 (halfway)
```

### Coupled creative network

```rust
use creative_engine::CreativeNetwork;

let expertises = vec![0.1, 0.5, 0.9];
let mut net = CreativeNetwork::new(&expertises);
net.run(1000);

println!("total quality: {:.3}", net.total_quality());
println!("total diversity: {:.3}", net.total_diversity());
```

Coupling flows from higher-expertise agents to lower-expertise ones — novices learn from experts.

### Adaptive thermostat

```rust
use creative_engine::CreativeThermostat;

let mut thermo = CreativeThermostat::new(28.0, 5.0);  // chaotic, target diversity 5.0
thermo.run_thermostat(50);  // 50 adaptation cycles

println!("converged ε: {:.3}", thermo.converged_epsilon());
```

The thermostat adjusts ε each cycle: if diversity is below target, ε increases (more exploration); if above, ε decreases (more refinement).

### Synchronization

```rust
use creative_engine::CreativeSystem;

let synced = vec![0.0; 10];
assert!(CreativeSystem::sync_order(&synced) > 0.99);  // perfect sync

let spread = vec![0.0, std::f64::consts::PI / 2.0, std::f64::consts::PI, 1.5 * std::f64::consts::PI];
assert!(CreativeSystem::sync_order(&spread) < 0.1);  // no sync
```

## Differences from creative-engine-c

| Feature | C version | Rust version |
|---|---|---|
| Regime thresholds | ρ < −0.01, ρ < 0.01 | ρ < 5, ρ ≤ 24.74 (Lorenz ρ-specific) |
| Coherence metric | Diff variance | Spectral concentration (DFT peak) |
| Network coupling | Hierarchical layers | Expertise-based directed coupling |
| Memory safety | Manual | Borrow-checked |
| Error handling | Return codes / asserts | Result types |

## API Reference

### `Regime`

```rust
pub enum Regime { FixedPoint, Periodic, Chaotic }
```

| Method | Description |
|---|---|
| `Regime::from_rho(rho)` | Classify from Lorenz ρ parameter |
| `.optimal_epsilon()` | Recommended ε for this regime |
| `.description()` | Human-readable description |

### `CreativeSystem`

```rust
pub struct CreativeSystem { sigma, rho, beta, state, dt, epsilon, outputs }
```

| Method | Description |
|---|---|
| `CreativeSystem::new(rho)` | Create system with given ρ |
| `.with_sigma(sigma)` | Builder: set σ |
| `.with_epsilon(eps)` | Builder: set ε |
| `.step()` | Single RK4 step, returns x |
| `.run(n_steps, discard)` | Run n steps, discard transient |
| `.detect_regime()` | Current regime |
| `.quality()` | `QualityMetrics` (novelty, coherence, quality) |
| `.diversity()` | Std deviation of outputs |
| `::soft_snap(x, eps)` | Static: soft quantization |
| `::sigmoid(x, k, x0)` | Static: sigmoid with steepness and midpoint |
| `::sync_order(phases)` | Static: Kuramoto order parameter |

### `QualityMetrics`

```rust
pub struct QualityMetrics { pub novelty: f64, pub coherence: f64, pub quality: f64 }
```

| Method | Description |
|---|---|
| `QualityMetrics::compute(outputs)` | Compute from a signal buffer |

- **Novelty**: standard deviation of the signal
- **Coherence**: spectral concentration (dominant frequency energy / total energy)
- **Quality**: novelty × coherence

### `CreativeNetwork`

```rust
pub struct CreativeNetwork { pub agents, pub coupling_matrix }
```

| Method | Description |
|---|---|
| `CreativeNetwork::new(expertises)` | Build network from expertise levels |
| `.step()` | Advance all agents one step |
| `.run(n_steps)` | Run for n steps |
| `.total_quality()` | Sum of all agent qualities |
| `.total_diversity()` | Sum of all agent diversities |

### `CreativeThermostat`

```rust
pub struct CreativeThermostat { pub system, pub target_diversity, pub learning_rate, pub history }
```

| Method | Description |
|---|---|
| `CreativeThermostat::new(rho, target_diversity)` | Initialize |
| `.adapt()` | One adaptation cycle, returns new ε |
| `.run_thermostat(n_cycles)` | Run n adaptation cycles |
| `.converged_epsilon()` | Average of last 10 ε values |

## Related Repos

- **[creative-engine-c](https://github.com/SuperInstance/creative-engine-c)** — Original C implementation
- **[constraint-toolkit](https://github.com/SuperInstance/constraint-toolkit)** — Dial space definitions and constraint solving
- **[superinstance-live](https://github.com/SuperInstance/superinstance-live)** — Live session controller
- **[flux-genome](https://github.com/SuperInstance/flux-genome)** — Genetic algorithm framework for evolving traditions

## License

MIT
