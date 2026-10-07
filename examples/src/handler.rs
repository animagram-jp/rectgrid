use alloc::{format, string::String, vec, vec::Vec};
use core::array::from_fn;

use rectgrid::{
    BBox, IncrementFunction, Px, RectGrid, Unit as GridUnit, corner_test, drag_resize,
    drag_translate,
    geometry::{Circle, as_on_circle},
    snap_bbox_to_unit, snap_point_to_unit,
};

use crate::{
    event::Event,
    js_client::{
        CanvasEvent, Command, EventType, Gesture, Keyword, PointerState, StyleProperty, StyleValue,
        Unit, VisibilityState,
        dom::{Id, Tag},
    },
};

const X_COLS: u32 = 5;
const Y_UNIT_REM: f64 = 4.0;
const REM_PX: f64 = 16.0;
const SECTION_PADDING_PX: f64 = 0.0;
const DROP_ZONE_ARTICLE: u32 = 4;
const CORNER_THRESHOLD: f64 = 0.1;

type Corner = [Option<bool>; 2];

pub struct Handler {
    articles:         Vec<(u32, BBox<2>)>,
    drag_target:      Option<u32>,
    drag_corner:      Option<Corner>,
    drag_pointer:     [f64; 2],
    drag_offset:      (f64, f64),
    rectgrid:         RectGrid<2>,
    section_width_px: f64,
    drop_zone:        Circle<2>,
    drop_zone_active: bool,
}

impl Handler {
    pub async fn ready(viewport_width_px: f64, _viewport_height_px: f64) -> Self {
        Self::new(viewport_width_px)
    }

    pub fn new(viewport_width_px: f64) -> Self {
        let section_width_px = viewport_width_px - SECTION_PADDING_PX;
        let x_unit = section_width_px / X_COLS as f64;
        let y_unit = Y_UNIT_REM * REM_PX;
        Self {
            articles: vec![
                (
                    1,
                    BBox::new(
                        [GridUnit::new(0.0), GridUnit::new(0.0)],
                        [GridUnit::new(0.0), GridUnit::new(0.0)],
                    ),
                ),
                (
                    2,
                    BBox::new(
                        [GridUnit::new(1.0), GridUnit::new(0.0)],
                        [GridUnit::new(0.0), GridUnit::new(0.0)],
                    ),
                ),
                (
                    3,
                    BBox::new(
                        [GridUnit::new(2.0), GridUnit::new(0.0)],
                        [GridUnit::new(2.0), GridUnit::new(3.0)],
                    ),
                ),
            ],
            drag_target: None,
            drag_corner: None,
            drag_pointer: [0.0; 2],
            drag_offset: (0.0, 0.0),
            rectgrid: RectGrid::new(
                [Px::new(0.0), Px::new(0.0)],
                [IncrementFunction::Scale(x_unit), IncrementFunction::Scale(y_unit)],
            )
            .unwrap(),
            section_width_px,
            drop_zone: Circle {
                center: [GridUnit::new(3.0), GridUnit::new(4.5)],
                radius: GridUnit::new(1.0),
            },
            drop_zone_active: false,
        }
    }

    pub fn close(&self) -> Vec<Command> {
        vec![]
    }

    pub fn initial_draw(&self) -> (Vec<Event>, Vec<Command>) {
        let mut commands: Vec<Command> = vec![];
        let boxes: Vec<BBox<2>> = self.articles.iter().map(|(_, bx)| *bx).collect();
        let resolved = self.rectgrid.box_as_px(&boxes);
        for (z, ((n, bx), px_result)) in self.articles.iter().zip(resolved).enumerate() {
            if let [Ok((base_x, size_x)), Ok((base_y, size_y))] = px_result {
                commands.push(translate_article(*n, base_x.get(), base_y.get()));
                if bx.has_size() {
                    commands.push(size_px(*n, StyleProperty::Width, size_x.get()));
                    commands.push(size_px(*n, StyleProperty::Height, size_y.get()));
                }
            }
            commands.push(z_index(*n, z as i32));
        }
        commands.push(grid_background(self.section_width_px));
        commands.extend(self.drop_zone_commands());
        (vec![], commands)
    }

    pub fn process_canvas(
        &mut self,
        event: &CanvasEvent,
        _state: &PointerState,
    ) -> (Vec<Event>, Vec<Command>) {
        if !matches!(event.event_type, EventType::PointerDown) {
            return (vec![], vec![]);
        }
        let (origin_x, origin_y) = event.root_origin();
        self.rectgrid.origin = [Px::new(origin_x), Px::new(origin_y)];

        let extend = Some((
            [GridUnit::new(-0.05), GridUnit::new(-0.05)],
            [GridUnit::new(0.05), GridUnit::new(0.05)],
        ));
        let point = [Px::new(event.x), Px::new(event.y)];
        let boxes: Vec<BBox<2>> = self.articles.iter().map(|(_, bx)| *bx).collect();
        let hit_index = self.rectgrid.hit_test(point, &boxes, extend);
        let hit_article = hit_index.map(|i| self.articles[i].0);
        self.drag_corner = None;
        let corner: Option<Corner> = hit_index.and_then(|i| {
            corner_test(&self.rectgrid, point, &boxes[i], CORNER_THRESHOLD, extend).1
        });
        let target = if corner.is_some() {
            self.drag_corner = corner;
            hit_article
        } else {
            article_index_at(&event.id).filter(|n| *n != DROP_ZONE_ARTICLE).or(hit_article)
        };
        let mut commands = vec![];
        if let Some(index) = target {
            if let Some((_, bx)) = self.articles.iter().find(|(n, _)| *n == index) {
                if self.drag_corner.is_none() {
                    let base_px: [Px; 2] = from_fn(|d| {
                        self.rectgrid.unit_to_px(d, &bx.base()[d]).unwrap_or(Px::new(0.0))
                    });
                    let offset = self.rectgrid.offset(point, base_px);
                    self.drag_offset = (offset[0].get(), offset[1].get());
                }
            }
            commands.push(z_index(index, self.articles.len() as i32));
            if let Some(cursor) = corner_cursor(self.drag_corner) {
                commands.push(style(section(), StyleProperty::Cursor, StyleValue::Keyword(cursor)));
            }
        }
        self.drag_target = target;
        (vec![], commands)
    }

    pub fn process_gesture(
        &mut self,
        gesture: &Gesture,
        _state: &PointerState,
        _origin: Option<&CanvasEvent>,
    ) -> (Vec<Event>, Vec<Command>) {
        match gesture {
            Gesture::Drag { x, y } => self.drag(Px::new(*x), Px::new(*y)),
            Gesture::DragEnd => self.drag_end(),
            Gesture::DragCancel => self.drag_cancel(),
            _ => (vec![], vec![]),
        }
    }

    pub fn process_resize(&mut self, width_px: f64, _height_px: f64) -> (Vec<Event>, Vec<Command>) {
        let section_width_px = width_px - SECTION_PADDING_PX;
        self.section_width_px = section_width_px;
        let _ = self
            .rectgrid
            .set_definition(IncrementFunction::Scale(section_width_px / X_COLS as f64), 0);
        let boxes: Vec<BBox<2>> = self.articles.iter().map(|(_, bx)| *bx).collect();
        let resolved = self.rectgrid.box_as_px(&boxes);
        let mut commands = vec![grid_background(section_width_px)];
        for ((n, bx), px_result) in self.articles.iter().zip(resolved) {
            let [Ok((base_x, size_x)), Ok((base_y, size_y))] = px_result else { continue };
            commands.push(translate_article(*n, base_x.get(), base_y.get()));
            if bx.has_size() {
                commands.push(size_px(*n, StyleProperty::Width, size_x.get()));
                commands.push(size_px(*n, StyleProperty::Height, size_y.get()));
            }
        }
        commands.extend(self.drop_zone_commands());
        (vec![], commands)
    }

    pub fn process_scroll(&mut self, _x: f64, _y: f64) -> (Vec<Event>, Vec<Command>) {
        (vec![], vec![])
    }

    pub fn process_visibility(&mut self, _state: VisibilityState) -> (Vec<Event>, Vec<Command>) {
        (vec![], vec![])
    }

    fn drag(&mut self, x: Px, y: Px) -> (Vec<Event>, Vec<Command>) {
        let pointer = [x, y];
        self.drag_pointer = [x.get(), y.get()];
        let Some(index) = self.drag_target else {
            return (vec![], vec![]);
        };
        let Some(entry) = self.articles.iter_mut().find(|(n, _)| *n == index) else {
            return (vec![], vec![]);
        };
        let bx = &mut entry.1;
        let drag_offset = [Px::new(self.drag_offset.0), Px::new(self.drag_offset.1)];
        if !bx.has_size() {
            let px = drag_translate(&self.rectgrid, pointer, drag_offset);
            return (vec![], vec![translate_article(index, px[0].get(), px[1].get())]);
        }
        if let Some(corner) = self.drag_corner {
            let Ok(new_bx) = drag_resize(&self.rectgrid, pointer, bx, corner) else {
                return (vec![], vec![]);
            };
            *bx = new_bx;
            let base_px = self.base_px(&new_bx);
            let size = self.size_px(&new_bx, base_px);
            let mut commands = vec![
                translate_article(index, base_px[0].get(), base_px[1].get()),
                size_px(index, StyleProperty::Width, size[0].get()),
                size_px(index, StyleProperty::Height, size[1].get()),
            ];
            commands.extend(self.check_drop_zone(base_px, new_bx.offset()));
            return (vec![], commands);
        }
        let offset = bx.offset();
        let px = drag_translate(&self.rectgrid, pointer, drag_offset);
        let mut commands = vec![translate_article(index, px[0].get(), px[1].get())];
        commands.extend(self.check_drop_zone(px, offset));
        (vec![], commands)
    }

    fn drag_end(&mut self) -> (Vec<Event>, Vec<Command>) {
        let mut commands = vec![];
        if let Some(index) = self.drag_target {
            if self.drag_corner.is_some() {
                commands.push(remove_cursor());
            }
            let pointer = [Px::new(self.drag_pointer[0]), Px::new(self.drag_pointer[1])];
            let drag_offset = [Px::new(self.drag_offset.0), Px::new(self.drag_offset.1)];
            let snapped = self.articles.iter_mut().find(|(n, _)| *n == index).and_then(|entry| {
                let bx = &mut entry.1;
                let snapped = if bx.has_size() {
                    if self.drag_corner.is_some() {
                        None
                    } else {
                        snap_bbox_to_unit(
                            &self.rectgrid,
                            pointer,
                            drag_offset,
                            bx,
                            Some([GridUnit::new(0.25), GridUnit::new(0.25)]),
                        )
                        .ok()
                    }
                } else {
                    snap_point_to_unit(
                        &self.rectgrid,
                        pointer,
                        drag_offset,
                        [GridUnit::new(0.25), GridUnit::new(0.25)],
                    )
                    .ok()
                };
                if let Some(new_bx) = snapped {
                    *bx = new_bx;
                }
                snapped
            });
            if let Some(new_bx) = snapped {
                let base_px = self.base_px(&new_bx);
                commands.push(translate_article(index, base_px[0].get(), base_px[1].get()));
            }
            if let Some(old_pos) = self.articles.iter().position(|(n, _)| *n == index) {
                let entry = self.articles.remove(old_pos);
                self.articles.push(entry);
                for (new_z, (n, _)) in self.articles[old_pos..].iter().enumerate() {
                    commands.push(z_index(*n, (old_pos + new_z) as i32));
                }
            }
        }
        commands.extend(self.clear_drop_zone());
        self.drag_target = None;
        self.drag_corner = None;
        (vec![], commands)
    }

    fn drag_cancel(&mut self) -> (Vec<Event>, Vec<Command>) {
        let mut commands = vec![];
        if let Some(index) = self.drag_target {
            if self.drag_corner.is_some() {
                commands.push(remove_cursor());
            }
            if let Some((_, bx)) = self.articles.iter().find(|(n, _)| *n == index) {
                let base_px = self.base_px(bx);
                commands.push(translate_article(index, base_px[0].get(), base_px[1].get()));
                if bx.has_size() {
                    let size = self.size_px(bx, base_px);
                    commands.push(size_px(index, StyleProperty::Width, size[0].get()));
                    commands.push(size_px(index, StyleProperty::Height, size[1].get()));
                }
            }
        }
        commands.extend(self.clear_drop_zone());
        self.drag_target = None;
        self.drag_corner = None;
        (vec![], commands)
    }

    fn base_px(&self, bx: &BBox<2>) -> [Px; 2] {
        from_fn(|d| self.rectgrid.unit_to_px(d, &bx.base()[d]).unwrap_or(Px::new(0.0)))
    }

    fn size_px(&self, bx: &BBox<2>, base_px: [Px; 2]) -> [Px; 2] {
        from_fn(|d| {
            self.rectgrid.unit_to_px(d, &(bx.base()[d] + bx.offset()[d])).unwrap_or(Px::new(0.0))
                - base_px[d]
        })
    }

    fn check_drop_zone(&mut self, base_local_px: [Px; 2], offset: [GridUnit; 2]) -> Vec<Command> {
        let base_global_px: [Px; 2] = from_fn(|d| base_local_px[d] + self.rectgrid.origin[d]);
        let base_unit: [GridUnit; 2] = from_fn(|d| {
            self.rectgrid.point_to_unit(base_global_px)[d].unwrap_or(GridUnit::new(f64::INFINITY))
        });
        let far_unit: [GridUnit; 2] = from_fn(|d| base_unit[d] + offset[d]);
        let nearest: [GridUnit; 2] = from_fn(|d| {
            GridUnit::new(
                self.drop_zone.center[d].get().max(base_unit[d].get()).min(far_unit[d].get()),
            )
        });
        let zone = Circle { center: self.drop_zone.center, radius: self.drop_zone.radius };
        let inside = as_on_circle(nearest, zone).is_ok_and(|g| g.signed_distance.get() <= 0.0);
        if inside == self.drop_zone_active {
            return vec![];
        }
        self.drop_zone_active = inside;
        highlight(DROP_ZONE_ARTICLE, inside)
    }

    fn clear_drop_zone(&mut self) -> Vec<Command> {
        if !self.drop_zone_active {
            return vec![];
        }
        self.drop_zone_active = false;
        highlight(DROP_ZONE_ARTICLE, false)
    }

    fn drop_zone_commands(&self) -> Vec<Command> {
        let radius = self.drop_zone.radius;
        let center_px: [Px; 2] = from_fn(|d| {
            self.rectgrid.unit_to_px(d, &self.drop_zone.center[d]).unwrap_or(Px::new(0.0))
        });
        let span_px: [Px; 2] = from_fn(|d| {
            let near = self
                .rectgrid
                .unit_to_px(d, &(self.drop_zone.center[d] - radius))
                .unwrap_or(Px::new(0.0));
            let far = self
                .rectgrid
                .unit_to_px(d, &(self.drop_zone.center[d] + radius))
                .unwrap_or(Px::new(0.0));
            far - near
        });
        let diameter = span_px[0].get().min(span_px[1].get());
        let base_px: [Px; 2] = from_fn(|d| center_px[d] - Px::new(diameter / 2.0));
        vec![
            translate_article(DROP_ZONE_ARTICLE, base_px[0].get(), base_px[1].get()),
            size_px(DROP_ZONE_ARTICLE, StyleProperty::Width, diameter),
            size_px(DROP_ZONE_ARTICLE, StyleProperty::Height, diameter),
        ]
    }
}

fn article(n: u32) -> Id {
    Id::new(&[(Tag::Main, None), (Tag::Article, Some(n))])
}

fn section() -> Id {
    Id::new(&[(Tag::Main, None)])
}

fn style(id: Id, property: StyleProperty, value: StyleValue) -> Command {
    Command::SetStyle { id, property, value }
}

fn size_px(n: u32, property: StyleProperty, px: f64) -> Command {
    style(article(n), property, StyleValue::Length((px as u32) as f32, Unit::Px))
}

fn z_index(n: u32, z: i32) -> Command {
    style(article(n), StyleProperty::ZIndex, StyleValue::Integer(z))
}

fn translate_article(n: u32, x: f64, y: f64) -> Command {
    style(
        article(n),
        StyleProperty::Translate,
        StyleValue::List(vec![
            StyleValue::Length(x as f32, Unit::Px),
            StyleValue::Length(y as f32, Unit::Px),
        ]),
    )
}

fn highlight(n: u32, on: bool) -> Vec<Command> {
    if on {
        vec![
            style(
                article(n),
                StyleProperty::Background,
                StyleValue::Text(String::from("var(--color-emphasis-ink)")),
            ),
            style(
                article(n),
                StyleProperty::Color,
                StyleValue::Text(String::from("var(--color-paper)")),
            ),
        ]
    } else {
        vec![
            Command::RemoveStyle { id: article(n), property: StyleProperty::Background },
            Command::RemoveStyle { id: article(n), property: StyleProperty::Color },
        ]
    }
}

fn remove_cursor() -> Command {
    Command::RemoveStyle { id: section(), property: StyleProperty::Cursor }
}

fn grid_background(section_width_px: f64) -> Command {
    let x_unit = section_width_px / X_COLS as f64;
    let y_unit_rem = Y_UNIT_REM;
    let background = format!(
        "repeating-linear-gradient(to right, var(--color-paper-mix) 0px, var(--color-paper-mix) 1px, transparent 1px, transparent {x_unit:.2}px), \
         repeating-linear-gradient(to bottom, var(--color-paper-mix) 0px, var(--color-paper-mix) 1px, transparent 1px, transparent {y_unit_rem}rem)"
    );
    style(section(), StyleProperty::Background, StyleValue::Text(background))
}

fn corner_cursor(corner: Option<Corner>) -> Option<Keyword> {
    match corner? {
        [Some(x_side), Some(y_side)] => {
            Some(if x_side == y_side { Keyword::NwseResize } else { Keyword::NeswResize })
        }
        [Some(_), None] => Some(Keyword::EwResize),
        [None, Some(_)] => Some(Keyword::NsResize),
        [None, None] => None,
    }
}

fn article_index_at(id: &Id) -> Option<u32> {
    id.0.iter().find_map(|seg| if matches!(seg.tag, Tag::Article) { seg.n } else { None })
}

#[cfg(test)]
mod tests {
    use alloc::{format, string::String, vec::Vec};

    use super::*;
    use crate::{
        app::App,
        event::EVENT_CANVAS,
        js_client::{dom, put_f32, put_str, put_u32},
    };

    const GOLDEN: &str = include_str!("../testdata/scenarios.txt");
    const ORIGIN: (f64, f64) = (10.0, 20.0);
    const POINTER_DOWN: u8 = 10;
    const POINTER_MOVE: u8 = 11;
    const POINTER_UP: u8 = 12;

    const SCENARIOS: [(&str, &[(&str, u32, f64, f64, f64)]); 4] = [
        (
            "A_move",
            &[
                ("down", 3, 500.0, 100.0, 0.0),
                ("move", 3, 520.0, 100.0, 16.0),
                ("move", 3, 560.0, 110.0, 32.0),
                ("move", 3, 620.0, 130.0, 48.0),
                ("move", 3, 700.0, 150.0, 64.0),
                ("up", 3, 700.0, 150.0, 80.0),
            ],
        ),
        (
            "B_corner_resize",
            &[
                ("down", 3, 806.0, 209.0, 0.0),
                ("move", 3, 830.0, 220.0, 16.0),
                ("move", 3, 900.0, 260.0, 32.0),
                ("move", 3, 950.0, 300.0, 48.0),
                ("up", 3, 950.0, 300.0, 64.0),
            ],
        ),
        (
            "C_drop_zone",
            &[
                ("down", 3, 500.0, 100.0, 0.0),
                ("move", 3, 510.0, 115.0, 16.0),
                ("move", 3, 520.0, 140.0, 32.0),
                ("move", 3, 530.0, 160.0, 48.0),
                ("move", 3, 530.0, 100.0, 64.0),
                ("move", 3, 530.0, 170.0, 80.0),
                ("up", 3, 530.0, 170.0, 96.0),
            ],
        ),
        (
            "D_fractional",
            &[
                ("down", 3, 503.37, 101.61, 0.0),
                ("move", 3, 518.71, 117.29, 16.5),
                ("move", 3, 561.13, 152.87, 33.25),
                ("move", 3, 611.93, 233.19, 49.5),
                ("move", 3, 646.41, 287.77, 66.0),
                ("up", 3, 646.41, 287.77, 82.5),
            ],
        ),
    ];

    fn id_string(id: &Id) -> String {
        id.0.iter().map(|s| format!("{:?}-{:?}", s.tag, s.n)).collect::<Vec<_>>().join("_")
    }

    fn describe(command: &Command) -> String {
        match command {
            Command::SetStyle { id, property, value } => match (property, value) {
                (StyleProperty::Translate, StyleValue::List(items)) => {
                    let [StyleValue::Length(x, Unit::Px), StyleValue::Length(y, Unit::Px)] =
                        items.as_slice()
                    else {
                        panic!("unexpected translate value");
                    };
                    format!("translate {} {:.3} {:.3}", id_string(id), x, y)
                }
                (StyleProperty::Width, StyleValue::Length(v, Unit::Px)) => {
                    format!("width {} {}", id_string(id), *v as u32)
                }
                (StyleProperty::Height, StyleValue::Length(v, Unit::Px)) => {
                    format!("height {} {}", id_string(id), *v as u32)
                }
                (StyleProperty::ZIndex, StyleValue::Integer(z)) => {
                    format!("z {} {}", id_string(id), z)
                }
                (StyleProperty::Background, StyleValue::Text(text)) => {
                    format!("background {} {}", id_string(id), text)
                }
                (StyleProperty::Color, StyleValue::Text(text)) => {
                    format!("color {} {}", id_string(id), text)
                }
                (StyleProperty::Cursor, StyleValue::Keyword(keyword)) => {
                    format!("cursor {} {:?}", id_string(id), keyword)
                }
                other => panic!("unexpected style command: {other:?}"),
            },
            Command::RemoveStyle { id, property: StyleProperty::Background } => {
                format!("background {} Unset", id_string(id))
            }
            Command::RemoveStyle { id, property: StyleProperty::Color } => {
                format!("color {} Unset", id_string(id))
            }
            Command::RemoveStyle { id, property: StyleProperty::Cursor } => {
                format!("cursor {} Unset", id_string(id))
            }
            Command::AddClass { id, value } => format!("addclass {} {:?}", id_string(id), value),
            Command::RemoveClass { id, value } => {
                format!("removeclass {} {:?}", id_string(id), value)
            }
            _ => String::from("other"),
        }
    }

    fn canvas_frame(event_type: u8, article: u32, x: f64, y: f64, time: f64) -> Vec<u8> {
        let mut frame = Vec::new();
        frame.push(EVENT_CANVAS);
        frame.push(event_type);
        dom::Id::new(&[(dom::Tag::Main, None), (dom::Tag::Article, Some(article))])
            .encode(&mut frame);
        frame.push(0);
        frame.push(0);
        put_str(&mut frame, "");
        put_f32(&mut frame, x as f32);
        put_f32(&mut frame, y as f32);
        put_f32(&mut frame, (x - ORIGIN.0) as f32);
        put_f32(&mut frame, (y - ORIGIN.1) as f32);
        frame.extend_from_slice(&time.to_le_bytes());
        put_u32(&mut frame, 1);
        frame
    }

    #[test]
    fn scenarios_match_the_recorded_command_sequences() {
        assert_eq!(EventType::decode_u8(POINTER_DOWN), EventType::PointerDown);
        assert_eq!(EventType::decode_u8(POINTER_MOVE), EventType::PointerMove);
        assert_eq!(EventType::decode_u8(POINTER_UP), EventType::PointerUp);

        let mut lines: Vec<String> = Vec::new();
        let mut handler = Handler::new(1000.0);
        for command in handler.initial_draw().1 {
            lines.push(format!("INIT {}", describe(&command)));
        }
        for command in handler.process_resize(1200.0, 800.0).1 {
            lines.push(format!("RESIZE {}", describe(&command)));
        }
        for (name, steps) in SCENARIOS {
            let mut app = App::new(false, Handler::new(1000.0));
            for (i, (kind, article, x, y, time)) in steps.iter().enumerate() {
                let event_type = match *kind {
                    "down" => POINTER_DOWN,
                    "move" => POINTER_MOVE,
                    _ => POINTER_UP,
                };
                app.clear();
                app.process(&canvas_frame(event_type, *article, *x, *y, *time));
                for command in app.commands() {
                    lines.push(format!("{name} {i} {kind} {}", describe(command)));
                }
            }
        }
        assert_eq!(lines.join("\n"), GOLDEN);
    }

    #[test]
    fn a_pointerdown_outside_every_article_starts_no_drag() {
        let mut app = App::new(false, Handler::new(1000.0));
        app.clear();
        app.process(&canvas_frame(POINTER_DOWN, 4, 700.0, 500.0, 0.0));
        assert!(app.commands().is_empty());
        app.process(&canvas_frame(POINTER_MOVE, 4, 740.0, 540.0, 16.0));
        app.process(&canvas_frame(POINTER_UP, 4, 740.0, 540.0, 32.0));
        assert!(app.commands().is_empty());
    }
}
