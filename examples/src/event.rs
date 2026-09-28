use alloc::vec::Vec;
use core::array::from_fn;

use rectgrid::{
    BBox, IncrementFunction, Px, RectGrid, Unit, corner_test, drag_resize, drag_translate,
    geometry::{Circle, as_on_circle},
    snap_point_to_unit, snap_region_to_unit,
};

use crate::js_client::{
    CanvasEvent, ClassName, Command, CursorValue, EventType, Gesture, PointerState,
    dom::{Id, Tag},
};

pub enum Event {
    Ready,
    Canvas(CanvasEvent),
    Gesture(Gesture),
    Resize { width: f64, height: f64, section_origin: [f64; 2] },
    Scroll { id: Id, x: f64, y: f64 },
    Shutdown,
}

const X_COLS: u32 = 5;
const Y_UNIT_REM: f64 = 4.0;
const REM_PX: f64 = 16.0;
const SECTION_PADDING_PX: f64 = 0.0;

pub struct Handler {
    articles:         Vec<(u32, BBox<2>)>,
    drag_target:      Option<u32>,
    drag_corner:      Option<[Option<bool>; 2]>,
    drag_pointer:     [f64; 2],
    drag_offset:      (f64, f64),
    drag_px:          (f64, f64),
    rectgrid:         RectGrid<2>,
    section_width_px: f64,
    drop_zone:        Circle<2>,
    drop_zone_active: bool,
}

const DROP_ZONE_ARTICLE: u32 = 4;

impl Handler {
    pub fn new(viewport_width_px: f64, section_origin_px: [f64; 2]) -> Self {
        let section_width_px = viewport_width_px - SECTION_PADDING_PX;
        let x_unit = section_width_px / X_COLS as f64;
        let y_unit = Y_UNIT_REM * REM_PX;
        Self {
            articles: alloc::vec![
                (1, BBox::new([Unit::new(0.0), Unit::new(0.0)], [Unit::new(0.0), Unit::new(0.0)])),
                (2, BBox::new([Unit::new(1.0), Unit::new(0.0)], [Unit::new(0.0), Unit::new(0.0)])),
                (3, BBox::new([Unit::new(2.0), Unit::new(0.0)], [Unit::new(2.0), Unit::new(3.0)])),
            ],
            drag_target: None,
            drag_corner: None,
            drag_pointer: [0.0; 2],
            drag_offset: (0.0, 0.0),
            drag_px: (0.0, 0.0),
            rectgrid: RectGrid::new(
                [Px::new(section_origin_px[0]), Px::new(section_origin_px[1])],
                [IncrementFunction::Scale(x_unit), IncrementFunction::Scale(y_unit)],
            )
            .unwrap(),
            section_width_px,
            // Fixed drop zone occupying the former article-4 slot: a circle inscribed
            // in a 2x3 unit square centered at [3.0, 4.5].
            drop_zone: Circle { center: [Unit::new(3.0), Unit::new(4.5)], radius: Unit::new(1.0) },
            drop_zone_active: false,
        }
    }
    pub fn close(&self) -> Vec<Command> {
        vec![]
    }

    pub fn initial_draw(&mut self) -> (Vec<Event>, Vec<Command>) {
        let mut cmds: Vec<Command> = vec![];
        let boxes: Vec<BBox<2>> = self.articles.iter().map(|(_, bx)| *bx).collect();
        let resolved = self.rectgrid.box_as_px(&boxes);
        for (z, ((n, bx), px_result)) in self.articles.iter().zip(resolved).enumerate() {
            let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(*n))]);
            if let Ok((base_px, offset_px)) = px_result {
                cmds.push(translate_card(*n, base_px[0].get(), base_px[1].get()));
                if bx.has_size() {
                    cmds.push(Command::SetWidth {
                        id: article.clone(),
                        px: offset_px[0].get() as u32,
                    });
                    cmds.push(Command::SetHeight {
                        id: article.clone(),
                        px: offset_px[1].get() as u32,
                    });
                }
            }
            cmds.push(Command::SetZIndex { id: article.clone(), z: z as i32 });
        }
        cmds.push(grid_background_cmd(self.section_width_px));
        cmds.extend(self.drop_zone_cmds());
        (vec![], cmds)
    }

    /// Checks whether a dragged card with area overlaps the circular drop zone, toggling the
    /// highlight only on a state change (entering/leaving). base_local_px is the card's current
    /// dragged-to base (local px, origin already subtracted); offset is its Unit-space size
    /// (unchanged during a move drag). Overlap is decided in Unit space via geometry::as_on_circle,
    /// by clamping the circle's center into the card's rect and testing the clamped (nearest) point
    /// — this catches edge/interior overlap, not just corners-inside-circle.
    /// point_to_unit expects a global (origin-relative) px, so origin is added back before the lookup.
    fn check_drop_zone(&mut self, base_local_px: [Px; 2], offset: [Unit; 2]) -> Vec<Command> {
        let base_global_px: [Px; 2] = from_fn(|d| base_local_px[d] + self.rectgrid.origin[d]);
        let base_unit: [Unit; 2] = from_fn(|d| {
            self.rectgrid.point_to_unit(base_global_px)[d].unwrap_or(Unit::new(f64::INFINITY))
        });
        let far_unit: [Unit; 2] = from_fn(|d| base_unit[d] + offset[d]);
        let nearest: [Unit; 2] = from_fn(|d| {
            Unit::new(self.drop_zone.center[d].get().clamp(base_unit[d].get(), far_unit[d].get()))
        });
        let zone = Circle { center: self.drop_zone.center, radius: self.drop_zone.radius };
        let result = as_on_circle(nearest, zone);
        let inside = result.signed_distance.get() <= 0.0;
        if inside == self.drop_zone_active {
            return vec![];
        }
        self.drop_zone_active = inside;
        let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(DROP_ZONE_ARTICLE))]);
        vec![if inside {
            Command::AddClass { id: article, value: ClassName::Highlighted }
        } else {
            Command::RemoveClass { id: article, value: ClassName::Highlighted }
        }]
    }

    /// Clears the drop zone highlight left over from a drag that just ended/was cancelled.
    fn clear_drop_zone(&mut self) -> Vec<Command> {
        if !self.drop_zone_active {
            return vec![];
        }
        self.drop_zone_active = false;
        let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(DROP_ZONE_ARTICLE))]);
        vec![Command::RemoveClass { id: article, value: ClassName::Highlighted }]
    }

    /// Positions/sizes the fixed circular drop zone (article-4) from drop_zone's Unit geometry.
    /// The grid's x/y axes have independent unit->px scales, so a circle defined in Unit space
    /// does not generally map to a square in px space; the diameter is pinned to the shorter of
    /// the two axis-wise px spans (border-radius: 50% in CSS then renders it as a true circle).
    fn drop_zone_cmds(&self) -> Vec<Command> {
        let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(DROP_ZONE_ARTICLE))]);
        let r = self.drop_zone.radius;
        let center_px: [Px; 2] = from_fn(|d| {
            self.rectgrid.unit_to_px(d, &self.drop_zone.center[d]).unwrap_or(Px::new(0.0))
        });
        let span_px: [Px; 2] = from_fn(|d| {
            let near = self.rectgrid.unit_to_px(d, &(self.drop_zone.center[d] - r)).unwrap_or(Px::new(0.0));
            let far = self.rectgrid.unit_to_px(d, &(self.drop_zone.center[d] + r)).unwrap_or(Px::new(0.0));
            far - near
        });
        let diameter = span_px[0].get().min(span_px[1].get());
        let base_px: [Px; 2] = from_fn(|d| center_px[d] - Px::new(diameter / 2.0));
        vec![
            translate_card(DROP_ZONE_ARTICLE, base_px[0].get(), base_px[1].get()),
            Command::SetWidth { id: article.clone(), px: diameter as u32 },
            Command::SetHeight { id: article.clone(), px: diameter as u32 },
        ]
    }

    pub fn process(
        &mut self,
        event: &CanvasEvent,
        _state: &PointerState,
    ) -> (Vec<Event>, Vec<Command>) {
        match &event.event_type {
            EventType::PointerDown => {
                let extend = Some((
                    [Unit::new(-0.05), Unit::new(-0.05)],
                    [Unit::new(0.05), Unit::new(0.05)],
                ));
                const CORNER_THRESHOLD: f64 = 0.1;
                let point = [Px::new(event.x), Px::new(event.y)];
                crate::debug_log!(
                    "event.x/y: {:?}, rectgrid.origin: {:?}",
                    (event.x, event.y),
                    (self.rectgrid.origin[0].get(), self.rectgrid.origin[1].get())
                );
                let boxes: Vec<BBox<2>> = self.articles.iter().map(|(_, bx)| *bx).collect();
                let hit_i = self.rectgrid.hit_test(point, &boxes, extend);
                let hit_n = hit_i.map(|i| self.articles[i].0);
                self.drag_corner = None;
                let corner: Option<[Option<bool>; 2]> = hit_i.and_then(|i| {
                    let (parameter, corner) =
                        corner_test(&self.rectgrid, point, &boxes[i], CORNER_THRESHOLD, extend);
                    crate::debug_log!(
                        "rectgrid parameter: {:?}, corner: {:?}",
                        parameter.map(|r| r.map(|p| p.get())),
                        corner
                    );
                    corner
                });
                let target = if corner.is_some() {
                    self.drag_corner = corner;
                    hit_n
                } else {
                    article_index_at(&event.id).filter(|n| *n != DROP_ZONE_ARTICLE).or(hit_n)
                };
                let mut cmds = vec![];
                if let Some(idx) = target {
                    if let Some((_, bx)) = self.articles.iter().find(|(n, _)| *n == idx) {
                        if self.drag_corner.is_none() {
                            let base_px: [Px; 2] = from_fn(|d| {
                                self.rectgrid.unit_to_px(d, &bx.base()[d]).unwrap_or(Px::new(0.0))
                            });
                            let offset = self.rectgrid.offset(point, base_px);
                            self.drag_offset = (offset[0].get(), offset[1].get());
                        }
                    }
                    let top_z = self.articles.len();
                    let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(idx))]);
                    cmds.push(Command::SetZIndex { id: article.clone(), z: top_z as i32 });
                    if let Some(cursor) = corner_cursor(self.drag_corner) {
                        let section = Id::new(&[(Tag::Section, None)]);
                        cmds.push(Command::SetCursor { id: section.clone(), value: cursor });
                    }
                }
                self.drag_target = target;
                (vec![], cmds)
            }
            EventType::KeyDown => todo!("keydown"),
            EventType::Input => todo!("input"),
            EventType::Change => todo!("change"),
            EventType::FocusOut => todo!("focusout"),
            EventType::Submit => todo!("submit"),
            _ => (vec![], vec![]),
        }
    }

    pub fn process_gesture(
        &mut self,
        gesture: &Gesture,
        _state: &PointerState,
    ) -> (Vec<Event>, Vec<Command>) {
        match gesture {
            Gesture::Drag { x, y } => {
                let pointer = [Px::new(*x), Px::new(*y)];
                self.drag_pointer = [*x, *y];
                let Some(idx) = self.drag_target else {
                    return (vec![], vec![]);
                };
                let Some(pos) = self.articles.iter_mut().find(|(n, _)| *n == idx) else {
                    return (vec![], vec![]);
                };
                let bx = &mut pos.1;
                let drag_offset = [Px::new(self.drag_offset.0), Px::new(self.drag_offset.1)];
                if bx.has_size() {
                    if let Some(corner) = self.drag_corner {
                        let Ok(new_bx) = drag_resize(&self.rectgrid, pointer, bx, corner) else {
                            return (vec![], vec![]);
                        };
                        *bx = new_bx;
                        let base_px: [Px; 2] = from_fn(|d| {
                            self.rectgrid.unit_to_px(d, &new_bx.base()[d]).unwrap_or(Px::new(0.0))
                        });
                        let size_px: [Px; 2] = from_fn(|d| {
                            self.rectgrid
                                .unit_to_px(d, &(new_bx.base()[d] + new_bx.offset()[d]))
                                .unwrap_or(Px::new(0.0))
                                - base_px[d]
                        });
                        let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(idx))]);
                        let mut cmds =
                            vec![translate_card(idx, base_px[0].get(), base_px[1].get())];
                        cmds.push(Command::SetWidth {
                            id: article.clone(),
                            px: size_px[0].get() as u32,
                        });
                        cmds.push(Command::SetHeight {
                            id: article.clone(),
                            px: size_px[1].get() as u32,
                        });
                        cmds.extend(self.check_drop_zone(base_px, new_bx.offset()));
                        return (vec![], cmds);
                    }
                    let offset = bx.offset();
                    let px = drag_translate(&self.rectgrid, pointer, drag_offset);
                    let mut cmds = vec![translate_card(idx, px[0].get(), px[1].get())];
                    cmds.extend(self.check_drop_zone(px, offset));
                    (vec![], cmds)
                } else {
                    // Point cards (no area) never trigger the drop zone.
                    let px = drag_translate(&self.rectgrid, pointer, drag_offset);
                    self.drag_px = (px[0].get(), px[1].get());
                    (vec![], vec![translate_card(idx, px[0].get(), px[1].get())])
                }
            }
            Gesture::DragEnd => {
                let mut cmds = vec![];
                if let Some(idx) = self.drag_target {
                    if self.drag_corner.is_some() {
                        let section = Id::new(&[(Tag::Section, None)]);
                        cmds.push(Command::SetCursor {
                            id:    section.clone(),
                            value: CursorValue::Unset,
                        });
                    }
                    if let Some(pos) = self.articles.iter_mut().find(|(n, _)| *n == idx) {
                        let bx = &mut pos.1;
                        let drag_pointer =
                            [Px::new(self.drag_pointer[0]), Px::new(self.drag_pointer[1])];
                        let drag_offset =
                            [Px::new(self.drag_offset.0), Px::new(self.drag_offset.1)];
                        if bx.has_size() {
                            if self.drag_corner.is_none() {
                                if let Ok(new_bx) = snap_region_to_unit(
                                    &self.rectgrid,
                                    drag_pointer,
                                    drag_offset,
                                    bx,
                                    Some([Unit::new(0.25), Unit::new(0.25)]),
                                ) {
                                    *bx = new_bx;
                                    let base_px: [Px; 2] = from_fn(|d| {
                                        self.rectgrid
                                            .unit_to_px(d, &new_bx.base()[d])
                                            .unwrap_or(Px::new(0.0))
                                    });
                                    cmds.push(translate_card(
                                        idx,
                                        base_px[0].get(),
                                        base_px[1].get(),
                                    ));
                                }
                            }
                        } else {
                            if let Ok(new_bx) = snap_point_to_unit(
                                &self.rectgrid,
                                drag_pointer,
                                drag_offset,
                                [Unit::new(0.25), Unit::new(0.25)],
                            ) {
                                *bx = new_bx;
                                let base_px: [Px; 2] = from_fn(|d| {
                                    self.rectgrid
                                        .unit_to_px(d, &new_bx.base()[d])
                                        .unwrap_or(Px::new(0.0))
                                });
                                cmds.push(translate_card(idx, base_px[0].get(), base_px[1].get()));
                            }
                        }
                    }
                    if let Some(old_pos) = self.articles.iter().position(|(n, _)| *n == idx) {
                        let entry = self.articles.remove(old_pos);
                        self.articles.push(entry);
                        for (new_z, (n, _)) in self.articles[old_pos..].iter().enumerate() {
                            let article =
                                Id::new(&[(Tag::Section, None), (Tag::Article, Some(*n))]);
                            cmds.push(Command::SetZIndex {
                                id: article.clone(),
                                z:  (old_pos + new_z) as i32,
                            });
                        }
                    }
                }
                cmds.extend(self.clear_drop_zone());
                self.drag_target = None;
                self.drag_corner = None;
                (vec![], cmds)
            }
            Gesture::DragCancel => {
                let mut cmds = vec![];
                if let Some(idx) = self.drag_target {
                    if self.drag_corner.is_some() {
                        let section = Id::new(&[(Tag::Section, None)]);
                        cmds.push(Command::SetCursor {
                            id:    section.clone(),
                            value: CursorValue::Unset,
                        });
                    }
                    if let Some((_, bx)) = self.articles.iter().find(|(n, _)| *n == idx) {
                        let base_px: [Px; 2] = from_fn(|d| {
                            self.rectgrid.unit_to_px(d, &bx.base()[d]).unwrap_or(Px::new(0.0))
                        });
                        cmds.push(translate_card(idx, base_px[0].get(), base_px[1].get()));
                        if bx.has_size() {
                            let size_px: [Px; 2] = from_fn(|d| {
                                self.rectgrid
                                    .unit_to_px(d, &(bx.base()[d] + bx.offset()[d]))
                                    .unwrap_or(Px::new(0.0))
                                    - base_px[d]
                            });
                            let article =
                                Id::new(&[(Tag::Section, None), (Tag::Article, Some(idx))]);
                            cmds.push(Command::SetWidth {
                                id: article.clone(),
                                px: size_px[0].get() as u32,
                            });
                            cmds.push(Command::SetHeight {
                                id: article.clone(),
                                px: size_px[1].get() as u32,
                            });
                        }
                    }
                }
                cmds.extend(self.clear_drop_zone());
                self.drag_target = None;
                self.drag_corner = None;
                (vec![], cmds)
            }
            _ => (vec![], vec![]),
        }
    }

    pub fn process_viewport(
        &mut self,
        width_px: f64,
        section_origin_px: [f64; 2],
    ) -> (Vec<Event>, Vec<Command>) {
        let section_width_px = width_px - SECTION_PADDING_PX;
        self.section_width_px = section_width_px;
        let _ = self
            .rectgrid
            .set_definition(IncrementFunction::Scale(section_width_px / X_COLS as f64), 0);
        self.rectgrid.origin = [Px::new(section_origin_px[0]), Px::new(section_origin_px[1])];
        let boxes: Vec<BBox<2>> = self.articles.iter().map(|(_, bx)| *bx).collect();
        let resolved = self.rectgrid.box_as_px(&boxes);
        let mut cmds = vec![grid_background_cmd(section_width_px)];
        for ((n, bx), px_result) in self.articles.iter().zip(resolved) {
            let Ok((base_px, offset_px)) = px_result else { continue };
            cmds.push(translate_card(*n, base_px[0].get(), base_px[1].get()));
            if bx.has_size() {
                let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(*n))]);
                cmds.push(Command::SetWidth { id: article.clone(), px: offset_px[0].get() as u32 });
                cmds.push(Command::SetHeight {
                    id: article.clone(),
                    px: offset_px[1].get() as u32,
                });
            }
        }
        cmds.extend(self.drop_zone_cmds());
        (vec![], cmds)
    }

    pub fn process_scroll(&mut self, _id: &Id, _x: f64, _y: f64) -> (Vec<Event>, Vec<Command>) {
        (vec![], vec![])
    }
}

fn grid_background_cmd(section_width_px: f64) -> Command {
    let x_unit = section_width_px / X_COLS as f64;
    let y_unit_rem = Y_UNIT_REM;
    let bg = format!(
        "repeating-linear-gradient(to right, var(--color-paper-mix) 0px, var(--color-paper-mix) 1px, transparent 1px, transparent {x_unit:.2}px), \
         repeating-linear-gradient(to bottom, var(--color-paper-mix) 0px, var(--color-paper-mix) 1px, transparent 1px, transparent {y_unit_rem}rem)"
    );
    let section = Id::new(&[(Tag::Section, None)]);
    Command::SetBackground { id: section.clone(), value: bg }
}

fn translate_card(n: u32, x: f64, y: f64) -> Command {
    let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(n))]);
    Command::SetTranslate { id: article.clone(), x, y }
}

fn corner_cursor(corner: Option<[Option<bool>; 2]>) -> Option<CursorValue> {
    match corner? {
        [Some(x_side), Some(y_side)] => {
            Some(if x_side == y_side { CursorValue::NwseResize } else { CursorValue::NeswResize })
        }
        [Some(_), None] => Some(CursorValue::EwResize),
        [None, Some(_)] => Some(CursorValue::NsResize),
        [None, None] => None,
    }
}

fn article_index_at(id: &Id) -> Option<u32> {
    id.0.iter().find_map(|seg| if matches!(seg.tag, Tag::Article) { seg.n } else { None })
}
