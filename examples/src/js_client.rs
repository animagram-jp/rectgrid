use js_sys::Reflect;
use serde::{Serialize, Serializer, ser::SerializeMap};
use wasm_bindgen::JsValue;

pub enum Command {
    SetText { id: String, value: String },
    SetValue { id: String, value: String },
    SetAttribute { id: String, attribute: Attribute, value: String },
    RemoveAttribute { id: String, attribute: Attribute },
    AddClass { id: String, value: ClassName },
    RemoveClass { id: String, value: ClassName },
    SetWidth { id: String, px: u32 },
    SetHeight { id: String, px: u32 },
    SetZIndex { id: String, z: i32 },
    SetBackground { id: String, value: String },
    SetTranslate { id: String, x: f64, y: f64 },
    SetCursor { id: String, value: CursorValue },
    ShowModal { id: String },
    CloseModal { id: String },
    Focus { id: String },
    JsFn { id: String, name: FnName },
    Error { message: String },
}

impl Serialize for Command {
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CursorValue {
    Default,
    Grab,
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

pub fn get_js_str(obj: &JsValue, key: &str) -> Option<String> {
    Reflect::get(obj, &JsValue::from_str(key)).ok().and_then(|v| v.as_string())
}

pub fn get_js_u32(obj: &JsValue, key: &str) -> u32 {
    Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_f64())
        .and_then(|f| {
            if f >= 0.0 && f <= u32::MAX as f64 && f.fract() == 0.0 { Some(f as u32) } else { None }
        })
        .unwrap_or(0)
}

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

pub fn get_js_f64(obj: &JsValue, key: &str) -> Option<f64> {
    Reflect::get(obj, &JsValue::from_str(key))
        .ok()
        .and_then(|v| v.as_f64())
        .and_then(|f| if f.is_finite() { Some(f) } else { None })
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
    Touch,
    Mouse,
}

pub fn detect_device(pointer_coarse: bool) -> Device {
    if pointer_coarse { Device::Touch } else { Device::Mouse }
}

#[derive(Debug, Clone, Copy)]
pub struct Thresholds {
    pub long_press_ms:      f64,
    pub long_press_slop_px: f64,
    pub drag_start_px:      f64,
    pub swipe_min_px:       f64,
    pub swipe_min_velocity: f64,
    pub swipe_max_ms:       f64,
    pub tap_max_ms:         f64,
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
    Tap,
    LongPress,
    SwipeUp,
    SwipeDown,
    SwipeLeft,
    SwipeRight,
    Drag { x: f64, y: f64 },
    DragEnd,
    DragCancel,
    Pinch { scale: f64, center_x: f64, center_y: f64 },
    PinchEnd,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PointerState {
    is_down:          bool,
    start_x:          f64,
    start_y:          f64,
    current_x:        f64,
    current_y:        f64,
    start_time:       f64,
    last_move_x:      f64,
    last_move_y:      f64,
    last_move_time:   f64,
    is_dragging:      bool,
    long_press_fired: bool,
    cancelled:        bool,
}

impl PointerState {
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

    pub const fn current(&self) -> (f64, f64) {
        (self.current_x, self.current_y)
    }

    fn distance(&self) -> f64 {
        let dx = self.current_x - self.start_x;
        let dy = self.current_y - self.start_y;
        (dx * dx + dy * dy).sqrt()
    }
}

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

fn detect_on_release(
    state: &mut PointerState,
    prev_state: &PointerState,
    current_time: f64,
    thresholds: &Thresholds,
) -> Option<Gesture> {
    if prev_state.is_dragging {
        state.is_dragging = false;
        return Some(if state.cancelled { Gesture::DragCancel } else { Gesture::DragEnd });
    }

    if state.cancelled {
        return None;
    }

    let dt = current_time - state.start_time;
    if dt <= 0.0 {
        return None;
    }
    let distance = state.distance();

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

    if state.long_press_fired {
        return None;
    }

    if dt > thresholds.long_press_ms && distance < thresholds.long_press_slop_px {
        return Some(Gesture::LongPress);
    }

    if dt < thresholds.tap_max_ms && distance < thresholds.tap_slop_px {
        return Some(Gesture::Tap);
    }

    None
}

fn detect_on_move(
    state: &mut PointerState,
    current_time: f64,
    thresholds: &Thresholds,
) -> Option<Gesture> {
    if !state.is_down {
        return None;
    }

    let distance = state.distance();

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

    if state.is_dragging {
        return Some(Gesture::Drag { x: state.current_x, y: state.current_y });
    }

    let dt = current_time - state.start_time;
    if dt > 0.0 && dt < thresholds.swipe_max_ms {
        let velocity = distance / dt;
        if velocity > thresholds.swipe_min_velocity && distance > thresholds.swipe_min_px {
            return None;
        }
    }

    state.is_dragging = true;
    Some(Gesture::Drag { x: state.current_x, y: state.current_y })
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum TwoFingerMode {
    #[default]
    Undetermined,
    Pan,
    Pinch,
}

const TWO_FINGER_COMMIT_PX: f64 = 8.0;

const PINCH_DOT_THRESHOLD: f64 = 0.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum FoldedInput {
    AsSinglePoint { x: f64, y: f64 },
    Pinch { scale: f64, center_x: f64, center_y: f64 },
    None,
}

#[derive(Debug, Clone, Copy, Default)]
struct TwoFingerState {
    primary:   Option<TouchPoint>,
    secondary: Option<TouchPoint>,
    mode:      TwoFingerMode,
}

impl TwoFingerState {
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

    fn primary_current(&self) -> Option<(f64, f64)> {
        self.primary.map(|p| (p.current_x, p.current_y))
    }

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

fn two_point_distance(x0: f64, y0: f64, x1: f64, y1: f64) -> f64 {
    let dx = x1 - x0;
    let dy = y1 - y0;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod two_finger_tests {
    use super::*;

    #[test]
    fn waits_for_both_fingers_before_classifying() {
        let mut state =
            TwoFingerState::default().touch_down(1, 100.0, 100.0).touch_down(2, 200.0, 100.0);

        state = state.touch_move(1, 110.0, 100.0);
        assert_eq!(state.fold(), FoldedInput::None);
        state = state.touch_move(1, 130.0, 100.0);
        assert_eq!(state.fold(), FoldedInput::None);

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

    #[test]
    fn parallel_motion_is_pan_not_pinch() {
        let mut state =
            TwoFingerState::default().touch_down(1, 100.0, 100.0).touch_down(2, 200.0, 100.0);
        state = state.touch_move(1, 120.0, 100.0).touch_move(2, 220.0, 100.0);
        assert_eq!(state.fold(), FoldedInput::AsSinglePoint { x: 170.0, y: 100.0 });
    }

    #[test]
    fn mode_latches_after_commit() {
        let mut state =
            TwoFingerState::default().touch_down(1, 100.0, 100.0).touch_down(2, 200.0, 100.0);
        state = state.touch_move(1, 140.0, 100.0).touch_move(2, 160.0, 100.0);
        assert!(matches!(state.fold(), FoldedInput::Pinch { .. }));

        state = state.touch_move(1, 140.0, 100.0).touch_move(2, 140.0, 100.0);
        assert!(matches!(state.fold(), FoldedInput::Pinch { .. }));
    }

    #[test]
    fn third_finger_is_ignored() {
        let state = TwoFingerState::default()
            .touch_down(1, 100.0, 100.0)
            .touch_down(2, 200.0, 100.0)
            .touch_down(3, 300.0, 100.0);
        assert_eq!(state.primary_id(), Some(1));
        assert_eq!(state.secondary_id(), Some(2));
    }

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
        assert_eq!(state.fold(), FoldedInput::None);
    }
}

#[derive(Debug, Default)]
pub struct TouchTracker {
    primary_state: PointerState,
    two_fingers:   TwoFingerState,
    pan_state:     Option<PointerState>,
}

impl TouchTracker {
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

    pub const fn active_state(&self) -> &PointerState {
        match &self.pan_state {
            Some(state) => state,
            None => &self.primary_state,
        }
    }

    fn on_down(&mut self, id: u32, x: f64, y: f64, time: f64) {
        if self.two_fingers.primary_id() == Some(id) || self.two_fingers.secondary_id() == Some(id)
        {
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
            return None;
        }
        self.two_fingers = self.two_fingers.touch_move(id, x, y);

        if self.two_fingers.secondary_id().is_some() {
            return self.fold_and_emit(time, thresholds);
        }

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
            let (next, ended_mode) = self.two_fingers.touch_up(id);
            self.two_fingers = next;
            let gesture = self.end_two_finger_session(event_type, ended_mode, time, thresholds);
            self.resync_primary(time);
            gesture
        } else if is_primary {
            let prev = self.primary_state;
            self.primary_state = self.primary_state.update(event_type, x, y, time);
            let gesture =
                detect_gesture(&mut self.primary_state, &prev, event_type, time, thresholds);
            self.two_fingers = self.two_fingers.touch_up(id).0;
            gesture
        } else {
            None
        }
    }

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

    fn run(events: &[(EventType, u32, f64, f64, f64)], th: &Thresholds) -> Vec<Option<Gesture>> {
        let mut tracker = TouchTracker::default();
        events
            .iter()
            .map(|(event_type, id, x, y, time)| tracker.handle(event_type, *id, *x, *y, *time, th))
            .collect()
    }

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

    #[test]
    fn second_finger_freezes_primary_until_pinch_commits() {
        let th = Thresholds::MOUSE;
        let got = run(
            &[
                (EventType::PointerDown, 1, 100.0, 100.0, 0.0),
                (EventType::PointerDown, 2, 200.0, 100.0, 0.0),
                (EventType::PointerMove, 1, 150.0, 100.0, 50.0),
                (EventType::PointerMove, 2, 150.0, 100.0, 60.0),
            ],
            &th,
        );
        assert_eq!(
            got,
            [
                None,
                None,
                None,
                Some(Gesture::Pinch { scale: 0.0, center_x: 150.0, center_y: 100.0 }),
            ]
        );
    }

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

    #[test]
    fn two_finger_pan_emits_drag_then_drag_end() {
        let th = Thresholds::MOUSE;
        let got = run(
            &[
                (EventType::PointerDown, 1, 0.0, 0.0, 0.0),
                (EventType::PointerDown, 2, 100.0, 0.0, 0.0),
                (EventType::PointerMove, 1, 30.0, 0.0, 50.0),
                (EventType::PointerMove, 2, 130.0, 0.0, 60.0),
                (EventType::PointerMove, 1, 60.0, 0.0, 120.0),
                (EventType::PointerUp, 2, 130.0, 0.0, 200.0),
            ],
            &th,
        );
        assert_eq!(got[0..4], [None, None, None, None]);
        assert_eq!(got[4], Some(Gesture::Drag { x: 95.0, y: 0.0 }));
        assert_eq!(got[5], Some(Gesture::DragEnd));
    }
}

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
        Drawer,
        Modal,
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
