use alloc::vec::Vec;
use core::array::from_fn;

use rectgrid::{
    BBox, IncrementFunction, Px, RectGrid, Unit, corner_test, drag_resize, drag_translate,
    snap_point_to_unit, snap_region_to_unit,
};

use crate::js_client::{
    CanvasEvent, Command, CursorValue, EventType, Gesture, PointerState,
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
}

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
                (4, BBox::new([Unit::new(2.0), Unit::new(3.0)], [Unit::new(2.0), Unit::new(3.0)])),
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
        (vec![], cmds)
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
                    article_index_at(&event.id).or(hit_n)
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
                        return (vec![], cmds);
                    }
                    let px = drag_translate(&self.rectgrid, pointer, drag_offset);
                    (vec![], vec![translate_card(idx, px[0].get(), px[1].get())])
                } else {
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
