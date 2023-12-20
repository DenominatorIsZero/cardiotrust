use bevy::{prelude::*, time::common_conditions::on_timer};

use serde_json::Value;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

#[allow(clippy::module_name_repetitions)]
pub struct WebsocketPlugin;

impl Plugin for WebsocketPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MessageBuffer>()
            .init_resource::<WebSocketResource>()
            .add_systems(Startup, init_websocket)
            .add_systems(
                Update,
                (
                    handle_websocket_messages,
                    send_heartbeat.run_if(on_timer(Duration::from_secs(1))),
                ),
            );
    }
}

#[derive(Resource, Debug)]
struct MessageBuffer {
    pub messages: Arc<Mutex<Vec<String>>>,
}

impl Default for MessageBuffer {
    fn default() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

struct WebSocketWrapper {
    pub websocket: WebSocket,
}

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for WebSocketWrapper {}
unsafe impl Sync for WebSocketWrapper {}

#[derive(Resource, Default)]
struct WebSocketResource {
    pub websocket: Option<WebSocketWrapper>,
}

#[allow(clippy::needless_pass_by_value)]
fn send_heartbeat(websocket_resource: Res<WebSocketResource>) {
    if let Some(websocket) = websocket_resource.websocket.as_ref() {
        if websocket.websocket.ready_state() == 1 {
            websocket
                .websocket
                .send_with_str(r#"{"h": "HBT", "p": {}}"#)
                .unwrap();
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_websocket_messages(message_buffer: Res<MessageBuffer>) {
    if let Some(message) = message_buffer.messages.lock().unwrap().pop() {
        let message: Value = serde_json::from_str(&message).unwrap();
        message.get("h").map_or_else(
            || {
                error!("Cannot process message. Missing header: {message}");
            },
            |header| {
                info!("Processing message with header {header}");
                let payload = message.get("p").unwrap();
                let id = payload.get("id").unwrap();
                info!("Message came with id {id}");
            },
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
fn init_websocket(
    message_buffer: Res<MessageBuffer>,
    mut websocket_resource: ResMut<WebSocketResource>,
) {
    // Connect to an echo server
    let ws = WebSocket::new("ws://127.0.0.1:3774/").unwrap();
    // For small binary messages, like CBOR, Arraybuffer is more efficient than Blob handling
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
    // create callback
    let _cloned_ws = ws.clone();
    let cloned_message_buffer = message_buffer.messages.clone();
    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        e.data().dyn_into::<js_sys::JsString>().map_or_else(
            |_| {
                error!("message event, received Unknown: {:?}", e.data());
            },
            |txt| {
                cloned_message_buffer.lock().unwrap().push(txt.into());
            },
        );
    });
    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
        error!("error event: {:?}", e);
    });
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let cloned_ws = ws.clone();
    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        info!("socket opened");
        match cloned_ws.send_with_str(r#"{"h": "SETP", "p": "CARDIO"}"#) {
            Ok(_) => info!("message successfully sent"),
            Err(err) => error!("error sending message: {:?}", err),
        }
    });
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();
    websocket_resource.websocket = Some(WebSocketWrapper { websocket: ws });
}
