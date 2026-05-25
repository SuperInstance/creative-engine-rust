# creative-engine-rust

> Rust port of the SuperInstance creative dynamics engine — Lorenz attractors, regime detection, and signal quality

Part of the [SuperInstance](https://github.com/SuperInstance) music constraint theory ecosystem. This is the Rust implementation of [creative-engine-c](https://github.com/SuperInstance/creative-engine-c), providing the same chaotic dynamics primitives with Rust's safety guarantees and zero-cost abstractions.

## What It Does

Implements dynamical systems for driving creative musical processes. The core is the **Lorenz attractor** — a chaotic system that, depending on parameters, produces fixed-point, periodic, or fully chaotic trajectories. The engine detects which regime is active and provides metrics for evaluating the creative quality of the output.

This Rust port mirrors the C implementation while leveraging Rust's type system for regime safety and its trait system for composable integration schemes.

## Key Features

- **Lorenz system integration** — 4th-order Runge-Kutta with configurable parameters (σ, ρ, β)
- **Regime detection** — classifies dynamics as fixed-point, periodic, or chaotic
- **Signal quality metrics** — evaluates creative viability of dynamical state
- **Coupling** — couples multiple Lorenz systems for multi-voice coordination
- **Safe API** — Rust's type system prevents invalid state transitions

## Building

```bash
git clone https://github.com/SuperInstance/creative-engine-rust.git
cd creative-engine-rust
cargo build                # Debug build
cargo build --release      # Optimized build
cargo test                 # Run tests
```

### Prerequisites

- Rust 1.70+ (edition 2021)

## Quick Start

```rust
use creative_engine::{LorenzState, lorenz_step, detect_regime, Regime};

fn main() {
    // Initialize with classic Lorenz parameters
    let mut state = LorenzState::new(0.1, 0.0, 0.0)
        .sigma(10.0)
        .rho(28.0)
        .beta(8.0 / 3.0);

    let dt = 0.01;

    // Integrate the attractor
    for _ in 0..1000 {
        lorenz_step(&mut state, dt);
    }

    // Detect the current regime
    let regime = detect_regime(&trajectory);
    match regime {
        Regime::FixedPoint => println!("Settled — stable output"),
        Regime::Periodic   => println!("Limit cycle — repeating patterns"),
        Regime::Chaotic    => println!("Strange attractor — rich material"),
    }
}
```

### Coupling two systems

```rust
use creative_engine::{LorenzState, lorenz_step_coupled};

let mut a = LorenzState::new(0.1, 0.0, 0.0);
let mut b = LorenzState::new(-0.1, 0.0, 0.0);

let coupling = 0.05;
let dt = 0.01;

for _ in 0..1000 {
    lorenz_step_coupled(&mut a, &mut b, coupling, dt);
}
```

## Relationship to creative-engine-c

| Aspect | creative-engine-c | creative-engine-rust |
|---|---|---|
| Language | C99 | Rust (edition 2021) |
| Memory safety | Manual | Compile-time guaranteed |
| Performance | Equivalent | Equivalent |
| Integration | Direct C FFI | Via [flux-ffi](https://github.com/SuperInstance/flux-ffi) |
| Use case | Embedded, real-time audio | Application-level, safe pipelines |

Both engines produce identical numerical results for the same inputs and parameters.

## API Reference

### `LorenzState`

```rust
let state = LorenzState::new(x, y, z)
    .sigma(10.0)    // Default: 10.0
    .rho(28.0)      // Default: 28.0
    .beta(8.0/3.0); // Default: 2.667

state.x  // Current x coordinate
state.y  // Current y coordinate
state.z  // Current z coordinate
```

### Functions

| Function | Description |
|---|---|
| `lorenz_step(state, dt)` | One RK4 integration step |
| `lorenz_step_coupled(a, b, coupling, dt)` | Coupled integration step |
| `detect_regime(trajectory)` | Classify trajectory regime |
| `signal_quality(state, history)` | Creative quality metric `[0.0, 1.0]` |

## Testing

```bash
cargo test                    # Run all tests
cargo test -- --nocapture     # Show output
cargo bench                   # Run benchmarks (if available)
```

## Related Repos

- [**creative-engine-c**](https://github.com/SuperInstance/creative-engine-c) — Original C implementation
- [**flux-ffi**](https://github.com/SuperInstance/flux-ffi) — FFI bindings bridging Rust and C
- [**superinstance-live**](https://github.com/SuperInstance/superinstance-live) — Live session controller
- [**constraint-toolkit**](https://github.com/SuperInstance/constraint-toolkit) — Constraint satisfaction engine
- [**flux-genome**](https://github.com/SuperInstance/flux-genome) — Genetic evolution of musical genomes

## License

MIT
