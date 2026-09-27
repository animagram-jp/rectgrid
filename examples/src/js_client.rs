use js_sys::Reflect;
use serde::{Serialize, Serializer, ser::SerializeMap};
use wasm_bindgen::JsValue;

// ============================================================
// send operation
// ============================================================

// operation番号はJS側 (init.js の execute) のswitch分岐と対応。
// 値を追加/変更する際は両方を揃えて更新する。
pub enum Command {
    SetText {
        id:    String,
        value: String,
    },
    SetValue {
        id:    String,
        value: String,
    },
    SetAttribute {
        id:        String,
        attribute: Attribute,
        value:     String,
    },
    RemoveAttribute {
        id:        String,
        attribute: Attribute,
    },
    AddClass {
        id:    String,
        value: ClassName,
    },
    RemoveClass {
        id:    String,
        value: ClassName,
    },
    SetWidth {
        id: String,
        px: u32,
    },
    SetHeight {
        id: String,
        px: u32,
    },
    SetZIndex {
        id: String,
        z:  i32,
    },
    SetBackground {
        id:    String,
        value: String,
    },
    SetTranslate {
        id: String,
        x:  f64,
        y:  f64,
    },
    SetCursor {
        id:    String,
        value: CursorValue,
    },
    ShowModal {
        id: String,
    },
    CloseModal {
        id: String,
    },
    Focus {
        id: String,
    },
    JsFn {
        id:   String,
        name: FnName,
    },
    /// 異常をJS側へ報告する。init.jsのexecuteがconsole.errorへ出力する。
    Error {
        message: String,
    },
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
                map.serialize_entry("attribute", &attribute.encode_u16())?;
                map.serialize_entry("value", value)?;
            }
            Self::RemoveAttribute { id, attribute } => {
                map.serialize_entry("operation", &4u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("attribute", &attribute.encode_u16())?;
            }
            Self::AddClass { id, value } => {
                map.serialize_entry("operation", &5u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("value", &value.encode_u16())?;
            }
            Self::RemoveClass { id, value } => {
                map.serialize_entry("operation", &6u8)?;
                map.serialize_entry("id", id)?;
                map.serialize_entry("value", &value.encode_u16())?;
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
                map.serialize_entry("value", &value.encode_u16())?;
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
                map.serialize_entry("name", &name.encode_u16())?;
            }
            Self::Error { message } => {
                map.serialize_entry("operation", &18u8)?;
                map.serialize_entry("message", message)?;
            }
        }
        map.end()
    }
}

// === static string index (init.js の各テーブルと index を揃える) ===
//
// app repositoryのjs_client.rsと同じ命名・同じindex。JSON上はu16の
// インデックスとしてそのまま送る (Serialize impl参照)。バイナリ方式へ
// 切り替える際も、init.js側のテーブル参照はそのまま使い回せる。

/// `init.js::ATTRIBUTES` の index。HTML属性名。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Attribute {
    Disabled,
    Hidden,
}

impl Attribute {
    fn encode_u16(self) -> u16 {
        self as u16
    }
}

/// `init.js::CLASS_NAMES` の index。CSSクラス名。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClassName {
    Hide,
    Show,
    Hidden,
}

impl ClassName {
    fn encode_u16(self) -> u16 {
        self as u16
    }
}

/// `init.js::CURSOR_VALUES` の index。CSS `cursor` の値。
///
/// `Default` / `Grab` はapp repositoryと共通のindex(0/1)。それ以降の
/// resize系カーソルはrectgrid examples固有の追加(角/辺ドラッグ用)。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CursorValue {
    Default,
    Grab,
    /// インラインstyleを外し、CSSの既定値に戻す (`el.style.cursor = ""`)。
    /// `Default`(`"default"`を明示指定)とは異なる。
    Unset,
    NwseResize,
    NeswResize,
    EwResize,
    NsResize,
}

impl CursorValue {
    fn encode_u16(self) -> u16 {
        self as u16
    }
}

/// `init.js::FN_NAMES` の index。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FnName {
    HideToast,
    ShowToast,
}

impl FnName {
    fn encode_u16(self) -> u16 {
        self as u16
    }
}

// ============================================================
// receive (js value)
// ============================================================

/// js由来の文字列をstrとして取得
pub fn get_js_str(obj: &JsValue, key: &str) -> Option<String> {
    Reflect::get(obj, &JsValue::from_str(key)).ok().and_then(|v| v.as_string())
}

/// js由来の整数をu32として取得
pub fn get_js_u32(obj: &JsValue, key: &str) -> u32 {
    Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_f64())
        .and_then(|f| {
            if f >= 0.0 && f <= u32::MAX as f64 && f.fract() == 0.0 { Some(f as u32) } else { None }
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
        .and_then(|f| if f.is_finite() { Some(f) } else { None })
}

/// js由来のデータを構造体のまま取得
pub fn get_js_field(obj: &JsValue, key: &str) -> Option<JsValue> {
    Reflect::get(obj, &JsValue::from_str(key)).ok()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    /// appはEVENT_SHUTDOWNを独立したフレーム種別として送るが、rectgrid
    /// examplesはフレーム種別を持たずJSONの`event_type`一本で表すため、
    /// ここに値を追加している(app repositoryのEventTypeには無い)。
    Shutdown,
    Other,
}

impl EventType {
    pub fn decode(event_type: &str) -> Self {
        match event_type {
            "submit" => Self::Submit,
            "click" => Self::Click,
            "contextmenu" => Self::ContextMenu,
            "keydown" => Self::KeyDown,
            "input" => Self::Input,
            "change" => Self::Change,
            "focusin" => Self::FocusIn,
            "focusout" => Self::FocusOut,
            "resize" => Self::Resize,
            "scroll" => Self::Scroll,
            "drop" => Self::Drop,
            "pointerdown" => Self::PointerDown,
            "pointerup" => Self::PointerUp,
            "pointermove" => Self::PointerMove,
            "pointercancel" => Self::PointerCancel,
            "shutdown" => Self::Shutdown,
            _ => Self::Other,
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
            "ArrowUp" => Self::ArrowUp,
            "ArrowDown" => Self::ArrowDown,
            "ArrowLeft" => Self::ArrowLeft,
            "ArrowRight" => Self::ArrowRight,
            "Enter" => Self::Enter,
            "Escape" => Self::Escape,
            "Tab" => Self::Tab,
            "Backspace" => Self::Backspace,
            _ => Self::Other,
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
// gesture: tap, long press, swipe (up,down,left,right), drag, pinch/pan
// ============================================================
//
// 判定の根拠(閾値の出典・velocity計算窓・LongPressのタイマーレス実装、
// 2本指pinch/panの判定方法など)は app repository の reference/Gesture.md
// を参照。Thresholds / PointerState / detect_gesture (+ detect_on_release /
// detect_on_move) に加え、複数指を pointer_id でルーティングして
// pinch/pan を判定する TouchTracker も app repository からそのまま移植した。

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
    Drag {
        x: f64,
        y: f64,
    },
    /// ドラッグ終了 (pointerup)。スナップ処理はここで行う。
    DragEnd,
    /// ドラッグ中断 (pointercancel)。DragEndと同一視すると割り込み時に
    /// ドロップを取り消せなくなるため区別する。
    DragCancel,
    /// 2本指のつまみ操作。継続中は毎フレーム発火する。
    ///
    /// `scale` は2本指の開始距離に対する現在距離の比であり、
    /// `center_x` / `center_y` は2本指の現在の中点。
    Pinch {
        scale:    f64,
        center_x: f64,
        center_y: f64,
    },
    /// つまみ操作の終了 (どちらかの指が離れた)。
    PinchEnd,
}

/// PointerCancel でも座標・時刻を保持し、is_down と is_dragging のフラグ
/// だけを倒す。判定は detect_gesture がこの直後に行うため、そこで必要な
/// 値を判定前に消さない。
///
/// drag_offset / drag_px のようなドラッグ対象固有の状態は、ここではなく
/// 呼び出し側の Handler が持つ。TouchTracker は複数指をこの型でまとめて
/// 追跡するため、特定のドラッグ対象に紐づく値をここに置くと使い回せない。
#[derive(Debug, Default, Clone, Copy)]
pub struct PointerState {
    is_down:          bool,
    start_x:          f64,
    start_y:          f64,
    current_x:        f64,
    current_y:        f64,
    start_time:       f64,
    /// 直近の PointerMove の座標・時刻 (無ければ PointerDown のそれ)。
    /// swipe の速度を「離す直前の実際の動き」から計算するために持つ。
    last_move_x:      f64,
    last_move_y:      f64,
    last_move_time:   f64,
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
                is_dragging:      false,
                long_press_fired: false,
                cancelled:        false,
            },
            EventType::PointerMove => Self {
                current_x: x,
                current_y: y,
                last_move_x: x,
                last_move_y: y,
                last_move_time: time,
                ..self
            },
            EventType::PointerUp => {
                Self { is_down: false, current_x: x, current_y: y, cancelled: false, ..self }
            }
            EventType::PointerCancel => {
                Self { is_down: false, current_x: x, current_y: y, cancelled: true, ..self }
            }
            _ => self,
        }
    }

    /// 現在座標。TouchTrackerが2本指セッション終了時に、合成ポインタへ
    /// 送るPointerUpの座標を作るのに使う。
    pub const fn current(&self) -> (f64, f64) {
        (self.current_x, self.current_y)
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
fn detect_on_move(
    state: &mut PointerState,
    current_time: f64,
    thresholds: &Thresholds,
) -> Option<Gesture> {
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
// gesture: two-finger (pinch / pan)
// ============================================================
//
// 2本指の入力を、逆向きの変位ならpinch (scale)、平行な変位ならpan
// (単一ポインタ用パイプラインへ渡す合成点) に振り分ける。
//
// TouchTrackerがpointer_idごとに指をprimary/secondaryへ振り分け、
// app.rsのApp::processから呼ばれる。

/// 2本指のうち一方の追跡状態。
#[derive(Debug, Clone, Copy)]
struct TouchPoint {
    id:        u32,
    start_x:   f64,
    start_y:   f64,
    current_x: f64,
    current_y: f64,
}

impl TouchPoint {
    const fn new(id: u32, x: f64, y: f64) -> Self {
        Self { id, start_x: x, start_y: y, current_x: x, current_y: y }
    }

    fn displacement(&self) -> (f64, f64) {
        (self.current_x - self.start_x, self.current_y - self.start_y)
    }
}

/// pan/pinchの確定状態。一度確定したら、2本指セッションが終わるまで
/// ラッチする ([`TwoFingerState::fold`] のdocを参照)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum TwoFingerMode {
    /// まだ確定していない。両方の指の変位がTWO_FINGER_COMMIT_PXを
    /// 超えるまでこのまま。
    #[default]
    Undetermined,
    /// 2本指パンとして確定。
    Pan,
    /// pinchとして確定。
    Pinch,
}

/// 各指がこの距離 (px) 動くまでpan/pinchを確定しない。
///
/// 変位ベクトルが(0,0)のままだと内積が常に0になり、片方の指だけ先に
/// 動いた瞬間がpan側(内積がpinch閾値未満にならない)に誤って倒れる。
/// 両方が動くまで待つことでこれを避ける。
const TWO_FINGER_COMMIT_PX: f64 = 8.0;

/// pinchと判定する際の、変位ベクトルの内積の閾値。
///
/// 内積が正(順向き)でも小さければ「ほぼ直交」であり、pinch側に倒しても
/// 実害が小さい。0.0 (符号だけで判定) から始めて実機で調整する想定。
const PINCH_DOT_THRESHOLD: f64 = 0.0;

/// 畳み込み結果。[`TwoFingerState::fold`] へ渡す「仮想の1点」か、
/// pinchとして確定したscaleと中心座標のどちらか。
#[derive(Debug, Clone, Copy, PartialEq)]
enum FoldedInput {
    /// 2本の指がほぼ平行に動いている。単一ポインタ用パイプラインへ渡す
    /// 合成座標。
    AsSinglePoint { x: f64, y: f64 },
    /// 2本の指が逆向きに動いている。pinchとして確定。
    Pinch { scale: f64, center_x: f64, center_y: f64 },
    /// 1本指のみ、または判定材料が揃っていない。
    None,
}

/// 2本指ジェスチャの追跡状態。primaryが埋まっていない状態でsecondary
/// だけ埋まることはない (1本目が離れたら2本目をprimaryへ繰り上げる)。
/// 3本目以降は無視する (zoom用途では不要と判断)。
#[derive(Debug, Clone, Copy, Default)]
struct TwoFingerState {
    primary:   Option<TouchPoint>,
    secondary: Option<TouchPoint>,
    mode:      TwoFingerMode,
}

impl TwoFingerState {
    /// 指が1本追加で触れた。3本目以降は無視する。
    fn touch_down(self, id: u32, x: f64, y: f64) -> Self {
        match (self.primary, self.secondary) {
            (None, _) => Self { primary: Some(TouchPoint::new(id, x, y)), ..self },
            (Some(_), None) => Self {
                secondary: Some(TouchPoint::new(id, x, y)),
                mode: TwoFingerMode::Undetermined,
                ..self
            },
            (Some(_), Some(_)) => self,
        }
    }

    /// idに一致する指が動いた。どちらにも一致しなければ無視する。
    fn touch_move(self, id: u32, x: f64, y: f64) -> Self {
        if self.primary.is_some_and(|p| p.id == id) {
            Self {
                primary: self.primary.map(|p| TouchPoint { current_x: x, current_y: y, ..p }),
                ..self
            }
        } else if self.secondary.is_some_and(|s| s.id == id) {
            Self {
                secondary: self.secondary.map(|s| TouchPoint { current_x: x, current_y: y, ..s }),
                ..self
            }
        } else {
            self
        }
    }

    /// idに一致する指が離れた。primaryならsecondaryを繰り上げる。戻り値の
    /// 2つ目は、離れる前の確定状態 (呼び出し側がGesture::PinchEndを出す
    /// かどうかの判断に使う。[`TouchTracker`]を参照)。
    fn touch_up(self, id: u32) -> (Self, TwoFingerMode) {
        let ended_mode = self.mode;
        if self.primary.is_some_and(|p| p.id == id) {
            (
                Self {
                    primary:   self.secondary,
                    secondary: None,
                    mode:      TwoFingerMode::Undetermined,
                },
                ended_mode,
            )
        } else if self.secondary.is_some_and(|s| s.id == id) {
            (Self { secondary: None, mode: TwoFingerMode::Undetermined, ..self }, ended_mode)
        } else {
            (self, TwoFingerMode::Undetermined)
        }
    }

    fn primary_id(&self) -> Option<u32> {
        self.primary.map(|p| p.id)
    }

    fn secondary_id(&self) -> Option<u32> {
        self.secondary.map(|p| p.id)
    }

    /// primaryの現在座標。2本指セッションが終わって1本指に戻る際、残った
    /// 指の位置でPointerStateを作り直すのに使う
    /// ([`TouchTracker::resync_primary`]を参照)。
    fn primary_current(&self) -> Option<(f64, f64)> {
        self.primary.map(|p| (p.current_x, p.current_y))
    }

    /// 現在の2本指の状態から、畳み込み結果を導出する。
    ///
    /// 2本とも揃っていなければFoldedInput::None。揃っていても、両方の
    /// 指の変位がTWO_FINGER_COMMIT_PXを超えるまでは判定を保留し
    /// FoldedInput::Noneを返す (doc冒頭の1.を参照)。一度Pan/Pinchを
    /// 確定したら、2本指セッションが終わるまで再判定しない
    /// (doc冒頭の2.を参照)。
    fn fold(&mut self) -> FoldedInput {
        let (Some(p), Some(s)) = (self.primary, self.secondary) else {
            return FoldedInput::None;
        };

        if self.mode == TwoFingerMode::Undetermined {
            let d1 = p.displacement();
            let d2 = s.displacement();
            let moved1 = (d1.0 * d1.0 + d1.1 * d1.1).sqrt() > TWO_FINGER_COMMIT_PX;
            let moved2 = (d2.0 * d2.0 + d2.1 * d2.1).sqrt() > TWO_FINGER_COMMIT_PX;
            if !(moved1 && moved2) {
                return FoldedInput::None;
            }
            let dot = d1.0 * d2.0 + d1.1 * d2.1;
            self.mode =
                if dot < PINCH_DOT_THRESHOLD { TwoFingerMode::Pinch } else { TwoFingerMode::Pan };
        }

        match self.mode {
            TwoFingerMode::Undetermined => FoldedInput::None,
            TwoFingerMode::Pan => FoldedInput::AsSinglePoint {
                x: (p.current_x + s.current_x) / 2.0,
                y: (p.current_y + s.current_y) / 2.0,
            },
            TwoFingerMode::Pinch => {
                let start_distance = two_point_distance(p.start_x, p.start_y, s.start_x, s.start_y);
                if start_distance <= 0.0 {
                    return FoldedInput::None;
                }
                let current_distance =
                    two_point_distance(p.current_x, p.current_y, s.current_x, s.current_y);
                FoldedInput::Pinch {
                    scale:    current_distance / start_distance,
                    center_x: (p.current_x + s.current_x) / 2.0,
                    center_y: (p.current_y + s.current_y) / 2.0,
                }
            }
        }
    }
}

/// 2点間の距離 (px)。PointerState::distanceと同じ式。
fn two_point_distance(x0: f64, y0: f64, x1: f64, y1: f64) -> f64 {
    let dx = x1 - x0;
    let dy = y1 - y0;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod two_finger_tests {
    use super::*;

    /// 片方の指だけが先に動いても、もう片方が動くまで確定しない。
    #[test]
    fn waits_for_both_fingers_before_classifying() {
        let mut state =
            TwoFingerState::default().touch_down(1, 100.0, 100.0).touch_down(2, 200.0, 100.0);

        // primaryだけが動く。secondaryの変位は(0,0)のまま。
        state = state.touch_move(1, 110.0, 100.0);
        assert_eq!(state.fold(), FoldedInput::None);
        state = state.touch_move(1, 130.0, 100.0);
        assert_eq!(state.fold(), FoldedInput::None);

        // secondaryも動き、両者が閾値を超えて初めて確定する。
        state = state.touch_move(1, 150.0, 100.0).touch_move(2, 150.0, 100.0);
        assert_eq!(
            state.fold(),
            FoldedInput::Pinch { scale: 0.0, center_x: 150.0, center_y: 100.0 }
        );
    }

    #[test]
    fn symmetric_pinch_in_reduces_scale() {
        let mut state =
            TwoFingerState::default().touch_down(1, 100.0, 100.0).touch_down(2, 200.0, 100.0);
        state = state.touch_move(1, 140.0, 100.0).touch_move(2, 160.0, 100.0);
        match state.fold() {
            FoldedInput::Pinch { scale, .. } => assert!(scale < 1.0, "scale = {scale}"),
            other => panic!("expected Pinch, got {other:?}"),
        }
    }

    #[test]
    fn symmetric_pinch_out_increases_scale() {
        let mut state =
            TwoFingerState::default().touch_down(1, 140.0, 100.0).touch_down(2, 160.0, 100.0);
        state = state.touch_move(1, 100.0, 100.0).touch_move(2, 200.0, 100.0);
        match state.fold() {
            FoldedInput::Pinch { scale, .. } => assert!(scale > 1.0, "scale = {scale}"),
            other => panic!("expected Pinch, got {other:?}"),
        }
    }

    /// 両指がほぼ同じ向き・同じ距離動けばpan (単一ポインタパイプラインへ
    /// の合成点) になる。
    #[test]
    fn parallel_motion_is_pan_not_pinch() {
        let mut state =
            TwoFingerState::default().touch_down(1, 100.0, 100.0).touch_down(2, 200.0, 100.0);
        state = state.touch_move(1, 120.0, 100.0).touch_move(2, 220.0, 100.0);
        assert_eq!(state.fold(), FoldedInput::AsSinglePoint { x: 170.0, y: 100.0 });
    }

    /// 一度確定したら、その後の入力で符号が変わっても再判定しない。
    #[test]
    fn mode_latches_after_commit() {
        let mut state =
            TwoFingerState::default().touch_down(1, 100.0, 100.0).touch_down(2, 200.0, 100.0);
        state = state.touch_move(1, 140.0, 100.0).touch_move(2, 160.0, 100.0);
        assert!(matches!(state.fold(), FoldedInput::Pinch { .. }));

        // 内積の符号だけで見ればもうpinchではない動きだが、ラッチして
        // いるためpanには切り替わらない。
        state = state.touch_move(1, 140.0, 100.0).touch_move(2, 140.0, 100.0);
        assert!(matches!(state.fold(), FoldedInput::Pinch { .. }));
    }

    /// 3本目以降は無視する。
    #[test]
    fn third_finger_is_ignored() {
        let state = TwoFingerState::default()
            .touch_down(1, 100.0, 100.0)
            .touch_down(2, 200.0, 100.0)
            .touch_down(3, 300.0, 100.0);
        assert_eq!(state.primary_id(), Some(1));
        assert_eq!(state.secondary_id(), Some(2));
    }

    /// 1本目が離れたら2本目がprimaryへ繰り上がり、モードは再判定待ちに
    /// 戻る。
    #[test]
    fn primary_release_promotes_secondary() {
        let mut state =
            TwoFingerState::default().touch_down(1, 100.0, 100.0).touch_down(2, 200.0, 100.0);
        state = state.touch_move(1, 140.0, 100.0).touch_move(2, 160.0, 100.0);
        assert!(matches!(state.fold(), FoldedInput::Pinch { .. }));

        let (next, ended_mode) = state.touch_up(1);
        state = next;
        assert_eq!(ended_mode, TwoFingerMode::Pinch);
        assert_eq!(state.primary_id(), Some(2));
        assert!(state.secondary_id().is_none());
        // 1本指しか残っていないので確定しない。
        assert_eq!(state.fold(), FoldedInput::None);
    }
}

// ============================================================
// gesture: TouchTracker (pointer_idによるルーティング)
// ============================================================
//
// 複数指のポインタ入力を、1系統のGestureへ落とす。
//
// 最初に触れた指をprimaryとし、既存の単一ポインタ用パイプライン
// (PointerState / detect_gesture) でそのままtap/press/swipe/dragを判定
// する。2本目が触れたらsecondaryとしてTwoFingerStateへ渡し、pan/pinchの
// 判定を始める。3本目以降は無視する。
//
// 2本指セッション中はprimaryの単一ポインタ判定を凍結する (PointerMove /
// PointerUpをprimary_stateへ回さない)。panと確定した場合のみ、2本指の
// 合成点を仮想の単一ポインタ (pan_state) として同じパイプラインに流し、
// Drag/Swipe/Tapをそのまま得る。pinchと確定した場合はPointerStateを
// 経由せず、Gesture::Pinchを直接返す。
//
// 2本指セッションが終わって1本指に戻るときは、残った指の現在位置で
// primary_stateをPointerDownし直す (resync_primary)。凍結中に動いた分の
// 距離を再開後の単一ポインタ判定へ持ち込まないためである。
#[derive(Debug, Default)]
pub struct TouchTracker {
    primary_state: PointerState,
    two_fingers:   TwoFingerState,
    pan_state:     Option<PointerState>,
}

impl TouchTracker {
    /// 1イベント分進めて、確定したジェスチャがあれば返す。
    pub fn handle(
        &mut self,
        event_type: &EventType,
        pointer_id: u32,
        x: f64,
        y: f64,
        time: f64,
        thresholds: &Thresholds,
    ) -> Option<Gesture> {
        match event_type {
            EventType::PointerDown => {
                self.on_down(pointer_id, x, y, time);
                None
            }
            EventType::PointerMove => self.on_move(pointer_id, x, y, time, thresholds),
            EventType::PointerUp | EventType::PointerCancel => {
                self.on_up(event_type, pointer_id, x, y, time, thresholds)
            }
            _ => None,
        }
    }

    /// 現在アクティブなPointerState。2本指pan中はその合成ポインタ、
    /// それ以外はprimaryのもの。Gesture::Pinch / PinchEndには対応する
    /// PointerStateが無いため、呼び出し側はそれらのvariantではこれを
    /// 参照しない。
    pub const fn active_state(&self) -> &PointerState {
        match &self.pan_state {
            Some(state) => state,
            None => &self.primary_state,
        }
    }

    fn on_down(&mut self, id: u32, x: f64, y: f64, time: f64) {
        if self.two_fingers.primary_id() == Some(id) || self.two_fingers.secondary_id() == Some(id)
        {
            // 既に追跡中のidへの重複PointerDown。新規の指としては扱わず、
            // 位置の更新だけ反映する。
            self.two_fingers = self.two_fingers.touch_move(id, x, y);
            return;
        }
        if self.two_fingers.primary_id().is_none() {
            self.primary_state = self.primary_state.update(&EventType::PointerDown, x, y, time);
        }
        self.two_fingers = self.two_fingers.touch_down(id, x, y);
    }

    fn on_move(
        &mut self,
        id: u32,
        x: f64,
        y: f64,
        time: f64,
        thresholds: &Thresholds,
    ) -> Option<Gesture> {
        let is_primary = self.two_fingers.primary_id() == Some(id);
        let is_secondary = self.two_fingers.secondary_id() == Some(id);
        if !is_primary && !is_secondary {
            return None; // 3本目以降、追跡していない指。
        }
        self.two_fingers = self.two_fingers.touch_move(id, x, y);

        if self.two_fingers.secondary_id().is_some() {
            // 2本指セッション中。primary単独の判定は凍結し、foldに譲る。
            return self.fold_and_emit(time, thresholds);
        }

        // 1本指のまま。既存のパイプラインで判定する。
        let prev = self.primary_state;
        self.primary_state = self.primary_state.update(&EventType::PointerMove, x, y, time);
        detect_gesture(&mut self.primary_state, &prev, &EventType::PointerMove, time, thresholds)
    }

    fn on_up(
        &mut self,
        event_type: &EventType,
        id: u32,
        x: f64,
        y: f64,
        time: f64,
        thresholds: &Thresholds,
    ) -> Option<Gesture> {
        let is_primary = self.two_fingers.primary_id() == Some(id);
        let is_secondary = self.two_fingers.secondary_id() == Some(id);
        let had_secondary = self.two_fingers.secondary_id().is_some();

        if is_secondary || (is_primary && had_secondary) {
            // 2本指セッションの終了 (どちらの指が離れても終わる)。
            let (next, ended_mode) = self.two_fingers.touch_up(id);
            self.two_fingers = next;
            let gesture = self.end_two_finger_session(event_type, ended_mode, time, thresholds);
            // 残った1本を今の位置から数え直す。
            self.resync_primary(time);
            gesture
        } else if is_primary {
            // 通常の単一ポインタの終了。既存のパイプラインそのまま。
            let prev = self.primary_state;
            self.primary_state = self.primary_state.update(event_type, x, y, time);
            let gesture =
                detect_gesture(&mut self.primary_state, &prev, event_type, time, thresholds);
            self.two_fingers = self.two_fingers.touch_up(id).0;
            gesture
        } else {
            None // 追跡していない指。
        }
    }

    /// 2本指セッションを閉じる。pinchだったらPinchEnd、panだったら合成
    /// ポインタへ最後のPointerUp / PointerCancelを送ってDragEnd /
    /// DragCancel / Swipe* / Tapを得る。まだ確定していなければ
    /// (Undetermined) 何も発行していないのでNone。
    fn end_two_finger_session(
        &mut self,
        event_type: &EventType,
        ended_mode: TwoFingerMode,
        time: f64,
        thresholds: &Thresholds,
    ) -> Option<Gesture> {
        match ended_mode {
            TwoFingerMode::Undetermined => None,
            TwoFingerMode::Pinch => Some(Gesture::PinchEnd),
            TwoFingerMode::Pan => {
                let state = self.pan_state.take()?;
                let (cx, cy) = state.current();
                let prev = state;
                let mut next = state.update(event_type, cx, cy, time);
                detect_gesture(&mut next, &prev, event_type, time, thresholds)
            }
        }
    }

    /// 2本指セッションが終わって残った1本を、今の位置からPointerDownし
    /// 直す。凍結中に動いた分を引きずらないため。
    fn resync_primary(&mut self, time: f64) {
        self.primary_state = match self.two_fingers.primary_current() {
            Some((x, y)) => PointerState::default().update(&EventType::PointerDown, x, y, time),
            None => PointerState::default(),
        };
    }

    fn fold_and_emit(&mut self, time: f64, thresholds: &Thresholds) -> Option<Gesture> {
        match self.two_fingers.fold() {
            FoldedInput::None => None,
            FoldedInput::Pinch { scale, center_x, center_y } => {
                Some(Gesture::Pinch { scale, center_x, center_y })
            }
            FoldedInput::AsSinglePoint { x, y } => match self.pan_state {
                None => {
                    self.pan_state =
                        Some(PointerState::default().update(&EventType::PointerDown, x, y, time));
                    None
                }
                Some(state) => {
                    let prev = state;
                    let mut next = state.update(&EventType::PointerMove, x, y, time);
                    let gesture =
                        detect_gesture(&mut next, &prev, &EventType::PointerMove, time, thresholds);
                    self.pan_state = Some(next);
                    gesture
                }
            },
        }
    }
}

#[cfg(test)]
mod touch_tracker_tests {
    use super::*;

    /// `App::process` と同じ順序で1イベントずつ`handle`に流す。
    fn run(events: &[(EventType, u32, f64, f64, f64)], th: &Thresholds) -> Vec<Option<Gesture>> {
        let mut tracker = TouchTracker::default();
        events
            .iter()
            .map(|(event_type, id, x, y, time)| tracker.handle(event_type, *id, *x, *y, *time, th))
            .collect()
    }

    /// 1本指のときは、これまでの単一指パイプラインと同じ結果になる。
    #[test]
    fn single_finger_behaves_like_before() {
        let th = Thresholds::MOUSE;
        let got = run(
            &[
                (EventType::PointerDown, 1, 100.0, 100.0, 0.0),
                (EventType::PointerMove, 1, 200.0, 100.0, 50.0),
                (EventType::PointerUp, 1, 260.0, 100.0, 100.0),
            ],
            &th,
        );
        assert_eq!(got, [None, None, Some(Gesture::SwipeRight)]);
    }

    /// 2本目が触れるとprimary単独の判定は凍結する。片方だけが先に動いても
    /// 、両方がTWO_FINGER_COMMIT_PXを超えて初めてpinchが確定する。
    #[test]
    fn second_finger_freezes_primary_until_pinch_commits() {
        let th = Thresholds::MOUSE;
        let got = run(
            &[
                (EventType::PointerDown, 1, 100.0, 100.0, 0.0),
                (EventType::PointerDown, 2, 200.0, 100.0, 0.0),
                (EventType::PointerMove, 1, 150.0, 100.0, 50.0), // primaryだけ動く
                (EventType::PointerMove, 2, 150.0, 100.0, 60.0), // secondaryも動き確定
            ],
            &th,
        );
        assert_eq!(
            got,
            [
                None,
                None,
                None, // primary単独ではDragも何も出ない (凍結中)
                Some(Gesture::Pinch { scale: 0.0, center_x: 150.0, center_y: 100.0 }),
            ]
        );
    }

    /// pinchが確定した後、2本目が離れるとPinchEnd。
    #[test]
    fn pinch_end_on_secondary_release() {
        let th = Thresholds::MOUSE;
        let got = run(
            &[
                (EventType::PointerDown, 1, 100.0, 100.0, 0.0),
                (EventType::PointerDown, 2, 200.0, 100.0, 0.0),
                (EventType::PointerMove, 1, 150.0, 100.0, 50.0),
                (EventType::PointerMove, 2, 150.0, 100.0, 60.0),
                (EventType::PointerUp, 2, 150.0, 100.0, 100.0),
            ],
            &th,
        );
        assert_eq!(got[4], Some(Gesture::PinchEnd));
    }

    /// 平行に動く2本指パンは、合成した中点を仮想の単一ポインタへ流し、
    /// 閾値を超えるとDragとして出る。離れるとDragEnd。
    #[test]
    fn two_finger_pan_emits_drag_then_drag_end() {
        let th = Thresholds::MOUSE;
        let got = run(
            &[
                (EventType::PointerDown, 1, 0.0, 0.0, 0.0),
                (EventType::PointerDown, 2, 100.0, 0.0, 0.0),
                (EventType::PointerMove, 1, 30.0, 0.0, 50.0), // まだ確定しない
                (EventType::PointerMove, 2, 130.0, 0.0, 60.0), // pan確定、合成点(80,0)でpan_stateを作る
                (EventType::PointerMove, 1, 60.0, 0.0, 120.0), // 合成点(95,0)、start(80,0)から15px
                (EventType::PointerUp, 2, 130.0, 0.0, 200.0),
            ],
            &th,
        );
        assert_eq!(got[0..4], [None, None, None, None]);
        assert_eq!(got[4], Some(Gesture::Drag { x: 95.0, y: 0.0 }));
        assert_eq!(got[5], Some(Gesture::DragEnd));
    }
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
    use alloc::{format, string::String, vec::Vec};
    use core::{
        clone::Clone,
        cmp::PartialEq,
        option::Option::{self, None, Some},
        result::Result::Ok,
    };

    #[derive(Debug, Clone, PartialEq)]
    pub enum Tag {
        Body,
        Header,
        H1,
        H2,
        H3,
        Ul,
        Li,
        Button,
        Main,
        Section,
        Span,
        Dl,
        Dt,
        Dd,
        Ol,
        P,
        Textarea,
        Drawer, // <dialog id="*drawer*">
        Modal,  // <dialog id="*modal*">
        Form,
        Input,
        Fieldset,
        Table,
        Thead,
        Tbody,
        Tr,
        Th,
        Td,
        Select,
        Footer,
        Output,
        Article,
        Other,
    }

    impl Tag {
        pub fn decode(s: &str) -> Self {
            match s {
                "body" => Self::Body,
                "header" => Self::Header,
                "h1" => Self::H1,
                "h2" => Self::H2,
                "h3" => Self::H3,
                "ul" => Self::Ul,
                "li" => Self::Li,
                "button" => Self::Button,
                "main" => Self::Main,
                "section" => Self::Section,
                "span" => Self::Span,
                "dl" => Self::Dl,
                "dt" => Self::Dt,
                "dd" => Self::Dd,
                "ol" => Self::Ol,
                "p" => Self::P,
                "textarea" => Self::Textarea,
                "drawer" => Self::Drawer,
                "modal" => Self::Modal,
                "form" => Self::Form,
                "input" => Self::Input,
                "fieldset" => Self::Fieldset,
                "table" => Self::Table,
                "thead" => Self::Thead,
                "tbody" => Self::Tbody,
                "tr" => Self::Tr,
                "th" => Self::Th,
                "td" => Self::Td,
                "select" => Self::Select,
                "footer" => Self::Footer,
                "output" => Self::Output,
                "article" => Self::Article,
                _ => Self::Other,
            }
        }

        pub fn encode(&self) -> &'static str {
            match self {
                Self::Body => "body",
                Self::Header => "header",
                Self::H1 => "h1",
                Self::H2 => "h2",
                Self::H3 => "h3",
                Self::Ul => "ul",
                Self::Li => "li",
                Self::Button => "button",
                Self::Main => "main",
                Self::Section => "section",
                Self::Span => "span",
                Self::Dl => "dl",
                Self::Dt => "dt",
                Self::Dd => "dd",
                Self::Ol => "ol",
                Self::P => "p",
                Self::Textarea => "textarea",
                Self::Drawer => "drawer",
                Self::Modal => "modal",
                Self::Form => "form",
                Self::Input => "input",
                Self::Fieldset => "fieldset",
                Self::Table => "table",
                Self::Thead => "thead",
                Self::Tbody => "tbody",
                Self::Tr => "tr",
                Self::Th => "th",
                Self::Td => "td",
                Self::Select => "select",
                Self::Footer => "footer",
                Self::Output => "output",
                Self::Article => "article",
                Self::Other => "",
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
        pub fn new(tag: Tag) -> Self {
            Self { tag, n: None }
        }
        pub fn numbered(tag: Tag, n: u32) -> Self {
            Self { tag, n: Some(n) }
        }

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
                None => self.tag.encode().to_string(),
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
            self.0.iter().map(Segment::encode).collect::<Vec<_>>().join("_")
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
    pub id:               dom::Id,
    pub key:              KeyName,
    pub value:            String,
    pub x:                f64,
    pub y:                f64,
    pub time:             f64,
    pub section_origin_x: f64,
    pub section_origin_y: f64,
    /// `PointerEvent.pointerId`。pointer系以外のイベントでは0
    /// (init.jsのsendが`e.pointerId ?? 0`で送る)。複数指の追跡に使う
    /// ([`TouchTracker`]を参照)。
    pub pointer_id:       u32,
}

impl CanvasEvent {
    pub fn decode(payload: &wasm_bindgen::JsValue) -> Self {
        let event_type = get_js_str(payload, "event_type")
            .as_deref()
            .map(EventType::decode)
            .unwrap_or(EventType::Other);
        let id = get_js_str(payload, "target_id")
            .as_deref()
            .map(dom::Id::decode)
            .unwrap_or_else(|| dom::Id(vec![]));
        let key =
            get_js_str(payload, "key").as_deref().map(KeyName::decode).unwrap_or(KeyName::Other);
        let value = get_js_str(payload, "value").unwrap_or_default();
        let x = get_js_f64(payload, "x").unwrap_or(0.0);
        let y = get_js_f64(payload, "y").unwrap_or(0.0);
        let time = get_js_f64(payload, "time").unwrap_or(0.0);
        let section_origin_x = get_js_f64(payload, "section_origin_x").unwrap_or(0.0);
        let section_origin_y = get_js_f64(payload, "section_origin_y").unwrap_or(0.0);
        let pointer_id = get_js_u32(payload, "pointer_id");
        Self {
            event_type,
            id,
            key,
            value,
            x,
            y,
            time,
            section_origin_x,
            section_origin_y,
            pointer_id,
        }
    }
}
