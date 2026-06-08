# INTEGRATION.md — creative-engine-rust

## Role in the SuperInstance Ecosystem

creative-engine-rust is the **nonlinear creative dynamics engine** for SuperInstance agents. It models each agent's creative state as a Lorenz system (σ, ρ, β) and provides regime detection, quality metrics, adaptive thermostats, and asymmetric coupling for multi-agent creative networks.

## SuperInstance Integration Points

### 1. si-runtime-python — Budget-Aware Creative Control
- `AgentBudget` (si-runtime-python) controls how much "creative energy" an agent may expend
- `CreativeThermostat` reads `budget.gamma` (exploration) and `budget.eta` (exploitation) to set its target diversity:
  ```rust
  target_diversity = gamma / (gamma + eta)
  ```
- When `budget.transfer(amount)` is called, the thermostat adjusts `epsilon` (coupling strength) to steer the system toward the new diversity target

### 2. constraint-dynamics-rs — Regime-Aware Constraints
- `CreativeSystem.quality()` returns `QualityMetrics` with `diversity` and `coherence`
- These metrics feed into `EnergyLandscape` as soft constraints:
  - diversity < threshold → loosen constraints (allow more chaos)
  - coherence < threshold → tighten constraints (enforce structure)
- The solver's `RelaxedSolution` reports how much regime adjustment is needed

### 3. flux-hyperbolic-rs — Tradition-Coupled Creativity
- `CreativeNetwork` coupling matrix can be initialized from hyperbolic tradition distances:
  ```rust
  let coupling = 1.0 / (1.0 + poincare.distance(&t_a, &t_b));
  network.set_coupling(i, j, coupling);
  ```
- Agents from distant traditions couple weakly (preserving distinct voices); close traditions couple strongly (fusion)
- `TraditionEmbedding::from_dial()` provides the 3D state vector for `CreativeSystem` initialization

### 4. superinstance-live — Real-Time Creative Rooms
- `FluxRoomPipeline` (superinstance-live) hosts one `CreativeSystem` per room
- Transport ticks drive `system.step()` at audio-rate or control-rate intervals
- `set_param("epsilon", value)` maps directly to `CreativeThermostat.set_target()`
- `trigger()` injects a perturbation into the Lorenz system (creative "spark")
- `PipelineEvent` carries `quality()` metrics back to the session's OSC bridge

### 5. si-cli — Fleet Health from Creative Metrics
- `si-cli check` validates that all `CreativeSystem` instances have:
  - Valid ρ values (ρ > 1 for non-trivial dynamics)
  - Non-negative σ, β
  - Quality metrics within expected ranges
- `si-cli rank` includes `diversity × coherence` as a capability metric in spectral ranking

### 6. plato-adapters — Adapter-Driven State I/O
- `plato-adapters.transform.normalize` scales raw sensor/OSC input into Lorenz state variables
- `AdapterRegistry.chain(["osc_to_state", "normalize_lorenz", "step_system"])` enables external control of creative dynamics without manual wiring

## Dial / Room / Snap Compatibility

| Primitive | Mapping |
|-----------|---------|
| **Dial**  | `rho` parameter directly: low ρ ≈ 0.1 (FixedPoint, rigid), high ρ ≈ 0.9 (Chaotic, free) |
| **Room**  | Each `CreativeSystem` is a Room; `CreativeNetwork` is a fleet of Rooms with coupling edges |
| **Snap**  | `system.snap()` forces ρ into Periodic regime (ρ ≈ 24.74), freezing creative output at a stable orbit |
| **Cascade**| Parent system's final state seeds child initial conditions; perturbation propagates through coupling matrix |

## Energy Conservation

The Lorenz system itself is energy-conserving in a generalized sense (dissipative but volume-contracting). SuperInstance maps this to the conservation budget:

```
γ (exploration) → drives ρ upward (toward chaos)
η (exploitation) → drives ρ downward (toward order)
total = γ + η = constant
```

`CreativeThermostat` implements a gradient descent on `|diversity - target_diversity|` using the conservation budget as its learning-rate envelope. It never spends more than `gamma` on exploratory perturbations or more than `eta` on convergence refinement.

## Quick Start

```rust
use creative_engine::{CreativeSystem, CreativeNetwork, Regime};

let mut sys = CreativeSystem::new(10.0, 28.0, 8.0 / 3.0);
sys.run(1000, 500); // burn-in
let q = sys.quality();
println!("novelty={:.3} coherence={:.3} quality={:.3}", q.novelty, q.coherence, q.quality);

// Couple two agents
let mut net = CreativeNetwork::new(vec![sys, CreativeSystem::default()]);
net.set_coupling(0, 1, 0.3);
net.step();
```

## Tests

```bash
cargo test
```

Regime detection, quality metric bounds, thermostat convergence, and network symmetry tests must pass.
