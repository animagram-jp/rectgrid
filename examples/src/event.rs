use alloc::vec::Vec;
use crate::js_client::{Command, Operation, EventType, Gesture, CanvasEvent, PointerState, dom::{Id, Tag}};
use crate::rectgrid::{
    Rectgrid, DefiningExpression, Region, Length,
    hit_test, corner_test, pointer_down_offset, drag_resize, drag_translate, snap_region_to_unit, snap_point_to_unit,
};

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
    articles:         Vec<(u32, Region<2>)>, // (article番号, Region)。末尾が最前面・judge優先
    drag_target:      Option<u32>,           // article番号
    drag_corner:      Option<[Option<bool>; 2]>, // article-3角ドラッグ: Some(true)=base側, Some(false)=offset側, None=軸ロック
    is_dragging:      bool,                  // Dragジェスチャが1回以上発火した
    rectgrid:         Rectgrid<2>,
    section_width_px: f64,
}

impl Handler {
    pub fn new(viewport_width_px: f64, section_origin_px: [f64; 2]) -> Self {
        let section_width_px = viewport_width_px - SECTION_PADDING_PX;
        let x_unit = section_width_px / X_COLS as f64;
        let y_unit = Y_UNIT_REM * REM_PX;
        Self {
            articles:         alloc::vec![
                                  (1, Region { base: [Length::Unit(0.0), Length::Unit(0.0)], offset: [Length::Unit(0.0), Length::Unit(0.0)] }),
                                  (2, Region { base: [Length::Unit(1.0), Length::Unit(0.0)], offset: [Length::Unit(0.0), Length::Unit(0.0)] }),
                                  (3, Region { base: [Length::Unit(2.0), Length::Unit(0.0)], offset: [Length::Unit(1.0), Length::Unit(3.0)] }),
                              ],
            drag_target:      None,
            drag_corner:      None,
            is_dragging:      false,
            rectgrid:            Rectgrid::new(
                                  section_origin_px,
                                  [DefiningExpression::Scale(x_unit), DefiningExpression::Scale(y_unit)],
                              ),
            section_width_px,
        }
    }
    pub fn close(&self) {}

    pub fn initial_draw(&mut self) -> (Vec<Event>, Vec<Command>) {
        let mut cmds: Vec<Command> = Vec::new();
        for (z, (n, region)) in self.articles.iter().enumerate() {
            let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(*n))]);
            if let Ok(base_px) = self.rectgrid.unit_point(&region.base) {
                cmds.push(translate_card(*n, base_px[0], base_px[1]));
            }
            cmds.push(Command::new(Operation::SetZIndex, &article.encode(), None, Some(&z.to_string())));
            // offsetが非ゼロの場合はサイズも設定(article 3など)
            if region.has_size() {
                if let Ok(offset_px) = self.rectgrid.unit_point(&region.offset) {
                    cmds.push(Command::new(Operation::SetWidth,  &article.encode(), None, Some(&format!("{:.2}", offset_px[0]))));
                    cmds.push(Command::new(Operation::SetHeight, &article.encode(), None, Some(&format!("{:.2}", offset_px[1]))));
                }
            }
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
                const EXTEND: Option<([f64; 2], [f64; 2])> = Some(([-0.05, -0.05], [0.05, 0.05]));
                const CORNER_THRESHOLD: f64 = 0.1;
                let coord = [event.x, event.y];
                // articles末尾から走査し、extendありjudgeでfirst hitを採用
                let regions: alloc::vec::Vec<Region<2>> = self.articles.iter()
                    .map(|(_, r)| *r)
                    .collect();
                let hit_i = hit_test(&mut self.rectgrid, coord, &regions, EXTEND);
                let hit_n = hit_i.map(|i| self.articles[i].0);
                // 角判定を最優先、次いでDOM hit、最後にRegion内部hit
                self.drag_corner = None;
                let corner: Option<[Option<bool>; 2]> = hit_i.and_then(|i| {
                    let (ratio, corner) = corner_test(&mut self.rectgrid, coord, &regions[i], EXTEND, CORNER_THRESHOLD);
                    crate::debug_log!("rectgrid ratio: {:?}, corner: {:?}", ratio, corner);
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
                    if let Some((_, region)) = self.articles.iter().find(|(n, _)| *n == idx) {
                        if self.drag_corner.is_none() {
                            if let Ok(offset) = pointer_down_offset(&mut self.rectgrid, coord, region) {
                                pointer_state.drag_offset = (offset[0], offset[1]);
                            }
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
                let pointer = [*x, *y];
                let Some(idx) = self.drag_target else { return (vec![], vec![]); };
                self.is_dragging = true;
                let Some(pos) = self.articles.iter_mut().find(|(n, _)| *n == idx) else {
                    return (vec![], vec![]);
                };
                let region = &mut pos.1;
                if region.has_size() {
                    if let Some(corner) = self.drag_corner {
                        // 角ハンドル: ポインタ絶対座標(viewport)からunit座標を求め、各軸のbase/offsetを動かす
                        let Ok((new_region, base_px, offset_px)) = drag_resize(&mut self.rectgrid, pointer, region, corner) else {
                            return (vec![], vec![]);
                        };
                        *region = new_region;
                        let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(idx))]);
                        let mut cmds = vec![translate_card(idx, base_px[0], base_px[1])];
                        cmds.push(Command::new(Operation::SetWidth,  &article.encode(), None, Some(&format!("{:.2}", offset_px[0]))));
                        cmds.push(Command::new(Operation::SetHeight, &article.encode(), None, Some(&format!("{:.2}", offset_px[1]))));
                        return (vec![], cmds);
                    }
                    let (new_region, px) = drag_translate(&mut self.rectgrid, pointer, [pointer_state.drag_offset.0, pointer_state.drag_offset.1], region);
                    *region = new_region;
                    (vec![], vec![translate_card(idx, px[0], px[1])])
                } else {
                    let (_, px) = drag_translate(&mut self.rectgrid, pointer, [pointer_state.drag_offset.0, pointer_state.drag_offset.1], region);
                    pointer_state.drag_px = (px[0], px[1]);
                    (vec![], vec![translate_card(idx, px[0], px[1])])
                }
            }
            Gesture::DragEnd => {
                let mut cmds = vec![];
                if let Some(idx) = self.drag_target {
                    if self.is_dragging {
                        if let Some(pos) = self.articles.iter_mut().find(|(n, _)| *n == idx) {
                            let region = &mut pos.1;
                            if region.has_size() {
                                if self.drag_corner.is_none() {
                                    // 移動ドラッグ: base を Px → Unit にスナップ
                                    if let Ok((new_region, base_px)) = snap_region_to_unit(&mut self.rectgrid, region, Some([0.25, 0.25])) {
                                        *region = new_region;
                                        cmds.push(translate_card(idx, base_px[0], base_px[1]));
                                    }
                                }
                                // 角ハンドルはDrag中に既にUnit確定済み
                            } else {
                                // カード(点Region): drag_pxからUnit座標へスナップ
                                let drag_px = [pointer_state.drag_px.0, pointer_state.drag_px.1];
                                if let Ok((new_region, base_px)) = snap_point_to_unit(&mut self.rectgrid, drag_px, [0.25, 0.25]) {
                                    *region = new_region;
                                    cmds.push(translate_card(idx, base_px[0], base_px[1]));
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
                self.rectgrid.set_expression(0, DefiningExpression::Scale(section_width_px / X_COLS as f64));
                self.rectgrid.set_origin(*section_origin_px);
                let regions: alloc::vec::Vec<Region<2>> = self.articles.iter().map(|(_, r)| *r).collect();
                let Ok(resolved) = self.rectgrid.update(regions) else { return (vec![], vec![]); };
                let mut cmds = vec![grid_background_cmd(section_width_px)];
                for ((n, region), (base_px, offset_px)) in self.articles.iter().zip(resolved.iter()) {
                    cmds.push(translate_card(*n, base_px[0], base_px[1]));
                    if region.has_size() {
                        let article = Id::new(&[(Tag::Section, None), (Tag::Article, Some(*n))]);
                        cmds.push(Command::new(Operation::SetWidth,  &article.encode(), None, Some(&format!("{:.2}", offset_px[0]))));
                        cmds.push(Command::new(Operation::SetHeight, &article.encode(), None, Some(&format!("{:.2}", offset_px[1]))));
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