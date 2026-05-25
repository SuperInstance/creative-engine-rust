# creative-engine-rust

Rust port of the [creative-engine-c](https://github.com/SuperInstance/creative-engine-c) dynamical systems engine. Lorenz attractors, regime detection, Kuramoto synchronization, quality metrics, coupling, and adaptive thermostats.

## Building

```bash
cargo build
cargo test
```

Requires Rust 1.70+ (edition 2021). No external dependencies.

## CreativeSystem

A single Lorenz system with integrated RK4 stepping, output tracking, and quality measurement.

```rust
use creative_engine::CreativeSystem;

let mut sys = CreativeSystem::new(28.0);  // ρ = 28 → chaotic regime

// Run 1000 steps, discarding 200 transient steps
sys.run(1000, 200);

// Access the raw state
println!("state: {:?}", sys.state);  // [x, y, z]

// All outputs from the run
println!("{} output samples", sys.outputs.len());

// Detect which regime the system is in
let regime = sys.detect_regime();
// Regime::Chaotic for ρ=28

// Compute quality metrics on outputs
let q = sys.quality();
println!("novelty={}, coherence={}, quality={}", q.novelty, q.coherence, q.quality);

// Diversity (standard deviation of outputs)
let div = sys.diversity();
```

Builder-style configuration:

```rust
let mut sys = CreativeSystem::new(28.0)
    .with_sigma(12.0)
    .with_epsilon(0.3);
```

Single-step integration:

```rust
let mut sys = CreativeSystem::new(28.0);
let output = sys.step();  // advances one RK4 step, returns x
```

## Regime Detection

Regime is determined by the ρ parameter (Rayleigh number):

| ρ range | Regime | Description |
|---|---|---|
| ρ < 5.0 | `FixedPoint` | Rigid, no creativity |
| 5.0 ≤ ρ ≤ 24.74 | `Periodic` | Oscillating, SR active |
| ρ > 24.74 | `Chaotic` | Strange attractor, full creativity |

```rust
use creative_engine::Regime;

let r = Regime::from_rho(28.0);
assert_eq!(r, Regime::Chaotic);

// Each regime has an optimal ε (exploration rate)
let eps = r.optimal_epsilon();
// FixedPoint → 1.5, Periodic → 0.5, Chaotic → 0.2

// Human-readable description
println!("{}", r.description());
```

## QualityMetrics

Evaluates creative output quality from a signal buffer:

```rust
use creative_engine::QualityMetrics;

let signal = vec![1.0, 1.5, 2.1, 2.8, 3.3, 3.9, 4.5];
let q = QualityMetrics::compute(&signal);

// novelty: standard deviation of the signal
// coherence: spectral concentration (DFT-based, dominant frequency power / total power)
// quality: novelty × coherence
println!("novelty={:.3} coherence={:.3} quality={:.3}", q.novelty, q.coherence, q.quality);
```

Coherence uses a direct DFT computation on up to 256 samples from the end of the signal. High coherence means the signal has a dominant frequency — it's structured, not noise.

## Soft Snap

Continuous snap function: `C(x, ε) = (1 − ε) · round(x) + ε · x`

```rust
use creative_engine::CreativeSystem;

let result = CreativeSystem::soft_snap(2.7, 0.0);  // → 3.0 (hard round)
let result = CreativeSystem::soft_snap(2.7, 1.0);  // → 2.7 (identity)
let result = CreativeSystem::soft_snap(2.6, 0.5);  // → 2.8
```

## Sigmoid

Parameterized sigmoid: `σ(x) = 1 / (1 + exp(−k(x − x₀)))`

```rust
let y = CreativeSystem::sigmoid(0.5, 1.0, 0.5);  // → 0.5 (centered at x₀)
let y = CreativeSystem::sigmoid(-10.0, 1.0, 0.5); // → ≈0 (far left)
let y = CreativeSystem::sigmoid(10.0, 1.0, 0.5);  // → ≈1 (far right)
```

## Kuramoto Order Parameter

Measures oscillator synchronization: `r = |1/N · Σ exp(i·θⱼ)|`

```rust
let synced = vec![0.0; 10];
let r = CreativeSystem::sync_order(&synced);
assert!(r > 0.99);  // fully synchronized

let spread: Vec<f64> = (0..100).map(|i| std::f64::consts::PI * i as f64 / 50.0).collect();
let r = CreativeSystem::sync_order(&spread);
assert!(r < 0.5);  // desynchronized
```

## CreativeNetwork

Coupled network of creative agents. Coupling flows from higher-expertise to lower-expertise agents.

```rust
use creative_engine::CreativeNetwork;

let expertises = vec![0.1, 0.5, 0.9];
let mut net = CreativeNetwork::new(&expertises);
// 3 agents with ρ values: 1.0+0.1*49=5.9, 1.0+0.5*49=25.5, 1.0+0.9*49=45.1
// Coupling: agent 0 receives from agents 1,2; agent 1 from agent 2

// Run 1000 coupled steps
net.run(1000);

// Each agent accumulates outputs
for (i, agent) in net.agents.iter().enumerate() {
    println!("Agent {}: {} outputs", i, agent.outputs.len());
}

// Aggregate metrics
let total_quality = net.total_quality();
let total_diversity = net.total_diversity();
```

## CreativeThermostat

Adapts ε (exploration rate) to track a target diversity:

```rust
use creative_engine::CreativeThermostat;

let mut thermo = CreativeThermostat::new(28.0, 5.0);  // ρ=28, target_diversity=5.0

// Run 50 adaptation cycles
// Each cycle: run system 100 steps (discard 50), measure diversity, adapt ε
thermo.run_thermostat(50);

// Get the converged ε
let eps = thermo.converged_epsilon();  // average of last 10 ε values

// Inspect history: Vec<(epsilon, diversity)>
for (eps, div) in &thermo.history {
    println!("ε={:.4} diversity={:.4}", eps, div);
}
```

Adaptation rule: if measured diversity < target, increase ε (more exploration). If diversity > target, decrease ε. Learning rate is 0.01 by default.

## Types

```rust
pub enum Regime { FixedPoint, Periodic, Chaotic }

pub struct QualityMetrics {
    pub novelty: f64,    // standard deviation of signal
    pub coherence: f64,  // spectral concentration [0, 1]
    pub quality: f64,    // novelty × coherence
}

pub struct CreativeSystem {
    pub sigma: f64,       // default 10.0
    pub rho: f64,         // determines regime
    pub beta: f64,        // default 8/3
    pub state: [f64; 3],  // [x, y, z]
    pub dt: f64,          // default 0.01
    pub epsilon: f64,     // exploration rate
    pub outputs: Vec<f64>,
}

pub struct CreativeNetwork {
    pub agents: Vec<CreativeSystem>,
    pub coupling_matrix: Vec<Vec<f64>>,
}

pub struct CreativeThermostat {
    pub system: CreativeSystem,
    pub target_diversity: f64,
    pub learning_rate: f64,
    pub history: Vec<(f64, f64)>,  // (epsilon, diversity) per cycle
}
```

## API Summary

| Type | Method | Description |
|---|---|---|
| `Regime` | `from_rho(rho)` | Classify from Rayleigh number |
| `Regime` | `optimal_epsilon()` | Suggested ε for this regime |
| `CreativeSystem` | `new(rho)` | Create with given ρ |
| `CreativeSystem` | `with_sigma(s)` / `with_epsilon(e)` | Builder config |
| `CreativeSystem` | `step()` | One RK4 step, returns x output |
| `CreativeSystem` | `run(n, discard)` | Run n steps, discard transient |
| `CreativeSystem` | `detect_regime()` | `Regime::from_rho(self.rho)` |
| `CreativeSystem` | `quality()` | `QualityMetrics` from outputs |
| `CreativeSystem` | `diversity()` | Std dev of outputs |
| `CreativeSystem` | `soft_snap(x, ε)` | `(1-ε)·round(x) + ε·x` |
| `CreativeSystem` | `sigmoid(x, k, x₀)` | Parameterized sigmoid |
| `CreativeSystem` | `sync_order(phases)` | Kuramoto order parameter |
| `QualityMetrics` | `compute(&[f64])` | Novelty + coherence + quality |
| `CreativeNetwork` | `new(&[expertise])` | Coupled agents |
| `CreativeNetwork` | `run(n)` | Run n coupled steps |
| `CreativeNetwork` | `total_quality()` / `total_diversity()` | Aggregate metrics |
| `CreativeThermostat` | `new(rho, target_div)` | Adaptive ε controller |
| `CreativeThermostat` | `adapt()` | One adaptation cycle |
| `CreativeThermostat` | `run_thermostat(n)` | Run n cycles |
| `CreativeThermostat` | `converged_epsilon()` | Mean of last 10 ε values |

## Related

- [creative-engine-c](https://github.com/SuperInstance/creative-engine-c) — Original C implementation
- [superinstance-live](https://github.com/SuperInstance/superinstance-live) — Live session controller
- [flux-genome](https://github.com/SuperInstance/flux-genome) — Genetic evolution of musical genomes

## License

MIT
