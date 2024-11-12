use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy::{prelude::*, time::common_conditions::on_timer};
use ndarray::Dim;
use serde_json::Value;
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

use crate::{
    core::{
        algorithm::refinement::Optimizer,
        data::{
            shapes::{Measurements, SystemStates},
            Data,
        },
        model::{spatial::voxels::VoxelType, Model},
        scenario::{calculate_plotting_arrays, results::Results, Scenario},
    },
    vis::{options::ColorOptions, sample_tracker::SampleTracker},
    ScenarioBundle, ScenarioList, SelectedSenario,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct WebsocketPlugin;

impl Plugin for WebsocketPlugin {
    #[tracing::instrument(level = "info", skip(app))]
    fn build(&self, app: &mut App) {
        info!("Initializing websocket plugin.");
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
    #[tracing::instrument(level = "debug")]
    fn default() -> Self {
        debug!("Initializing message buffer.");
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[derive(Debug)]
struct WebSocketWrapper {
    pub websocket: WebSocket,
}

#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for WebSocketWrapper {}
unsafe impl Sync for WebSocketWrapper {}

#[derive(Resource, Default, Debug)]
struct WebSocketResource {
    pub websocket: Option<WebSocketWrapper>,
}

/// Sends a heartbeat message over the websocket connection in `websocket_resource`
/// if it is in a ready state. This keeps the connection alive.
#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "debug")]
fn send_heartbeat(websocket_resource: Res<WebSocketResource>) {
    debug!("Sending heartbeat.");
    if let Some(websocket) = websocket_resource.websocket.as_ref() {
        if websocket.websocket.ready_state() == 1 {
            websocket
                .websocket
                .send_with_str(r#"{"h": "HBT", "p": {}}"#)
                .unwrap();
        }
    }
}

/// Handles incoming websocket messages by processing the message header and
/// payload. The header determines which handler function is called. The handlers
/// update the simulation state and visualizations.
#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "warn")]
fn handle_websocket_messages(
    message_buffer: Res<MessageBuffer>,
    mut sample_tracker: ResMut<SampleTracker>,
    mut selected_scenario: ResMut<SelectedSenario>,
    mut scenario_list: ResMut<ScenarioList>,
    mut vis_options: ResMut<ColorOptions>,
) {
    trace!("Running handle_websocket_messages system");
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
                    r#""UPDATE_SIM""# => {
                        handle_update_sim_message(payload, &selected_scenario, &mut scenario_list);
                        vis_options.set_changed();
                    }
                    r#""UPDATE_EST""# => {
                        handle_update_est_message(payload, &selected_scenario, &mut scenario_list);
                        vis_options.set_changed();
                    }
                    _ => warn!("Do not know header {header}"),
                }
            },
        );
    }
}

/// Initializes the simulation struct from the payload of an `INIT_SIM` websocket message.
/// Sets voxel size, initializes voxel types, positions, numbers, and sensor positions/orientations.
/// Requires that the selected scenario is already set.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
#[tracing::instrument(level = "info", skip_all)]
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
    initialize_sensor_positions(model, payload);
    initialize_sensor_orientations(model, payload);
}

/// Initializes the estimation struct from the payload of an `INIT_EST` websocket message.
/// Sets voxel size, initializes voxel types, positions, numbers, and sensor positions/orientations.
/// Requires that the selected scenario is already set.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
#[tracing::instrument(level = "info", skip_all)]
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
    initialize_sensor_positions(model, payload);
    initialize_sensor_orientations(model, payload);
}

/// Updates the simulation values from the payload of an `UPDATE_SIM`
/// websocket message. Updates the system states and measurements
/// in the selected scenario's simulation struct.
#[tracing::instrument(level = "debug", skip_all)]
fn handle_update_sim_message(
    payload: &Value,
    selected_scenario: &SelectedSenario,
    scenario_list: &mut ScenarioList,
) {
    debug!("Updateing simulation values.");
    let scenario = &mut scenario_list.entries[selected_scenario
        .index
        .expect("Selected scenario to be some.")]
    .scenario;
    let simulation = &mut scenario
        .data
        .as_mut()
        .expect("Data should be some")
        .simulation;
    let states = &mut simulation.system_states;
    let measurements = &mut simulation.measurements;

    update_values(payload, states, measurements);
    simulation.calculate_plotting_arrays();
}

/// Updates the estimation values from the payload of an `UPDATE_EST`
/// websocket message. Updates the system states and measurements
/// in the selected scenario's estimation struct.
#[tracing::instrument(level = "debug", skip_all)]
fn handle_update_est_message(
    payload: &Value,
    selected_scenario: &SelectedSenario,
    scenario_list: &mut ScenarioList,
) {
    debug!("Updateing estimation values.");
    let scenario = &mut scenario_list.entries[selected_scenario
        .index
        .expect("Selected scenario to be some.")]
    .scenario;
    let results = scenario.results.as_mut().expect("Results should be some.");
    let data = scenario.data.as_ref().expect("Data should be some");
    let states = &mut results.estimations.system_states;
    let measurements = &mut results.estimations.measurements;

    update_values(payload, states, measurements);

    calculate_plotting_arrays(results, data);
}

/// Updates the system state and measurement values in the provided
/// `ArraySystemStates` and `ArrayMeasurements` from the given payload.
/// Extracts the state and measurement values from the "ppfStatesToExoBuffer"
/// and "ppfMeasurementsToExoBuffer" keys in the payload.
#[allow(
    clippy::similar_names,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
#[tracing::instrument(level = "debug")]
fn update_values(payload: &Value, states: &mut SystemStates, measurements: &mut Measurements) {
    debug!("Updating values.");
    let key = "ppfStatesToExoBuffer";
    let ppf_state_buffer = payload
        .get(key)
        .unwrap_or_else(|| panic!("Key {key} should exist"))
        .as_array()
        .unwrap_or_else(|| panic!("{key} to be array."));
    for index_sample in 0..states.num_steps() {
        let pf_state_buffer = ppf_state_buffer[index_sample]
            .as_array()
            .expect("pfStateBuffer should be an array.");
        for index_state in 0..states.num_states() {
            let state = pf_state_buffer[index_state]
                .as_f64()
                .expect("State should be a float");
            states[(index_sample, index_state)] = state as f32;
        }
    }
    let key = "ppfMeasurementsToExoBuffer";
    let ppf_measurement_buffer = payload
        .get(key)
        .unwrap_or_else(|| panic!("Key {key} should exist"))
        .as_array()
        .unwrap_or_else(|| panic!("{key} to be array."));
    for index_sample in 0..measurements.num_steps() {
        let pf_measurement_buffer = ppf_measurement_buffer[index_sample]
            .as_array()
            .expect("pfMeasurementBuffer should be an array.");
        for index_sensor in 0..measurements.num_sensors() {
            let state = pf_measurement_buffer[index_sensor]
                .as_f64()
                .expect("Measurement should be a float");
            measurements[(0, index_sample, index_sensor)] = state as f32;
        }
    }
}

/// Initializes the voxel types in the model by mapping values from the
/// "pppcVoxelTypes" key in the payload to `VoxelType` enum variants.
#[allow(clippy::similar_names)]
#[tracing::instrument(level = "debug", skip_all)]
fn initialize_voxel_types(model: &mut Model, payload: &Value) {
    debug!("Initializing voxel types.");
    let types = &mut model.spatial_description.voxels.types;
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

/// Initializes voxel positions in the model by mapping values from the
/// "ppppfVoxelPositionsMm" key in the payload to the positions field of the
/// voxels struct. Loops through the nested arrays in the payload to populate
/// the 4D positions array.
#[allow(
    clippy::similar_names,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
#[tracing::instrument(level = "debug", skip_all)]
fn initialize_voxel_positions(model: &mut Model, payload: &Value) {
    debug!("Initializing voxel positions.");
    let positions = &mut model.spatial_description.voxels.positions_mm;
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

/// Initializes voxel numbers in the model by mapping values from the
/// "pppuVoxelNumbers" key in the payload to the numbers field of the
/// voxels struct. Loops through the nested arrays in the payload to populate
/// the 3D numbers array.
#[allow(
    clippy::similar_names,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
#[tracing::instrument(level = "debug", skip_all)]
fn initialize_voxel_numbers(model: &mut Model, payload: &Value) {
    debug!("Initializing voxel numbers.");
    let numbers = &mut model.spatial_description.voxels.numbers;
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

/// Initializes sensor positions in the model by mapping values from the
/// "ppfSensorPositionsMm" key in the payload to the positions field of the
/// sensors struct. Loops through the nested arrays in the payload to populate  
/// the 3D positions array.
#[allow(
    clippy::similar_names,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]
#[tracing::instrument(level = "debug", skip_all)]
fn initialize_sensor_positions(model: &mut Model, payload: &Value) {
    debug!("Initializing sensor positions.");
    let positions = &mut model.spatial_description.sensors.positions_mm;
    let key = "ppfSensorPositionsMm";
    let ppf_sensor_positions_mm = payload
        .get(key)
        .unwrap_or_else(|| panic!("Key {key} should exist"))
        .as_array()
        .unwrap_or_else(|| panic!("{key} to be array."));
    for i in 0..positions.shape()[0] {
        let pf_sensor_position_mm = ppf_sensor_positions_mm[i]
            .as_array()
            .expect("ppf_sensor_positons to be array");
        for d in 0..3 {
            let position = pf_sensor_position_mm[d]
                .as_f64()
                .expect("Sensor position to be float") as f32;
            positions[(i, d)] = position;
        }
    }
}
/// Initializes sensor orientations in the model by mapping values from the
/// "ppfSensorOrientations" key in the payload to the orientations field of the
/// sensors struct. Loops through the nested arrays in the payload to populate
/// the 3D orientations array.
#[allow(
    clippy::similar_names,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]
#[tracing::instrument(level = "debug", skip_all)]
fn initialize_sensor_orientations(model: &mut Model, payload: &Value) {
    debug!("Initializing sensor orientations.");
    let orientations = &mut model.spatial_description.sensors.orientations_xyz;
    let key = "ppfSensorOrientations";
    let ppf_sensor_orientations = payload
        .get(key)
        .unwrap_or_else(|| panic!("Key {key} should exist"))
        .as_array()
        .unwrap_or_else(|| panic!("{key} to be array."));
    for i in 0..orientations.shape()[0] {
        let pf_sensor_orientation = ppf_sensor_orientations[i]
            .as_array()
            .expect("ppf_sensor_orientations to be array");
        for d in 0..3 {
            let orientation = pf_sensor_orientation[d]
                .as_f64()
                .expect("Sensor orientation to be float") as f32;
            orientations[(i, d)] = orientation;
        }
    }
}

/// Initializes a scenario by mapping values from the payload to a new
/// Scenario struct. Extracts the number of sensors, states, sample rate,
/// etc. and uses them to construct a new Scenario with empty Data and
/// Results structs. Also adds the scenario to the scenario list and
/// initializes the sample tracker.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
#[tracing::instrument(level = "debug", skip_all)]
fn init_scenario(
    payload: &Value,
    selected_scenario: &mut SelectedSenario,
    scenario_list: &mut ScenarioList,
    sample_tracker: &mut SampleTracker,
) {
    debug!("Initializing scenario struct.");
    let number_of_sensors = usize::try_from(
        payload
            .get("iNumberOfSensors")
            .expect("Key iNumberOfSensors should exist.")
            .as_i64()
            .expect("iNumberOfSensors should be an int."),
    )
    .unwrap();
    info!("Number of sensors: {number_of_sensors}");
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
        1,
    );
    let mut results = Results::new(
        number_of_epochs,
        number_of_steps,
        number_of_sensors,
        number_of_states,
        1,
        1,
        Optimizer::Sgd,
    );
    let est_model = Model::empty(
        number_of_states,
        number_of_sensors,
        number_of_steps,
        voxels_in_dims,
        1,
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

/// Initializes a `WebSocket` connection and sets up callbacks to handle
/// incoming messages and connection events.
///
/// Creates a new `WebSocket` connection to the specified URL. Configures the
/// `WebSocket` to us`ArrayBuffers`rs for binary messages. Sets up callbacks to
/// handle incoming messages, errors, and open events. The message callback
/// pushes received messages onto the shared message buffer. The open callback
/// sends an initial message over the socket. Stores the `WebSocket` in the
/// `WebSocketResource`.
#[allow(clippy::needless_pass_by_value)]
#[tracing::instrument(level = "info", skip_all)]
fn init_websocket(
    message_buffer: Res<MessageBuffer>,
    mut websocket_resource: ResMut<WebSocketResource>,
) {
    info!("Initializing WebSocket.");
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
            Ok(()) => info!("message successfully sent"),
            Err(err) => error!("error sending message: {:?}", err),
        }
    });
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();
    websocket_resource.websocket = Some(WebSocketWrapper { websocket: ws });
}
