use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use serde_wasm_bindgen::to_value;
use crate::js_client::{Command, EventType, detect_device, PointerState, detect_gesture, CanvasEvent};
use crate::event::{Handler, Event};

// ============================================================
// App
// ============================================================

#[wasm_bindgen]
pub struct App {
    pointer_state: PointerState,
    events:        Vec<Event>,
    handler:       Handler,
}

#[wasm_bindgen]
impl App {
    pub fn init(pointer_coarse: bool, viewport_width_px: f64, _viewport_height_px: f64, section_origin_x: f64, section_origin_y: f64) -> App {
        detect_device(pointer_coarse);

        let mut app = App {
            pointer_state: PointerState::default(),
            events:        Vec::new(),
            handler:       Handler::new(viewport_width_px, [section_origin_x, section_origin_y]),
        };

        app.events.push(Event::Ready);
        app
    }

    pub fn close(&self) {
        self.handler.close();
    }

    pub fn process(&mut self, payload: JsValue) -> JsValue {
        let mut commands = Vec::new();
        let canvas_event = CanvasEvent::decode(&payload);
        let prev_state = self.pointer_state;
        self.pointer_state = self.pointer_state.update(
            &canvas_event.event_type,
            canvas_event.x, canvas_event.y, canvas_event.time,
        );
        match detect_gesture(&mut self.pointer_state, &prev_state, &canvas_event.event_type, canvas_event.time) {
            Some(gesture) => self.events.push(Event::Gesture(gesture)),
            None => match &canvas_event.event_type {
                EventType::PointerDown => self.events.push(Event::Canvas(canvas_event)),
                EventType::PointerMove |
                EventType::PointerUp   | EventType::PointerCancel => {},
                _ => self.events.push(Event::Canvas(canvas_event)),
            },
        }
        while let Some(event) = self.events.pop() {
            let (new_events, new_commands) = self.dispatch(event);
            self.events.extend(new_events);
            commands.extend(new_commands);
        }
        to_value(&commands).unwrap_or(JsValue::NULL)
    }

    fn dispatch(&mut self, event: Event) -> (Vec<Event>, Vec<Command>) {
        let Self { handler, pointer_state, .. } = self;
        match event {
            Event::Ready             => handler.initial_draw(),
            Event::Canvas(e)         => handler.process(&e, pointer_state),
            Event::Gesture(g)        => handler.process_gesture(&g, pointer_state),
            Event::Ugrid(e)          => handler.process_ugrid(&e),
        }
    }
}
