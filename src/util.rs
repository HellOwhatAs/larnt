//! Utility functions.
//!
//! This module provides utility functions for angle conversion, median computation,
//! parsing, and small geometry helpers.

use crate::common::EPS;
use crate::path::Paths;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ParamPoint {
    pub(crate) u: f64,
    pub(crate) v: f64,
}

impl ParamPoint {
    pub(crate) fn new(u: f64, v: f64) -> Self {
        Self { u, v }
    }

    fn lerp(self, other: Self, t: f64) -> Self {
        Self {
            u: self.u + (other.u - self.u) * t,
            v: self.v + (other.v - self.v) * t,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ParamEdge {
    U(usize),
    V(usize),
    Diagonal { u: usize, v: usize },
}

#[derive(Debug, Clone, Copy)]
struct EdgeEvent {
    lambda: f64,
    edge: ParamEdge,
}

/// Converts degrees to radians.
///
/// # Example
///
/// ```
/// use larnt::radians;
///
/// let rad = radians(90.0);
/// assert!((rad - std::f64::consts::PI / 2.0).abs() < 1e-10);
/// ```
pub fn radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

/// Converts radians to degrees.
///
/// # Example
///
/// ```
/// use larnt::degrees;
///
/// let deg = degrees(std::f64::consts::PI);
/// assert!((deg - 180.0).abs() < 1e-10);
/// ```
pub fn degrees(radians: f64) -> f64 {
    radians * 180.0 / std::f64::consts::PI
}

/// Computes the median of a sorted slice of floats.
///
/// # Arguments
/// * `items` - A sorted slice of f64 values
///
/// # Returns
/// The median value. Returns 0.0 for empty slices.
///
/// # Note
/// The caller must ensure the slice is sorted before calling this function.
pub fn median(items: &[f64]) -> f64 {
    let n = items.len();
    match n {
        0 => 0.0,
        _ if n % 2 == 1 => items[n / 2],
        _ => {
            let a = items[n / 2 - 1];
            let b = items[n / 2];
            (a + b) / 2.0
        }
    }
}

pub fn parse_floats(items: &[&str]) -> Vec<f64> {
    items
        .iter()
        .map(|s| s.parse::<f64>().unwrap_or(0.0))
        .collect()
}

fn param_in_bounds(u_steps: usize, v_steps: usize, point: ParamPoint) -> bool {
    (-EPS..=u_steps as f64 + EPS).contains(&point.u)
        && (-EPS..=v_steps as f64 + EPS).contains(&point.v)
}

fn integer_step(value: f64, steps: usize) -> Option<usize> {
    let step = value.round();
    if (value - step).abs() < EPS && step >= 0.0 && step <= steps as f64 {
        Some(step as usize)
    } else {
        None
    }
}

fn diagonal_cell_at(u_steps: usize, v_steps: usize, point: ParamPoint) -> Option<(usize, usize)> {
    let u_floor = point.u.floor() as isize;
    let v_floor = point.v.floor() as isize;

    for u in [u_floor - 1, u_floor] {
        for v in [v_floor - 1, v_floor] {
            if u < 0 || v < 0 || u >= u_steps as isize || v >= v_steps as isize {
                continue;
            }

            let s = point.u - u as f64;
            let t = point.v - v as f64;
            if (-EPS..=1.0 + EPS).contains(&s)
                && (-EPS..=1.0 + EPS).contains(&t)
                && (s + t - 1.0).abs() < EPS
            {
                return Some((u as usize, v as usize));
            }
        }
    }

    None
}

fn edge_at_point(u_steps: usize, v_steps: usize, point: ParamPoint) -> Option<ParamEdge> {
    if !param_in_bounds(u_steps, v_steps, point) {
        return None;
    }

    if let Some(u) = integer_step(point.u, u_steps) {
        return Some(ParamEdge::U(u));
    }

    if let Some(v) = integer_step(point.v, v_steps) {
        return Some(ParamEdge::V(v));
    }

    diagonal_cell_at(u_steps, v_steps, point).map(|(u, v)| ParamEdge::Diagonal { u, v })
}

fn normalize_lambda(lambda: f64) -> Option<f64> {
    (-EPS..=1.0 + EPS)
        .contains(&lambda)
        .then(|| lambda.clamp(0.0, 1.0))
}

fn sorted_edge_events(
    u_steps: usize,
    v_steps: usize,
    a: ParamPoint,
    b: ParamPoint,
) -> Vec<EdgeEvent> {
    if u_steps == 0 || v_steps == 0 {
        return Vec::new();
    }

    let du = b.u - a.u;
    let dv = b.v - a.v;
    let mut events = Vec::new();

    if let Some(edge) = edge_at_point(u_steps, v_steps, a) {
        events.push(EdgeEvent { lambda: 0.0, edge });
    }
    if let Some(edge) = edge_at_point(u_steps, v_steps, b) {
        events.push(EdgeEvent { lambda: 1.0, edge });
    }

    if du.abs() > EPS {
        for u in 0..=u_steps {
            let lambda = (u as f64 - a.u) / du;
            if let Some(lambda) = normalize_lambda(lambda) {
                let point = a.lerp(b, lambda);
                if (-EPS..=v_steps as f64 + EPS).contains(&point.v) {
                    events.push(EdgeEvent {
                        lambda,
                        edge: ParamEdge::U(u),
                    });
                }
            }
        }
    }

    if dv.abs() > EPS {
        for v in 0..=v_steps {
            let lambda = (v as f64 - a.v) / dv;
            if let Some(lambda) = normalize_lambda(lambda) {
                let point = a.lerp(b, lambda);
                if (-EPS..=u_steps as f64 + EPS).contains(&point.u) {
                    events.push(EdgeEvent {
                        lambda,
                        edge: ParamEdge::V(v),
                    });
                }
            }
        }
    }

    let dsum = du + dv;
    if dsum.abs() > EPS {
        let sum0 = a.u + a.v;
        let sum1 = b.u + b.v;
        let min_sum = sum0.min(sum1);
        let max_sum = sum0.max(sum1);
        let min_edge = min_sum.ceil() as isize;
        let max_edge = max_sum.floor() as isize;
        let max_diagonal = u_steps + v_steps - 1;

        for sum in min_edge..=max_edge {
            if sum < 1 || sum > max_diagonal as isize {
                continue;
            }

            let lambda = (sum as f64 - sum0) / dsum;
            if let Some(lambda) = normalize_lambda(lambda) {
                let point = a.lerp(b, lambda);
                if param_in_bounds(u_steps, v_steps, point)
                    && let Some((u, v)) = diagonal_cell_at(u_steps, v_steps, point)
                {
                    events.push(EdgeEvent {
                        lambda,
                        edge: ParamEdge::Diagonal { u, v },
                    });
                }
            }
        }
    }

    events.sort_by(|a, b| a.lambda.total_cmp(&b.lambda));

    let mut unique = Vec::with_capacity(events.len());
    for event in events {
        if unique
            .last()
            .is_some_and(|prev: &EdgeEvent| (prev.lambda - event.lambda).abs() < EPS)
        {
            continue;
        }
        unique.push(event);
    }
    unique
}

fn push_segment_point<T>(
    segment: &mut Vec<T>,
    point: T,
    same_point: &mut impl FnMut(&T, &T) -> bool,
) {
    if let Some(last) = segment.last()
        && same_point(last, &point)
    {
        return;
    }
    segment.push(point);
}

fn flush_segment<T>(paths: &mut Paths<T>, segment: &mut Vec<T>) {
    if segment.len() >= 2 {
        paths.new_path().extend(segment.drain(..));
    } else {
        segment.clear();
    }
}

fn append_edge_crossings<T>(
    steps: (usize, usize),
    paths: &mut Paths<T>,
    segment: &mut Vec<T>,
    a: ParamPoint,
    b: ParamPoint,
    edge_point: &mut impl FnMut(ParamEdge, ParamPoint) -> T,
    same_point: &mut impl FnMut(&T, &T) -> bool,
) {
    let (u_steps, v_steps) = steps;
    let events = sorted_edge_events(u_steps, v_steps, a, b);

    if events.is_empty() {
        if !param_in_bounds(u_steps, v_steps, b) {
            flush_segment(paths, segment);
        }
        return;
    }

    let mut previous_lambda = 0.0;
    for (i, &event) in events.iter().enumerate() {
        let lambda = event.lambda;
        let before_inside = if lambda > previous_lambda + EPS {
            param_in_bounds(
                u_steps,
                v_steps,
                a.lerp(b, (previous_lambda + lambda) * 0.5),
            )
        } else {
            param_in_bounds(u_steps, v_steps, a.lerp(b, lambda))
        };

        let next_lambda = events.get(i + 1).map_or(1.0, |event| event.lambda);
        let after_inside = if next_lambda > lambda + EPS {
            param_in_bounds(u_steps, v_steps, a.lerp(b, (lambda + next_lambda) * 0.5))
        } else {
            param_in_bounds(u_steps, v_steps, b)
        };

        if before_inside || after_inside {
            let point = edge_point(event.edge, a.lerp(b, lambda));
            push_segment_point(segment, point, same_point);
        }

        if !after_inside {
            flush_segment(paths, segment);
        }

        previous_lambda = lambda;
    }

    if !param_in_bounds(u_steps, v_steps, b) {
        flush_segment(paths, segment);
    }
}

pub(crate) fn trace_parametric_edge_curve<T>(
    u_steps: usize,
    v_steps: usize,
    samples: impl IntoIterator<Item = Option<ParamPoint>>,
    mut edge_point: impl FnMut(ParamEdge, ParamPoint) -> T,
    mut same_point: impl FnMut(&T, &T) -> bool,
) -> Paths<T> {
    let mut paths = Paths::new();
    let mut segment = Vec::new();
    let mut previous = None;

    for sample in samples {
        let Some(point) = sample else {
            previous = None;
            flush_segment(&mut paths, &mut segment);
            continue;
        };

        if let Some(prev_point) = previous {
            append_edge_crossings(
                (u_steps, v_steps),
                &mut paths,
                &mut segment,
                prev_point,
                point,
                &mut edge_point,
                &mut same_point,
            );
        }
        previous = Some(point);
    }

    flush_segment(&mut paths, &mut segment);
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    fn same_sample(a: &(ParamEdge, ParamPoint), b: &(ParamEdge, ParamPoint)) -> bool {
        a.0 == b.0 && (a.1.u - b.1.u).abs() < EPS && (a.1.v - b.1.v).abs() < EPS
    }

    fn assert_sample(actual: &(ParamEdge, ParamPoint), edge: ParamEdge, u: f64, v: f64) {
        assert_eq!(actual.0, edge);
        assert!((actual.1.u - u).abs() < EPS);
        assert!((actual.1.v - v).abs() < EPS);
    }

    #[test]
    fn traced_curve_outputs_only_edge_points() {
        let paths = trace_parametric_edge_curve(
            2,
            1,
            [
                Some(ParamPoint::new(0.25, 0.25)),
                Some(ParamPoint::new(1.75, 0.25)),
            ],
            |edge, point| (edge, point),
            same_sample,
        );

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].len(), 3);
        assert_sample(&paths[0][0], ParamEdge::Diagonal { u: 0, v: 0 }, 0.75, 0.25);
        assert_sample(&paths[0][1], ParamEdge::U(1), 1.0, 0.25);
        assert_sample(&paths[0][2], ParamEdge::Diagonal { u: 1, v: 0 }, 1.75, 0.25);
    }

    #[test]
    fn traced_curve_keeps_edge_aligned_segments() {
        let paths = trace_parametric_edge_curve(
            2,
            1,
            [
                Some(ParamPoint::new(1.0, 0.2)),
                Some(ParamPoint::new(1.0, 0.8)),
            ],
            |edge, point| (edge, point),
            same_sample,
        );

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].len(), 2);
        assert_sample(&paths[0][0], ParamEdge::U(1), 1.0, 0.2);
        assert_sample(&paths[0][1], ParamEdge::U(1), 1.0, 0.8);
    }
}
