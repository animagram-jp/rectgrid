use alloc::vec::Vec;
use core::array::from_fn;
use crate::js_client::{Command, Operation, EventType, Gesture, CanvasEvent, PointerState, dom::{Id, Tag}};
use rectgrid::{RectGrid, IncrementFunction, BBox, Px, Unit, corner_test, drag_resize, drag_translate, snap_region_to_unit, snap_point_to_unit};

// ============================================================
// Event
// ============================================================

pub enum RectgridEvent {
    Resize { width_px: f64, section_origin_px: [f64; 2] },
}

pub enum Event {
    Ready,
    Canvas(CanvasEvent),
    Gesture(Gesture),
    Rectgrid(RectgridEvent),
}

// ============================================================
// Grid constants
// ============================================================

const X_COLS:             u32 = 5;    // x方向の分割数
const Y_UNIT_REM:         f64 = 4.0;  // y方向1マスのrem数
const REM_PX:             f64 = 16.0; // 1rem = 16px 基準
const SECTION_PADDING_PX: f64 = 0.0;  // viewport左端から#section内側までの水平余白合計

// ============================================================
// Handler
// ============================================================

pub struct Handler {
    articles:         Vec<(u32, BBox<2>)>,    // (article番号, BBox)。末尾が最前面・hit優先
    drag_target:      Option<u32>,             // article番号
    drag_corner:      Option<[Option<bool>; 2]>, // article-3角ドラッグ: Some(true)=base側, Some(false)=offset側, None=軸ロック
    is_dragging:      bool,                    // Dragジェスチャが1回以上発火した
    drag_pointer:     [f64; 2],               // Drag中の最終raw viewport座標(DragEndのsnap_*呼び出し用)
    rectgrid:         RectGrid<2>,
    section_width_px: f64,
}

impl Handler {
    pub fn new(viewport_width_px: f64, section_origin_px: [f64; 2]) -> Self {
        let section_width_px = viewport_width_px - SECTION_PADDING_PX;
        let x_unit = section_width_px / X_COLS as f64;
        let y_unit = Y_UNIT_REM * REM_PX;
        Self {
            articles:     alloc::vec![
                              (1, BBox { base: [Unit::new(0.0), Unit::new(0.0)], offset: [Unit::new(0.0), Unit::new(0.0)] }),
                              (2, BBox { base: [Unit::new(1.0), Unit::new(0.0)], offset: [Unit::new(0.0), Unit::new(0.0)] }),
                              (3, BBox { base: [Unit::new(2.0), Unit::new(0.0)], offset: [Unit::new(1.0), Unit::new(3.0)] }),
                          ],
            drag_target:  None,
            drag_corner:  None,
            is_dragging:  false,
            drag_pointer: [0.0; 2],
            rectgrid:     RectGrid::new(
                              [Px::new(section_origin_px[0]), Px::new(section_origin_px[1])],
                              [IncrementFunction::Scale(x_unit), IncrementFunction::Scale(y_unit)],
                          ).unwrap(),
            section_width_px,
        }
    }
    pub fn close(&self) {}

    pub fn initial_draw(&mut self) -> (Vec<Event>, Vec<Command>) {
        let mut cmds: Vec<Command> = Vec::new();
        let boxes: Vec<BBox<2>> = self.articles.iter().map(|(_, bx)| *bx).collect();
        let resolved = self.rectgrid.box_as_px(&boxes);
        for (z, ((n, bx), px_result)) in self.articles.iter().zip(resolved).enumerate() {
            let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(*n))]);
            if let Ok((base_px, offset_px)) = px_result {
                cmds.push(translate_card(*n, base_px[0].get(), base_px[1].get()));
                if bx.has_size() {
                    cmds.push(Command::new(Operation::SetWidth,  &article.encode(), None, Some(&format!("{:.2}", offset_px[0].get()))));
                    cmds.push(Command::new(Operation::SetHeight, &article.encode(), None, Some(&format!("{:.2}", offset_px[1].get()))));
                }
            }
            cmds.push(Command::new(Operation::SetZIndex, &article.encode(), None, Some(&z.to_string())));
        }
        cmds.push(grid_background_cmd(self.section_width_px));
        (vec![], cmds)
    }

    pub fn process(&mut self, event: &CanvasEvent, pointer_state: &mut PointerState) -> (Vec<Event>, Vec<Command>) {
        match &event.event_type {
            EventType::Resize => (vec![Event::Rectgrid(RectgridEvent::Resize {
                width_px:          event.x,
                section_origin_px: [event.section_origin_x, event.section_origin_y],
            })], vec![]),
            EventType::PointerDown => {
                let extend = Some(([Unit::new(-0.05), Unit::new(-0.05)], [Unit::new(0.05), Unit::new(0.05)]));
                const CORNER_THRESHOLD: f64 = 0.1;
                let point = [Px::new(event.x), Px::new(event.y)];
                // articles末尾から走査し、extendありhit_testでfirst hitを採用
                let boxes: Vec<BBox<2>> = self.articles.iter().map(|(_, bx)| *bx).collect();
                let hit_i = self.rectgrid.hit_test(point, &boxes, extend);
                let hit_n = hit_i.map(|i| self.articles[i].0);
                // 角判定を最優先、次いでDOM hit、最後にBBox内部hit
                self.drag_corner = None;
                let corner: Option<[Option<bool>; 2]> = hit_i.and_then(|i| {
                    let (ratio, corner) = corner_test(&self.rectgrid, point, &boxes[i], CORNER_THRESHOLD);
                    crate::debug_log!("rectgrid ratio: {:?}, corner: {:?}", ratio.map(|r| r.map(|p| p.get())), corner);
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
                            let base_px: [Px; 2] = from_fn(|d| self.rectgrid.unit_to_px(d, &bx.base[d]).unwrap_or(Px::new(0.0)));
                            let offset = self.rectgrid.offset(point, base_px);
                            pointer_state.drag_offset = (offset[0].get(), offset[1].get());
                        }
                    }
                    // drag開始時点で対象を最前面z-indexに
                    let top_z = self.articles.len();
                    let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(idx))]);
                    cmds.push(Command::new(Operation::SetZIndex, &article.encode(), None, Some(&top_z.to_string())));
                }
                self.drag_target = target;
                self.is_dragging = false;
                (vec![], cmds)
            }
            EventType::KeyDown  => todo!("keydown"),
            EventType::Input    => todo!("input"),
            EventType::Change   => todo!("change"),
            EventType::FocusOut => todo!("focusout"),
            EventType::Submit   => todo!("submit"),
            _                   => (vec![], vec![]),
        }
    }

    pub fn process_gesture(&mut self, gesture: &Gesture, pointer_state: &mut PointerState) -> (Vec<Event>, Vec<Command>) {
        match gesture {
            Gesture::Drag { x, y } => {
                let pointer = [Px::new(*x), Px::new(*y)];
                self.drag_pointer = [*x, *y];
                let Some(idx) = self.drag_target else { return (vec![], vec![]); };
                self.is_dragging = true;
                let Some(pos) = self.articles.iter_mut().find(|(n, _)| *n == idx) else {
                    return (vec![], vec![]);
                };
                let bx = &mut pos.1;
                let drag_offset = [Px::new(pointer_state.drag_offset.0), Px::new(pointer_state.drag_offset.1)];
                if bx.has_size() {
                    if let Some(corner) = self.drag_corner {
                        // 角ハンドル: ポインタ絶対座標(viewport)からunit座標を求め、各軸のbase/offsetを動かす
                        let Ok(new_bx) = drag_resize(&self.rectgrid, pointer, bx, corner) else {
                            return (vec![], vec![]);
                        };
                        *bx = new_bx;
                        let base_px: [Px; 2] = from_fn(|d| self.rectgrid.unit_to_px(d, &new_bx.base[d]).unwrap_or(Px::new(0.0)));
                        let size_px: [Px; 2] = from_fn(|d| {
                            self.rectgrid.unit_to_px(d, &(new_bx.base[d] + new_bx.offset[d])).unwrap_or(Px::new(0.0)) - base_px[d]
                        });
                        let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(idx))]);
                        let mut cmds = vec![translate_card(idx, base_px[0].get(), base_px[1].get())];
                        cmds.push(Command::new(Operation::SetWidth,  &article.encode(), None, Some(&format!("{:.2}", size_px[0].get()))));
                        cmds.push(Command::new(Operation::SetHeight, &article.encode(), None, Some(&format!("{:.2}", size_px[1].get()))));
                        return (vec![], cmds);
                    }
                    // 移動ドラッグ: BBoxはUnit座標のまま維持し、DragEndでスナップ
                    let px = drag_translate(&self.rectgrid, pointer, drag_offset);
                    (vec![], vec![translate_card(idx, px[0].get(), px[1].get())])
                } else {
                    // 点BBox: 移動中の描画位置のみ更新
                    let px = drag_translate(&self.rectgrid, pointer, drag_offset);
                    pointer_state.drag_px = (px[0].get(), px[1].get());
                    (vec![], vec![translate_card(idx, px[0].get(), px[1].get())])
                }
            }
            Gesture::DragEnd => {
                let mut cmds = vec![];
                if let Some(idx) = self.drag_target {
                    if self.is_dragging {
                        if let Some(pos) = self.articles.iter_mut().find(|(n, _)| *n == idx) {
                            let bx = &mut pos.1;
                            let drag_pointer = [Px::new(self.drag_pointer[0]), Px::new(self.drag_pointer[1])];
                            let drag_offset  = [Px::new(pointer_state.drag_offset.0), Px::new(pointer_state.drag_offset.1)];
                            if bx.has_size() {
                                if self.drag_corner.is_none() {
                                    // 移動ドラッグ: base を Unit格子にスナップ
                                    if let Ok(new_bx) = snap_region_to_unit(&self.rectgrid, drag_pointer, drag_offset, bx, Some([Unit::new(0.25), Unit::new(0.25)])) {
                                        *bx = new_bx;
                                        let base_px: [Px; 2] = from_fn(|d| self.rectgrid.unit_to_px(d, &new_bx.base[d]).unwrap_or(Px::new(0.0)));
                                        cmds.push(translate_card(idx, base_px[0].get(), base_px[1].get()));
                                    }
                                }
                                // 角ハンドルはDrag中に既にUnit確定済み
                            } else {
                                // 点BBox: drag_pointerからUnit格子にスナップ
                                if let Ok(new_bx) = snap_point_to_unit(&self.rectgrid, drag_pointer, drag_offset, [Unit::new(0.25), Unit::new(0.25)]) {
                                    *bx = new_bx;
                                    let base_px: [Px; 2] = from_fn(|d| self.rectgrid.unit_to_px(d, &new_bx.base[d]).unwrap_or(Px::new(0.0)));
                                    cmds.push(translate_card(idx, base_px[0].get(), base_px[1].get()));
                                }
                            }
                        }
                        // 動かしたarticleを末尾(最前面)へ移動し、全articleのz-indexを再割り当て
                        if let Some(pos) = self.articles.iter().position(|(n, _)| *n == idx) {
                            let entry = self.articles.remove(pos);
                            self.articles.push(entry);
                        }
                        for (z, (n, _)) in self.articles.iter().enumerate() {
                            let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(*n))]);
                            cmds.push(Command::new(Operation::SetZIndex, &article.encode(), None, Some(&z.to_string())));
                        }
                    }
                }
                self.drag_target = None;
                self.drag_corner = None;
                self.is_dragging = false;
                (vec![], cmds)
            }
            _ => (vec![], vec![]),
        }
    }

    pub fn process_rectgrid(&mut self, event: &RectgridEvent) -> (Vec<Event>, Vec<Command>) {
        match event {
            RectgridEvent::Resize { width_px, section_origin_px } => {
                let section_width_px = width_px - SECTION_PADDING_PX;
                self.section_width_px = section_width_px;
                let _ = self.rectgrid.set_definition(IncrementFunction::Scale(section_width_px / X_COLS as f64), 0);
                self.rectgrid.origin = [Px::new(section_origin_px[0]), Px::new(section_origin_px[1])];
                let boxes: Vec<BBox<2>> = self.articles.iter().map(|(_, bx)| *bx).collect();
                let resolved = self.rectgrid.box_as_px(&boxes);
                let mut cmds = vec![grid_background_cmd(section_width_px)];
                for ((n, bx), px_result) in self.articles.iter().zip(resolved) {
                    let Ok((base_px, offset_px)) = px_result else { continue };
                    cmds.push(translate_card(*n, base_px[0].get(), base_px[1].get()));
                    if bx.has_size() {
                        let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(*n))]);
                        cmds.push(Command::new(Operation::SetWidth,  &article.encode(), None, Some(&format!("{:.2}", offset_px[0].get()))));
                        cmds.push(Command::new(Operation::SetHeight, &article.encode(), None, Some(&format!("{:.2}", offset_px[1].get()))));
                    }
                }
                (vec![], cmds)
            }
        }
    }
}

// ============================================================
// grid helpers
// ============================================================

fn grid_background_cmd(section_width_px: f64) -> Command {
    let x_unit = section_width_px / X_COLS as f64;
    let y_unit_rem = Y_UNIT_REM;
    // vertical lines every x_unit px, horizontal lines every y_unit_rem rem
    // 線色: rgba(0,0,0,0.08) の1px線
    let bg = format!(
        "repeating-linear-gradient(to right, rgba(0,0,0,0.08) 0px, rgba(0,0,0,0.08) 1px, transparent 1px, transparent {x_unit:.2}px), \
         repeating-linear-gradient(to bottom, rgba(0,0,0,0.08) 0px, rgba(0,0,0,0.08) 1px, transparent 1px, transparent {y_unit_rem}rem)"
    );
    let section = Id::new(&[(Tag::Section, None)]);
    Command::new(Operation::SetBackground, &section.encode(), None, Some(&bg))
}

// ============================================================
// drag helpers
// ============================================================

fn translate_card(n: u32, x: f64, y: f64) -> Command {
    let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(n))]);
    Command::new(
        Operation::SetTranslate,
        &article.encode(),
        Some(&(x as i64).to_string()),
        Some(&(y as i64).to_string()),
    )
}

/// イベントターゲットIDからarticle番号を抽出する
/// section_article-N_... の形式で、article セグメントの番号を返す
fn article_index_at(id: &Id) -> Option<u32> {
    id.0.iter().find_map(|seg| {
        if matches!(seg.tag, Tag::Article) { seg.n } else { None }
    })
}
