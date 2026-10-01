// This file includes untranslated text (ja).

use alloc::vec::Vec;
use core::primitive::{f64, usize};

use libm;

use crate::{Parameter, Point, Unit};

pub struct Line<const D: usize> {
    pub start: Point<D>,
    pub end:   Point<D>,
}

/// ```
/// use rectgrid::Unit;
/// use rectgrid::geometry::*;
///
/// let line = Line { start: [Unit::new(0.0), Unit::new(0.0)], end: [Unit::new(10.0), Unit::new(0.0)] };
/// let result = as_on_line([Unit::new(5.0), Unit::new(5.0)], line);
/// assert_eq!(result.t.get(), 0.5);
/// assert_eq!(result.signed_distance.get().abs(), 5.0);
/// ```
pub fn as_on_line(point: [Unit; 2], line: Line<2>) -> PointOnGeometry<2> {
    let px = point[0].get();
    let py = point[1].get();
    let x1 = line.start[0].get();
    let y1 = line.start[1].get();
    let x2 = line.end[0].get();
    let y2 = line.end[1].get();

    let vx = x2 - x1;
    let vy = y2 - y1;
    let wx = px - x1;
    let wy = py - y1;

    let length_squared = vx * vx + vy * vy;
    let t = if length_squared == 0.0 { 0.0 } else { (wx * vx + wy * vy) / length_squared };

    let proj_x = x1 + t * vx;
    let proj_y = y1 + t * vy;

    let dx = px - proj_x;
    let dy = py - proj_y;
    let distance = libm::sqrt(dx * dx + dy * dy);

    let cross = vx * wy - vy * wx;
    let sign = if cross < 0.0 { -1.0 } else { 1.0 };

    PointOnGeometry {
        t:               Parameter::new(t),
        projected:       [Unit::new(proj_x), Unit::new(proj_y)],
        signed_distance: Unit::new(sign * distance),
    }
}

pub struct Circle<const D: usize> {
    pub center: Point<D>,
    pub radius: Unit,
}

impl Circle<2> {
    /// ```
    /// use rectgrid::Unit;
    /// use rectgrid::geometry::*;
    ///
    /// let result = Circle::from_three_points(
    ///     [Unit::new(0.0), Unit::new(1.0)],
    ///     [Unit::new(1.0), Unit::new(0.0)],
    ///     [Unit::new(0.0), Unit::new(-1.0)],
    /// ).unwrap();
    /// assert!((result.center[0].get()).abs() < 1e-8);
    /// assert!((result.center[1].get()).abs() < 1e-8);
    /// assert!((result.radius.get() - 1.0).abs() < 1e-8);
    /// ```
    pub fn from_three_points(a: Point<2>, b: Point<2>, c: Point<2>) -> Option<Self> {
        // Collinearity threshold relative to the longest side: cross = 2 * area, so
        // cross / longest² is scale-invariant (an absolute threshold rejects small circles).
        const RELATIVE_EPSILON: f64 = 1e-12;

        let ax = a[0].get();
        let ay = a[1].get();
        // Solve relative to `a`: roundoff then scales with the differences between the points,
        // not with their absolute coordinates (Shewchuk, "Lecture Notes on Geometric Robustness").
        let bx = b[0].get() - ax;
        let by = b[1].get() - ay;
        let cx = c[0].get() - ax;
        let cy = c[1].get() - ay;

        let b_sq = bx * bx + by * by;
        let c_sq = cx * cx + cy * cy;
        let bc_sq = (bx - cx) * (bx - cx) + (by - cy) * (by - cy);
        let longest_sq = b_sq.max(c_sq).max(bc_sq);

        let cross = bx * cy - by * cx;
        if cross.abs() <= RELATIVE_EPSILON * longest_sq {
            return None;
        }

        let denominator = 2.0 * cross;
        let center_x = (cy * b_sq - by * c_sq) / denominator;
        let center_y = (bx * c_sq - cx * b_sq) / denominator;

        let radius = libm::sqrt(center_x * center_x + center_y * center_y);

        Some(Circle {
            center: [Unit::new(ax + center_x), Unit::new(ay + center_y)],
            radius: Unit::new(radius),
        })
    }
}

/// ```
/// use rectgrid::Unit;
/// use rectgrid::geometry::*;
///
/// let circle = Circle { center: [Unit::new(0.0), Unit::new(0.0)], radius: Unit::new(5.0) };
/// let result = as_on_circle([Unit::new(0.0), Unit::new(0.0)], circle);
/// assert_eq!(result.signed_distance.get(), -5.0);
/// ```
pub fn as_on_circle(point: [Unit; 2], circle: Circle<2>) -> PointOnGeometry<2> {
    let px = point[0].get();
    let py = point[1].get();
    let cx = circle.center[0].get();
    let cy = circle.center[1].get();
    let radius = circle.radius.get();

    let dx = px - cx;
    let dy = py - cy;
    let distance_from_center = libm::sqrt(dx * dx + dy * dy);

    let t = libm::atan2(dy, dx);

    let (proj_x, proj_y) = if distance_from_center == 0.0 {
        (cx + radius, cy)
    } else {
        let scale = radius / distance_from_center;
        (cx + dx * scale, cy + dy * scale)
    };

    PointOnGeometry {
        t:               Parameter::new(t),
        projected:       [Unit::new(proj_x), Unit::new(proj_y)],
        signed_distance: Unit::new(distance_from_center - radius),
    }
}

pub struct Ellipse<const D: usize> {
    pub center: Point<D>,
    pub rx:     Unit,
    pub ry:     Unit,
}

/// Exact signed distance to the ellipse and its closest point (the foot of the normal), after
/// Eberly, "Distance from a Point to an Ellipse, an Ellipsoid, or a Hyperellipsoid" (2013).
/// `t` is the parametric (eccentric) angle of the closest point.
///
/// ```
/// use rectgrid::Unit;
/// use rectgrid::geometry::*;
///
/// let ellipse = Ellipse { center: [Unit::new(0.0), Unit::new(0.0)], rx: Unit::new(4.0), ry: Unit::new(3.0) };
/// let result = as_on_ellipse([Unit::new(8.0), Unit::new(0.0)], ellipse);
/// assert_eq!(result.signed_distance.get(), 4.0);
/// ```
pub fn as_on_ellipse(point: [Unit; 2], ellipse: Ellipse<2>) -> PointOnGeometry<2> {
    as_on_ellipse_with_root(point, ellipse, ellipse_root)
}

/// `as_on_ellipse` with the root finder injected, so bisection and Newton can be compared.
/// `root(e0, e1, delta, y0, y1)` solves F(u) = 0 as documented on `ellipse_root`.
fn as_on_ellipse_with_root(
    point: [Unit; 2],
    ellipse: Ellipse<2>,
    root: impl Fn(f64, f64, f64, f64, f64) -> f64,
) -> PointOnGeometry<2> {
    let px = point[0].get();
    let py = point[1].get();
    let cx = ellipse.center[0].get();
    let cy = ellipse.center[1].get();
    let rx = ellipse.rx.get();
    let ry = ellipse.ry.get();

    let raw_dx = px - cx;
    let raw_dy = py - cy;

    if rx == 0.0 || ry == 0.0 {
        let distance = libm::sqrt(raw_dx * raw_dx + raw_dy * raw_dy);
        return PointOnGeometry {
            t:               Parameter::new(libm::atan2(raw_dy, raw_dx)),
            projected:       [Unit::new(cx), Unit::new(cy)],
            signed_distance: Unit::new(distance),
        };
    }

    let rx = rx.abs();
    let ry = ry.abs();

    // Reduce to the first quadrant with the semi-major axis first (e0 >= e1), then undo.
    let swap = ry > rx;
    let (e0, e1) = if swap { (ry, rx) } else { (rx, ry) };
    let (y0, y1) = if swap { (raw_dy.abs(), raw_dx.abs()) } else { (raw_dx.abs(), raw_dy.abs()) };

    let (x0, x1) = ellipse_closest_in_first_quadrant(e0, e1, y0, y1, root);

    let (foot_x, foot_y) = if swap { (x1, x0) } else { (x0, x1) };
    let foot_x = if raw_dx < 0.0 { -foot_x } else { foot_x };
    let foot_y = if raw_dy < 0.0 { -foot_y } else { foot_y };

    let distance = libm::sqrt((x0 - y0) * (x0 - y0) + (x1 - y1) * (x1 - y1));
    let ratio0 = y0 / e0;
    let ratio1 = y1 / e1;
    let signed_distance =
        if ratio0 * ratio0 + ratio1 * ratio1 < 1.0 { -distance } else { distance };

    PointOnGeometry {
        t:               Parameter::new(libm::atan2(foot_y / ry, foot_x / rx)),
        projected:       [Unit::new(cx + foot_x), Unit::new(cy + foot_y)],
        signed_distance: Unit::new(signed_distance),
    }
}

/// Closest point on the ellipse (x0/e0)² + (x1/e1)² = 1 to (y0, y1), where e0 >= e1 > 0 and
/// y0, y1 >= 0 (Eberly, Listing 1; the circle is handled separately).
fn ellipse_closest_in_first_quadrant(
    e0: f64,
    e1: f64,
    y0: f64,
    y1: f64,
    root: impl Fn(f64, f64, f64, f64, f64) -> f64,
) -> (f64, f64) {
    if e0 == e1 {
        let norm = libm::sqrt(y0 * y0 + y1 * y1);
        return if norm == 0.0 { (e0, 0.0) } else { (e0 * y0 / norm, e0 * y1 / norm) };
    }

    let e0_sq = e0 * e0;
    let e1_sq = e1 * e1;

    if y1 > 0.0 {
        if y0 > 0.0 {
            let delta = e0_sq - e1_sq;
            let u = root(e0, e1, delta, y0, y1);
            (e0_sq * y0 / (u + delta), e1_sq * y1 / u)
        } else {
            (0.0, e1)
        }
    } else {
        let denominator = e0_sq - e1_sq;
        if y0 < denominator / e0 {
            let x0 = e0_sq * y0 / denominator;
            let ratio = x0 / e0;
            (x0, e1 * libm::sqrt((1.0 - ratio * ratio).max(0.0)))
        } else {
            (e0, 0.0)
        }
    }
}

/// The unique root of F(u) = (e0*y0/(u + delta))² + (e1*y1/u)² - 1 on (0, inf), delta = e0² - e1²,
/// found by bisection. This is Eberly's F(t) with u = t + e1²: the shift keeps the root a
/// floating-point number with *relative* precision. Searching t directly stores the root as
/// -e1² + u, which loses u's digits when u is tiny (a query point close to the major axis).
///
/// F is strictly decreasing, so [u0, u1] below always brackets the root and the iteration is
/// guaranteed to converge (Eberly §2.8.1); it stops once the midpoint rounds to an endpoint.
/// Requires e0 > e1 > 0 and y0, y1 > 0.
fn ellipse_root(e0: f64, e1: f64, delta: f64, y0: f64, y1: f64) -> f64 {
    // Eberly's bound for double precision is 1074 iterations (digits - min_exponent).
    const MAX_ITERATIONS: usize = 1074;

    let n0 = e0 * y0;
    let n1 = e1 * y1;

    let mut u0 = n1;
    let mut u1 = libm::sqrt(n0 * n0 + n1 * n1);

    for _ in 0..MAX_ITERATIONS {
        let u = 0.5 * (u0 + u1);
        if u == u0 || u == u1 {
            break;
        }
        let r0 = n0 / (u + delta);
        let r1 = n1 / u;
        let f = r0 * r0 + r1 * r1 - 1.0;
        if f > 0.0 {
            u0 = u;
        } else if f < 0.0 {
            u1 = u;
        } else {
            return u;
        }
    }
    0.5 * (u0 + u1)
}

/// Same root as `ellipse_root`, by Newton's method started at the left bracket end u = e1*y1.
/// F is convex and strictly decreasing with F(u0) > 0, so every tangent step stays left of the root
/// and the iterates increase monotonically toward it (Eberly §2.8.2, "initial guess to the left");
/// it stops when F reaches 0 (or rounds negative) or a step no longer moves u.
#[cfg_attr(not(test), allow(dead_code))]
fn ellipse_root_newton(e0: f64, e1: f64, delta: f64, y0: f64, y1: f64) -> f64 {
    const MAX_ITERATIONS: usize = 1074;

    let n0 = e0 * y0;
    let n1 = e1 * y1;

    let mut u = n1;
    for _ in 0..MAX_ITERATIONS {
        let r0 = n0 / (u + delta);
        let r1 = n1 / u;
        let f = r0 * r0 + r1 * r1 - 1.0;
        if f <= 0.0 {
            break;
        }
        let slope = -2.0 * (r0 * r0 / (u + delta) + r1 * r1 / u);
        let next = u - f / slope;
        if !(next > u) {
            break;
        }
        u = next;
    }
    u
}

pub struct Polygon<const D: usize> {
    pub vertices: Vec<Point<D>>,
}

/// Signed distance to the polygon boundary: the nearest point is searched over the *segments*
/// (not their infinite lines), and `t` / `projected` describe that point (t in [0, 1] on the edge
/// `edge_index`). Outside is positive for a counter-clockwise polygon (the sign flips for clockwise).
///
/// When the nearest point is a vertex, the sign comes from the vertex pseudo-normal (the sum of the
/// two adjacent edge normals) instead of a single edge, which stays correct at reflex vertices
/// (Bærentzen & Aanæs, "Signed Distance Computation Using the Angle Weighted Pseudonormal", 2005).
///
/// ```
/// use rectgrid::Unit;
/// use rectgrid::geometry::*;
///
/// let polygon = Polygon { vertices: vec![
///     [Unit::new(0.0), Unit::new(0.0)],
///     [Unit::new(10.0), Unit::new(0.0)],
///     [Unit::new(10.0), Unit::new(10.0)],
/// ] };
/// let (result, edge_index) = as_on_polygon([Unit::new(5.0), Unit::new(-2.0)], polygon);
/// assert_eq!(result.signed_distance.get(), 2.0);
/// assert_eq!(edge_index, 0);
/// ```
pub fn as_on_polygon(point: [Unit; 2], polygon: Polygon<2>) -> (PointOnGeometry<2>, usize) {
    let n = polygon.vertices.len();
    assert!(n >= 3, "polygon must have at least 3 vertices");

    let px = point[0].get();
    let py = point[1].get();
    let vertex = |i: usize| -> (f64, f64) {
        let v = polygon.vertices[i % n];
        (v[0].get(), v[1].get())
    };

    // Nearest edge by squared distance to the segment (no sqrt per edge); first edge wins ties.
    let mut best_edge = 0;
    let mut best_t = 0.0;
    let mut best_sq = f64::INFINITY;
    for i in 0..n {
        let (x1, y1) = vertex(i);
        let (x2, y2) = vertex(i + 1);
        let (vx, vy) = (x2 - x1, y2 - y1);
        let (wx, wy) = (px - x1, py - y1);

        let length_sq = vx * vx + vy * vy;
        let t =
            if length_sq == 0.0 { 0.0 } else { ((wx * vx + wy * vy) / length_sq).clamp(0.0, 1.0) };
        let (dx, dy) = (wx - t * vx, wy - t * vy);
        let distance_sq = dx * dx + dy * dy;

        if i == 0 || distance_sq < best_sq {
            best_edge = i;
            best_t = t;
            best_sq = distance_sq;
        }
    }

    let (x1, y1) = vertex(best_edge);
    let (x2, y2) = vertex(best_edge + 1);
    let (vx, vy) = (x2 - x1, y2 - y1);
    let projected = [Unit::new(x1 + best_t * vx), Unit::new(y1 + best_t * vy)];
    let distance = libm::sqrt(best_sq);

    // Outward (right-hand) unit normal of edge `i` for a counter-clockwise polygon.
    let outward_normal = |i: usize| -> (f64, f64) {
        let (ax, ay) = vertex(i);
        let (bx, by) = vertex(i + 1);
        let (ex, ey) = (bx - ax, by - ay);
        let length = libm::sqrt(ex * ex + ey * ey);
        if length == 0.0 { (0.0, 0.0) } else { (ey / length, -ex / length) }
    };

    let outside = if best_t > 0.0 && best_t < 1.0 {
        // cross(v, p - a) < 0: right-hand side of the edge, i.e. outside for a counter-clockwise polygon
        vx * (py - y1) - vy * (px - x1) < 0.0
    } else {
        let (vertex_index, previous_edge, next_edge) = if best_t <= 0.0 {
            (best_edge, best_edge + n - 1, best_edge)
        } else {
            (best_edge + 1, best_edge, best_edge + 1)
        };
        let (qx, qy) = vertex(vertex_index);
        let (n0x, n0y) = outward_normal(previous_edge);
        let (n1x, n1y) = outward_normal(next_edge);
        (px - qx) * (n0x + n1x) + (py - qy) * (n0y + n1y) > 0.0
    };

    let signed_distance = if distance == 0.0 {
        0.0
    } else if outside {
        distance
    } else {
        -distance
    };

    (
        PointOnGeometry {
            t: Parameter::new(best_t),
            projected,
            signed_distance: Unit::new(signed_distance),
        },
        best_edge,
    )
}

pub struct PointOnGeometry<const D: usize> {
    pub t:               Parameter,
    pub projected:       Point<D>,
    pub signed_distance: Unit,
}

/// Pre-refinement implementations, kept (test builds only) so the new ones can be compared against
/// them in accuracy and cost. Delete once the experiment is settled.
#[cfg(test)]
mod legacy {
    use alloc::vec::Vec;

    use super::*;

    pub fn from_three_points(a: Point<2>, b: Point<2>, c: Point<2>) -> Option<Circle<2>> {
        const EPSILON: f64 = 1e-8;

        let ax = a[0].get();
        let ay = a[1].get();
        let bx = b[0].get();
        let by = b[1].get();
        let cx = c[0].get();
        let cy = c[1].get();

        let denominator = ((bx - cx) * (cy - ay) + (cx - ax) * (cy - by)) * 2.0;
        if denominator.abs() < EPSILON {
            return None;
        }

        let a_sq = ax * ax + ay * ay;
        let b_sq = bx * bx + by * by;
        let c_sq = cx * cx + cy * cy;

        let center_x = (a_sq * (by - cy) + b_sq * (cy - ay) + c_sq * (ay - by)) / denominator;
        let center_y = (a_sq * (cx - bx) + b_sq * (ax - cx) + c_sq * (bx - ax)) / denominator;

        let radius =
            libm::sqrt((center_x - ax) * (center_x - ax) + (center_y - ay) * (center_y - ay));

        Some(Circle {
            center: [Unit::new(center_x), Unit::new(center_y)],
            radius: Unit::new(radius),
        })
    }

    pub fn as_on_ellipse(point: [Unit; 2], ellipse: Ellipse<2>) -> PointOnGeometry<2> {
        let px = point[0].get();
        let py = point[1].get();
        let cx = ellipse.center[0].get();
        let cy = ellipse.center[1].get();
        let rx = ellipse.rx.get();
        let ry = ellipse.ry.get();

        let raw_dx = px - cx;
        let raw_dy = py - cy;

        if rx == 0.0 || ry == 0.0 {
            let distance = libm::sqrt(raw_dx * raw_dx + raw_dy * raw_dy);
            return PointOnGeometry {
                t:               Parameter::new(libm::atan2(raw_dy, raw_dx)),
                projected:       [Unit::new(cx), Unit::new(cy)],
                signed_distance: Unit::new(distance),
            };
        }

        let dx = raw_dx / rx;
        let dy = raw_dy / ry;
        let ellipse_value = dx * dx + dy * dy;
        let t = libm::atan2(dy, dx);

        let (proj_x, proj_y) = if ellipse_value == 0.0 {
            (cx + rx, cy)
        } else {
            let scale = 1.0 / libm::sqrt(ellipse_value);
            (cx + raw_dx * scale, cy + raw_dy * scale)
        };

        let approx_distance = (libm::sqrt(ellipse_value) - 1.0) * rx.min(ry);

        PointOnGeometry {
            t:               Parameter::new(t),
            projected:       [Unit::new(proj_x), Unit::new(proj_y)],
            signed_distance: Unit::new(approx_distance),
        }
    }

    fn edges(polygon: &Polygon<2>) -> Vec<Line<2>> {
        let n = polygon.vertices.len();
        (0..n)
            .map(|i| Line { start: polygon.vertices[i], end: polygon.vertices[(i + 1) % n] })
            .collect()
    }

    pub fn as_on_polygon(point: [Unit; 2], polygon: Polygon<2>) -> (PointOnGeometry<2>, usize) {
        assert!(polygon.vertices.len() >= 3, "polygon must have at least 3 vertices");

        let px = point[0].get();
        let py = point[1].get();

        let mut best: Option<(f64, PointOnGeometry<2>, usize)> = None;

        for (i, edge) in edges(&polygon).into_iter().enumerate() {
            let x1 = edge.start[0].get();
            let y1 = edge.start[1].get();
            let x2 = edge.end[0].get();
            let y2 = edge.end[1].get();

            let result = as_on_line(
                [Unit::new(px), Unit::new(py)],
                Line {
                    start: [Unit::new(x1), Unit::new(y1)],
                    end:   [Unit::new(x2), Unit::new(y2)],
                },
            );

            let raw_t = result.t.get();
            let clamped_t = raw_t.max(0.0).min(1.0);
            let clamped_proj_x = x1 + clamped_t * (x2 - x1);
            let clamped_proj_y = y1 + clamped_t * (y2 - y1);
            let bounded_distance = libm::sqrt(
                (px - clamped_proj_x) * (px - clamped_proj_x)
                    + (py - clamped_proj_y) * (py - clamped_proj_y),
            );

            if best.as_ref().map_or(true, |(d, _, _)| bounded_distance < *d) {
                best = Some((bounded_distance, result, i));
            }
        }

        let (_, nearest, edge_index) = best.expect("polygon must have at least one edge");

        (
            PointOnGeometry {
                t:               nearest.t,
                projected:       nearest.projected,
                signed_distance: Unit::new(-nearest.signed_distance.get()),
            },
            edge_index,
        )
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn p(x: f64, y: f64) -> Point<2> {
        [Unit::new(x), Unit::new(y)]
    }

    #[test]
    fn as_on_line_returns_zero_when_point_lies_on_segment() {
        let line = Line { start: p(-5.0, -5.0), end: p(5.0, 5.0) };
        let result = as_on_line(p(0.0, 0.0), line);
        assert!(result.signed_distance.get().abs() < 1e-8);
    }

    #[test]
    fn as_on_line_handles_zero_length_segment() {
        let line = Line { start: p(2.0, 2.0), end: p(2.0, 2.0) };
        let result = as_on_line(p(5.0, 6.0), line);
        assert_eq!(result.signed_distance.get().abs(), 5.0);
    }

    #[test]
    fn as_on_line_t_outside_zero_one_means_beyond_segment() {
        let line = Line { start: p(0.0, 0.0), end: p(10.0, 0.0) };
        let result = as_on_line(p(-5.0, 0.0), line);
        assert!(result.t.get() < 0.0);
    }

    #[test]
    fn as_on_circle_negative_coordinates_and_decimals() {
        let circle = Circle { center: p(-4.0, 4.0), radius: Unit::new(5.0) };
        let result = as_on_circle(p(1.0, 4.0), circle);
        assert!(result.signed_distance.get().abs() < 1e-8);
    }

    #[test]
    fn as_on_circle_outside_is_positive() {
        let circle = Circle { center: p(0.0, 0.0), radius: Unit::new(5.0) };
        let result = as_on_circle(p(10.0, 0.0), circle);
        assert_eq!(result.signed_distance.get(), 5.0);
    }

    #[test]
    fn from_three_points_returns_none_for_colinear_points() {
        let result = Circle::from_three_points(p(0.0, 0.0), p(1.0, 1.0), p(2.0, 2.0));
        assert!(result.is_none());
    }

    #[test]
    fn from_three_points_handles_matching_y_on_first_and_third_point() {
        let result = Circle::from_three_points(p(0.0, 0.0), p(1.0, 3.0), p(4.0, 0.0)).unwrap();
        assert!((result.center[0].get() - 2.0).abs() < 1e-8);
        assert!((result.center[1].get() - 1.0).abs() < 1e-8);
        assert!((result.radius.get() - 5.0_f64.sqrt()).abs() < 1e-8);
    }

    #[test]
    fn as_on_ellipse_inside_is_negative() {
        let ellipse =
            Ellipse { center: p(0.0, 0.0), rx: Unit::new(4.0), ry: Unit::new(3.0) };
        let result = as_on_ellipse(p(2.0, 0.0), ellipse);
        assert!(result.signed_distance.get() < 0.0);
    }

    #[test]
    fn as_on_ellipse_degenerate_ry_zero_falls_back_to_center_distance() {
        let ellipse =
            Ellipse { center: p(0.0, 0.0), rx: Unit::new(4.0), ry: Unit::new(0.0) };
        let result = as_on_ellipse(p(3.0, 4.0), ellipse);
        assert_eq!(result.signed_distance.get(), 5.0);
    }

    #[test]
    fn as_on_polygon_inside_is_negative() {
        let polygon =
            Polygon { vertices: vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)] };
        let (result, _) = as_on_polygon(p(5.0, 5.0), polygon);
        assert!(result.signed_distance.get() < 0.0);
    }

    #[test]
    fn as_on_polygon_picks_nearest_among_multiple_shapes_edge() {
        let near = Polygon { vertices: vec![p(20.0, 0.0), p(30.0, 0.0), p(30.0, 10.0)] };
        let (result, _) = as_on_polygon(p(25.0, -2.0), near);
        assert_eq!(result.signed_distance.get(), 2.0);
    }

    #[test]
    fn as_on_polygon_cw_inverts_sign() {
        let ccw =
            Polygon { vertices: vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)] };
        let cw = Polygon { vertices: vec![p(0.0, 0.0), p(0.0, 10.0), p(10.0, 10.0), p(10.0, 0.0)] };
        let inside = p(5.0, 5.0);
        let (result_ccw, _) = as_on_polygon(inside, ccw);
        let (result_cw, _) = as_on_polygon(inside, cw);
        assert!(result_ccw.signed_distance.get() < 0.0);
        assert!(result_cw.signed_distance.get() > 0.0);
    }

    #[test]
    fn as_on_polygon_returns_nearest_edge_index() {
        let polygon =
            Polygon { vertices: vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)] };
        let (_, edge_index) = as_on_polygon(p(5.0, -2.0), polygon);
        assert_eq!(edge_index, 0);
    }

    #[test]
    #[should_panic(expected = "at least 3 vertices")]
    fn as_on_polygon_panics_for_less_than_three_vertices() {
        let polygon = Polygon { vertices: vec![p(0.0, 0.0), p(1.0, 1.0)] };
        as_on_polygon(p(0.0, 0.0), polygon);
    }
    // ---- accuracy: expected values come from an independent high-precision (Decimal, 60 digits)
    // ---- evaluation of the same problem, cross-checked by brute-force minimisation to 1e-14.

    fn close(actual: f64, expected: f64, tolerance: f64) -> bool {
        (actual - expected).abs() <= tolerance * (1.0 + expected.abs())
    }

    fn square() -> Polygon<2> {
        Polygon { vertices: vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)] }
    }

    fn l_shape() -> Polygon<2> {
        Polygon {
            vertices: vec![
                p(0.0, 0.0),
                p(10.0, 0.0),
                p(10.0, 4.0),
                p(4.0, 4.0),
                p(4.0, 10.0),
                p(0.0, 10.0),
            ],
        }
    }

    #[test]
    fn as_on_polygon_nearest_convex_vertex_gives_euclidean_distance() {
        // 3-4-5 triangles: the nearest point is the vertex, not the edge's infinite line.
        let (result, edge) = as_on_polygon(p(13.0, 14.0), square());
        assert_eq!(result.signed_distance.get(), 5.0);
        assert_eq!((result.projected[0].get(), result.projected[1].get()), (10.0, 10.0));
        assert_eq!((edge, result.t.get()), (1, 1.0));

        let (result, edge) = as_on_polygon(p(-3.0, -4.0), square());
        assert_eq!(result.signed_distance.get(), 5.0);
        assert_eq!((result.projected[0].get(), result.projected[1].get()), (0.0, 0.0));
        assert_eq!((edge, result.t.get()), (0, 0.0));

        // legacy: distance to the infinite line of the chosen edge (3 and 4 instead of 5)
        assert_eq!(legacy::as_on_polygon(p(13.0, 14.0), square()).0.signed_distance.get(), 3.0);
        assert_eq!(legacy::as_on_polygon(p(-3.0, -4.0), square()).0.signed_distance.get(), 4.0);
    }

    #[test]
    fn as_on_polygon_edge_interior_reports_clamped_t_and_projection() {
        let (result, edge) = as_on_polygon(p(20.0, 0.5), square());
        assert_eq!(result.signed_distance.get(), 10.0);
        assert!(close(result.t.get(), 0.05, 1e-15));
        assert!(close(result.projected[0].get(), 10.0, 1e-15));
        assert!(close(result.projected[1].get(), 0.5, 1e-15));
        assert_eq!(edge, 1);
    }

    #[test]
    fn as_on_polygon_reflex_vertex_inside_is_negative_with_euclidean_distance() {
        // (3, 3) is inside the L; its nearest boundary point is the reflex vertex (4, 4).
        let (result, edge) = as_on_polygon(p(3.0, 3.0), l_shape());
        assert!(close(result.signed_distance.get(), -core::f64::consts::SQRT_2, 1e-15));
        assert_eq!((result.projected[0].get(), result.projected[1].get()), (4.0, 4.0));
        assert_eq!((edge, result.t.get()), (2, 1.0));

        // legacy: distance to the infinite line y = 4 (1.0 instead of sqrt(2))
        let legacy_distance = legacy::as_on_polygon(p(3.0, 3.0), l_shape()).0.signed_distance.get();
        assert_eq!(legacy_distance, -1.0);
    }

    #[test]
    fn as_on_polygon_notch_outside_is_positive() {
        let (result, edge) = as_on_polygon(p(5.0, 5.0), l_shape());
        assert_eq!(result.signed_distance.get(), 1.0);
        assert_eq!((result.projected[0].get(), result.projected[1].get()), (5.0, 4.0));
        assert_eq!(edge, 2);
    }

    #[test]
    fn as_on_polygon_vertex_sign_follows_orientation() {
        let cw = Polygon { vertices: vec![p(0.0, 0.0), p(0.0, 10.0), p(10.0, 10.0), p(10.0, 0.0)] };
        let (outside, _) = as_on_polygon(p(13.0, 14.0), cw);
        assert_eq!(outside.signed_distance.get(), -5.0);
    }

    #[test]
    fn as_on_polygon_on_boundary_is_exactly_zero() {
        assert_eq!(as_on_polygon(p(5.0, 0.0), square()).0.signed_distance.get(), 0.0);
        assert_eq!(as_on_polygon(p(10.0, 10.0), square()).0.signed_distance.get(), 0.0);
    }

    #[test]
    fn as_on_polygon_matches_brute_force_distance_and_even_odd_sign() {
        let polygon = l_shape();
        let vertices = polygon.vertices.clone();
        let n = vertices.len();
        let v = |i: usize| (vertices[i % n][0].get(), vertices[i % n][1].get());

        for ix in -6..=24 {
            for iy in -6..=24 {
                let (qx, qy) = (ix as f64 * 0.5, iy as f64 * 0.5);

                let mut min_sq = f64::INFINITY;
                let mut inside = false;
                for i in 0..n {
                    let ((x1, y1), (x2, y2)) = (v(i), v(i + 1));
                    let (vx, vy) = (x2 - x1, y2 - y1);
                    let t =
                        (((qx - x1) * vx + (qy - y1) * vy) / (vx * vx + vy * vy)).clamp(0.0, 1.0);
                    let (dx, dy) = (qx - x1 - t * vx, qy - y1 - t * vy);
                    min_sq = min_sq.min(dx * dx + dy * dy);
                    if (y1 > qy) != (y2 > qy) && qx < x1 + (qy - y1) / (y2 - y1) * (x2 - x1) {
                        inside = !inside;
                    }
                }
                let expected = if min_sq == 0.0 {
                    0.0
                } else if inside {
                    -libm::sqrt(min_sq)
                } else {
                    libm::sqrt(min_sq)
                };

                let actual = as_on_polygon(p(qx, qy), Polygon { vertices: vertices.clone() })
                    .0
                    .signed_distance
                    .get();
                assert!(close(actual, expected, 1e-12), "q = ({qx}, {qy}): {actual} vs {expected}");
            }
        }
    }

    // (name, rx, ry, dx, dy, signed distance, closest point relative to the center)
    const ELLIPSE_CASES: [(&str, f64, f64, f64, f64, f64, (f64, f64)); 18] = [
        (
            "4x1 (3,2)",
            4.0,
            1.0,
            3.0,
            2.0,
            1.2973054925552014,
            (2.7090557089436533, 0.7357401530873434),
        ),
        (
            "4x1 (10,10)",
            4.0,
            1.0,
            10.0,
            10.0,
            11.498686021746554,
            (3.733379239352857, 0.35898114985061225),
        ),
        (
            "4x1 (1.5,0.5) inside",
            4.0,
            1.0,
            1.5,
            0.5,
            -0.42478368617597306,
            (1.5442009682459554, 0.9224777561568983),
        ),
        ("4x1 (5,0) on major axis", 4.0, 1.0, 5.0, 0.0, 1.0, (4.0, 0.0)),
        ("4x1 (0,5) on minor axis", 4.0, 1.0, 0.0, 5.0, 4.0, (0.0, 1.0)),
        (
            "1x4 (2,3) axes swapped",
            1.0,
            4.0,
            2.0,
            3.0,
            1.2973054925552014,
            (0.7357401530873434, 2.7090557089436533),
        ),
        (
            "4x1 (-3,-2) third quadrant",
            4.0,
            1.0,
            -3.0,
            -2.0,
            1.2973054925552014,
            (-2.7090557089436533, -0.7357401530873434),
        ),
        ("5x5 (6,8) circle", 5.0, 5.0, 6.0, 8.0, 5.0, (3.0, 4.0)),
        ("4x1 center", 4.0, 1.0, 0.0, 0.0, -1.0, (0.0, 1.0)),
        (
            "100x0.01 (50,1) eccentric",
            100.0,
            0.01,
            50.0,
            1.0,
            0.9913397443099252,
            (49.99994276506063, 0.00866025734230263),
        ),
        (
            "100x0.01 (120,0.005) eccentric",
            100.0,
            0.01,
            120.0,
            0.005,
            20.00000062499996,
            (99.99999999999997, 2.4999998750000017e-10),
        ),
        (
            "4x1 (3,1e-12) near axis",
            4.0,
            1.0,
            3.0,
            1e-12,
            -0.6324555320327272,
            (3.1999999999996445, 0.6000000000001185),
        ),
        (
            "4x1 (3.7,1e-6) near axis",
            4.0,
            1.0,
            3.7,
            1e-6,
            -0.29552158605376616,
            (3.946665050074008, 0.16275652493147685),
        ),
        (
            "4x1 (0.001,5) near minor axis",
            4.0,
            1.0,
            0.001,
            5.0,
            4.000000025,
            (0.000799999996, 0.99999998),
        ),
        (
            "3x2 (1,1) inside",
            3.0,
            2.0,
            1.0,
            1.0,
            -0.8554637244664959,
            (1.249987537492453, 1.8181224938702933),
        ),
        (
            "4x1 (3,1e-15) on the axis side",
            4.0,
            1.0,
            3.0,
            1e-15,
            -0.6324555320336749,
            (3.1999999999999997, 0.6000000000000001),
        ),
        (
            "4x1 (3.7,1e-14) on the axis side",
            4.0,
            1.0,
            3.7,
            1e-14,
            -0.2955221367906812,
            (3.9466666666666503, 0.16275407487647386),
        ),
        (
            "100x0.01 (50,1e-9) eccentric",
            100.0,
            0.01,
            50.0,
            1e-9,
            -0.00866025302341063,
            (50.00000049999995, 0.008660254008976876),
        ),
    ];

    type Root = fn(f64, f64, f64, f64, f64) -> f64;

    #[test]
    fn as_on_ellipse_matches_high_precision_reference() {
        matches_high_precision_reference(ellipse_root);
    }

    #[test]
    fn as_on_ellipse_newton_matches_high_precision_reference() {
        matches_high_precision_reference(ellipse_root_newton);
    }

    fn matches_high_precision_reference(root: Root) {
        for (name, rx, ry, dx, dy, signed, foot) in ELLIPSE_CASES {
            let ellipse =
                Ellipse { center: p(0.0, 0.0), rx: Unit::new(rx), ry: Unit::new(ry) };
            let result = as_on_ellipse_with_root(p(dx, dy), ellipse, root);
            let got = result.signed_distance.get();
            assert!(close(got, signed, 1e-12), "{name}: distance {got} vs {signed}");
            let (fx, fy) = (result.projected[0].get(), result.projected[1].get());
            assert!(close(fx, foot.0, 1e-9), "{name}: foot x {fx} vs {}", foot.0);
            assert!(close(fy, foot.1, 1e-9), "{name}: foot y {fy} vs {}", foot.1);
        }
    }

    #[test]
    fn as_on_ellipse_is_translation_invariant_for_large_center() {
        let ellipse =
            Ellipse { center: p(1e9, -5e8), rx: Unit::new(4.0), ry: Unit::new(1.0) };
        let result = as_on_ellipse(p(1e9 + 3.0, -5e8 + 2.0), ellipse);
        assert!(close(result.signed_distance.get(), 1.2973054925552014, 1e-12));
        assert!(close(result.projected[0].get() - 1e9, 2.7090557089436533, 1e-6));
    }

    #[test]
    fn as_on_ellipse_points_on_the_ellipse_are_zero() {
        let ellipse =
            || Ellipse { center: p(0.0, 0.0), rx: Unit::new(4.0), ry: Unit::new(1.0) };
        assert_eq!(as_on_ellipse(p(4.0, 0.0), ellipse()).signed_distance.get(), 0.0);
        assert_eq!(as_on_ellipse(p(0.0, 1.0), ellipse()).signed_distance.get(), 0.0);
        // (4 cos(pi/3), sin(pi/3)) is on the ellipse up to rounding of the inputs
        let on = as_on_ellipse(p(2.0, libm::sqrt(0.75)), ellipse());
        assert!(on.signed_distance.get().abs() < 1e-15, "{}", on.signed_distance.get());
    }

    #[test]
    fn as_on_ellipse_closest_point_is_the_foot_of_the_normal() {
        closest_point_is_the_foot_of_the_normal(ellipse_root);
    }

    #[test]
    fn as_on_ellipse_newton_closest_point_is_the_foot_of_the_normal() {
        closest_point_is_the_foot_of_the_normal(ellipse_root_newton);
    }

    fn closest_point_is_the_foot_of_the_normal(root: Root) {
        for (rx, ry) in [(4.0, 1.0), (1.0, 4.0), (3.0, 3.0), (100.0, 0.01), (7.0, 6.9)] {
            for ix in -12..=12 {
                for iy in -12..=12 {
                    let (dx, dy) = (ix as f64 * 0.9 * rx / 6.0, iy as f64 * 0.9 * ry / 6.0);
                    let ellipse = Ellipse {
                        center: p(1.0, -2.0),
                        rx:     Unit::new(rx),
                        ry:     Unit::new(ry),
                    };
                    let r = as_on_ellipse_with_root(p(1.0 + dx, -2.0 + dy), ellipse, root);
                    let (fx, fy) = (r.projected[0].get() - 1.0, r.projected[1].get() + 2.0);
                    let d = r.signed_distance.get();
                    let scale = rx.max(ry);

                    // the closest point lies on the ellipse
                    let residual = (fx / rx) * (fx / rx) + (fy / ry) * (fy / ry) - 1.0;
                    assert!(residual.abs() < 1e-12, "({rx},{ry}) ({dx},{dy}): residual {residual}");
                    // its distance to the query is the reported distance
                    let actual = libm::sqrt((dx - fx) * (dx - fx) + (dy - fy) * (dy - fy));
                    assert!((actual - d.abs()).abs() <= 1e-12 * scale, "({rx},{ry}) ({dx},{dy})");
                    // no sampled ellipse point is closer
                    for k in 0..720 {
                        let a = k as f64 * core::f64::consts::PI / 360.0;
                        let (sx, sy) = (rx * libm::cos(a) - dx, ry * libm::sin(a) - dy);
                        assert!(libm::sqrt(sx * sx + sy * sy) >= d.abs() - 1e-12 * scale);
                    }
                }
            }
        }
    }

    #[test]
    fn as_on_ellipse_legacy_radial_approximation_error() {
        let ellipse =
            || Ellipse { center: p(0.0, 0.0), rx: Unit::new(4.0), ry: Unit::new(1.0) };
        // on the major axis (5, 0): approximation 0.25, exact 1.0
        assert_eq!(legacy::as_on_ellipse(p(5.0, 0.0), ellipse()).signed_distance.get(), 0.25);
        // (10, 10): approximation 9.3078, exact 11.4987
        let approx = legacy::as_on_ellipse(p(10.0, 10.0), ellipse()).signed_distance.get();
        assert!(close(approx, 9.30776406404415, 1e-12));
    }

    #[test]
    fn from_three_points_far_from_origin_keeps_precision() {
        // a unit right triangle translated far from the origin: center (+0.5, +0.5), radius sqrt(1/2)
        for offset in [1e6, 1e9, 1e12] {
            let c = Circle::from_three_points(
                p(offset, offset),
                p(offset + 1.0, offset),
                p(offset, offset + 1.0),
            )
            .unwrap();
            assert_eq!(c.center[0].get() - offset, 0.5, "offset {offset}");
            assert_eq!(c.center[1].get() - offset, 0.5, "offset {offset}");
            assert_eq!(c.radius.get(), 0.7071067811865476, "offset {offset}");
        }

        // legacy: radius 0 at 1e9 and garbage at 1e12
        let at_1e9 = legacy::from_three_points(p(1e9, 1e9), p(1e9 + 1.0, 1e9), p(1e9, 1e9 + 1.0));
        assert_eq!(at_1e9.unwrap().radius.get(), 0.0);
        let at_1e12 =
            legacy::from_three_points(p(1e12, 1e12), p(1e12 + 1.0, 1e12), p(1e12, 1e12 + 1.0));
        assert!((at_1e12.unwrap().radius.get() - 0.7071067811865476).abs() > 1e6);
    }

    #[test]
    fn from_three_points_is_scale_invariant() {
        // (0,0), (4,0), (1,3): center (2,1), radius sqrt(5)
        for scale in [1e-9, 1e-5, 1.0, 1e5, 1e9] {
            let c = Circle::from_three_points(
                p(0.0, 0.0),
                p(4.0 * scale, 0.0),
                p(1.0 * scale, 3.0 * scale),
            )
            .unwrap();
            assert!(close(c.center[0].get() / scale, 2.0, 1e-14), "scale {scale}");
            assert!(close(c.center[1].get() / scale, 1.0, 1e-14), "scale {scale}");
            assert!(close(c.radius.get() / scale, libm::sqrt(5.0), 1e-14), "scale {scale}");
        }

        // legacy rejects any circle smaller than the absolute threshold
        let tiny = legacy::from_three_points(p(0.0, 0.0), p(1e-5, 0.0), p(0.0, 1e-5));
        assert!(tiny.is_none());
        let tiny = Circle::from_three_points(p(0.0, 0.0), p(1e-5, 0.0), p(0.0, 1e-5)).unwrap();
        assert!(close(tiny.center[0].get() / 5e-6, 1.0, 1e-14));
        assert!(close(tiny.radius.get() / 7.0710678118654755e-6, 1.0, 1e-14));
    }

    #[test]
    fn from_three_points_nearly_collinear_but_valid() {
        // cross / longest² = 2.5e-7: far above the relative threshold, radius ~ 1e6
        let c = Circle::from_three_points(p(0.0, 0.0), p(1.0, 0.0), p(2.0, 1e-6)).unwrap();
        assert!(close(c.center[0].get(), 0.5, 1e-9));
        assert!(close(c.radius.get() / 1e6, 1.0, 1e-5), "{}", c.radius.get());
        for q in [p(0.0, 0.0), p(1.0, 0.0), p(2.0, 1e-6)] {
            let (dx, dy) = (q[0].get() - c.center[0].get(), q[1].get() - c.center[1].get());
            assert!(close(libm::sqrt(dx * dx + dy * dy) / c.radius.get(), 1.0, 1e-9));
        }
    }

    #[test]
    fn from_three_points_rejects_coincident_points() {
        assert!(Circle::from_three_points(p(3.0, 3.0), p(3.0, 3.0), p(3.0, 3.0)).is_none());
        assert!(Circle::from_three_points(p(0.0, 0.0), p(1e-9, 1e-9), p(2e-9, 2e-9)).is_none());
    }

    // ---- cost comparison (run: cargo test --release --all-features perf -- --ignored --nocapture) ----

    fn lcg(state: &mut u64) -> f64 {
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*state >> 11) as f64 / (1u64 << 53) as f64
    }

    fn time_ns(iterations: usize, mut f: impl FnMut(usize) -> f64) -> f64 {
        let mut sink = 0.0;
        for i in 0..iterations.min(1000) {
            sink += f(i);
        }
        let start = std::time::Instant::now();
        for i in 0..iterations {
            sink += f(i);
        }
        let elapsed = start.elapsed().as_nanos() as f64 / iterations as f64;
        core::hint::black_box(sink);
        elapsed
    }

    fn random_points(count: usize, half_extent: (f64, f64), seed: u64) -> Vec<(f64, f64)> {
        let mut state = seed;
        (0..count)
            .map(|_| {
                (
                    (lcg(&mut state) * 2.0 - 1.0) * half_extent.0,
                    (lcg(&mut state) * 2.0 - 1.0) * half_extent.1,
                )
            })
            .collect()
    }

    /// Iteration count of the bisection in `ellipse_root` (a replica of its loop).
    fn bisection_iterations(e0: f64, e1: f64, y0: f64, y1: f64) -> usize {
        let delta = e0 * e0 - e1 * e1;
        let (n0, n1) = (e0 * y0, e1 * y1);
        let (mut u0, mut u1) = (n1, libm::sqrt(n0 * n0 + n1 * n1));
        for count in 0..1074 {
            let u = 0.5 * (u0 + u1);
            if u == u0 || u == u1 {
                return count;
            }
            let (r0, r1) = (n0 / (u + delta), n1 / u);
            let f = r0 * r0 + r1 * r1 - 1.0;
            if f > 0.0 {
                u0 = u;
            } else if f < 0.0 {
                u1 = u;
            } else {
                return count;
            }
        }
        1074
    }

    #[test]
    #[ignore]
    fn perf_polygon_new_vs_legacy() {
        for n in [4usize, 32, 256, 4096] {
            let vertices: Vec<Point<2>> = (0..n)
                .map(|i| {
                    let a = i as f64 * core::f64::consts::TAU / n as f64;
                    p(100.0 * libm::cos(a), 100.0 * libm::sin(a))
                })
                .collect();
            let points = random_points(1024, (150.0, 150.0), 1);
            let iterations = (4_000_000 / n).max(2000);

            let clone_only = time_ns(iterations, |i| {
                core::hint::black_box(vertices.clone()).len() as f64 + i as f64
            });
            let legacy_ns = time_ns(iterations, |i| {
                let (x, y) = points[i % 1024];
                legacy::as_on_polygon(p(x, y), Polygon { vertices: vertices.clone() })
                    .0
                    .signed_distance
                    .get()
            });
            let new_ns = time_ns(iterations, |i| {
                let (x, y) = points[i % 1024];
                as_on_polygon(p(x, y), Polygon { vertices: vertices.clone() })
                    .0
                    .signed_distance
                    .get()
            });
            std::println!(
                "polygon n={n:5}: legacy {legacy_ns:10.1} ns  new {new_ns:10.1} ns  (clone only {clone_only:9.1} ns)  net: legacy {:9.1}  new {:9.1}  ratio {:.2}",
                legacy_ns - clone_only,
                new_ns - clone_only,
                (new_ns - clone_only) / (legacy_ns - clone_only)
            );
        }
    }

    #[test]
    #[ignore]
    fn perf_ellipse_new_vs_legacy() {
        for (rx, ry) in [(4.0f64, 1.0f64), (100.0, 0.01), (5.0, 5.0)] {
            let extent = rx.max(ry) * 1.3;
            let points = random_points(1024, (extent, extent), 2);
            let ellipse =
                || Ellipse { center: p(0.0, 0.0), rx: Unit::new(rx), ry: Unit::new(ry) };
            let iterations = 2_000_000;

            let legacy_ns = time_ns(iterations, |i| {
                let (x, y) = points[i % 1024];
                legacy::as_on_ellipse(p(x, y), ellipse()).signed_distance.get()
            });
            let new_ns = time_ns(iterations, |i| {
                let (x, y) = points[i % 1024];
                as_on_ellipse(p(x, y), ellipse()).signed_distance.get()
            });
            let newton_ns = time_ns(iterations, |i| {
                let (x, y) = points[i % 1024];
                as_on_ellipse_with_root(p(x, y), ellipse(), ellipse_root_newton)
                    .signed_distance
                    .get()
            });

            let (e0, e1) = (rx.max(ry), rx.min(ry));
            let counts: Vec<usize> = points
                .iter()
                .filter(|(x, y)| x.abs() > 0.0 && y.abs() > 0.0 && e0 != e1)
                .map(|(x, y)| {
                    let (a, b) = if ry > rx { (y.abs(), x.abs()) } else { (x.abs(), y.abs()) };
                    bisection_iterations(e0, e1, a, b)
                })
                .collect();
            let mean = counts.iter().sum::<usize>() as f64 / counts.len().max(1) as f64;
            std::println!(
                "ellipse {rx}x{ry}: legacy {legacy_ns:7.1} ns  bisection {new_ns:7.1} ns (x{:.1})  newton {newton_ns:7.1} ns (x{:.1})  (bisection iterations: mean {mean:.1}, max {})",
                new_ns / legacy_ns,
                newton_ns / legacy_ns,
                counts.iter().max().copied().unwrap_or(0)
            );
        }
    }

    #[test]
    #[ignore]
    fn perf_circle_three_points_new_vs_legacy() {
        let points = random_points(3072, (50.0, 50.0), 3);
        let iterations = 5_000_000;
        let legacy_ns = time_ns(iterations, |i| {
            let j = (i * 3) % 3072;
            let (a, b, c) = (points[j], points[j + 1], points[j + 2]);
            legacy::from_three_points(p(a.0, a.1), p(b.0, b.1), p(c.0, c.1))
                .map_or(0.0, |c| c.radius.get())
        });
        let new_ns = time_ns(iterations, |i| {
            let j = (i * 3) % 3072;
            let (a, b, c) = (points[j], points[j + 1], points[j + 2]);
            Circle::from_three_points(p(a.0, a.1), p(b.0, b.1), p(c.0, c.1))
                .map_or(0.0, |c| c.radius.get())
        });
        std::println!(
            "circle from_three_points: legacy {legacy_ns:6.1} ns  new {new_ns:6.1} ns  ratio {:.2}",
            new_ns / legacy_ns
        );
    }
}
