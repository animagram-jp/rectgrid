use alloc::vec::Vec;
use core::primitive::{f64, usize};

use libm;

use crate::{Error, Parameter, Point, Unit};

fn reject_nan(values: &[f64]) -> Result<(), Error> {
    if values.iter().any(|v| v.is_nan()) { Err(Error::InvalidInput) } else { Ok(()) }
}

pub struct Line<const D: usize> {
    pub start: Point<D>,
    pub end:   Point<D>,
}

/// ```
/// use rectgrid::Unit;
/// use rectgrid::geometry::*;
///
/// let line = Line { start: [Unit::new(0.0), Unit::new(0.0)], end: [Unit::new(10.0), Unit::new(0.0)] };
/// let result = as_on_line([Unit::new(5.0), Unit::new(5.0)], line).unwrap();
/// assert_eq!(result.t.get(), 0.5);
/// assert_eq!(result.signed_distance.get().abs(), 5.0);
/// ```
pub fn as_on_line(point: [Unit; 2], line: Line<2>) -> Result<PointOnGeometry<2>, Error> {
    let px = point[0].get();
    let py = point[1].get();
    let x1 = line.start[0].get();
    let y1 = line.start[1].get();
    let x2 = line.end[0].get();
    let y2 = line.end[1].get();
    reject_nan(&[px, py, x1, y1, x2, y2])?;

    let vx = x2 - x1;
    let vy = y2 - y1;
    let wx = px - x1;
    let wy = py - y1;

    let length_squared = vx * vx + vy * vy;
    let t = if length_squared == 0.0 { 0.0 } else { (wx * vx + wy * vy) / length_squared };

    let proj_x = x1 + t * vx;
    let proj_y = y1 + t * vy;

    // |cross| / |v| uses differences only, so the error does not grow with the coordinates' magnitude
    let signed_distance = if length_squared == 0.0 {
        libm::sqrt(wx * wx + wy * wy)
    } else {
        let cross = vx * wy - vy * wx;
        if cross == 0.0 { 0.0 } else { cross / libm::sqrt(length_squared) }
    };

    Ok(PointOnGeometry {
        t:               Parameter::new(t),
        projected:       [Unit::new(proj_x), Unit::new(proj_y)],
        signed_distance: Unit::new(signed_distance),
    })
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
    /// ).unwrap().unwrap();
    /// assert!((result.center[0].get()).abs() < 1e-8);
    /// assert!((result.center[1].get()).abs() < 1e-8);
    /// assert!((result.radius.get() - 1.0).abs() < 1e-8);
    /// ```
    pub fn from_three_points(a: Point<2>, b: Point<2>, c: Point<2>) -> Result<Option<Self>, Error> {
        // collinear when cross <= RELATIVE_EPSILON * longest_side², which is scale-invariant
        const RELATIVE_EPSILON: f64 = 1e-12;

        let ax = a[0].get();
        let ay = a[1].get();
        reject_nan(&[ax, ay, b[0].get(), b[1].get(), c[0].get(), c[1].get()])?;
        // solve relative to `a` so roundoff follows the point differences (Shewchuk, 1999)
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
            return Ok(None);
        }

        let denominator = 2.0 * cross;
        let center_x = (cy * b_sq - by * c_sq) / denominator;
        let center_y = (bx * c_sq - cx * b_sq) / denominator;

        let radius = libm::sqrt(center_x * center_x + center_y * center_y);

        Ok(Some(Circle {
            center: [Unit::new(ax + center_x), Unit::new(ay + center_y)],
            radius: Unit::new(radius),
        }))
    }
}

/// ```
/// use rectgrid::Unit;
/// use rectgrid::geometry::*;
///
/// let circle = Circle { center: [Unit::new(0.0), Unit::new(0.0)], radius: Unit::new(5.0) };
/// let result = as_on_circle([Unit::new(0.0), Unit::new(0.0)], circle).unwrap();
/// assert_eq!(result.signed_distance.get(), -5.0);
/// ```
pub fn as_on_circle(point: [Unit; 2], circle: Circle<2>) -> Result<PointOnGeometry<2>, Error> {
    let px = point[0].get();
    let py = point[1].get();
    let cx = circle.center[0].get();
    let cy = circle.center[1].get();
    let radius = circle.radius.get();
    reject_nan(&[px, py, cx, cy, radius])?;

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

    Ok(PointOnGeometry {
        t:               Parameter::new(t),
        projected:       [Unit::new(proj_x), Unit::new(proj_y)],
        signed_distance: Unit::new(distance_from_center - radius),
    })
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
/// let result = as_on_ellipse([Unit::new(8.0), Unit::new(0.0)], ellipse).unwrap();
/// assert_eq!(result.signed_distance.get(), 4.0);
/// ```
pub fn as_on_ellipse(point: [Unit; 2], ellipse: Ellipse<2>) -> Result<PointOnGeometry<2>, Error> {
    let px = point[0].get();
    let py = point[1].get();
    let cx = ellipse.center[0].get();
    let cy = ellipse.center[1].get();
    let rx = ellipse.rx.get();
    let ry = ellipse.ry.get();
    reject_nan(&[px, py, cx, cy, rx, ry])?;

    let raw_dx = px - cx;
    let raw_dy = py - cy;

    if rx == 0.0 || ry == 0.0 {
        let distance = libm::sqrt(raw_dx * raw_dx + raw_dy * raw_dy);
        return Ok(PointOnGeometry {
            t:               Parameter::new(libm::atan2(raw_dy, raw_dx)),
            projected:       [Unit::new(cx), Unit::new(cy)],
            signed_distance: Unit::new(distance),
        });
    }

    let rx = rx.abs();
    let ry = ry.abs();

    let swap = ry > rx;
    let (e0, e1) = if swap { (ry, rx) } else { (rx, ry) };
    let (y0, y1) = if swap { (raw_dy.abs(), raw_dx.abs()) } else { (raw_dx.abs(), raw_dy.abs()) };

    let (x0, x1) = ellipse_closest_in_first_quadrant(e0, e1, y0, y1);

    let (foot_x, foot_y) = if swap { (x1, x0) } else { (x0, x1) };
    let foot_x = if raw_dx < 0.0 { -foot_x } else { foot_x };
    let foot_y = if raw_dy < 0.0 { -foot_y } else { foot_y };

    let distance = libm::sqrt((x0 - y0) * (x0 - y0) + (x1 - y1) * (x1 - y1));
    let ratio0 = y0 / e0;
    let ratio1 = y1 / e1;
    let signed_distance =
        if ratio0 * ratio0 + ratio1 * ratio1 < 1.0 { -distance } else { distance };

    Ok(PointOnGeometry {
        t:               Parameter::new(libm::atan2(foot_y / ry, foot_x / rx)),
        projected:       [Unit::new(cx + foot_x), Unit::new(cy + foot_y)],
        signed_distance: Unit::new(signed_distance),
    })
}

/// Below this, `e1 * y1` is near the subnormal range and the root `u >= e1 * y1` loses precision, so the
/// point is treated as lying on the major axis (error ~1e-97 at worst, at the evolute cusp).
const MIN_RESOLVABLE_PRODUCT: f64 = 1e-290;

/// Closest point on the ellipse (x0/e0)² + (x1/e1)² = 1 to (y0, y1), where e0 >= e1 > 0 and
/// y0, y1 >= 0 (Eberly, Listing 1; the circle is handled separately). Inputs must be finite and
/// below ~1e150 in magnitude (squared distances are formed).
fn ellipse_closest_in_first_quadrant(e0: f64, e1: f64, y0: f64, y1: f64) -> (f64, f64) {
    if e0 == e1 {
        let norm = libm::sqrt(y0 * y0 + y1 * y1);
        return if norm == 0.0 { (e0, 0.0) } else { (e0 * y0 / norm, e0 * y1 / norm) };
    }

    // e0² - e1² as a product of differences: no cancellation for nearly circular ellipses
    let delta = (e0 - e1) * (e0 + e1);
    let n0 = e0 * y0;
    let n1 = e1 * y1;

    if n1 >= MIN_RESOLVABLE_PRODUCT {
        if y0 > 0.0 {
            let u = ellipse_root(e0, e1, delta, y0, y1);
            (e0 * (n0 / (u + delta)), e1 * (n1 / u))
        } else {
            (0.0, e1)
        }
    } else if n0 < delta {
        // on the major axis, inside the evolute: x0 = e0² y0 / delta
        let ratio = n0 / delta;
        (e0 * ratio, e1 * libm::sqrt((1.0 - ratio) * (1.0 + ratio)))
    } else {
        (e0, 0.0)
    }
}

/// Root of F(u) = (e0*y0/(u + delta))² + (e1*y1/u)² - 1 on (0, inf), delta = e0² - e1², by bracketed Newton.
///
/// F is Eberly's F(t) with u = t + e1², which keeps the root relatively precise near the major axis; F is
/// strictly decreasing and convex. It is evaluated as `(c - u)(n0 + delta + u)/(u + delta)² + (n1/u)²` with
/// `c = n0 - delta`, avoiding the cancellation in `(n0/(u + delta))² - 1` at the evolute cusp.
///
/// The iteration count is bounded independently of the data:
/// 1. The root lies in [lo, hi] with lo = max(e1*y1, c) and hi = hypot(e0*y0, e1*y1).
/// 2. At most 11 geometric bisections (mid = sqrt(lo * hi)) bring hi / lo below 1.5; Newton alone advances
///    only ~1.5x per step when the root is far from lo.
/// 3. Newton from `lo` converges monotonically (F is convex and decreasing); 7 steps reach double precision.
///
/// Worst case is 19 evaluations of F (11 bracketing + 8 Newton), typically a handful.
/// Requires e0 > e1 > 0, y0 > 0 and e1 * y1 >= MIN_RESOLVABLE_PRODUCT.
fn ellipse_root(e0: f64, e1: f64, delta: f64, y0: f64, y1: f64) -> f64 {
    ellipse_root_counted(e0, e1, delta, y0, y1).0
}

const MAX_BRACKET_STEPS: usize = 12;
const MAX_NEWTON_STEPS: usize = 16;

/// F(u) of `ellipse_root` (see there), with c = n0 - delta passed in.
#[inline]
fn ellipse_root_function(n0: f64, n1: f64, c: f64, delta: f64, u: f64) -> f64 {
    let s = u + delta;
    let r1 = n1 / u;
    // two O(1) ratios avoid overflowing a product of two large factors
    ((c - u) / s) * ((n0 + s) / s) + r1 * r1
}

/// `ellipse_root` and the number of F evaluations it spent (the bound above is asserted in tests).
fn ellipse_root_counted(e0: f64, e1: f64, delta: f64, y0: f64, y1: f64) -> (f64, usize) {
    let n0 = e0 * y0;
    let n1 = e1 * y1;
    let c = n0 - delta;

    let scale = n0.max(n1);
    let mut lo = n1.max(c);
    let mut hi = scale * libm::sqrt((n0 / scale) * (n0 / scale) + (n1 / scale) * (n1 / scale));

    let mut evaluations = 0;
    for _ in 0..MAX_BRACKET_STEPS {
        if hi <= 1.5 * lo {
            break;
        }
        let mid = libm::sqrt(lo) * libm::sqrt(hi);
        if !(mid > lo && mid < hi) {
            break;
        }
        evaluations += 1;
        if ellipse_root_function(n0, n1, c, delta, mid) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    let mut u = lo;
    for _ in 0..MAX_NEWTON_STEPS {
        let value = ellipse_root_function(n0, n1, c, delta, u);
        evaluations += 1;
        if value <= 0.0 {
            break;
        }
        let s = u + delta;
        let (r0, r1) = (n0 / s, n1 / u);
        let slope = -2.0 * (r0 * r0 / s + r1 * r1 / u);
        let next = u - value / slope;
        if next.is_nan() || next <= u {
            break;
        }
        u = next;
    }
    (u, evaluations)
}

/// Reference root finder for tests: geometric bisection of the same F (independent of Newton).
#[cfg(test)]
fn ellipse_root_reference(e0: f64, e1: f64, delta: f64, y0: f64, y1: f64) -> f64 {
    let n0 = e0 * y0;
    let n1 = e1 * y1;
    let c = n0 - delta;
    let scale = n0.max(n1);
    let mut lo = n1.max(c);
    let mut hi = scale * libm::sqrt((n0 / scale) * (n0 / scale) + (n1 / scale) * (n1 / scale));
    for _ in 0..2200 {
        let mut mid = libm::sqrt(lo) * libm::sqrt(hi);
        if !(mid > lo && mid < hi) {
            mid = 0.5 * (lo + hi);
            if !(mid > lo && mid < hi) {
                break;
            }
        }
        if ellipse_root_function(n0, n1, c, delta, mid) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

pub struct Polygon<const D: usize> {
    pub vertices: Vec<Point<D>>,
}

/// Signed distance to the polygon boundary, searched over the segments; `t` / `projected` describe the nearest
/// point (t in [0, 1] on the edge `edge_index`). Outside is positive for a counter-clockwise polygon and
/// negative for a clockwise one.
///
/// When the nearest point is a vertex, the sign comes from the vertex pseudo-normal (the sum of the two
/// adjacent edge normals), which is correct at reflex vertices
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
/// let (result, edge_index) = as_on_polygon([Unit::new(5.0), Unit::new(-2.0)], polygon).unwrap();
/// assert_eq!(result.signed_distance.get(), 2.0);
/// assert_eq!(edge_index, 0);
/// ```
pub fn as_on_polygon(
    point: [Unit; 2],
    polygon: Polygon<2>,
) -> Result<(PointOnGeometry<2>, usize), Error> {
    let n = polygon.vertices.len();
    if n < 3 {
        return Err(Error::InvalidInput);
    }

    let px = point[0].get();
    let py = point[1].get();
    reject_nan(&[px, py])?;
    for v in &polygon.vertices {
        reject_nan(&[v[0].get(), v[1].get()])?;
    }
    let vertex = |i: usize| -> (f64, f64) {
        let v = polygon.vertices[i % n];
        (v[0].get(), v[1].get())
    };

    // nearest edge by squared segment distance; the first edge wins ties
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

    // outward unit normal of the first non-degenerate edge from `from` (skips duplicated vertices)
    let outward_normal = |from: usize, step: usize| -> (f64, f64) {
        for k in 0..n {
            let i = (from + k * step) % n;
            let (ax, ay) = vertex(i);
            let (bx, by) = vertex(i + 1);
            let (ex, ey) = (bx - ax, by - ay);
            let length = libm::sqrt(ex * ex + ey * ey);
            if length != 0.0 {
                return (ey / length, -ex / length);
            }
        }
        (0.0, 0.0)
    };

    let outside = if best_t > 0.0 && best_t < 1.0 {
        vx * (py - y1) - vy * (px - x1) < 0.0
    } else {
        let (vertex_index, previous_edge, next_edge) = if best_t <= 0.0 {
            (best_edge, best_edge + n - 1, best_edge)
        } else {
            (best_edge + 1, best_edge, best_edge + 1)
        };
        let (qx, qy) = vertex(vertex_index);
        let (n0x, n0y) = outward_normal(previous_edge, n - 1);
        let (n1x, n1y) = outward_normal(next_edge, 1);
        (px - qx) * (n0x + n1x) + (py - qy) * (n0y + n1y) > 0.0
    };

    let signed_distance = if distance == 0.0 {
        0.0
    } else if outside {
        distance
    } else {
        -distance
    };

    Ok((
        PointOnGeometry {
            t: Parameter::new(best_t),
            projected,
            signed_distance: Unit::new(signed_distance),
        },
        best_edge,
    ))
}

pub struct PointOnGeometry<const D: usize> {
    pub t:               Parameter,
    pub projected:       Point<D>,
    pub signed_distance: Unit,
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
        let result = as_on_line(p(0.0, 0.0), line).unwrap();
        assert!(result.signed_distance.get().abs() < 1e-8);
    }

    #[test]
    fn as_on_line_handles_zero_length_segment() {
        let line = Line { start: p(2.0, 2.0), end: p(2.0, 2.0) };
        let result = as_on_line(p(5.0, 6.0), line).unwrap();
        assert_eq!(result.signed_distance.get().abs(), 5.0);
    }

    #[test]
    fn as_on_line_t_outside_zero_one_means_beyond_segment() {
        let line = Line { start: p(0.0, 0.0), end: p(10.0, 0.0) };
        let result = as_on_line(p(-5.0, 0.0), line).unwrap();
        assert!(result.t.get() < 0.0);
    }

    #[test]
    fn as_on_circle_negative_coordinates_and_decimals() {
        let circle = Circle { center: p(-4.0, 4.0), radius: Unit::new(5.0) };
        let result = as_on_circle(p(1.0, 4.0), circle).unwrap();
        assert!(result.signed_distance.get().abs() < 1e-8);
    }

    #[test]
    fn as_on_circle_outside_is_positive() {
        let circle = Circle { center: p(0.0, 0.0), radius: Unit::new(5.0) };
        let result = as_on_circle(p(10.0, 0.0), circle).unwrap();
        assert_eq!(result.signed_distance.get(), 5.0);
    }

    #[test]
    fn from_three_points_returns_none_for_colinear_points() {
        let result = Circle::from_three_points(p(0.0, 0.0), p(1.0, 1.0), p(2.0, 2.0)).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn from_three_points_handles_matching_y_on_first_and_third_point() {
        let result =
            Circle::from_three_points(p(0.0, 0.0), p(1.0, 3.0), p(4.0, 0.0)).unwrap().unwrap();
        assert!((result.center[0].get() - 2.0).abs() < 1e-8);
        assert!((result.center[1].get() - 1.0).abs() < 1e-8);
        assert!((result.radius.get() - 5.0_f64.sqrt()).abs() < 1e-8);
    }

    #[test]
    fn as_on_ellipse_inside_is_negative() {
        let ellipse =
            Ellipse { center: p(0.0, 0.0), rx: Unit::new(4.0), ry: Unit::new(3.0) };
        let result = as_on_ellipse(p(2.0, 0.0), ellipse).unwrap();
        assert!(result.signed_distance.get() < 0.0);
    }

    #[test]
    fn as_on_ellipse_degenerate_ry_zero_falls_back_to_center_distance() {
        let ellipse =
            Ellipse { center: p(0.0, 0.0), rx: Unit::new(4.0), ry: Unit::new(0.0) };
        let result = as_on_ellipse(p(3.0, 4.0), ellipse).unwrap();
        assert_eq!(result.signed_distance.get(), 5.0);
    }

    #[test]
    fn as_on_polygon_inside_is_negative() {
        let polygon =
            Polygon { vertices: vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)] };
        let (result, _) = as_on_polygon(p(5.0, 5.0), polygon).unwrap();
        assert!(result.signed_distance.get() < 0.0);
    }

    #[test]
    fn as_on_polygon_picks_nearest_among_multiple_shapes_edge() {
        let near = Polygon { vertices: vec![p(20.0, 0.0), p(30.0, 0.0), p(30.0, 10.0)] };
        let (result, _) = as_on_polygon(p(25.0, -2.0), near).unwrap();
        assert_eq!(result.signed_distance.get(), 2.0);
    }

    #[test]
    fn as_on_polygon_cw_inverts_sign() {
        let ccw =
            Polygon { vertices: vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)] };
        let cw = Polygon { vertices: vec![p(0.0, 0.0), p(0.0, 10.0), p(10.0, 10.0), p(10.0, 0.0)] };
        let inside = p(5.0, 5.0);
        let (result_ccw, _) = as_on_polygon(inside, ccw).unwrap();
        let (result_cw, _) = as_on_polygon(inside, cw).unwrap();
        assert!(result_ccw.signed_distance.get() < 0.0);
        assert!(result_cw.signed_distance.get() > 0.0);
    }

    #[test]
    fn as_on_polygon_returns_nearest_edge_index() {
        let polygon =
            Polygon { vertices: vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)] };
        let (_, edge_index) = as_on_polygon(p(5.0, -2.0), polygon).unwrap();
        assert_eq!(edge_index, 0);
    }

    #[test]
    fn as_on_polygon_rejects_less_than_three_vertices() {
        let polygon = Polygon { vertices: vec![p(0.0, 0.0), p(1.0, 1.0)] };
        assert!(matches!(as_on_polygon(p(0.0, 0.0), polygon), Err(Error::InvalidInput)));
    }

    #[test]
    fn nan_inputs_are_rejected_as_invalid_input() {
        let nan = f64::NAN;
        let line = |s: Point<2>| Line { start: s, end: p(1.0, 1.0) };
        assert!(matches!(as_on_line(p(nan, 0.0), line(p(0.0, 0.0))), Err(Error::InvalidInput)));
        assert!(matches!(as_on_line(p(0.0, 0.0), line(p(0.0, nan))), Err(Error::InvalidInput)));
        let circle = |r: f64| Circle { center: p(0.0, 0.0), radius: Unit::new(r) };
        assert!(matches!(as_on_circle(p(0.0, nan), circle(1.0)), Err(Error::InvalidInput)));
        assert!(matches!(as_on_circle(p(0.0, 0.0), circle(nan)), Err(Error::InvalidInput)));
        let ellipse = |rx: f64| Ellipse {
            center: p(0.0, 0.0),
            rx:     Unit::new(rx),
            ry:     Unit::new(1.0),
        };
        assert!(matches!(as_on_ellipse(p(nan, 0.0), ellipse(2.0)), Err(Error::InvalidInput)));
        assert!(matches!(as_on_ellipse(p(0.0, 0.0), ellipse(nan)), Err(Error::InvalidInput)));
        let polygon = |v: Point<2>| Polygon { vertices: vec![p(0.0, 0.0), p(1.0, 0.0), v] };
        assert!(matches!(
            as_on_polygon(p(nan, 0.0), polygon(p(0.0, 1.0))),
            Err(Error::InvalidInput)
        ));
        assert!(matches!(
            as_on_polygon(p(0.0, 0.0), polygon(p(nan, 1.0))),
            Err(Error::InvalidInput)
        ));
        assert!(matches!(
            Circle::from_three_points(p(0.0, 0.0), p(1.0, 0.0), p(nan, 1.0)),
            Err(Error::InvalidInput)
        ));
    }

    fn close(actual: f64, expected: f64, tolerance: f64) -> bool {
        (actual - expected).abs() <= tolerance * (1.0 + expected.abs())
    }

    fn lcg(state: &mut u64) -> f64 {
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (*state >> 11) as f64 / (1u64 << 53) as f64
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

    /// 22-vertex star (radius 10 / 4 alternating), counter-clockwise: acute convex tips, obtuse
    /// reflex valleys, no axis-aligned edges.
    fn star(clockwise: bool) -> Polygon<2> {
        let mut vertices: Vec<Point<2>> = (0..22)
            .map(|i| {
                let a = i as f64 * core::f64::consts::TAU / 22.0;
                let r = if i % 2 == 0 { 10.0 } else { 4.0 };
                p(r * libm::cos(a), r * libm::sin(a))
            })
            .collect();
        if clockwise {
            vertices.reverse();
        }
        Polygon { vertices }
    }

    /// Brute-force oracle: (min distance to the segments, point inside by the even-odd rule).
    fn polygon_oracle(vertices: &[Point<2>], q: (f64, f64)) -> (f64, bool) {
        let n = vertices.len();
        let v = |i: usize| (vertices[i % n][0].get(), vertices[i % n][1].get());
        let mut min_sq = f64::INFINITY;
        let mut inside = false;
        for i in 0..n {
            let ((x1, y1), (x2, y2)) = (v(i), v(i + 1));
            let (vx, vy) = (x2 - x1, y2 - y1);
            let len_sq = vx * vx + vy * vy;
            let t = if len_sq == 0.0 {
                0.0
            } else {
                (((q.0 - x1) * vx + (q.1 - y1) * vy) / len_sq).clamp(0.0, 1.0)
            };
            let (dx, dy) = (q.0 - x1 - t * vx, q.1 - y1 - t * vy);
            min_sq = min_sq.min(dx * dx + dy * dy);
            if (y1 > q.1) != (y2 > q.1) && q.0 < x1 + (q.1 - y1) / (y2 - y1) * (x2 - x1) {
                inside = !inside;
            }
        }
        (libm::sqrt(min_sq), inside)
    }

    /// Expected signed distance: outside positive for counter-clockwise, flipped for clockwise.
    fn expected_signed(polygon: &Polygon<2>, q: (f64, f64), clockwise: bool) -> f64 {
        let (distance, inside) = polygon_oracle(&polygon.vertices, q);
        let outside_positive = if distance == 0.0 {
            0.0
        } else if inside {
            -distance
        } else {
            distance
        };
        if clockwise { -outside_positive } else { outside_positive }
    }

    /// In 3-4-5 triangles the nearest boundary point is the vertex, not the edge's line.
    #[test]
    fn as_on_polygon_nearest_convex_vertex_gives_euclidean_distance() {
        let (result, edge) = as_on_polygon(p(13.0, 14.0), square()).unwrap();
        assert_eq!(result.signed_distance.get(), 5.0);
        assert_eq!((result.projected[0].get(), result.projected[1].get()), (10.0, 10.0));
        assert_eq!((edge, result.t.get()), (1, 1.0));

        let (result, edge) = as_on_polygon(p(-3.0, -4.0), square()).unwrap();
        assert_eq!(result.signed_distance.get(), 5.0);
        assert_eq!((result.projected[0].get(), result.projected[1].get()), (0.0, 0.0));
        assert_eq!((edge, result.t.get()), (0, 0.0));
    }

    #[test]
    fn as_on_polygon_edge_interior_reports_clamped_t_and_projection() {
        let (result, edge) = as_on_polygon(p(20.0, 0.5), square()).unwrap();
        assert_eq!(result.signed_distance.get(), 10.0);
        assert!(close(result.t.get(), 0.05, 1e-15));
        assert!(close(result.projected[0].get(), 10.0, 1e-15));
        assert!(close(result.projected[1].get(), 0.5, 1e-15));
        assert_eq!(edge, 1);
    }

    /// (3, 3) is inside the L and its nearest boundary point is the reflex vertex (4, 4).
    #[test]
    fn as_on_polygon_reflex_vertex_inside_is_negative_with_euclidean_distance() {
        let (result, edge) = as_on_polygon(p(3.0, 3.0), l_shape()).unwrap();
        assert!(close(result.signed_distance.get(), -core::f64::consts::SQRT_2, 1e-15));
        assert_eq!((result.projected[0].get(), result.projected[1].get()), (4.0, 4.0));
        assert_eq!((edge, result.t.get()), (2, 1.0));
    }

    #[test]
    fn as_on_polygon_notch_outside_is_positive() {
        let (result, edge) = as_on_polygon(p(5.0, 5.0), l_shape()).unwrap();
        assert_eq!(result.signed_distance.get(), 1.0);
        assert_eq!((result.projected[0].get(), result.projected[1].get()), (5.0, 4.0));
        assert_eq!(edge, 2);
    }

    #[test]
    fn as_on_polygon_vertex_sign_follows_orientation() {
        let cw = Polygon { vertices: vec![p(0.0, 0.0), p(0.0, 10.0), p(10.0, 10.0), p(10.0, 0.0)] };
        let (outside, _) = as_on_polygon(p(13.0, 14.0), cw).unwrap();
        assert_eq!(outside.signed_distance.get(), -5.0);
    }

    #[test]
    fn as_on_polygon_on_boundary_is_exactly_zero() {
        assert_eq!(as_on_polygon(p(5.0, 0.0), square()).unwrap().0.signed_distance.get(), 0.0);
        assert_eq!(as_on_polygon(p(10.0, 10.0), square()).unwrap().0.signed_distance.get(), 0.0);
    }

    #[test]
    fn as_on_polygon_l_shape_grid_matches_oracle() {
        let polygon = l_shape();
        for ix in -6..=24 {
            for iy in -6..=24 {
                let q = (ix as f64 * 0.5, iy as f64 * 0.5);
                let expected = expected_signed(&polygon, q, false);
                let actual =
                    as_on_polygon(p(q.0, q.1), Polygon { vertices: polygon.vertices.clone() })
                        .unwrap()
                        .0
                        .signed_distance
                        .get();
                assert!(close(actual, expected, 1e-12), "q = {q:?}: {actual} vs {expected}");
            }
        }
    }

    #[test]
    fn as_on_polygon_star_random_points_match_oracle_for_both_orientations() {
        for clockwise in [false, true] {
            let polygon = star(clockwise);
            let mut state = 7;
            for _ in 0..3000 {
                let q =
                    ((lcg(&mut state) * 2.0 - 1.0) * 13.0, (lcg(&mut state) * 2.0 - 1.0) * 13.0);
                let expected = expected_signed(&polygon, q, clockwise);
                let actual =
                    as_on_polygon(p(q.0, q.1), Polygon { vertices: polygon.vertices.clone() })
                        .unwrap()
                        .0
                        .signed_distance
                        .get();
                assert!(
                    close(actual, expected, 1e-12),
                    "cw={clockwise} q={q:?}: {actual} vs {expected}"
                );
            }
        }
    }

    /// Around tips (acute convex) and valleys (obtuse reflex) the sign must come from the pseudo-normal.
    #[test]
    fn as_on_polygon_star_points_around_every_vertex_match_oracle() {
        for clockwise in [false, true] {
            let polygon = star(clockwise);
            for vertex in &polygon.vertices {
                for radius in [1e-3, 0.5, 3.0] {
                    for k in 0..16 {
                        let a = 0.1 + k as f64 * core::f64::consts::TAU / 16.0;
                        let q = (
                            vertex[0].get() + radius * libm::cos(a),
                            vertex[1].get() + radius * libm::sin(a),
                        );
                        let expected = expected_signed(&polygon, q, clockwise);
                        let actual = as_on_polygon(
                            p(q.0, q.1),
                            Polygon { vertices: polygon.vertices.clone() },
                        )
                        .unwrap()
                        .0
                        .signed_distance
                        .get();
                        assert!(
                            close(actual, expected, 1e-12),
                            "cw={clockwise} q={q:?}: {actual} vs {expected}"
                        );
                    }
                }
            }
        }
    }

    /// A repeated vertex gives a zero-length edge that must not drop a neighbour from the sign; a collinear vertex changes nothing.
    #[test]
    fn as_on_polygon_duplicate_and_collinear_vertices() {
        let repeated = || Polygon {
            vertices: vec![p(0.0, 0.0), p(10.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)],
        };
        assert_eq!(as_on_polygon(p(13.0, 14.0), repeated()).unwrap().0.signed_distance.get(), 5.0);
        assert_eq!(as_on_polygon(p(13.0, -4.0), repeated()).unwrap().0.signed_distance.get(), 5.0);
        assert_eq!(as_on_polygon(p(9.0, 1.0), repeated()).unwrap().0.signed_distance.get(), -1.0);

        let collinear = || Polygon {
            vertices: vec![p(0.0, 0.0), p(5.0, 0.0), p(10.0, 0.0), p(10.0, 10.0), p(0.0, 10.0)],
        };
        let (outside, _) = as_on_polygon(p(5.0, -2.0), collinear()).unwrap();
        assert_eq!(outside.signed_distance.get(), 2.0);
        assert_eq!((outside.projected[0].get(), outside.projected[1].get()), (5.0, 0.0));
        assert_eq!(as_on_polygon(p(5.0, 2.0), collinear()).unwrap().0.signed_distance.get(), -2.0);
        assert_eq!(as_on_polygon(p(13.0, 14.0), collinear()).unwrap().0.signed_distance.get(), 5.0);
    }

    /// Polygon and query both move by 1e8; the tolerance 1e-6 covers the input grid ulp(1e8) = 1.5e-8.
    #[test]
    fn as_on_polygon_is_stable_under_translation() {
        let offset = 1e8;
        let moved = Polygon {
            vertices: star(false)
                .vertices
                .iter()
                .map(|v| p(v[0].get() + offset, v[1].get() + offset))
                .collect(),
        };
        let mut state = 11;
        for _ in 0..200 {
            let q = ((lcg(&mut state) * 2.0 - 1.0) * 13.0, (lcg(&mut state) * 2.0 - 1.0) * 13.0);
            let expected = expected_signed(&moved, (q.0 + offset, q.1 + offset), false);
            let actual = as_on_polygon(
                p(q.0 + offset, q.1 + offset),
                Polygon { vertices: moved.vertices.clone() },
            )
            .unwrap()
            .0
            .signed_distance
            .get();
            assert!((actual - expected).abs() < 1e-6, "q={q:?}: {actual} vs {expected}");
        }
    }

    /// (name, rx, ry, dx, dy, signed distance, closest point relative to the center)
    type EllipseCase = (&'static str, f64, f64, f64, f64, f64, (f64, f64));

    const ELLIPSE_CASES: [EllipseCase; 39] = [
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
            1e-06,
            -0.295521586053766,
            (3.9466650500740084, 0.16275652493147658),
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
            -0.295522136790681,
            (3.946666666666651, 0.1627540748764736),
        ),
        (
            "100x0.01 (50,1e-9) eccentric",
            100.0,
            0.01,
            50.0,
            1e-09,
            -0.00866025302341063,
            (50.00000049999995, 0.008660254008976876),
        ),
        ("2x1 cusp (1.5,1e-300)", 2.0, 1.0, 1.5, 1e-300, -0.5, (2.0, 8.735804647362988e-101)),
        ("2x1 cusp (1.5,1e-100)", 2.0, 1.0, 1.5, 1e-100, -0.5, (2.0, 4.054801330382267e-34)),
        ("2x1 cusp (1.5,1e-30)", 2.0, 1.0, 1.5, 1e-30, -0.5, (2.0, 8.735804647362989e-11)),
        (
            "2x1 cusp (1.5,1e-8)",
            2.0,
            1.0,
            1.5,
            1e-08,
            -0.499999999971769,
            (1.9999964578079061, 0.0018820703910963743),
        ),
        (
            "2x1 just below cusp (1.5(1-1e-9),1e-12)",
            2.0,
            1.0,
            1.4999999985,
            1e-12,
            -0.5000000014999998,
            (1.9999999909803683, 9.497174164576132e-05),
        ),
        (
            "2x1 just above cusp (1.5(1+1e-9),1e-12)",
            2.0,
            1.0,
            1.5000000015000001,
            1e-12,
            -0.49999999849999976,
            (1.9999999936403032, 7.974770708447724e-05),
        ),
        (
            "2x1 beyond cusp (1.6,1e-6)",
            2.0,
            1.0,
            1.6,
            1e-06,
            -0.3999999999949999,
            (1.999999999975, 4.999999998999995e-06),
        ),
        (
            "near-circle 1+1e-12 inside (700,800)",
            1000.000000001,
            1000.0,
            700.0,
            800.0,
            63.01458127303132,
            (658.5046078688479, 752.5766947071655),
        ),
        (
            "near-circle 1+1e-12 outside (3000,4000)",
            1000.000000001,
            1000.0,
            3000.0,
            4000.0,
            3999.99999999964,
            (600.0000000008304, 799.9999999998272),
        ),
        (
            "near-circle 1+1e-8 inside (0.6,0.8)",
            1.00000001,
            1.0,
            0.6,
            0.8,
            -3.599999921356644e-09,
            (0.6000000021599999, 0.80000000288),
        ),
        (
            "near-circle 1+1e-8 outside (3,4)",
            1.00000001,
            1.0,
            3.0,
            4.0,
            3.9999999964,
            (0.6000000083039999, 0.799999998272),
        ),
        (
            "1e6x1 (5e5,0.5)",
            1000000.0,
            1.0,
            500000.0,
            0.5,
            -0.36602540378437765,
            (500000.00000021135, 0.8660254037843166),
        ),
        (
            "1e6x1 (2e6,3)",
            1000000.0,
            1.0,
            2000000.0,
            3.0,
            1000000.0000045,
            (1000000.0, 2.999999999997e-12),
        ),
        (
            "1e8x1 (3e7,1)",
            100000000.0,
            1.0,
            30000000.0,
            1.0,
            0.04606079858305435,
            (30000000.0, 0.9539392014169457),
        ),
        (
            "4x1 far (1e6,1e6)",
            4.0,
            1.0,
            1000000.0,
            1000000.0,
            1414210.6468994874,
            (3.880569170127413, 0.24253645548873687),
        ),
        (
            "4x1 far on major side (1e9,1)",
            4.0,
            1.0,
            1000000000.0,
            1.0,
            999999996.0,
            (4.0, 2.500000009375e-10),
        ),
        (
            "4e100x1e100 (3e100,2e100)",
            4e+100,
            1e+100,
            3e+100,
            2e+100,
            1.2973054925552015e+100,
            (2.7090557089436533e+100, 7.357401530873434e+99),
        ),
        (
            "4e-100x1e-100 (3e-100,2e-100)",
            4e-100,
            1e-100,
            3e-100,
            2e-100,
            1.2973054925552015e-100,
            (2.709055708943653e-100, 7.357401530873435e-101),
        ),
        ("4x1 (3,1e-320) subnormal y", 4.0, 1.0, 3.0, 1e-320, -0.6324555320336759, (3.2, 0.6)),
        ("4x1 (1e-300,5) tiny x", 4.0, 1.0, 1e-300, 5.0, 4.0, (8e-301, 1.0)),
        (
            "4x1 (1e-300,1e-300) tiny both",
            4.0,
            1.0,
            1e-300,
            1e-300,
            -1.0,
            (1.0666666666666666e-300, 1.0),
        ),
    ];

    /// The tolerance is a few ulp of the largest magnitude involved.
    #[test]
    fn as_on_ellipse_matches_high_precision_reference() {
        for (name, rx, ry, dx, dy, signed, foot) in ELLIPSE_CASES {
            let ellipse =
                Ellipse { center: p(0.0, 0.0), rx: Unit::new(rx), ry: Unit::new(ry) };
            let result = as_on_ellipse(p(dx, dy), ellipse).unwrap();
            let magnitude = rx.max(ry).max(dx.abs()).max(dy.abs());
            let tolerance = 1e-14 * magnitude;
            let got = result.signed_distance.get();
            assert!((got - signed).abs() <= tolerance, "{name}: distance {got:e} vs {signed:e}");
            let (fx, fy) = (result.projected[0].get(), result.projected[1].get());
            assert!((fx - foot.0).abs() <= tolerance, "{name}: foot x {fx:e} vs {:e}", foot.0);
            assert!((fy - foot.1).abs() <= tolerance, "{name}: foot y {fy:e} vs {:e}", foot.1);
        }
    }

    #[test]
    fn as_on_ellipse_is_translation_invariant_for_large_center() {
        let ellipse =
            Ellipse { center: p(1e9, -5e8), rx: Unit::new(4.0), ry: Unit::new(1.0) };
        let result = as_on_ellipse(p(1e9 + 3.0, -5e8 + 2.0), ellipse).unwrap();
        assert!(close(result.signed_distance.get(), 1.2973054925552014, 1e-12));
        assert!(close(result.projected[0].get() - 1e9, 2.7090557089436533, 1e-6));
    }

    #[test]
    fn as_on_ellipse_non_finite_inputs_terminate() {
        let ellipse =
            || Ellipse { center: p(0.0, 0.0), rx: Unit::new(4.0), ry: Unit::new(1.0) };
        for (x, y) in [(f64::INFINITY, 1.0), (1.0, f64::INFINITY)] {
            let _ = as_on_ellipse(p(x, y), ellipse());
        }
    }

    /// (2, sqrt(3/4)) = (4 cos(pi/3), sin(pi/3)) is on the 4x1 ellipse up to rounding of the inputs.
    #[test]
    fn as_on_ellipse_points_on_the_ellipse_are_zero() {
        let ellipse =
            || Ellipse { center: p(0.0, 0.0), rx: Unit::new(4.0), ry: Unit::new(1.0) };
        assert_eq!(as_on_ellipse(p(4.0, 0.0), ellipse()).unwrap().signed_distance.get(), 0.0);
        assert_eq!(as_on_ellipse(p(0.0, 1.0), ellipse()).unwrap().signed_distance.get(), 0.0);
        let on = as_on_ellipse(p(2.0, libm::sqrt(0.75)), ellipse()).unwrap();
        assert!(on.signed_distance.get().abs() < 1e-15, "{}", on.signed_distance.get());
    }

    /// The foot lies on the ellipse, query - foot is normal there, and no sampled ellipse point is closer.
    #[test]
    fn as_on_ellipse_closest_point_is_the_foot_of_the_normal() {
        for (rx, ry) in [(4.0, 1.0), (1.0, 4.0), (3.0, 3.0), (100.0, 0.01), (7.0, 6.9)] {
            for ix in -12..=12 {
                for iy in -12..=12 {
                    let (dx, dy) = (ix as f64 * 0.9 * rx / 6.0, iy as f64 * 0.9 * ry / 6.0);
                    let ellipse = Ellipse {
                        center: p(1.0, -2.0),
                        rx:     Unit::new(rx),
                        ry:     Unit::new(ry),
                    };
                    let r = as_on_ellipse(p(1.0 + dx, -2.0 + dy), ellipse).unwrap();
                    let (fx, fy) = (r.projected[0].get() - 1.0, r.projected[1].get() + 2.0);
                    let d = r.signed_distance.get();
                    let scale = rx.max(ry);

                    let residual = (fx / rx) * (fx / rx) + (fy / ry) * (fy / ry) - 1.0;
                    assert!(residual.abs() < 1e-12, "({rx},{ry}) ({dx},{dy}): residual {residual}");

                    let actual = libm::sqrt((dx - fx) * (dx - fx) + (dy - fy) * (dy - fy));
                    assert!((actual - d.abs()).abs() <= 1e-12 * scale, "({rx},{ry}) ({dx},{dy})");

                    // orthogonality: (query - foot) . tangent = 0, tangent = (-rx sin t, ry cos t)
                    let t = r.t.get();
                    let (tx, ty) = (-rx * libm::sin(t), ry * libm::cos(t));
                    let dot = (dx - fx) * tx + (dy - fy) * ty;
                    assert!(
                        dot.abs() <= 1e-9 * (1.0 + actual) * libm::sqrt(tx * tx + ty * ty),
                        "({rx},{ry}) ({dx},{dy}): dot {dot}"
                    );

                    for k in 0..720 {
                        let a = k as f64 * core::f64::consts::PI / 360.0;
                        let (sx, sy) = (rx * libm::cos(a) - dx, ry * libm::sin(a) - dy);
                        assert!(libm::sqrt(sx * sx + sy * sy) >= d.abs() - 1e-12 * scale);
                    }
                }
            }
        }
    }

    /// Sweeps the hard cases (nearly circular, extreme eccentricity, the evolute cusp, tiny and huge
    /// coordinates): the iteration bound holds and the root agrees with a geometric bisection.
    #[test]
    fn ellipse_root_iteration_count_is_bounded_and_agrees_with_reference() {
        let mut worst = 0;
        let mut cases = 0;
        for ratio in [1.0 + 1e-12, 1.0 + 1e-8, 1.001, 1.5, 2.0, 4.0, 100.0, 1e4, 1e8] {
            for e1 in [1e-3, 1.0, 1e3] {
                let e0 = e1 * ratio;
                let delta = (e0 - e1) * (e0 + e1);
                let cusp = delta / e0;

                let mut y0s: Vec<f64> =
                    (-16..=14).map(|k| libm::pow(10.0, k as f64) * e1).collect();
                for k in 1..=15 {
                    let d = libm::pow(10.0, -(k as f64));
                    y0s.push(cusp * (1.0 - d));
                    y0s.push(cusp * (1.0 + d));
                }
                y0s.push(cusp);
                let mut y1s: Vec<f64> =
                    (-16..=14).map(|k| libm::pow(10.0, k as f64) * e1).collect();
                y1s.extend([1e-300, 1e-200, 1e-100, 1e-50, 1e-30, 1e-20].map(|y| y * e1.max(1.0)));

                for &y0 in &y0s {
                    for &y1 in &y1s {
                        if !(y0 > 0.0 && e1 * y1 >= MIN_RESOLVABLE_PRODUCT) {
                            continue;
                        }
                        cases += 1;
                        let (root, count) = ellipse_root_counted(e0, e1, delta, y0, y1);
                        worst = worst.max(count);
                        assert!(
                            count <= 19,
                            "e0={e0:e} e1={e1:e} y0={y0:e} y1={y1:e}: {count} evaluations"
                        );

                        let reference = ellipse_root_reference(e0, e1, delta, y0, y1);
                        let foot = |u: f64| (e0 * (e0 * y0 / (u + delta)), e1 * (e1 * y1 / u));
                        let (a, b) = (foot(root), foot(reference));
                        let scale = e0 + y0 + y1;
                        assert!(
                            (a.0 - b.0).abs() <= 1e-13 * scale
                                && (a.1 - b.1).abs() <= 1e-13 * scale,
                            "e0={e0:e} e1={e1:e} y0={y0:e} y1={y1:e}: {a:?} vs {b:?}"
                        );
                    }
                }
            }
        }
        assert!(cases > 10_000 && worst >= 10, "cases {cases}, worst {worst}");
    }

    /// On the (2, 1) ellipse the evolute cusp is at y0 = (e0² - e1²) / e0 = 1.5.
    #[test]
    fn ellipse_root_exact_cusp_with_tiny_y_stays_within_bound() {
        let (e0, e1, delta) = (2.0, 1.0, 3.0);
        for y1 in [1e-300, 1e-200, 1e-100, 1e-30, 1e-8] {
            let (_, count) = ellipse_root_counted(e0, e1, delta, 1.5, y1);
            assert!(count <= 19, "y1 = {y1:e}: {count} evaluations");
        }
    }

    /// A unit right triangle translated far from the origin keeps center (+0.5, +0.5) and radius sqrt(1/2).
    #[test]
    fn from_three_points_far_from_origin_keeps_precision() {
        for offset in [1e6, 1e9, 1e12] {
            let c = Circle::from_three_points(
                p(offset, offset),
                p(offset + 1.0, offset),
                p(offset, offset + 1.0),
            )
            .unwrap()
            .unwrap();
            assert_eq!(c.center[0].get() - offset, 0.5, "offset {offset}");
            assert_eq!(c.center[1].get() - offset, 0.5, "offset {offset}");
            assert_eq!(c.radius.get(), core::f64::consts::FRAC_1_SQRT_2, "offset {offset}");
        }
    }

    /// The triangle (0,0), (4,0), (1,3) has center (2,1) and radius sqrt(5) at every scale.
    #[test]
    fn from_three_points_is_scale_invariant() {
        for scale in [1e-9, 1e-5, 1.0, 1e5, 1e9] {
            let c = Circle::from_three_points(
                p(0.0, 0.0),
                p(4.0 * scale, 0.0),
                p(1.0 * scale, 3.0 * scale),
            )
            .unwrap()
            .unwrap();
            assert!(close(c.center[0].get() / scale, 2.0, 1e-14), "scale {scale}");
            assert!(close(c.center[1].get() / scale, 1.0, 1e-14), "scale {scale}");
            assert!(close(c.radius.get() / scale, libm::sqrt(5.0), 1e-14), "scale {scale}");
        }

        let tiny =
            Circle::from_three_points(p(0.0, 0.0), p(1e-5, 0.0), p(0.0, 1e-5)).unwrap().unwrap();
        assert!(close(tiny.center[0].get() / 5e-6, 1.0, 1e-14));
        assert!(close(tiny.radius.get() / 7.0710678118654755e-6, 1.0, 1e-14));
    }

    #[test]
    fn from_three_points_random_triangles_are_equidistant() {
        let mut state = 5;
        let mut checked = 0;
        for offset in [0.0, 1e3, 1e9] {
            for scale in [1e-6, 1.0, 1e6] {
                for _ in 0..200 {
                    let pts: Vec<(f64, f64)> = (0..3)
                        .map(|_| {
                            (offset + scale * lcg(&mut state), -offset + scale * lcg(&mut state))
                        })
                        .collect();
                    // skip thin triangles and offsets the input grid cannot resolve
                    let cross = (pts[1].0 - pts[0].0) * (pts[2].1 - pts[0].1)
                        - (pts[1].1 - pts[0].1) * (pts[2].0 - pts[0].0);
                    let longest_sq = (0..3)
                        .map(|i| {
                            let (a, b) = (pts[i], pts[(i + 1) % 3]);
                            (a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1)
                        })
                        .fold(0.0, f64::max);
                    let ulp = f64::EPSILON * offset;
                    if cross.abs() < 0.1 * longest_sq || scale < 1e3 * ulp {
                        continue;
                    }
                    let c = Circle::from_three_points(
                        p(pts[0].0, pts[0].1),
                        p(pts[1].0, pts[1].1),
                        p(pts[2].0, pts[2].1),
                    )
                    .unwrap()
                    .unwrap();
                    let radius = c.radius.get();
                    for q in &pts {
                        let (dx, dy) = (q.0 - c.center[0].get(), q.1 - c.center[1].get());
                        let distance = libm::sqrt(dx * dx + dy * dy);
                        assert!(
                            (distance - radius).abs() <= 1e-12 * radius + 4.0 * ulp,
                            "offset {offset} scale {scale}: {distance} vs {radius}"
                        );
                    }
                    checked += 1;
                }
            }
        }
        assert!(checked > 1000, "checked {checked}");
    }

    /// cross / longest² = 2.5e-7 is far above the relative threshold, so a circle of radius ~1e6 is returned.
    #[test]
    fn from_three_points_nearly_collinear_but_valid() {
        let c = Circle::from_three_points(p(0.0, 0.0), p(1.0, 0.0), p(2.0, 1e-6)).unwrap().unwrap();
        assert!(close(c.center[0].get(), 0.5, 1e-9));
        assert!(close(c.radius.get() / 1e6, 1.0, 1e-5), "{}", c.radius.get());
        for q in [p(0.0, 0.0), p(1.0, 0.0), p(2.0, 1e-6)] {
            let (dx, dy) = (q[0].get() - c.center[0].get(), q[1].get() - c.center[1].get());
            assert!(close(libm::sqrt(dx * dx + dy * dy) / c.radius.get(), 1.0, 1e-9));
        }
    }

    /// With apex height h over a unit base, cross / longest² = h and the threshold is 1e-12; the decision is scale-invariant.
    #[test]
    fn from_three_points_collinearity_threshold_is_relative_to_the_longest_side() {
        assert!(
            Circle::from_three_points(p(0.0, 0.0), p(1.0, 0.0), p(0.5, 0.9e-12)).unwrap().is_none()
        );
        assert!(
            Circle::from_three_points(p(0.0, 0.0), p(1.0, 0.0), p(0.5, 1.1e-12)).unwrap().is_some()
        );
        assert!(
            Circle::from_three_points(p(0.0, 0.0), p(1e9, 0.0), p(0.5e9, 0.9e-3))
                .unwrap()
                .is_none()
        );
        assert!(
            Circle::from_three_points(p(0.0, 0.0), p(1e9, 0.0), p(0.5e9, 1.1e-3))
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn from_three_points_rejects_coincident_points() {
        assert!(
            Circle::from_three_points(p(3.0, 3.0), p(3.0, 3.0), p(3.0, 3.0)).unwrap().is_none()
        );
        assert!(
            Circle::from_three_points(p(0.0, 0.0), p(1e-9, 1e-9), p(2e-9, 2e-9)).unwrap().is_none()
        );
    }

    /// With v = (3, 4) and |v| = 5: w = (1, 0) gives cross -4 (-0.8), w = (7, 2) gives -4.4, w = (-1, 2) gives +2, at every offset.
    #[test]
    fn as_on_line_distance_is_exact_for_integer_geometry_at_any_offset() {
        for offset in [0.0, 1e6, 1e9, 1e12] {
            let line = || Line { start: p(offset, offset), end: p(offset + 3.0, offset + 4.0) };
            let at = |wx: f64, wy: f64| as_on_line(p(offset + wx, offset + wy), line()).unwrap();
            assert_eq!(at(1.0, 0.0).signed_distance.get(), -0.8, "offset {offset}");
            assert_eq!(at(1.0, 0.0).t.get(), 0.12, "offset {offset}");
            assert_eq!(at(7.0, 2.0).signed_distance.get(), -4.4, "offset {offset}");
            assert_eq!(at(-1.0, 2.0).signed_distance.get(), 2.0, "offset {offset}");
            assert_eq!(at(3.0, 4.0).signed_distance.get(), 0.0, "offset {offset}");
            assert_eq!(at(6.0, 8.0).signed_distance.get(), 0.0, "offset {offset}");
        }
    }

    #[test]
    fn as_on_line_random_integer_lines_satisfy_the_exact_distance_identity() {
        // d² |v|² = cross² exactly in the reals; cross and |v|² are integers here
        let mut state = 21;
        for offset in [0.0, 1e6, 1e9, 1e12] {
            for _ in 0..300 {
                let mut coordinate = || (lcg(&mut state) * 2000.0 - 1000.0).floor();
                let (vx, vy, wx, wy) = (coordinate(), coordinate(), coordinate(), coordinate());
                if vx == 0.0 && vy == 0.0 {
                    continue;
                }
                let line = Line { start: p(offset, offset), end: p(offset + vx, offset + vy) };
                let d =
                    as_on_line(p(offset + wx, offset + wy), line).unwrap().signed_distance.get();

                let cross = (vx as i64 * wy as i64 - vy as i64 * wx as i64) as f64;
                let length_squared = vx * vx + vy * vy;
                assert!(
                    (d * d * length_squared - cross * cross).abs() <= 1e-15 * cross * cross,
                    "offset {offset} v=({vx},{vy}) w=({wx},{wy}): d = {d}"
                );
                assert_eq!(d < 0.0, cross < 0.0, "offset {offset} v=({vx},{vy}) w=({wx},{wy})");
            }
        }
    }
}
