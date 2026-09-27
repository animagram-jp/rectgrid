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

/// ```
/// use rectgrid::Unit;
/// use rectgrid::geometry::*;
///
/// let ellipse = Ellipse { center: [Unit::new(0.0), Unit::new(0.0)], rx: Unit::new(4.0), ry: Unit::new(3.0) };
/// let result = as_on_ellipse([Unit::new(8.0), Unit::new(0.0)], ellipse);
/// assert_eq!(result.signed_distance.get(), 3.0);
/// ```
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

pub struct Polygon<const D: usize> {
    pub vertices: Vec<Point<D>>,
}

impl Polygon<2> {
    fn edges(&self) -> Vec<Line<2>> {
        let n = self.vertices.len();
        (0..n).map(|i| Line { start: self.vertices[i], end: self.vertices[(i + 1) % n] }).collect()
    }
}

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
    assert!(polygon.vertices.len() >= 3, "polygon must have at least 3 vertices");

    let px = point[0].get();
    let py = point[1].get();

    let mut best: Option<(f64, PointOnGeometry<2>, usize)> = None;

    for (i, edge) in polygon.edges().into_iter().enumerate() {
        let x1 = edge.start[0].get();
        let y1 = edge.start[1].get();
        let x2 = edge.end[0].get();
        let y2 = edge.end[1].get();

        let result = as_on_line(
            [Unit::new(px), Unit::new(py)],
            Line { start: [Unit::new(x1), Unit::new(y1)], end: [Unit::new(x2), Unit::new(y2)] },
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
}
