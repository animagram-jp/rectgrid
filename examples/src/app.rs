use serde::Serialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::{JsValue, prelude::*};

use crate::{
    event::{Event, Handler},
    js_client::{CanvasEvent, Command, EventType, Thresholds, TouchTracker, detect_device},
};

// ============================================================
// App
// ============================================================

#[wasm_bindgen]
pub struct App {
    touch:      TouchTracker,
    thresholds: Thresholds,
    events:     Vec<Event>,
    handler:    Handler,
}

#[wasm_bindgen]
impl App {
    pub fn init(
        pointer_coarse: bool,
        viewport_width_px: f64,
        _viewport_height_px: f64,
        section_origin_x: f64,
        section_origin_y: f64,
    ) -> App {
        let mut app = App {
            touch:      TouchTracker::default(),
            thresholds: Thresholds::for_device(detect_device(pointer_coarse)),
            events:     Vec::new(),
            handler:    Handler::new(viewport_width_px, [section_origin_x, section_origin_y]),
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
        match self.touch.handle(
            &canvas_event.event_type,
            canvas_event.pointer_id,
            canvas_event.x,
            canvas_event.y,
            canvas_event.time,
            &self.thresholds,
        ) {
            Some(gesture) => self.events.push(Event::Gesture(gesture)),
            None => match &canvas_event.event_type {
                EventType::PointerDown => self.events.push(Event::Canvas(canvas_event)),
                EventType::PointerMove | EventType::PointerUp | EventType::PointerCancel => {}
                _ => self.events.push(Event::Canvas(canvas_event)),
            },
        }
        while let Some(event) = self.events.pop() {
            let (new_events, new_commands) = self.dispatch(event);
            self.events.extend(new_events);
            commands.extend(new_commands);
        }
        // Command::serialize uses serialize_map; force plain JS objects (not Map) so init.js's cmd.field access works.
        let serializer = Serializer::new().serialize_maps_as_objects(true);
        commands.serialize(&serializer).unwrap_or(JsValue::NULL)
    }

    fn dispatch(&mut self, event: Event) -> (Vec<Event>, Vec<Command>) {
        let Self { handler, touch, .. } = self;
        match event {
            Event::Ready => handler.initial_draw(),
            Event::Canvas(e) => handler.process(&e, touch.active_state()),
            Event::Gesture(g) => handler.process_gesture(&g, touch.active_state()),
            Event::Rectgrid(e) => handler.process_rectgrid(&e),
        }
    }
}
