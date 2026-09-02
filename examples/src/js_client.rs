use wasm_bindgen::JsValue;
use js_sys::Reflect;
use serde::{Serialize, Serializer, ser::SerializeMap};

// ============================================================
// send operation
// ============================================================

// operation番号はJS側 (init.js の execute) のswitch分岐と対応。
// 値を追加/変更する際は両方を揃えて更新する。
pub enum Command {
    SetText         { id: String, value: String },
    SetValue        { id: String, value: String },
    SetAttribute    { id: String, attribute: String, value: String },
    RemoveAttribute { id: String, attribute: String },
    AddClass        { id: String, value: String },
    RemoveClass     { id: String, value: String },
    SetWidth        { id: String, px: u32 },
    SetHeight       { id: String, px: u32 },
    SetZIndex       { id: String, z: i32 },
    SetBackground   { id: String, value: String },
    SetTranslate    { id: String, x: f64, y: f64 },
    SetCursor       { id: String, value: String },
    ShowModal       { id: String },
    CloseModal      { id: String },
    Focus           { id: String },
    JsFn            { id: String, name: String },
}

impl Serialize for Command {
    // opは元のu8のまま保持し、フィールドと同じ階層にフラットに並べる
    // (例: {"operation":11,"id":"...","x":1.0,"y":2.0})。
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        match self {
            Self::SetText { id, value } => {
                map.serialize_entry("operation", &1u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("value", value)?;
            }
            Self::SetValue { id, value } => {
                map.serialize_entry("operation", &2u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("value", value)?;
            }
            Self::SetAttribute { id, attribute, value } => {
                map.serialize_entry("operation", &3u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("attribute", attribute)?;
                map.serialize_entry("value", value)?;
            }
            Self::RemoveAttribute { id, attribute } => {
                map.serialize_entry("operation", &4u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("attribute", attribute)?;
            }
            Self::AddClass { id, value } => {
                map.serialize_entry("operation", &5u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("value", value)?;
            }
            Self::RemoveClass { id, value } => {
                map.serialize_entry("operation", &6u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("value", value)?;
            }
            Self::SetWidth { id, px } => {
                map.serialize_entry("operation", &7u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("px", px)?;
            }
            Self::SetHeight { id, px } => {
                map.serialize_entry("operation", &8u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("px", px)?;
            }
            Self::SetZIndex { id, z } => {
                map.serialize_entry("operation", &9u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("z", z)?;
            }
            Self::SetBackground { id, value } => {
                map.serialize_entry("operation", &10u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("value", value)?;
            }
            Self::SetTranslate { id, x, y } => {
                map.serialize_entry("operation", &11u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("x", x)?;
                map.serialize_entry("y", y)?;
            }
            Self::SetCursor { id, value } => {
                map.serialize_entry("operation", &12u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("value", value)?;
            }
            Self::ShowModal { id } => {
                map.serialize_entry("operation", &13u8)?;
                map.serialize_entry("id", id)?;
            }
            Self::CloseModal { id } => {
                map.serialize_entry("operation", &14u8)?;
                map.serialize_entry("id", id)?;
            }
            Self::Focus { id } => {
                map.serialize_entry("operation", &15u8)?;
                map.serialize_entry("id", id)?;
            }
            Self::JsFn { id, name } => {
                map.serialize_entry("operation", &16u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("name", name)?;
            }
        }
        map.end()
    }
}

// ============================================================
// receive (js value)
// ============================================================

/// js由来の文字列をstrとして取得
pub fn get_js_str(obj: &JsValue, key: &str) -> Option<String> {
    Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_string())
}

/// js由来の整数をu32として取得
pub fn get_js_u32(obj: &JsValue, key: &str) -> u32 {
    Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_f64())
        .and_then(|f| {
            if f >= 0.0 && f <= u32::MAX as f64 && f.fract() == 0.0 {
                Some(f as u32)
            } else {
                None
            }
        })
        .unwrap_or(0)
}

/// js由来の整数をi32として取得
pub fn get_js_i32(obj: &JsValue, key: &str) -> i32 {
    Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_f64())
        .and_then(|f| {
            if f >= i32::MIN as f64 && f <= i32::MAX as f64 && f.fract() == 0.0 {
                Some(f as i32)
            } else {
                None
            }
        })
        .unwrap_or(0)
}

/// js由来の小数をf64として取得
pub fn get_js_f64(obj: &JsValue, key: &str) -> Option<f64> {
    Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_f64())
        .and_then(|f| {
            if f.is_finite() {
                Some(f)
            } else {
                None
            }
        })
}

/// js由来のデータを構造体のまま取得
pub fn get_js_field(obj: &JsValue, key: &str) -> Option<JsValue> {
    Reflect::get(obj, &JsValue::from_str(key)).ok()
}

pub enum EventType {
    Submit,
    Click,
    ContextMenu,
    KeyDown,
    Input,
    Change,
    FocusIn,
    FocusOut,
    Resize,
    Scroll,
    Drop,
    PointerDown,
    PointerUp,
    PointerMove,
    PointerCancel,
    Other,
}

impl EventType {
    pub fn decode(event_type: &str) -> Self {
        match event_type {
            "submit"       => Self::Submit,
            "click"        => Self::Click,
            "contextmenu"  => Self::ContextMenu,
            "keydown"      => Self::KeyDown,
            "input"        => Self::Input,
            "change"       => Self::Change,
            "focusin"      => Self::FocusIn,
            "focusout"     => Self::FocusOut,
            "resize"       => Self::Resize,
            "scroll"       => Self::Scroll,
            "drop"         => Self::Drop,
            "pointerdown"  => Self::PointerDown,
            "pointerup"    => Self::PointerUp,
            "pointermove"  => Self::PointerMove,
            "pointercancel"=> Self::PointerCancel,
            _              => Self::Other,
        }
    }
}

pub enum KeyName {
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Enter,
    Escape,
    Tab,
    Backspace,
    Other,
}

impl KeyName {
    pub fn decode(key_name: &str) -> Self {
        match key_name {
            "ArrowUp"    => Self::ArrowUp,
            "ArrowDown"  => Self::ArrowDown,
            "ArrowLeft"  => Self::ArrowLeft,
            "ArrowRight" => Self::ArrowRight,
            "Enter"      => Self::Enter,
            "Escape"     => Self::Escape,
            "Tab"        => Self::Tab,
            "Backspace"  => Self::Backspace,
            _            => Self::Other,
        }
    }
}

// ============================================================
// device
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
    Touch,
    Mouse,
}

// pointer_coarse: window.matchMedia('(pointer: coarse)').matches
pub fn detect_device(pointer_coarse: bool) -> Device {
    if pointer_coarse { Device::Touch } else { Device::Mouse }
}

// ============================================================
// gesture: tap, long press, swipe (up,down,left,right), drag
// ============================================================
//
// 判定の根拠(閾値の出典・velocity計算窓・LongPressのタイマーレス実装など)は
// app repository の reference/Gesture.md を参照。ここでの実装はその
// Thresholds / PointerState / detect_gesture (+ detect_on_release /
// detect_on_move) を単一ポインタ版としてそのまま移植したもの。app repository
// はこれを複数指対応の TouchTracker で包んでいるが、rectgrid の examples は
// 単一ポインタで十分なため TouchTracker 自体は移植していない。

/// ジェスチャ判定の閾値。すべて CSS px と ms。
///
/// 装置ごとに閾値を分ける。指の接触面はマウスカーソルより広く、押下中の
/// 座標のブレも大きいため、タッチでは許容を広げる。
#[derive(Debug, Clone, Copy)]
pub struct Thresholds {
    /// 長押しと見なす最短時間 (ms)。
    pub long_press_ms:      f64,
    /// 長押し中に許容する座標のブレ (px)。これを超えたら長押しを取り消す。
    pub long_press_slop_px: f64,
    /// ドラッグ開始と見なす移動距離 (px)。
    pub drag_start_px:      f64,
    /// スワイプと見なす最短距離 (px)。
    pub swipe_min_px:       f64,
    /// スワイプと見なす最低速度 (px/ms)。
    pub swipe_min_velocity: f64,
    /// スワイプと見なす最長時間 (ms)。これを超えたらドラッグ扱い。
    pub swipe_max_ms:       f64,
    /// タップと見なす最長時間 (ms)。
    pub tap_max_ms:         f64,
    /// タップ中に許容する座標のブレ (px)。
    pub tap_slop_px:        f64,
}

impl Thresholds {
    pub const MOUSE: Self = Self {
        long_press_ms:      251.0,
        long_press_slop_px: 9.0,
        drag_start_px:      10.0,
        swipe_min_px:       50.0,
        swipe_min_velocity: 0.5,
        swipe_max_ms:       250.0,
        tap_max_ms:         250.0,
        tap_slop_px:        9.0,
    };

    /// タッチ向けの既定値。ブレ許容と開始距離をマウスより広く取る。
    pub const TOUCH: Self = Self {
        long_press_ms:      500.0,
        long_press_slop_px: 16.0,
        drag_start_px:      16.0,
        swipe_min_px:       50.0,
        swipe_min_velocity: 0.5,
        swipe_max_ms:       300.0,
        tap_max_ms:         300.0,
        tap_slop_px:        16.0,
    };

    #[must_use]
    pub const fn for_device(device: Device) -> Self {
        match device {
            Device::Mouse => Self::MOUSE,
            Device::Touch => Self::TOUCH,
        }
    }
}

impl Default for Thresholds {
    fn default() -> Self {
        Self::MOUSE
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    /// 単純なタップ / クリック。
    Tap,
    /// 長押し。押下したまま long_press_ms を超えた時点で1度だけ発火する。
    /// 動かさずに保持した場合、実際の発火は次の PointerMove / PointerUp
    /// まで遅延する ([`detect_gesture`] の doc を参照)。
    LongPress,
    SwipeUp,
    SwipeDown,
    SwipeLeft,
    SwipeRight,
    Drag { x: f64, y: f64 },
    /// ドラッグ終了 (pointerup)。スナップ処理はここで行う。
    DragEnd,
    /// ドラッグ中断 (pointercancel)。DragEndと同一視すると割り込み時に
    /// ドロップを取り消せなくなるため区別する。
    DragCancel,
}

/// PointerCancel でも座標・時刻・drag_offset / drag_px を保持し、is_down と
/// is_dragging のフラグだけを倒す。判定は detect_gesture がこの直後に
/// 行うため、そこで必要な値を判定前に消さない。
#[derive(Default, Clone, Copy)]
pub struct PointerState {
    is_down:    bool,
    start_x:    f64,
    start_y:    f64,
    current_x:  f64,
    current_y:  f64,
    start_time: f64,
    /// 直近の PointerMove の座標・時刻 (無ければ PointerDown のそれ)。
    /// swipe の速度を「離す直前の実際の動き」から計算するために持つ。
    last_move_x:    f64,
    last_move_y:    f64,
    last_move_time: f64,
    pub drag_offset: (f64, f64), // PointerDown時の (pointer_px - カード左上px)
    pub drag_px:     (f64, f64), // Drag中のカード左上px座標(一時)
    is_dragging:      bool, // Dragジェスチャが1回以上発火した
    /// 長押しを発火済みか。連続発火を防ぐラッチ。
    long_press_fired: bool,
    /// 直前の終了が PointerCancel だったか。
    cancelled:        bool,
}

impl PointerState {
    // payloadから必要な値を全て引数で受け取り、新しい状態を返す
    pub fn update(self, event_type: &EventType, x: f64, y: f64, time: f64) -> Self {
        match event_type {
            EventType::PointerDown => Self {
                is_down:          true,
                start_x:          x,
                start_y:          y,
                current_x:        x,
                current_y:        y,
                start_time:       time,
                last_move_x:      x,
                last_move_y:      y,
                last_move_time:   time,
                drag_offset:      (0.0, 0.0),
                drag_px:          (0.0, 0.0),
                is_dragging:      false,
                long_press_fired: false,
                cancelled:        false,
            },
            EventType::PointerMove => Self {
                current_x:      x,
                current_y:      y,
                last_move_x:    x,
                last_move_y:    y,
                last_move_time: time,
                ..self
            },
            EventType::PointerUp => Self {
                is_down: false, current_x: x, current_y: y, cancelled: false, ..self
            },
            EventType::PointerCancel => Self {
                is_down: false, current_x: x, current_y: y, cancelled: true, ..self
            },
            _ => self,
        }
    }

    /// 押下開始からの移動距離 (px)。
    fn distance(&self) -> f64 {
        let dx = self.current_x - self.start_x;
        let dy = self.current_y - self.start_y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// pointer 状態の遷移からジェスチャを認識する。
///
/// is_down == false でも、PointerUp / PointerCancel なら終了時ジェスチャ
/// (DragEnd / DragCancel / Swipe* / Tap / LongPress) の判定へ進む。
///
/// # 判定順
///
/// 1. 終了イベント (PointerUp / PointerCancel)
///    - ドラッグ中なら DragEnd / DragCancel
///    - 速い + 遠い + 短い なら Swipe*
///    - 長押し発火済みなら何も返さない (発火済みのため)
///    - 保持時間超過 + ブレ小 なら LongPress (動かないまま離した場合)
///    - 短い + ブレ小 なら Tap
/// 2. 移動イベント (PointerMove)
///    - 保持時間超過 + ブレ小 かつ未発火なら LongPress
///      (動かないまま保持時間を超え、その後わずかに動いた場合)
///    - 既にドラッグ中、または swipe 条件を満たさない移動なら Drag
///
/// LongPress はタイマーを持たない。動かないまま保持され続けた場合は
/// 次の PointerMove / PointerUp まで発火が遅延する。
pub fn detect_gesture(
    state: &mut PointerState,
    prev_state: &PointerState,
    event_type: &EventType,
    current_time: f64,
    thresholds: &Thresholds,
) -> Option<Gesture> {
    match event_type {
        EventType::PointerUp | EventType::PointerCancel => {
            detect_on_release(state, prev_state, current_time, thresholds)
        }
        EventType::PointerMove => detect_on_move(state, current_time, thresholds),
        _ => None,
    }
}

/// 終了イベントの判定。
fn detect_on_release(
    state: &mut PointerState,
    prev_state: &PointerState,
    current_time: f64,
    thresholds: &Thresholds,
) -> Option<Gesture> {
    // ドラッグしていたなら、終了種別を返して確定させる。
    if prev_state.is_dragging {
        state.is_dragging = false;
        return Some(if state.cancelled { Gesture::DragCancel } else { Gesture::DragEnd });
    }

    // キャンセルはここで打ち切る。タップにもスワイプにもしない。
    if state.cancelled {
        return None;
    }

    let dt = current_time - state.start_time;
    if dt <= 0.0 {
        return None;
    }
    let distance = state.distance();

    // swipe: 速い + 遠い + 短い。
    //
    // 速度は start からの平均ではなく、直近の PointerMove から current
    // までの区間で計算する。平均だと、序盤に大きく動いた後指を止めたまま
    // 保持してから離した場合でも、距離が大きいままなので速度が閾値を超え
    // 続け、実際には止まっていたのに swipe と誤判定されうる。直近区間で
    // 計算すれば、動きが止まっていた分だけ move_dt が伸びて速度は自然に
    // 下がる。PointerMove が一度も無ければ last_move_* は start と同じ
    // なので、平均と一致する。
    let move_dt = current_time - state.last_move_time;
    let velocity = if move_dt > 0.0 {
        let mdx = state.current_x - state.last_move_x;
        let mdy = state.current_y - state.last_move_y;
        (mdx * mdx + mdy * mdy).sqrt() / move_dt
    } else {
        0.0
    };
    if velocity > thresholds.swipe_min_velocity
        && distance > thresholds.swipe_min_px
        && dt < thresholds.swipe_max_ms
    {
        let dx = state.current_x - state.start_x;
        let dy = state.current_y - state.start_y;
        return Some(if dx.abs() > dy.abs() {
            if dx > 0.0 { Gesture::SwipeRight } else { Gesture::SwipeLeft }
        } else if dy > 0.0 {
            Gesture::SwipeDown
        } else {
            Gesture::SwipeUp
        });
    }

    // 長押しは detect_on_move で既に発火済み。ここで tap を重ねて返さない。
    if state.long_press_fired {
        return None;
    }

    // long press: 指を動かさないまま保持時間を超えて離した場合、
    // PointerMove が一度も来ていないため detect_on_move 側では拾えて
    // いない。ここが最後の判定機会になる。
    if dt > thresholds.long_press_ms && distance < thresholds.long_press_slop_px {
        return Some(Gesture::LongPress);
    }

    // tap: 短い + ブレ小。
    if dt < thresholds.tap_max_ms && distance < thresholds.tap_slop_px {
        return Some(Gesture::Tap);
    }

    None
}

/// 移動イベントの判定。
fn detect_on_move(state: &mut PointerState, current_time: f64, thresholds: &Thresholds) -> Option<Gesture> {
    if !state.is_down {
        return None;
    }

    let distance = state.distance();

    // long press: 動いていない状態で保持時間を超えたら、この PointerMove
    // で確定させる。指を完全に静止させたままなら次の PointerUp で
    // detect_on_release が拾う。
    if !state.long_press_fired
        && !state.is_dragging
        && distance < thresholds.long_press_slop_px
        && current_time - state.start_time > thresholds.long_press_ms
    {
        state.long_press_fired = true;
        return Some(Gesture::LongPress);
    }

    if distance <= thresholds.drag_start_px {
        return None;
    }

    // 既にドラッグ中なら継続する。
    if state.is_dragging {
        return Some(Gesture::Drag { x: state.current_x, y: state.current_y });
    }

    // まだドラッグに入っていない場合、swipe になりうる動きは譲る。
    // (先に Drag へ倒すと後続の PointerUp で swipe が判定不能になる —
    // 「drag が swipe を横取りする」バグの原因だった。)
    let dt = current_time - state.start_time;
    if dt > 0.0 && dt < thresholds.swipe_max_ms {
        let velocity = distance / dt;
        if velocity > thresholds.swipe_min_velocity && distance > thresholds.swipe_min_px {
            // まだ確定させない。PointerUp で swipe か drag かを決める。
            return None;
        }
    }

    state.is_dragging = true;
    Some(Gesture::Drag { x: state.current_x, y: state.current_y })
}

// ============================================================
// dom (rust item <=> element id)
// ============================================================
//
// id規則:
//   "_" = 親子セグメント区切り  例: main_div_section-1
//   "-N" = 同タグ内の連番       例: span-3, th-2
//   連番なし = その階層に1つだけ 例: thead_tr, legend_h5
//
// dom::Id::encode()  -> "seg1_seg2_seg-N_..."
// dom::Id::decode()  -> Vec<dom::Segment> のパース

pub mod dom {
    use core::{option::Option::{self, Some, None}, result::Result::Ok, cmp::PartialEq, clone::Clone};
    use alloc::{vec::Vec, string::String, format};

    #[derive(Debug, Clone, PartialEq)]
    pub enum Tag {
        Body,
        Head,
        Header,
        H1, H2, H3,
        Ul, Li,
        Button,
        Main,
        Section,
        Span,
        Dl, Dt, Dd,
        Ol,
        P,
        Textarea,
        Drawer, // <dialog id="*drawer*">
        Modal,  // <dialog id="*modal*">
        Form,
        Input,
        Fieldset,
        Table, Thead, Tbody, Tr, Th, Td,
        Select,
        Footer,
        Output, Article,
        Other,
    }

    impl Tag {
        pub fn decode(s: &str) -> Self {
            match s {
                "body"     => Self::Body,
                "head"     => Self::Head,
                "header"   => Self::Header,
                "h1"       => Self::H1,
                "h2"       => Self::H2,
                "h3"       => Self::H3,
                "ul"       => Self::Ul,
                "li"       => Self::Li,
                "button"   => Self::Button,
                "main"     => Self::Main,
                "section"  => Self::Section,
                "span"     => Self::Span,
                "dl"       => Self::Dl,
                "dt"       => Self::Dt,
                "dd"       => Self::Dd,
                "ol"       => Self::Ol,
                "p"        => Self::P,
                "textarea" => Self::Textarea,
                "drawer"   => Self::Drawer,
                "modal"    => Self::Modal,
                "form"     => Self::Form,
                "input"    => Self::Input,
                "fieldset" => Self::Fieldset,
                "table"    => Self::Table,
                "thead"    => Self::Thead,
                "tbody"    => Self::Tbody,
                "tr"       => Self::Tr,
                "th"       => Self::Th,
                "td"       => Self::Td,
                "select"   => Self::Select,
                "footer"   => Self::Footer,
                "output"   => Self::Output,
                "article"  => Self::Article,
                _          => Self::Other,
            }
        }

        pub fn encode(&self) -> &'static str {
            match self {
                Self::Body     => "body",
                Self::Head     => "head",
                Self::Header   => "header",
                Self::H1       => "h1",
                Self::H2       => "h2",
                Self::H3       => "h3",
                Self::Ul       => "ul",
                Self::Li       => "li",
                Self::Button   => "button",
                Self::Main     => "main",
                Self::Section  => "section",
                Self::Span     => "span",
                Self::Dl       => "dl",
                Self::Dt       => "dt",
                Self::Dd       => "dd",
                Self::Ol       => "ol",
                Self::P        => "p",
                Self::Textarea => "textarea",
                Self::Drawer   => "drawer",
                Self::Modal    => "modal",
                Self::Form     => "form",
                Self::Input    => "input",
                Self::Fieldset => "fieldset",
                Self::Table    => "table",
                Self::Thead    => "thead",
                Self::Tbody    => "tbody",
                Self::Tr       => "tr",
                Self::Th       => "th",
                Self::Td       => "td",
                Self::Select   => "select",
                Self::Footer   => "footer",
                Self::Output   => "output",
                Self::Article  => "article",
                Self::Other    => "",
            }
        }
    }

    // セグメント1つ: タグ + オプション連番
    #[derive(Debug, Clone, PartialEq)]
    pub struct Segment {
        pub tag: Tag,
        pub n:   Option<u32>,
    }

    impl Segment {
        pub fn new(tag: Tag) -> Self { Self { tag, n: None } }
        pub fn numbered(tag: Tag, n: u32) -> Self { Self { tag, n: Some(n) } }

        pub fn decode(s: &str) -> Self {
            if let Some(pos) = s.rfind('-') {
                let (tag, num) = s.split_at(pos);
                if let Ok(n) = num[1..].parse::<u32>() {
                    return Self::numbered(Tag::decode(tag), n);
                }
            }
            Self::new(Tag::decode(s))
        }

        pub fn encode(&self) -> String {
            match self.n {
                Some(n) => format!("{}-{}", self.tag.encode(), n),
                None    => self.tag.encode().to_string(),
            }
        }
    }

    // id全体: セグメントのリスト
    #[derive(Debug, Clone, PartialEq)]
    pub struct Id(pub Vec<Segment>);

    impl Id {
        pub fn new(segs: &[(Tag, Option<u32>)]) -> Self {
            Self(segs.iter().map(|(tag, n)| Segment { tag: tag.clone(), n: *n }).collect())
        }

        pub fn decode(id: &str) -> Self {
            Self(id.split('_').map(Segment::decode).collect())
        }

        pub fn encode(&self) -> String {
            self.0.iter()
                .map(Segment::encode)
                .collect::<Vec<_>>()
                .join("_")
        }

        pub fn last_tag(&self) -> Option<&Tag> {
            self.0.last().map(|s| &s.tag)
        }
    }
}

// ============================================================
// canvas event
// ============================================================

pub struct CanvasEvent {
    pub event_type:       EventType,
    pub id:                dom::Id,
    pub key:                KeyName,
    pub value:               String,
    pub x:                      f64,
    pub y:                      f64,
    pub time:                   f64,
    pub section_origin_x:       f64,
    pub section_origin_y:       f64,
}

impl CanvasEvent {
    pub fn decode(payload: &wasm_bindgen::JsValue) -> Self {
        let event_type      = get_js_str(payload, "event_type").as_deref().map(EventType::decode).unwrap_or(EventType::Other);
        let id               = get_js_str(payload, "target_id").as_deref().map(dom::Id::decode).unwrap_or_else(|| dom::Id(vec![]));
        let key              = get_js_str(payload, "key").as_deref().map(KeyName::decode).unwrap_or(KeyName::Other);
        let value            = get_js_str(payload, "value").unwrap_or_default();
        let x                = get_js_f64(payload, "x").unwrap_or(0.0);
        let y                = get_js_f64(payload, "y").unwrap_or(0.0);
        let time             = get_js_f64(payload, "time").unwrap_or(0.0);
        let section_origin_x = get_js_f64(payload, "section_origin_x").unwrap_or(0.0);
        let section_origin_y = get_js_f64(payload, "section_origin_y").unwrap_or(0.0);
        Self { event_type, id, key, value, x, y, time, section_origin_x, section_origin_y }
    }
}