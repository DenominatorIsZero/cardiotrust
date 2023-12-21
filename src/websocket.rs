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
    let scenario = &mut scenario_list.entries[selected_scenario
        .index
        .expect("Selected scenario to be some.")]
    .scenario;
    let model = &mut scenario
        .data
        .as_mut()
        .expect("Data to be some")
        .simulation
        .as_mut()
        .expect("Simulation to be some.")
        .model;
    model.spatial_description.voxels.size_mm = payload
        .get("fVoxelSizeMm")
        .expect("Key fVoxelSizeMm should exist.")
        .as_f64()
        .expect("fVoxelSizeMm should be a float")
        as f32;
    initialize_voxel_types(model, payload);
    initialize_voxel_positions(model, payload);
    initialize_voxel_numbers(model, payload);
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
        .expect("Selected scenario should be some.")]
    .scenario;
    let model = scenario
        .results
        .as_mut()
        .expect("Results should be some.")
        .model
        .as_mut()
        .expect("Model should be some");
    model.spatial_description.voxels.size_mm = payload
        .get("fVoxelSizeMm")
        .expect("Key fVoxelSizeMm should exist.")
        .as_f64()
        .expect("fVoxelSizeMm should be a float")
        as f32;
    initialize_voxel_types(model, payload);
    initialize_voxel_positions(model, payload);
    initialize_voxel_numbers(model, payload);
}

fn initialize_voxel_types(model: &mut Model, payload: &Value) {
    let types = &mut model.spatial_description.voxels.types.values;
    let key = "pppcVoxelTypes";
    let pppc_voxel_types = payload
        .get(key)
        .unwrap_or_else(|| panic!("Key {key} should exist"))
        .as_array()
        .unwrap_or_else(|| panic!("{key} to be array."));
    for x in 0..types.shape()[0] {
        let ppc_voxel_types = pppc_voxel_types[x]
            .as_array()
            .expect("ppc_voxel_types should be an array.");
        for y in 0..types.shape()[1] {
            let pc_voxel_types = ppc_voxel_types[y]
                .as_array()
                .expect("pc_voxel_types should be an array.");
            for z in 0..types.shape()[2] {
                let voxel_type = pc_voxel_types[z].as_i64().expect("Voxeltype to be int");
                types[(x, y, z)] = match voxel_type {
                    0 => VoxelType::None,
                    1 => VoxelType::Sinoatrial,
                    2 => VoxelType::Atrium,
                    3 => VoxelType::Atrioventricular,
                    4 => VoxelType::HPS,
                    5 => VoxelType::Ventricle,
                    6 => VoxelType::Pathological,
                    _ => panic!("Voxel type should be in 0..=6"),
                }
            }
        }
    }
}

fn initialize_voxel_positions(model: &mut Model, payload: &Value) {
    let positions = &mut model.spatial_description.voxels.positions_mm.values;
    let key = "ppppfVoxelPositionsMm";
    let ppppf_voxel_positions = payload
        .get(key)
        .unwrap_or_else(|| panic!("Key {key} should exist"))
        .as_array()
        .unwrap_or_else(|| panic!("{key} to be array."));
    for x in 0..positions.shape()[0] {
        let pppf_voxel_positions = ppppf_voxel_positions[x]
            .as_array()
            .expect("pppf_voxel_positons to be array");
        for y in 0..positions.shape()[1] {
            let ppf_voxel_types = pppf_voxel_positions[y]
                .as_array()
                .expect("ppf_voxel_positions to be array");
            for z in 0..positions.shape()[2] {
                let pf_voxel_positions = ppf_voxel_types[z]
                    .as_array()
                    .expect("pf_voxel_positions to be array");
                for d in 0..3 {
                    let position = pf_voxel_positions[d]
                        .as_f64()
                        .expect("Voxel position to be float")
                        as f32;
                    positions[(x, y, z, d)] = position;
                }
            }
        }
    }
}

fn initialize_voxel_numbers(model: &mut Model, payload: &Value) {
    let numbers = &mut model.spatial_description.voxels.numbers.values;
    let key = "pppuVoxelNumbers";
    let pppu_voxel_numbers = payload
        .get(key)
        .unwrap_or_else(|| panic!("Key {key} should exist"))
        .as_array()
        .unwrap_or_else(|| panic!("{key} should be an array."));
    for x in 0..numbers.shape()[0] {
        let ppu_voxel_numbers = pppu_voxel_numbers[x]
            .as_array()
            .expect("ppu_voxel_numbers should be an array");
        for y in 0..numbers.shape()[1] {
            let pu_voxel_numbers = ppu_voxel_numbers[y]
                .as_array()
                .expect("pu_voxel_numbers should be an array");
            for z in 0..numbers.shape()[2] {
                let number = pu_voxel_numbers[z]
                    .as_i64()
                    .expect("Voxel number should be an int") as usize;
                numbers[(x, y, z)] = Some(number);
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
            .expect("Key iNumberOfSensors should exist.")
            .as_i64()
            .expect("iNumberOfSensors should be an int."),
    )
    .unwrap();
    let number_of_states = usize::try_from(
        payload
            .get("iNumberOfStates")
            .expect("Key iNumberOfStates to exist.")
            .as_i64()
            .expect("iNumberOfStates should be an int."),
    )
    .unwrap();
    let samplerate = payload
        .get("fSampleRate")
        .expect("Key fSampleRate should exist.")
        .as_f64()
        .expect("fSampleRate should be a float.") as f32;
    let number_of_steps = (samplerate) as usize;
    let number_of_epochs = 1;
    let voxels_in_dims_json = payload
        .get("piVoxelsPerDim")
        .expect("Key piVoxelsPerDim should exist.")
        .as_array()
        .expect("piVoxelsPerDim should be an array.");
    let voxels_in_dims = Dim([
        voxels_in_dims_json[0]
            .as_i64()
            .expect("voxels in dim should be an int.") as usize,
        voxels_in_dims_json[1]
            .as_i64()
            .expect("voxels in dim should be an int.") as usize,
        voxels_in_dims_json[2]
            .as_i64()
            .expect("voxels in dim should be an int.") as usize,
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
