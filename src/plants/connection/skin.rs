//! Explicit two-bone Dirichlet extension over the newly generated iso domain.
use super::Skin;
use crate::{
    mesh_material_finish::Plan,
    mesh_topology::Topology,
    meshes::{Geometry, Weight},
    model::{Result, invalid},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Report {
    pub iterations: u16,
    pub residual_max: f64,
    pub max_clamp: f64,
    pub minimum_weight: f64,
    pub maximum_weight: f64,
}

pub(super) fn extend(
    g: &Geometry,
    plan: &Plan,
    retained: &[Weight],
    root: usize,
    anchor: usize,
    options: &Skin,
) -> Result<(Vec<Weight>, Report)> {
    let start = plan.retained_vertices as usize;
    let end = start + plan.iso_vertices as usize;
    if start != retained.len() || end > g.positions.len() || end == start {
        return Err(invalid(
            "Connection skin provenance has inconsistent ranges",
        ));
    }
    let topology = Topology::inspect(g)?;
    let mut offsets = vec![0; g.positions.len() + 1];
    for e in &topology.edges {
        for i in e.vertices {
            offsets[i as usize + 1] += 1;
        }
    }
    for i in 1..offsets.len() {
        offsets[i] += offsets[i - 1];
    }
    let mut next = offsets[..g.positions.len()].to_vec();
    let mut adjacent = vec![0; offsets[g.positions.len()]];
    for e in &topology.edges {
        let [a, b] = e.vertices.map(|v| v as usize);
        adjacent[next[a]] = b;
        next[a] += 1;
        adjacent[next[b]] = a;
        next[b] += 1;
    }
    let neighbors = |i: usize| &adjacent[offsets[i]..offsets[i + 1]];
    let mut values = vec![0.; g.positions.len()];
    for (i, w) in retained.iter().enumerate() {
        let sum = w.influences().map(|(_, v)| f64::from(v)).sum::<f64>();
        values[i] = w
            .influences()
            .filter(|(j, _)| *j as usize == anchor)
            .map(|(_, w)| f64::from(w))
            .sum::<f64>()
            / sum;
    }
    let mut output = retained.to_vec();
    output.resize(
        g.positions.len(),
        Weight {
            extra: None,
            joints: [root as u8, anchor as u8, 0, 0],
            weights: [1., 0., 0., 0.],
        },
    );
    let mut guards = vec![false; g.positions.len() - end];
    for &[source, guard] in &plan.pairs {
        let (source, guard) = (source as usize, guard as usize);
        if source >= start || guard < end || guard >= values.len() || guards[guard - end] {
            return Err(invalid(
                "Connection skin needs exact unique guard provenance",
            ));
        }
        guards[guard - end] = true;
        values[guard] = values[source];
        output[guard] = retained[source].clone();
    }
    if guards.iter().any(|v| !*v) {
        return Err(invalid("Connection skin has an unbound guard"));
    }
    let count = end - start;
    let diagonal: Vec<_> = (start..end).map(|i| neighbors(i).len() as f64).collect();
    if diagonal.contains(&0.) {
        return Err(invalid("Connection skin has an isolated unknown"));
    }
    let rhs: Vec<_> = (start..end)
        .map(|i| {
            neighbors(i)
                .iter()
                .filter(|&&j| j < start || j >= end)
                .map(|&j| values[j])
                .sum::<f64>()
        })
        .collect();
    let apply = |x: &[f64], out: &mut [f64]| {
        for (row, i) in (start..end).enumerate() {
            out[row] = diagonal[row] * x[row]
                - neighbors(i)
                    .iter()
                    .filter(|&&j| (start..end).contains(&j))
                    .map(|&j| x[j - start])
                    .sum::<f64>();
        }
    };
    let mut x = vec![0.5; count];
    let mut product = vec![0.; count];
    apply(&x, &mut product);
    let mut residual: Vec<_> = rhs.iter().zip(&product).map(|(b, a)| b - a).collect();
    let mut z: Vec<_> = residual.iter().zip(&diagonal).map(|(r, d)| r / d).collect();
    let mut direction = z.clone();
    let mut rho = residual.iter().zip(&z).map(|(a, b)| a * b).sum::<f64>();
    let tolerance = f64::from(options.tolerance);
    let mut iterations = 0;
    for step in 0..options.iterations {
        if residual.iter().all(|r| r.abs() <= tolerance) {
            break;
        }
        apply(&direction, &mut product);
        let denominator = direction
            .iter()
            .zip(&product)
            .map(|(a, b)| a * b)
            .sum::<f64>();
        if !denominator.is_finite() || denominator <= 0. || !rho.is_finite() {
            return Err(invalid("Connection skin solver has no positive direction"));
        }
        let alpha = rho / denominator;
        for i in 0..count {
            x[i] += alpha * direction[i];
            residual[i] -= alpha * product[i];
            z[i] = residual[i] / diagonal[i];
        }
        let next = residual.iter().zip(&z).map(|(a, b)| a * b).sum::<f64>();
        for i in 0..count {
            direction[i] = z[i] + next / rho * direction[i];
        }
        rho = next;
        iterations = step + 1;
    }
    apply(&x, &mut product);
    let maximum = rhs
        .iter()
        .zip(&product)
        .map(|(b, a)| (b - a).abs())
        .fold(0., f64::max);
    if !maximum.is_finite() || maximum > tolerance {
        return Err(invalid(
            "Connection skin did not converge within its iteration budget",
        ));
    }
    let mut report = Report {
        iterations,
        residual_max: maximum,
        max_clamp: 0.,
        minimum_weight: f64::INFINITY,
        maximum_weight: f64::NEG_INFINITY,
    };
    for (i, value) in x.into_iter().enumerate() {
        if !value.is_finite() || value < -tolerance || value > 1. + tolerance {
            return Err(invalid(
                "Connection skin exceeds its maximum-principle tolerance",
            ));
        }
        let bounded = value.clamp(0., 1.);
        report.max_clamp = report.max_clamp.max((bounded - value).abs());
        report.minimum_weight = report.minimum_weight.min(bounded);
        report.maximum_weight = report.maximum_weight.max(bounded);
        output[start + i] = Weight {
            extra: None,
            joints: [root as u8, anchor as u8, 0, 0],
            weights: [(1. - bounded) as f32, bounded as f32, 0., 0.],
        };
    }
    Ok((output, report))
}
