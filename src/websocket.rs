use bevy::{prelude::*, time::common_conditions::on_timer};

use ndarray::{arr1, Dim};
use serde_json::Value;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

use crate::{
    core::{
        data::Data,
        model::{spatial::voxels::VoxelType, Model},
        scenario::{self, results::Results, Scenario},
    },
    vis::sample_tracker::{self, SampleTracker},
    ScenarioBundle, ScenarioList, SelectedSenario,
};

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
fn handle_websocket_messages(
    message_buffer: Res<MessageBuffer>,
    mut sample_tracker: ResMut<SampleTracker>,
    mut selected_scenario: ResMut<SelectedSenario>,
    mut scenario_list: ResMut<ScenarioList>,
) {
    if let Some(message) = message_buffer.messages.lock().unwrap().pop() {
        let message: Value = serde_json::from_str(&message).unwrap();
        message.get("h").map_or_else(
            || {
                error!("Cannot process message. Missing header: {message}");
            },
            |header| {
                let header = header.to_string();
                info!("Processing message with header {header}");
                let payload = message.get("p").unwrap();
                let id = payload.get("id").unwrap();
                info!("Message came with id {id}");
                match header.as_str() {
                    r#""INIT_EST""# => handle_init_est_message(
                        payload,
                        &mut sample_tracker,
                        &mut selected_scenario,
                        &mut scenario_list,
                    ),
                    r#""INIT_SIM""# => handle_init_sim_message(
                        payload,
                        &mut sample_tracker,
                        &mut selected_scenario,
                        &mut scenario_list,
                    ),
                    _ => info!("Do not know header {header}"),
                }
            },
        );
    }
}

fn handle_init_sim_message(
    payload: &Value,
    sample_tracker: &mut SampleTracker,
    selected_scenario: &mut SelectedSenario,
    scenario_list: &mut ScenarioList,
) {
    info!("Initializing simulation struct.");
    if selected_scenario.index.is_none() {
        init_scenario(payload, selected_scenario, scenario_list, sample_tracker);
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn handle_init_est_message(
    payload: &Value,
    sample_tracker: &mut SampleTracker,
    selected_scenario: &mut SelectedSenario,
    scenario_list: &mut ScenarioList,
) {
    info!("Initializing estimation struct.");
    if selected_scenario.index.is_none() {
        init_scenario(payload, selected_scenario, scenario_list, sample_tracker);
    }
    let scenario = &mut scenario_list.entries[selected_scenario
        .index
        .expect("Selected scenario to be some.")]
    .scenario;
    let pppc_voxel_types = payload
        .get("pppcVoxelTypes")
        .expect("Key to exist")
        .as_array()
        .expect("Value to be array.");
    let model = scenario
        .results
        .as_mut()
        .expect("Results to be some.")
        .model
        .as_mut()
        .expect("Model to be some");
    let types = &mut model.spatial_description.voxels.types.values;
    for x in 0..types.shape()[0] {
        let ppc_voxel_types = pppc_voxel_types[x].as_array().expect("Value to be array");
        for y in 0..types.shape()[1] {
            let pc_voxel_types = ppc_voxel_types[y].as_array().expect("Value to be array");
            for z in 0..types.shape()[2] {
                let voxel_type = pc_voxel_types[z].as_i64().expect("Value to be int");
                types[(x, y, z)] = match voxel_type {
                    0 => VoxelType::None,
                    1 => VoxelType::Sinoatrial,
                    2 => VoxelType::Atrium,
                    3 => VoxelType::Atrioventricular,
                    4 => VoxelType::HPS,
                    5 => VoxelType::Ventricle,
                    6 => VoxelType::Pathological,
                    _ => panic!("Got unexpected voxel type."),
                }
            }
        }
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn init_scenario(
    payload: &Value,
    selected_scenario: &mut SelectedSenario,
    scenario_list: &mut ScenarioList,
    sample_tracker: &mut SampleTracker,
) {
    let number_of_sensors = usize::try_from(
        payload
            .get("iNumberOfSensors")
            .expect("Key to exist.")
            .as_i64()
            .expect("Value to be int."),
    )
    .unwrap();
    let number_of_states = usize::try_from(
        payload
            .get("iNumberOfStates")
            .expect("Key to exist.")
            .as_i64()
            .expect("Value to be int."),
    )
    .unwrap();
    let samplerate = payload
        .get("fSampleRate")
        .expect("Key to exist.")
        .as_f64()
        .expect("Value to be float.") as f32;
    let number_of_steps = (samplerate) as usize;
    let number_of_epochs = 1;
    let voxels_in_dims_json = payload
        .get("piVoxelsPerDim")
        .expect("Key to exist.")
        .as_array()
        .expect("Value to be int.");
    let voxels_in_dims = Dim([
        voxels_in_dims_json[0].as_i64().expect("Value to be int.") as usize,
        voxels_in_dims_json[1].as_i64().expect("Value to be int.") as usize,
        voxels_in_dims_json[2].as_i64().expect("Value to be int.") as usize,
    ]);
    selected_scenario.index = Some(0);
    let mut scenario = Scenario::empty();
    let data = Data::empty(
        number_of_sensors,
        number_of_states,
        number_of_steps,
        voxels_in_dims,
    );
    let mut results = Results::new(
        number_of_epochs,
        number_of_steps,
        number_of_sensors,
        number_of_states,
    );
    let est_model = Model::empty(
        number_of_states,
        number_of_sensors,
        number_of_steps,
        voxels_in_dims,
    );

    scenario.data = Some(data);
    results.model = Some(est_model);
    scenario.results = Some(results);
    let bundle = ScenarioBundle {
        scenario,
        join_handle: None,
        epoch_rx: None,
        summary_rx: None,
    };
    scenario_list.entries.push(bundle);
    sample_tracker.sample_rate = samplerate;
    sample_tracker.max_sample = number_of_steps;
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
