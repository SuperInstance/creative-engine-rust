//! Creative Dynamics Engine
//! Lorenz-based creative system with regime detection, adaptive ε, quality metrics, and coupling.

/// The three creative regimes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Regime {
    FixedPoint,  // ρ < ~5, rigid, no creativity
    Periodic,    // 5 < ρ < 24.74, oscillating, SR active
    Chaotic,     // ρ > 24.74, strange attractor, full creativity
}

impl Regime {
    pub fn from_rho(rho: f64) -> Self {
        if rho < 5.0 {
            Regime::FixedPoint
        } else if rho <= 24.74 {
            Regime::Periodic
        } else {
            Regime::Chaotic
        }
    }

    pub fn optimal_epsilon(&self) -> f64 {
        match self {
            Regime::FixedPoint => 1.5,
            Regime::Periodic => 0.5,
            Regime::Chaotic => 0.2,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Regime::FixedPoint => "Rigid — needs noise/freedom to escape",
            Regime::Periodic => "Oscillating — stochastic resonance active",
            Regime::Chaotic => "Creative — strange attractor, needs focus",
        }
    }
}

/// Quality metrics for creative output
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub novelty: f64,
    pub coherence: f64,
    pub quality: f64,
}

impl QualityMetrics {
    pub fn compute(outputs: &[f64]) -> Self {
        if outputs.is_empty() {
            return QualityMetrics { novelty: 0.0, coherence: 0.0, quality: 0.0 };
        }

        let mean = outputs.iter().sum::<f64>() / outputs.len() as f64;
        let variance = outputs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / outputs.len() as f64;
        let novelty = variance.sqrt();

        let n = outputs.len().min(256);
        let slice = &outputs[outputs.len() - n..];

        let mut spectral_sum = 0.0;
        let mut spectral_max = 0.0;
        for k in 0..n / 2 {
            let mut re = 0.0;
            let mut im = 0.0;
            for (i, &x) in slice.iter().enumerate() {
                let angle = 2.0 * std::f64::consts::PI * k as f64 * i as f64 / n as f64;
                re += x * angle.cos();
                im -= x * angle.sin();
            }
            let mag = (re * re + im * im).sqrt();
            spectral_sum += mag;
            if mag > spectral_max { spectral_max = mag; }
        }

        let coherence = if spectral_sum > 0.0 {
            (spectral_max / spectral_sum * (n / 2) as f64).min(1.0)
        } else {
            0.0
        };

        let quality = novelty * coherence;

        QualityMetrics { novelty, coherence, quality }
    }
}

/// A single Lorenz creative system
#[derive(Debug, Clone)]
pub struct CreativeSystem {
    pub sigma: f64,
    pub rho: f64,
    pub beta: f64,
    pub state: [f64; 3],
    pub dt: f64,
    pub epsilon: f64,
    pub outputs: Vec<f64>,
}

impl CreativeSystem {
    pub fn new(rho: f64) -> Self {
        CreativeSystem {
            sigma: 10.0,
            rho,
            beta: 8.0 / 3.0,
            state: [0.1, 0.1, 0.1],
            dt: 0.01,
            epsilon: Regime::from_rho(rho).optimal_epsilon(),
            outputs: Vec::new(),
        }
    }

    pub fn with_sigma(mut self, sigma: f64) -> Self {
        self.sigma = sigma;
        self
    }

    pub fn with_epsilon(mut self, epsilon: f64) -> Self {
        self.epsilon = epsilon;
        self
    }

    /// Single RK4 step
    pub fn step(&mut self) -> f64 {
        let [x, y, z] = self.state;
        let s = self.sigma;
        let r = self.rho;
        let b = self.beta;
        let dt = self.dt;

        let k1x = s * (y - x);
        let k1y = x * (r - z) - y;
        let k1z = x * y - b * z;

        let x2 = x + 0.5 * dt * k1x;
        let y2 = y + 0.5 * dt * k1y;
        let z2 = z + 0.5 * dt * k1z;
        let k2x = s * (y2 - x2);
        let k2y = x2 * (r - z2) - y2;
        let k2z = x2 * y2 - b * z2;

        let x3 = x + 0.5 * dt * k2x;
        let y3 = y + 0.5 * dt * k2y;
        let z3 = z + 0.5 * dt * k2z;
        let k3x = s * (y3 - x3);
        let k3y = x3 * (r - z3) - y3;
        let k3z = x3 * y3 - b * z3;

        let x4 = x + dt * k3x;
        let y4 = y + dt * k3y;
        let z4 = z + dt * k3z;
        let k4x = s * (y4 - x4);
        let k4y = x4 * (r - z4) - y4;
        let k4z = x4 * y4 - b * z4;

        self.state[0] += (dt / 6.0) * (k1x + 2.0 * k2x + 2.0 * k3x + k4x);
        self.state[1] += (dt / 6.0) * (k1y + 2.0 * k2y + 2.0 * k3y + k4y);
        self.state[2] += (dt / 6.0) * (k1z + 2.0 * k2z + 2.0 * k3z + k4z);

        self.outputs.push(self.state[0]);
        self.state[0]
    }

    /// Run for N steps, discard transient
    pub fn run(&mut self, n_steps: usize, discard: usize) {
        for _ in 0..discard { self.step(); }
        self.outputs.clear();
        for _ in 0..n_steps { self.step(); }
    }

    pub fn detect_regime(&self) -> Regime {
        Regime::from_rho(self.rho)
    }

    pub fn quality(&self) -> QualityMetrics {
        QualityMetrics::compute(&self.outputs)
    }

    pub fn diversity(&self) -> f64 {
        if self.outputs.is_empty() { return 0.0; }
        let mean = self.outputs.iter().sum::<f64>() / self.outputs.len() as f64;
        let var = self.outputs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / self.outputs.len() as f64;
        var.sqrt()
    }

    pub fn soft_snap(x: f64, epsilon: f64) -> f64 {
        let snapped = x.round();
        (1.0 - epsilon) * snapped + epsilon * x
    }

    pub fn sigmoid(x: f64, k: f64, x0: f64) -> f64 {
        1.0 / (1.0 + (-k * (x - x0)).exp())
    }

    pub fn sync_order(phases: &[f64]) -> f64 {
        let n = phases.len() as f64;
        let cos_sum: f64 = phases.iter().map(|p| p.cos()).sum();
        let sin_sum: f64 = phases.iter().map(|p| p.sin()).sum();
        ((cos_sum / n).powi(2) + (sin_sum / n).powi(2)).sqrt()
    }
}

/// A network of coupled creative agents
#[derive(Debug, Clone)]
pub struct CreativeNetwork {
    pub agents: Vec<CreativeSystem>,
    pub coupling_matrix: Vec<Vec<f64>>,
}

impl CreativeNetwork {
    pub fn new(expertises: &[f64]) -> Self {
        let agents: Vec<CreativeSystem> = expertises.iter()
            .map(|&e| CreativeSystem::new(1.0 + e * 49.0))
            .collect();

        let n = agents.len();

        let mut coupling = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                if i != j && expertises[j] > expertises[i] {
                    coupling[i][j] = 0.01;
                }
            }
        }

        CreativeNetwork { agents, coupling_matrix: coupling }
    }

    pub fn step(&mut self) -> Vec<f64> {
        let outputs: Vec<f64> = self.agents.iter_mut().map(|a| a.step()).collect();

        for i in 0..self.agents.len() {
            let mut signal = 0.0;
            for j in 0..self.agents.len() {
                if self.coupling_matrix[i][j] > 0.0 {
                    signal += self.coupling_matrix[i][j] * (outputs[j] - outputs[i]);
                }
            }
            self.agents[i].state[0] += signal * self.agents[i].dt;
        }

        outputs
    }

    pub fn run(&mut self, n_steps: usize) {
        for _ in 0..n_steps { self.step(); }
    }

    pub fn total_quality(&self) -> f64 {
        self.agents.iter().map(|a| a.quality().quality).sum()
    }

    pub fn total_diversity(&self) -> f64 {
        self.agents.iter().map(|a| a.diversity()).sum()
    }
}

/// The Creative Thermostat — adaptive ε that tracks regime
#[derive(Debug, Clone)]
pub struct CreativeThermostat {
    pub system: CreativeSystem,
    pub target_diversity: f64,
    pub learning_rate: f64,
    pub history: Vec<(f64, f64)>,
}

impl CreativeThermostat {
    pub fn new(rho: f64, target_diversity: f64) -> Self {
        CreativeThermostat {
            system: CreativeSystem::new(rho),
            target_diversity,
            learning_rate: 0.01,
            history: Vec::new(),
        }
    }

    pub fn adapt(&mut self) -> f64 {
        self.system.run(100, 50);
        let div = self.system.diversity();

        let error = self.target_diversity - div;
        self.system.epsilon += self.learning_rate * error;
        self.system.epsilon = self.system.epsilon.max(0.01).min(2.0);

        self.history.push((self.system.epsilon, div));
        self.system.epsilon
    }

    pub fn run_thermostat(&mut self, n_cycles: usize) {
        for _ in 0..n_cycles { self.adapt(); }
    }

    pub fn converged_epsilon(&self) -> f64 {
        if self.history.is_empty() { return self.system.epsilon; }
        let recent: Vec<f64> = self.history.iter().rev().take(10).map(|(e, _)| *e).collect();
        recent.iter().sum::<f64>() / recent.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regime_detection() {
        assert_eq!(Regime::from_rho(1.0), Regime::FixedPoint);
        assert_eq!(Regime::from_rho(15.0), Regime::Periodic);
        assert_eq!(Regime::from_rho(28.0), Regime::Chaotic);
        assert_eq!(Regime::from_rho(24.74), Regime::Periodic);
    }

    #[test]
    fn test_optimal_epsilon() {
        assert!(Regime::FixedPoint.optimal_epsilon() > Regime::Periodic.optimal_epsilon());
        assert!(Regime::Periodic.optimal_epsilon() > Regime::Chaotic.optimal_epsilon());
    }

    #[test]
    fn test_lorenz_step() {
        let mut sys = CreativeSystem::new(28.0);
        let initial = sys.state;
        sys.step();
        assert_ne!(sys.state, initial);
    }

    #[test]
    fn test_lorenz_chaotic_regime() {
        let mut sys = CreativeSystem::new(28.0);
        sys.run(5000, 1000);
        let div = sys.diversity();
        assert!(div > 1.0, "Chaotic regime should have diversity > 1.0, got {}", div);
    }

    #[test]
    fn test_lorenz_fixed_point() {
        let mut sys = CreativeSystem::new(1.0);
        sys.run(5000, 1000);
        let div = sys.diversity();
        assert!(div < 1.0, "Fixed-point regime should have diversity < 1.0, got {}", div);
    }

    #[test]
    fn test_soft_snap() {
        assert_eq!(CreativeSystem::soft_snap(2.7, 0.0), 3.0);
        assert_eq!(CreativeSystem::soft_snap(2.3, 0.0), 2.0);
        assert!((CreativeSystem::soft_snap(2.7, 1.0) - 2.7).abs() < 1e-10);
        let result = CreativeSystem::soft_snap(2.6, 0.5);
        assert!((result - 2.8).abs() < 1e-10);
    }

    #[test]
    fn test_sigmoid() {
        assert!((CreativeSystem::sigmoid(0.5, 1.0, 0.5) - 0.5).abs() < 1e-10);
        assert!(CreativeSystem::sigmoid(-10.0, 1.0, 0.5) < 0.01);
        assert!(CreativeSystem::sigmoid(10.0, 1.0, 0.5) > 0.99);
    }

    #[test]
    fn test_sync_order() {
        let synced = vec![0.0; 10];
        assert!(CreativeSystem::sync_order(&synced) > 0.99);

        let random: Vec<f64> = (0..100).map(|i| std::f64::consts::PI * i as f64 / 50.0).collect();
        let order = CreativeSystem::sync_order(&random);
        assert!(order < 0.5, "Random phases should have low order, got {}", order);
    }

    #[test]
    fn test_quality_metrics() {
        let constant = vec![1.0; 100];
        let q = QualityMetrics::compute(&constant);
        assert_eq!(q.novelty, 0.0);
        assert_eq!(q.quality, 0.0);

        let mut sys = CreativeSystem::new(28.0);
        sys.run(1000, 500);
        let q = sys.quality();
        assert!(q.novelty > 0.0);
    }

    #[test]
    fn test_network_creation() {
        let expertises = vec![0.1, 0.5, 0.9];
        let net = CreativeNetwork::new(&expertises);
        assert_eq!(net.agents.len(), 3);
        assert!(net.coupling_matrix[0][1] > 0.0);
        assert!(net.coupling_matrix[0][2] > 0.0);
        assert_eq!(net.coupling_matrix[2][0], 0.0);
    }

    #[test]
    fn test_network_runs() {
        let expertises = vec![0.1, 0.5, 0.9];
        let mut net = CreativeNetwork::new(&expertises);
        net.run(1000);
        for agent in &net.agents {
            assert!(!agent.outputs.is_empty());
        }
        let total_q = net.total_quality();
        assert!(total_q >= 0.0);
    }

    #[test]
    fn test_thermostat_adapts() {
        let mut thermo = CreativeThermostat::new(28.0, 5.0);
        let initial_eps = thermo.system.epsilon;
        thermo.run_thermostat(50);
        let final_eps = thermo.converged_epsilon();
        assert_ne!(final_eps, initial_eps);
        assert!(!thermo.history.is_empty());
    }

    // --- Additional coverage tests ---

    #[test]
    fn test_regime_descriptions() {
        assert!(!Regime::FixedPoint.description().is_empty());
        assert!(!Regime::Periodic.description().is_empty());
        assert!(!Regime::Chaotic.description().is_empty());
        assert!(Regime::FixedPoint.description().contains("Rigid"));
        assert!(Regime::Periodic.description().contains("Oscillating"));
        assert!(Regime::Chaotic.description().contains("Creative"));
    }

    #[test]
    fn test_regime_boundary_values() {
        assert_eq!(Regime::from_rho(0.0), Regime::FixedPoint);
        assert_eq!(Regime::from_rho(4.999), Regime::FixedPoint);
        assert_eq!(Regime::from_rho(5.0), Regime::Periodic);
        assert_eq!(Regime::from_rho(24.739), Regime::Periodic);
        assert_eq!(Regime::from_rho(24.75), Regime::Chaotic);
        assert_eq!(Regime::from_rho(100.0), Regime::Chaotic);
    }

    #[test]
    fn test_regime_from_rho_matches_detect_regime() {
        for rho in [1.0, 5.0, 10.0, 24.74, 28.0, 50.0] {
            let mut sys = CreativeSystem::new(rho);
            assert_eq!(sys.detect_regime(), Regime::from_rho(rho));
        }
    }

    #[test]
    fn test_with_sigma_builder() {
        let sys = CreativeSystem::new(28.0).with_sigma(5.0);
        assert!((sys.sigma - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_with_epsilon_builder() {
        let sys = CreativeSystem::new(28.0).with_epsilon(0.99);
        assert!((sys.epsilon - 0.99).abs() < 1e-10);
    }

    #[test]
    fn test_chained_builders() {
        let sys = CreativeSystem::new(15.0).with_sigma(7.0).with_epsilon(0.3);
        assert!((sys.sigma - 7.0).abs() < 1e-10);
        assert!((sys.epsilon - 0.3).abs() < 1e-10);
        assert!((sys.rho - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_step_returns_output() {
        let mut sys = CreativeSystem::new(28.0);
        let out = sys.step();
        assert!(!out.is_nan());
        assert_eq!(sys.outputs.len(), 1);
        assert!((sys.outputs[0] - out).abs() < 1e-10);
    }

    #[test]
    fn test_run_clears_and_fills_outputs() {
        let mut sys = CreativeSystem::new(28.0);
        sys.step();
        assert_eq!(sys.outputs.len(), 1);
        sys.run(200, 50);
        assert_eq!(sys.outputs.len(), 200);
    }

    #[test]
    fn test_default_params() {
        let sys = CreativeSystem::new(28.0);
        assert!((sys.sigma - 10.0).abs() < 1e-10);
        assert!((sys.beta - 8.0 / 3.0).abs() < 1e-10);
        assert!((sys.dt - 0.01).abs() < 1e-10);
        assert_eq!(sys.state, [0.1, 0.1, 0.1]);
    }

    #[test]
    fn test_quality_metrics_empty() {
        let q = QualityMetrics::compute(&[]);
        assert_eq!(q.novelty, 0.0);
        assert_eq!(q.coherence, 0.0);
        assert_eq!(q.quality, 0.0);
    }

    #[test]
    fn test_quality_metrics_single_value() {
        let q = QualityMetrics::compute(&[42.0]);
        assert_eq!(q.novelty, 0.0);
        // Single element => coherence should be 0 or 1 depending on DFT of len-1
        assert!(q.coherence >= 0.0 && q.coherence <= 1.0);
    }

    #[test]
    fn test_diversity_empty_outputs() {
        let sys = CreativeSystem::new(28.0);
        assert_eq!(sys.diversity(), 0.0);
    }

    #[test]
    fn test_diversity_matches_novelty() {
        let mut sys = CreativeSystem::new(28.0);
        sys.run(500, 100);
        let div = sys.diversity();
        let nov = sys.quality().novelty;
        assert!((div - nov).abs() < 1e-10, "diversity {} should equal novelty {}", div, nov);
    }

    #[test]
    fn test_soft_snap_edge_cases() {
        // Exact integer
        assert_eq!(CreativeSystem::soft_snap(3.0, 0.5), 3.0 * 0.5 + 3.0 * 0.5);
        // Negative values
        let result = CreativeSystem::soft_snap(-2.3, 0.0);
        assert_eq!(result, -2.0);
    }

    #[test]
    fn test_sigmoid_steepness() {
        let gentle = CreativeSystem::sigmoid(1.0, 0.1, 0.0);
        let steep = CreativeSystem::sigmoid(1.0, 10.0, 0.0);
        assert!(steep > gentle, "Steeper k should give higher sigmoid for positive x-x0");
    }

    #[test]
    fn test_sync_order_uniform_phases() {
        // Uniformly distributed phases => low order
        let phases: Vec<f64> = (0..100).map(|i| 2.0 * std::f64::consts::PI * i as f64 / 100.0).collect();
        let order = CreativeSystem::sync_order(&phases);
        assert!(order < 0.1, "Uniform phases should have near-zero order, got {}", order);
    }

    #[test]
    fn test_network_coupling_asymmetry() {
        let expertises = vec![0.2, 0.5, 0.8];
        let net = CreativeNetwork::new(&expertises);
        // Low expertise should couple to higher
        assert!(net.coupling_matrix[0][2] > 0.0, "Low expertise should couple to high");
        // High expertise should NOT couple to lower
        assert_eq!(net.coupling_matrix[2][0], 0.0, "High expertise should not couple to low");
        // No self-coupling
        for i in 0..3 {
            assert_eq!(net.coupling_matrix[i][i], 0.0, "No self-coupling at index {}", i);
        }
    }

    #[test]
    fn test_network_step_returns_outputs() {
        let expertises = vec![0.3, 0.6, 0.9];
        let mut net = CreativeNetwork::new(&expertises);
        let outputs = net.step();
        assert_eq!(outputs.len(), 3);
        for out in &outputs {
            assert!(!out.is_nan());
        }
    }

    #[test]
    fn test_network_total_diversity() {
        let expertises = vec![0.3, 0.6, 0.9];
        let mut net = CreativeNetwork::new(&expertises);
        net.run(500);
        let div = net.total_diversity();
        assert!(div >= 0.0);
    }

    #[test]
    fn test_thermostat_converged_epsilon_empty_history() {
        let thermo = CreativeThermostat::new(28.0, 5.0);
        let eps = thermo.converged_epsilon();
        assert!((eps - thermo.system.epsilon).abs() < 1e-10);
    }

    #[test]
    fn test_thermostat_epsilon_bounded() {
        let mut thermo = CreativeThermostat::new(28.0, 5.0);
        thermo.run_thermostat(100);
        for (eps, _) in &thermo.history {
            assert!(*eps >= 0.01 && *eps <= 2.0, "epsilon {} out of bounds", eps);
        }
    }

    #[test]
    fn test_thermostat_history_grows() {
        let mut thermo = CreativeThermostat::new(28.0, 5.0);
        assert!(thermo.history.is_empty());
        thermo.adapt();
        assert_eq!(thermo.history.len(), 1);
        thermo.adapt();
        assert_eq!(thermo.history.len(), 2);
    }

    #[test]
    fn test_regime_epsilon_decreases() {
        let mut epsilons = Vec::new();
        for rho in [5.0, 15.0, 28.0, 45.0] {
            let mut thermo = CreativeThermostat::new(rho, 5.0);
            thermo.run_thermostat(30);
            epsilons.push(thermo.converged_epsilon());
        }
        assert!(epsilons[0] >= epsilons[2],
            "Beginner ε ({}) should be >= expert ε ({})", epsilons[0], epsilons[2]);
    }
}
