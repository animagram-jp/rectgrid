use core::{primitive::{u32, usize, f64}, result::Result, array::from_fn, marker::PhantomData, ops::{Add, AddAssign, Sub, Mul, Div}};
use alloc::{vec::Vec, boxed::Box, rc::Rc};

use crate::RectgridError;

// When treating the rectgrid module of x and y as 2D coordinates,
// x represents the axis that becomes the width in the viewport.
// y represents the height direction in the viewport.
// The origin (0,0) is assumed to be the top-left corner.

// Px coordinate contract: px (point/pointer) passed in as a function argument from outside this
// module is received as global (an external coordinate not yet corrected for origin, e.g. a
// viewport coordinate), and each function subtracts origin internally to make it local.
// Conversely, px derived from a box (base/offset) — the return value of unit_to_px, and anything
// built on it such as hit_test/as_px results — is always local (origin=0 as the reference).
// If a caller passes such a value back across a RectGrid boundary, treat it as local px.

/// f64 value tagged with a unit system. The tag is zero-sized and has no runtime representation.
pub struct Value<Tag>(f64, PhantomData<Tag>);

impl<Tag> Clone for Value<Tag> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Tag> Copy for Value<Tag> {}

impl<Tag> Value<Tag> {
    pub fn new(v: f64) -> Self {
        Self(v, PhantomData)
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

impl<Tag> From<f64> for Value<Tag> {
    fn from(v: f64) -> Self {
        Self::new(v)
    }
}

impl<Tag> Add for Value<Tag> {
    type Output = Value<Tag>;
    fn add(self, rhs: Self) -> Self::Output {
        Value::new(self.0 + rhs.0)
    }
}

impl<Tag> AddAssign for Value<Tag> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<Tag> Sub for Value<Tag> {
    type Output = Value<Tag>;
    fn sub(self, rhs: Self) -> Self::Output {
        Value::new(self.0 - rhs.0)
    }
}

impl<Tag> Mul<f64> for Value<Tag> {
    type Output = Value<Tag>;
    fn mul(self, rhs: f64) -> Self::Output {
        Value::new(self.0 * rhs)
    }
}

impl<Tag> Div for Value<Tag> {
    type Output = f64;
    fn div(self, rhs: Self) -> f64 {
        self.0 / rhs.0
    }
}

pub struct PxTag;
pub struct UnitTag;
pub struct ParameterTag;

/// Orthogonal, axis-independent unit system extending infinitely in the positive direction from the origin.
pub type Px = Value<PxTag>;

/// Arbitrary orthogonal unit system, finite or infinite per axis, whose value is an ordinal from its own origin in the positive direction.
pub type Unit = Value<UnitTag>;

/// Unbounded local coordinate system for a single BBox, where each side length is 1 and sign follows the unit coordinate.
pub type Parameter = Value<ParameterTag>;

pub type Point<const D: usize>  = [Unit; D];

#[derive(Clone, Copy)]
pub struct BBox<const D: usize> {
    pub base:   Point<D>,
    pub offset: Point<D>,
}

impl<const D: usize> BBox<D> {
    /// Floors base/offset to snap them to an integer grid.
    /// extend applies to base only, added before flooring.
    ///
    /// ```
    /// use rectgrid::{BBox, Unit};
    /// let mut bx = BBox {
    ///     base:   [Unit::new(2.6), Unit::new(0.6)],
    ///     offset: [Unit::new(1.9), Unit::new(3.0)],
    /// };
    /// bx.snap_floor(Some([Unit::new(-0.5), Unit::new(-0.5)]));
    /// assert_eq!(bx.base[0].get(), 2.0);
    /// assert_eq!(bx.base[1].get(), 0.0);
    /// assert_eq!(bx.offset[0].get(), 1.0);
    /// assert_eq!(bx.offset[1].get(), 3.0);
    /// ```
    pub fn snap_floor(&mut self, extend: Option<[Unit; D]>) -> &mut Self {
        for (d, u) in self.base.iter_mut().enumerate() {
            let v = if let Some(ext) = extend { *u + ext[d] } else { *u };
            *u = Unit::new(libm::floor(v.get()));
        }
        for u in self.offset.iter_mut() {
            *u = Unit::new(libm::floor(u.get()));
        }
        self
    }

    /// Returns whether every axis of offset is nonzero (i.e., the BBox has geometric area/volume).
    /// If any axis is 0, it is treated as a segment or point with no area, and false is returned.
    ///
    /// ```
    /// use rectgrid::{BBox, Unit};
    /// let area = BBox { base: [Unit::new(0.0); 2], offset: [Unit::new(1.0), Unit::new(3.0)] };
    /// assert!(area.has_size());
    /// let segment = BBox { base: [Unit::new(0.0); 2], offset: [Unit::new(0.0), Unit::new(3.0)] };
    /// assert!(!segment.has_size());
    /// let point = BBox { base: [Unit::new(0.0); 2], offset: [Unit::new(0.0); 2] };
    /// assert!(!point.has_size());
    /// ```
    pub fn has_size(&self) -> bool {
        self.offset.iter().all(|u| u.get() != 0.0)
    }
}

// todo: Option-based expression, geometry definition implementation part
// pub type Region<const D: usize> = (Vec<BBox<D>>, Option<Box<dyn Fn(u32) -> Result<Px, RectgridError>>>);

pub enum IncrementFunction {
    /// Fn(i) = points[i+1] - points[i]
    /// OutOfIndex means boundary; the argument is the difference's index (integer).
    /// The closure must, when out of range, saturate to the last valid index within range and return it as OutOfIndex.
    /// todo: verify whether a scattered distribution of OutOfIndex is acceptable
    ForwardDifference(Rc<dyn Fn(u32) -> Result<Px, RectgridError>>),
    /// boundary
    /// Array enumerating unit values of the interval from the origin in the positive direction.
    VectorList(Vec<Px>),
    /// unboundary
    Scale(f64),
}

impl IncrementFunction {
    /// Builds an evaluation closure from the definition: unit coordinate (f64) -> px coordinate (distance relative to origin).
    /// Fractional parts are converted to px by linear interpolation.
    /// Returns InvalidDefinition if the definition cannot be evaluated, e.g. an empty VectorList.
    ///
    /// ```
    /// extern crate alloc;
    /// use rectgrid::{IncrementFunction, Px};
    ///
    /// let f = IncrementFunction::Scale(10.0).accumulate().unwrap();
    /// assert_eq!(f(2.5).unwrap().get(), 25.0);
    ///
    /// let f = IncrementFunction::VectorList(alloc::vec![Px::new(0.0), Px::new(10.0), Px::new(30.0)]).accumulate().unwrap();
    /// assert_eq!(f(1.0).unwrap().get(), 10.0);
    /// assert_eq!(f(1.5).unwrap().get(), 20.0);
    ///
    /// use rectgrid::RectgridError;
    /// assert!(matches!(IncrementFunction::VectorList(alloc::vec![]).accumulate(), Err(RectgridError::InvalidDefinition)));
    /// ```
    pub fn accumulate(&self) -> Result<Box<dyn Fn(f64) -> Result<Px, RectgridError>>, RectgridError> {
        match self {
            // Accumulate differences over 0..floor(x); the fractional remainder is added via linear interpolation of the next difference.
            Self::ForwardDifference(f) => {
                let f = f.clone();
                Ok(Box::new(move |x| {
                    let n = libm::floor(x) as u32;
                    let frac = x - n as f64;
                    let mut acc = Px::new(0.0);
                    for k in 0..n {
                        acc += f(k)?;
                    }
                    if frac != 0.0 {
                        acc += f(n)? * frac;
                    }
                    Ok(acc)
                }))
            }
            // Array of accumulated coordinates. Indexed by integer; fractions are linearly interpolated with the neighbor. Out of range is a boundary.
            Self::VectorList(pxs) => {
                if pxs.is_empty() {
                    return Err(RectgridError::InvalidDefinition);
                }
                let pxs = pxs.clone();
                Ok(Box::new(move |x| {
                    let last = (pxs.len() - 1) as u32;
                    let n = libm::floor(x) as usize;
                    let frac = x - n as f64;
                    let lo = *pxs.get(n).ok_or(RectgridError::OutOfIndex(last))?;
                    if frac == 0.0 {
                        return Ok(lo);
                    }
                    let hi = *pxs.get(n + 1).ok_or(RectgridError::OutOfIndex(last))?;
                    Ok(lo + (hi - lo) * frac)
                }))
            }
            Self::Scale(s) => {
                let s = *s;
                Ok(Box::new(move |x| Ok(Px::new(s * x))))
            }
        }
    }
}

pub struct RectGrid<const D: usize> {
    pub origin: [Px; D],
    /// f([Unit; D]) -> f([Px; D])
    accumulator: [Box<dyn Fn(f64) -> Result<Px, RectgridError>>; D],
}

impl<const D: usize> RectGrid<D> {
    pub fn new(origin: [Px; D], definitions: [IncrementFunction; D]) -> Result<Self, RectgridError> {
        let accumulator: Vec<_> = definitions.into_iter()
            .map(|d| d.accumulate())
            .collect::<Result<_, _>>()?;
        let accumulator = match accumulator.try_into() {
            Ok(a) => a,
            Err(_) => unreachable!("definitions and accumulator share length D"),
        };
        Ok(Self { origin, accumulator })
    }

    /// Replaces the definition for axis d. Subsequent calls to unit_to_px/point_to_unit etc. evaluate against this new definition.
    ///
    /// ```
    /// use rectgrid::{RectGrid, IncrementFunction, Px, Unit};
    /// let mut grid = RectGrid::<1>::new([Px::new(0.0)], [IncrementFunction::Scale(10.0)]).unwrap();
    /// assert_eq!(grid.unit_to_px(0, &Unit::new(2.0)).unwrap().get(), 20.0);
    /// grid.set_definition(IncrementFunction::Scale(100.0), 0).unwrap();
    /// assert_eq!(grid.unit_to_px(0, &Unit::new(2.0)).unwrap().get(), 200.0);
    /// ```
    pub fn set_definition(&mut self, definition: IncrementFunction, d: usize) -> Result<(), RectgridError> {
        self.accumulator[d] = definition.accumulate()?;
        Ok(())
    }

    /// Numerically inverts px to unit (the accumulator only holds a one-way Unit -> Px closure).
    /// point may be passed as-is as an external px coordinate (e.g. viewport); it is corrected to a local coordinate by subtracting origin before conversion.
    /// Caller contract: each axis's accumulator must be monotonically non-decreasing over Unit >= 0.
    /// If it is not (e.g. a negative value given to Scale, or ForwardDifference returning a decreasing difference), the result is not guaranteed.
    ///
    /// ```
    /// use rectgrid::{RectGrid, IncrementFunction, Px};
    /// let grid = RectGrid::<1>::new([Px::new(0.0)], [IncrementFunction::Scale(10.0)]).unwrap();
    /// let result = grid.point_to_unit([Px::new(25.0)]);
    /// assert!((result[0].unwrap().get() - 2.5).abs() < 1e-6);
    /// ```
    pub fn point_to_unit(&self, point: [Px; D]) -> [Result<Unit, RectgridError>; D] {
        from_fn(|i| self.px_to_unit_axis(i, point[i] - self.origin[i]))
    }

    fn px_to_unit_axis(&self, i: usize, target: Px) -> Result<Unit, RectgridError> {
        let target = target.get();
        let f = |x: f64| self.accumulator[i](x);

        let mut lo = 0.0;
        let mut hi = 1.0;
        loop {
            match f(hi) {
                Ok(px) if px.get() >= target => break,
                Ok(_) => {
                    lo = hi;
                    hi *= 2.0;
                }
                // Reached the end of the domain; check whether target is reachable within that range.
                Err(RectgridError::OutOfIndex(last)) => {
                    hi = last as f64;
                    if f(hi)?.get() < target {
                        return Err(RectgridError::OutOfIndex(last));
                    }
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        const EPSILON: f64 = 1e-9;
        for _ in 0..64 {
            if hi - lo < EPSILON {
                break;
            }
            let mid = (lo + hi) / 2.0;
            if f(mid)?.get() < target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Ok(Unit::new((lo + hi) / 2.0))
    }

    /// Converts a unit coordinate to px (evaluates the accumulator directly). unit must be an absolute value from the origin.
    ///
    /// ```
    /// use rectgrid::{RectGrid, IncrementFunction, Px, Unit};
    /// let grid = RectGrid::<1>::new([Px::new(0.0)], [IncrementFunction::Scale(200.0)]).unwrap();
    /// assert_eq!(grid.unit_to_px(0, &Unit::new(2.25)).unwrap().get(), 450.0);
    /// ```
    pub fn unit_to_px(&self, d: usize, unit: &Unit) -> Result<Px, RectgridError> {
        self.accumulator[d](unit.get())
    }

    /// Converts multiple unit coordinate points to px. Returns Err for a point with an unevaluable axis (evaluation stops per point; other points are unaffected).
    ///
    /// ```
    /// extern crate alloc;
    /// use rectgrid::{RectGrid, IncrementFunction, Px, Unit};
    /// use rectgrid::RectgridError;
    /// let grid = RectGrid::<1>::new(
    ///     [Px::new(0.0)],
    ///     [IncrementFunction::VectorList(alloc::vec![Px::new(0.0), Px::new(10.0)])],
    /// ).unwrap();
    /// let points = alloc::vec![[Unit::new(0.5)], [Unit::new(5.0)]];
    /// let px = grid.point_as_px(&points);
    /// assert_eq!(px[0].as_ref().unwrap()[0].get(), 5.0);
    /// // the 2nd point exceeds the VectorList's domain (0..=1), so it is OutOfIndex
    /// assert!(matches!(px[1], Err(RectgridError::OutOfIndex(1))));
    /// ```
    pub fn point_as_px(&self, points: &Vec<Point<D>>) -> Vec<Result<[Px; D], RectgridError>> {
        points.iter().map(|pt| -> Result<[Px; D], RectgridError> {
            let mut px = [Px::new(0.0); D];
            for d in 0..D {
                px[d] = self.unit_to_px(d, &pt[d])?;
            }
            Ok(px)
        }).collect()
    }

    /// Determines per axis whether point is contained in boxes[i] (extend included).
    /// point may be passed as-is in viewport coordinates (origin is subtracted internally).
    /// extend is added to base/offset in unit space before conversion to px
    /// (if converted to px individually and added afterward, the width would drift depending on boundary position for a nonlinear accumulator).
    /// Returns: (whether it hit, base_px without extend, offset_px without extend).
    /// base_px/offset_px are returned alongside the hit test so they can be reused directly for parameter calculation.
    fn contains(&self, point: [Px; D], bx: &BBox<D>, extend: Option<([Unit; D], [Unit; D])>) -> (bool, [Px; D], [Px; D]) {
        let local: [Px; D] = from_fn(|d| point[d] - self.origin[d]);
        let base_px:   [Px; D] = from_fn(|d| self.unit_to_px(d, &bx.base[d]).unwrap_or(Px::new(0.0)));
        let offset_px: [Px; D] = from_fn(|d| self.unit_to_px(d, &(bx.base[d] + bx.offset[d])).unwrap_or(Px::new(0.0)));
        let (lo, hi): ([Px; D], [Px; D]) = if let Some((eb, eo)) = extend {
            (from_fn(|d| self.unit_to_px(d, &(bx.base[d] + eb[d])).unwrap_or(Px::new(0.0))),
             from_fn(|d| self.unit_to_px(d, &(bx.base[d] + bx.offset[d] + eo[d])).unwrap_or(Px::new(0.0))))
        } else {
            (base_px, offset_px)
        };
        let hit = (0..D).all(|d| local[d].get() >= lo[d].get() && local[d].get() <= hi[d].get());
        (hit, base_px, offset_px)
    }

    /// `ξ_d = (point_d − base_d) / offset_d`
    /// Signed local coordinate (parameter) for a single box, with each side length (offset) normalized to 1.
    /// base_px/offset_px are the px-converted values of the unit coordinates base/base+offset (same shape as contains' return value).
    fn parameter_from_px(point: [Px; D], base_px: [Px; D], offset_px: [Px; D]) -> [Parameter; D] {
        from_fn(|d| {
            let width = offset_px[d] - base_px[d];
            if width.get() == 0.0 { Parameter::new(0.0) } else { Parameter::new((point[d] - base_px[d]) / width) }
        })
    }

    /// Returns the highest index among the boxes that point hits (a higher index in boxes is treated as higher priority).
    /// When multiple boxes hit, the higher index wins, so the scan runs from the tail.
    ///
    /// ```
    /// extern crate alloc;
    /// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit};
    /// let grid = RectGrid::<2>::new(
    ///     [Px::new(0.0), Px::new(0.0)],
    ///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
    /// ).unwrap();
    /// let boxes = alloc::vec![
    ///     BBox { base: [Unit::new(0.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] },
    ///     BBox { base: [Unit::new(2.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] },
    /// ];
    /// assert_eq!(grid.hit_test([Px::new(450.0), Px::new(10.0)], &boxes, None), Some(1));
    /// assert_eq!(grid.hit_test([Px::new(-100.0), Px::new(-100.0)], &boxes, None), None);
    /// ```
    pub fn hit_test(&self, point: [Px; D], boxes: &Vec<BBox<D>>, extend: Option<([Unit; D], [Unit; D])>) -> Option<usize> {
        boxes.iter()
            .enumerate()
            .rev()
            .find_map(|(i, bx)| self.contains(point, bx, extend).0.then_some(i))
    }

    /// Like hit_test, returns the highest-index hit, and additionally the get_parameter-equivalent value for the hit box.
    /// parameter is unaffected by extend; it is a ratio relative to the box interior (base side = 0.0, offset side = 1.0).
    ///
    /// ```
    /// extern crate alloc;
    /// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit};
    /// let grid = RectGrid::<2>::new(
    ///     [Px::new(0.0), Px::new(0.0)],
    ///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
    /// ).unwrap();
    /// let boxes = alloc::vec![BBox { base: [Unit::new(0.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] }];
    /// let (i, parameter) = grid.hit_test_with_parameter([Px::new(100.0), Px::new(32.0)], &boxes, None).unwrap();
    /// assert_eq!(i, 0);
    /// assert!((parameter[0].get() - 0.5).abs() < 1e-9);
    /// assert!((parameter[1].get() - 0.5).abs() < 1e-9);
    /// ```
    pub fn hit_test_with_parameter(&self, point: [Px; D], boxes: &Vec<BBox<D>>, extend: Option<([Unit; D], [Unit; D])>) -> Option<(usize, [Parameter; D])> {
        let local: [Px; D] = from_fn(|d| point[d] - self.origin[d]);
        boxes.iter()
            .enumerate()
            .rev()
            .find_map(|(i, bx)| {
                let (hit, base_px, offset_px) = self.contains(point, bx, extend);
                hit.then(|| (i, Self::parameter_from_px(local, base_px, offset_px)))
            })
    }

    /// Scans all boxes point hits, returning hit/no-hit for each, in a Vec the same length as boxes.
    ///
    /// ```
    /// extern crate alloc;
    /// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit};
    /// let grid = RectGrid::<2>::new(
    ///     [Px::new(0.0), Px::new(0.0)],
    ///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
    /// ).unwrap();
    /// let boxes = alloc::vec![
    ///     BBox { base: [Unit::new(0.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] },
    ///     BBox { base: [Unit::new(0.5), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] },
    /// ];
    /// assert_eq!(grid.hit_tests([Px::new(150.0), Px::new(10.0)], &boxes, None), alloc::vec![true, true]);
    /// ```
    pub fn hit_tests(&self, point: [Px; D], boxes: &Vec<BBox<D>>, extend: Option<([Unit; D], [Unit; D])>) -> Vec<bool> {
        boxes.iter()
            .map(|bx| self.contains(point, bx, extend).0)
            .collect()
    }

    /// `ξ_d = (point_d − base_d) / offset_d`
    /// Signed local coordinate (parameter) for a single box, with each side length (offset) normalized to 1.
    /// point may be passed as-is as an external px coordinate (e.g. viewport); origin is subtracted internally.
    ///
    /// ```
    /// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit};
    /// let grid = RectGrid::<2>::new(
    ///     [Px::new(0.0), Px::new(0.0)],
    ///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
    /// ).unwrap();
    /// let bx = BBox { base: [Unit::new(1.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] };
    /// let parameter = grid.get_parameter([Px::new(300.0), Px::new(16.0)], bx);
    /// assert!((parameter[0].get() - 0.5).abs() < 1e-9);
    /// assert!((parameter[1].get() - 0.25).abs() < 1e-9);
    /// // outside the box (base side), parameter goes negative
    /// let parameter = grid.get_parameter([Px::new(100.0), Px::new(0.0)], bx);
    /// assert!((parameter[0].get() - (-0.5)).abs() < 1e-9);
    /// ```
    pub fn get_parameter(&self, point: [Px; D], bx: BBox<D>) -> [Parameter; D] {
        let local: [Px; D] = from_fn(|d| point[d] - self.origin[d]);
        let base_px   = from_fn(|d| self.unit_to_px(d, &bx.base[d]).unwrap_or(Px::new(0.0)));
        let offset_px = from_fn(|d| self.unit_to_px(d, &(bx.base[d] + bx.offset[d])).unwrap_or(Px::new(1.0)));
        Self::parameter_from_px(local, base_px, offset_px)
    }

    /// Converts multiple BBox to (base_px, offset_px). offset_px is the actual side length accounting for base position
    /// (unit_to_px(base+offset) - unit_to_px(base)), which stays correct for base position even under a nonlinear
    /// accumulator (ForwardDifference/VectorList). Returns Err for a box with an unevaluable axis (evaluation stops per box; other boxes are unaffected).
    ///
    /// ```
    /// extern crate alloc;
    /// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit};
    /// let grid = RectGrid::<1>::new([Px::new(0.0)], [IncrementFunction::Scale(100.0)]).unwrap();
    /// let boxes = alloc::vec![BBox { base: [Unit::new(1.0)], offset: [Unit::new(2.0)] }];
    /// let result = grid.box_as_px(&boxes);
    /// let (base_px, offset_px) = result[0].as_ref().unwrap();
    /// assert_eq!(base_px[0].get(), 100.0);
    /// assert_eq!(offset_px[0].get(), 200.0);
    /// ```
    pub fn box_as_px(&self, boxes: &Vec<BBox<D>>) -> Vec<Result<([Px; D], [Px; D]), RectgridError>> {
        boxes.iter().map(|bx| -> Result<([Px; D], [Px; D]), RectgridError> {
            let mut base_px   = [Px::new(0.0); D];
            let mut offset_px = [Px::new(0.0); D];
            for d in 0..D {
                base_px[d]   = self.unit_to_px(d, &bx.base[d])?;
                offset_px[d] = self.unit_to_px(d, &(bx.base[d] + bx.offset[d]))? - base_px[d];
            }
            Ok((base_px, offset_px))
        }).collect()
    }

    /// Returns pointer's local coordinate (after origin correction) with z subtracted.
    /// pointer may be passed as-is as an external px coordinate (e.g. viewport); origin is subtracted internally.
    /// At drag start, passing base_px (the element's reference position) as z yields the drag offset;
    /// during drag, passing that offset as z yields the element's current reference position.
    ///
    /// ```
    /// use rectgrid::{RectGrid, IncrementFunction, Px};
    /// let grid = RectGrid::<2>::new(
    ///     [Px::new(10.0), Px::new(20.0)],
    ///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
    /// ).unwrap();
    /// let drag_offset = grid.offset([Px::new(230.0), Px::new(50.0)], [Px::new(200.0), Px::new(0.0)]);
    /// assert_eq!((drag_offset[0].get(), drag_offset[1].get()), (20.0, 30.0));
    /// ```
    pub fn offset(&self, pointer: [Px; D], z: [Px; D]) -> [Px; D] {
        from_fn(|d| (pointer[d] - self.origin[d]) - z[d])
    }
}

// ============================================================
// pointer / drag interaction (stateless helpers)
//
// Stateless pure functions depending only on RectGrid<D> and BBox<D>.
// All of them require only &RectGrid (RectGrid's accumulator is already built at new() time, so no
// internal state needs to change per call).
// Because BBox always keeps base/offset in Unit and cannot represent a mixed Px/Unit intermediate
// state, the px position during drag is never written back into BBox — it is only returned as a
// value. The caller holds onto px during the drag and finalizes it into BBox (Unit) via a snap_*
// function at the DragEnd-equivalent moment.
// ============================================================

/// For a BBox with area, determines per axis whether point is near an edge (within threshold).
/// Returns a pair of (each axis's parameter, if obtainable) and the corner test result.
/// Each element of the corner result: Some(true) = near the base-side edge ([0, threshold]), Some(false) = near the offset-side edge ([1-threshold, 1]), None = not applicable.
/// If at least one axis is Some, it is treated as a handle hit (all axes Some = corner, only one axis Some = edge).
/// If all axes are None, returns None to signal no handle (the caller should fall back to e.g. a move drag).
/// parameter is None when the box lacks size (has_size is false) or point is outside bx.
///
/// ```
/// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit, corner_test};
/// let grid = RectGrid::<2>::new(
///     [Px::new(0.0), Px::new(0.0)],
///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
/// ).unwrap();
/// let bx = BBox { base: [Unit::new(2.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(3.0)] };
/// // clicking near bx's top-left corner (400, 0)px -> corner on the base side for both axes
/// let (_, corner) = corner_test(&grid, [Px::new(400.0), Px::new(0.0)], &bx, 0.1);
/// assert_eq!(corner, Some([Some(true), Some(true)]));
/// // clicking near the middle of bx's top edge (x center, y = base side) -> a y-only handle (edge drag)
/// let (_, corner) = corner_test(&grid, [Px::new(500.0), Px::new(0.0)], &bx, 0.1);
/// assert_eq!(corner, Some([None, Some(true)]));
/// // near the center of bx matches neither edge nor corner, but parameter is still obtainable
/// let (parameter, corner) = corner_test(&grid, [Px::new(500.0), Px::new(96.0)], &bx, 0.1);
/// assert!(parameter.is_some());
/// assert_eq!(corner, None);
/// // outside bx, parameter is also unobtainable
/// let (parameter, corner) = corner_test(&grid, [Px::new(400.0), Px::new(-10.0)], &bx, 0.1);
/// assert!(parameter.is_none());
/// assert_eq!(corner, None);
/// ```
pub fn corner_test<const D: usize>(
    grid:      &RectGrid<D>,
    point:     [Px; D],
    bx:        &BBox<D>,
    threshold: f64,
) -> (Option<[Parameter; D]>, Option<[Option<bool>; D]>) {
    if !bx.has_size() { return (None, None); }
    let parameter = grid.get_parameter(point, *bx);
    let inside = parameter.iter().all(|r| r.get() >= 0.0 && r.get() <= 1.0);
    if !inside { return (None, None); }
    let corner: [Option<bool>; D] = from_fn(|d| {
        let r = parameter[d].get();
        if r <= threshold { Some(true) }
        else if r >= 1.0 - threshold { Some(false) }
        else { None }
    });
    let corner = if corner.iter().any(Option::is_some) { Some(corner) } else { None };
    (Some(parameter), corner)
}

/// During drag, updates BBox's base/offset via a corner-handle drag and returns the updated BBox.
/// corner[d] = Some(base_side): if base_side == true, moves the base-side edge; if false, the offset-side edge.
/// The new offset is guaranteed to be at least 1.0 unit (so base/offset never cross).
///
/// ```
/// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit, drag_resize};
/// let grid = RectGrid::<2>::new(
///     [Px::new(0.0), Px::new(0.0)],
///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
/// ).unwrap();
/// let bx = BBox { base: [Unit::new(2.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(3.0)] };
/// // dragging the top-left (base-side) corner to (150, 0)px -> the x axis's base shrinks to 0 unit
/// let resized = drag_resize(&grid, [Px::new(150.0), Px::new(0.0)], &bx, [Some(true), None]).unwrap();
/// assert_eq!(resized.base[0].get(), 0.0);
/// assert_eq!(resized.offset[0].get(), 3.0); // base 2.0 + offset 1.0 - new_base 0.0
/// assert_eq!(resized.offset[1].get(), 3.0); // y axis unchanged
/// ```
pub fn drag_resize<const D: usize>(
    grid:    &RectGrid<D>,
    pointer: [Px; D],
    bx:      &BBox<D>,
    corner:  [Option<bool>; D],
) -> Result<BBox<D>, RectgridError> {
    let unit = grid.point_to_unit(pointer);
    let mut resized = *bx;
    for d in 0..D {
        let Some(base_side) = corner[d] else { continue };
        let new_u = Unit::new(libm::floor(unit[d]?.get()));
        let base_u   = bx.base[d];
        let offset_u = bx.offset[d];
        if base_side {
            let new_offset = ((base_u + offset_u) - new_u).get().max(1.0);
            resized.base[d]   = new_u;
            resized.offset[d] = Unit::new(new_offset);
        } else {
            let new_offset = (new_u - base_u).get().max(1.0);
            resized.offset[d] = Unit::new(new_offset);
        }
    }
    Ok(resized)
}

/// During drag, computes base's px position for a move drag (as opposed to a corner-handle drag).
/// pointer may be passed as-is as an external px coordinate (e.g. viewport); origin is subtracted internally.
/// Because BBox always keeps base in Unit, the px position during drag is only returned here without
/// updating BBox; it is committed to BBox via snap_region_to_unit at the DragEnd-equivalent moment.
///
/// ```
/// use rectgrid::{RectGrid, IncrementFunction, Px, drag_translate};
/// let grid = RectGrid::<2>::new(
///     [Px::new(10.0), Px::new(20.0)],
///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
/// ).unwrap();
/// let px = drag_translate(&grid, [Px::new(230.0), Px::new(50.0)], [Px::new(20.0), Px::new(30.0)]);
/// assert_eq!((px[0].get(), px[1].get()), (200.0, 0.0));
/// ```
pub fn drag_translate<const D: usize>(
    grid:        &RectGrid<D>,
    pointer:     [Px; D],
    drag_offset: [Px; D],
) -> [Px; D] {
    grid.offset(pointer, drag_offset)
}

/// At DragEnd, snaps the move-drag result of a BBox with area to the Unit grid and returns the updated BBox (base).
/// pointer may be passed as-is as an external px coordinate (e.g. viewport); origin is subtracted internally. drag_offset must be the same value passed to drag_translate.
/// extend is added to the unit-converted value before flooring.
///
/// ```
/// use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit, snap_region_to_unit};
/// let grid = RectGrid::<2>::new(
///     [Px::new(0.0), Px::new(0.0)],
///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
/// ).unwrap();
/// let bx = BBox { base: [Unit::new(0.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] };
/// let snapped = snap_region_to_unit(
///     &grid, [Px::new(430.0), Px::new(70.0)], [Px::new(0.0), Px::new(0.0)], &bx,
///     Some([Unit::new(0.25), Unit::new(0.25)]),
/// ).unwrap();
/// assert_eq!(snapped.base[0].get(), 2.0);
/// assert_eq!(snapped.base[1].get(), 1.0);
/// assert_eq!(snapped.offset[0].get(), 1.0); // offset is already floored, so it stays as-is
/// ```
pub fn snap_region_to_unit<const D: usize>(
    grid:        &RectGrid<D>,
    pointer:     [Px; D],
    drag_offset: [Px; D],
    bx:          &BBox<D>,
    extend:      Option<[Unit; D]>,
) -> Result<BBox<D>, RectgridError> {
    let mut snapped = *bx;
    for d in 0..D {
        let local = pointer[d] - grid.origin[d] - drag_offset[d];
        snapped.base[d] = grid.px_to_unit_axis(d, local)?;
    }
    snapped.snap_floor(extend);
    Ok(snapped)
}

/// At DragEnd, computes a BBox snapped to the Unit grid from the move-drag result of a point BBox (no area).
/// pointer may be passed as-is as an external px coordinate (e.g. viewport); origin is subtracted internally. drag_offset must be the same value passed to drag_translate.
///
/// ```
/// use rectgrid::{RectGrid, IncrementFunction, Px, Unit, snap_point_to_unit};
/// let grid = RectGrid::<2>::new(
///     [Px::new(0.0), Px::new(0.0)],
///     [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
/// ).unwrap();
/// let snapped = snap_point_to_unit(
///     &grid, [Px::new(430.0), Px::new(70.0)], [Px::new(0.0), Px::new(0.0)], [Unit::new(0.25), Unit::new(0.25)],
/// ).unwrap();
/// assert_eq!(snapped.base[0].get(), 2.0);
/// assert_eq!(snapped.base[1].get(), 1.0);
/// assert!(!snapped.has_size());
/// ```
pub fn snap_point_to_unit<const D: usize>(
    grid:        &RectGrid<D>,
    pointer:     [Px; D],
    drag_offset: [Px; D],
    snap:        [Unit; D],
) -> Result<BBox<D>, RectgridError> {
    let mut base: Point<D> = [Unit::new(0.0); D];
    for d in 0..D {
        let local = pointer[d] - grid.origin[d] - drag_offset[d];
        let u = grid.px_to_unit_axis(d, local)?;
        base[d] = Unit::new(libm::floor((u + snap[d]).get()));
    }
    Ok(BBox { base, offset: from_fn(|_| Unit::new(0.0)) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::rc::Rc;

    #[test]
    fn point_to_unit_scale_roundtrip() {
        let grid = RectGrid::<2>::new(
            [Px::new(0.0), Px::new(0.0)],
            [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
        ).unwrap();
        let result = grid.point_to_unit([Px::new(450.0), Px::new(10.0)]);
        let x = result[0].as_ref().unwrap().get();
        let y = result[1].as_ref().unwrap().get();
        assert!((x - 2.25).abs() < 1e-6, "x = {}", x);
        assert!((y - 0.15625).abs() < 1e-6, "y = {}", y);
    }

    #[test]
    fn point_to_unit_vector_list_roundtrip() {
        let grid = RectGrid::<1>::new(
            [Px::new(0.0)],
            [IncrementFunction::VectorList(alloc::vec![Px::new(0.0), Px::new(10.0), Px::new(30.0), Px::new(60.0)])],
        ).unwrap();
        let result = grid.point_to_unit([Px::new(45.0)]);
        let x = result[0].as_ref().unwrap().get();
        assert!((x - 2.5).abs() < 1e-6, "x = {}", x);
    }

    #[test]
    fn point_to_unit_vector_list_out_of_range() {
        let grid = RectGrid::<1>::new(
            [Px::new(0.0)],
            [IncrementFunction::VectorList(alloc::vec![Px::new(0.0), Px::new(10.0), Px::new(30.0), Px::new(60.0)])],
        ).unwrap();
        let result = grid.point_to_unit([Px::new(100.0)]);
        assert!(matches!(result[0], Err(RectgridError::OutOfIndex(3))));
    }

    #[test]
    fn point_to_unit_applies_origin_offset() {
        let grid = RectGrid::<2>::new(
            [Px::new(10.0), Px::new(20.0)],
            [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
        ).unwrap();
        let result = grid.point_to_unit([Px::new(230.0), Px::new(50.0)]);
        let x = result[0].as_ref().unwrap().get();
        let y = result[1].as_ref().unwrap().get();
        assert!((x - 1.1).abs() < 1e-6, "x = {}", x);
        assert!((y - 0.46875).abs() < 1e-6, "y = {}", y);
    }

    // ForwardDifference is the only variant not covered by the doctests or the unit tests above.
    #[test]
    fn accumulate_forward_difference() {
        // f(i) = (i+1)*10 -> accumulated at x=2.5: 10+20+0.5*30 = 45
        let f = IncrementFunction::ForwardDifference(Rc::new(|i| Ok(Px::new((i + 1) as f64 * 10.0))));
        let acc = f.accumulate().unwrap();
        assert_eq!(acc(0.0).unwrap().get(), 0.0);
        assert!((acc(2.5).unwrap().get() - 45.0).abs() < 1e-9);
    }

    #[test]
    fn snap_floor_no_extend() {
        let mut bx = BBox { base: [Unit::new(2.7)], offset: [Unit::new(1.9)] };
        bx.snap_floor(None);
        assert_eq!(bx.base[0].get(), 2.0);
        assert_eq!(bx.offset[0].get(), 1.0);
    }

    #[test]
    fn box_as_px_out_of_index() {
        let grid = RectGrid::<1>::new(
            [Px::new(0.0)],
            [IncrementFunction::VectorList(alloc::vec![Px::new(0.0), Px::new(10.0)])],
        ).unwrap();
        let boxes = alloc::vec![BBox { base: [Unit::new(0.0)], offset: [Unit::new(5.0)] }];
        assert!(matches!(grid.box_as_px(&boxes)[0], Err(RectgridError::OutOfIndex(1))));
    }

    #[test]
    fn hit_test_with_parameter_no_hit() {
        let grid = RectGrid::<2>::new(
            [Px::new(0.0), Px::new(0.0)],
            [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
        ).unwrap();
        let boxes = alloc::vec![
            BBox { base: [Unit::new(0.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] },
        ];
        assert!(grid.hit_test_with_parameter([Px::new(500.0), Px::new(10.0)], &boxes, None).is_none());
    }

    #[test]
    fn hit_tests_partial_and_no_hit() {
        let grid = RectGrid::<2>::new(
            [Px::new(0.0), Px::new(0.0)],
            [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
        ).unwrap();
        let boxes = alloc::vec![
            BBox { base: [Unit::new(0.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] },
            BBox { base: [Unit::new(2.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(1.0)] },
        ];
        assert_eq!(grid.hit_tests([Px::new(50.0), Px::new(10.0)], &boxes, None), alloc::vec![true, false]);
        assert_eq!(grid.hit_tests([Px::new(-1.0), Px::new(-1.0)], &boxes, None), alloc::vec![false, false]);
    }

    #[test]
    fn drag_resize_offset_side() {
        let grid = RectGrid::<2>::new(
            [Px::new(0.0), Px::new(0.0)],
            [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
        ).unwrap();
        let bx = BBox {
            base:   [Unit::new(2.0), Unit::new(0.0)],
            offset: [Unit::new(1.0), Unit::new(3.0)],
        };
        // Drag the offset-side (right) edge to 900px (unit 4.5 -> floor=4): offset = 4 - 2 = 2, base unchanged.
        // Use a non-integer point: at an integer boundary (800px = unit 4.0), binary-search convergence error can shift the floored result.
        let resized = drag_resize(&grid, [Px::new(900.0), Px::new(0.0)], &bx, [Some(false), None]).unwrap();
        assert_eq!(resized.base[0].get(), 2.0);
        assert_eq!(resized.offset[0].get(), 2.0);
    }

    #[test]
    fn drag_resize_minimum_clamp() {
        let grid = RectGrid::<2>::new(
            [Px::new(0.0), Px::new(0.0)],
            [IncrementFunction::Scale(200.0), IncrementFunction::Scale(64.0)],
        ).unwrap();
        let bx = BBox {
            base:   [Unit::new(2.0), Unit::new(0.0)],
            offset: [Unit::new(2.0), Unit::new(3.0)],
        };
        // Drag the base side past end (unit 4.0) to unit 5.5 (=1100px): offset clamps to a minimum of 1.0.
        let resized = drag_resize(&grid, [Px::new(1100.0), Px::new(0.0)], &bx, [Some(true), None]).unwrap();
        assert_eq!(resized.base[0].get(), 5.0);
        assert_eq!(resized.offset[0].get(), 1.0);
    }
}
