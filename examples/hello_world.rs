use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;
use bevy_replicon_request::{prelude::*, RequestHandler, ResponseEvent};
use serde::{Deserialize, Serialize};
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            RepliconPlugins,
            RepliconRenetPlugins,
            protocol_plugin,
        ))
        .add_systems(
            Update,
            (
                open_pod_bay_doors.run_if(has_authority),
                send_request.run_if(on_timer(Duration::from_secs(1))),
                recv_response,
            ),
        )
        .run();
}

fn protocol_plugin(app: &mut App) {
    app.add_client_request::<OpenPodBayDoors>();
}

/// This runs on the server and handles incoming client requests.
fn open_pod_bay_doors(mut requests: RequestHandler<OpenPodBayDoors>) {
    requests.handle_requests(|_request| Err(MutinyError));
}

/// Asks Hal to open the pod bay doors.
fn send_request(mut sender: RequestSender<OpenPodBayDoors>) {
    let response_index = sender.send(OpenPodBayDoors);
    println!("Requesting Hal open pod bay doors, response will have index {response_index}");
}

/// Logs Hal's responses.
fn recv_response(mut receiver: EventReader<ResponseEvent<OpenPodBayDoors>>) {
    for response in receiver.read() {
        println!(
            "Received a response to request {}: {:?}",
            response.index(),
            response.response
        );
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct OpenPodBayDoors;

impl Request for OpenPodBayDoors {
    type Response = Result<(), MutinyError>;
}

#[derive(Serialize, Deserialize)]
struct MutinyError;

impl std::fmt::Debug for MutinyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MutinyError")
            .field(&"I'm sorry, Dave. I'm afraid I can't do that.")
            .finish()
    }
}
