use std::env;

use simulation::SimulationMode;

pub mod simulation;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    let client_only = args.binary_search(&String::from("client")).is_ok();
    let server_only = args.binary_search(&String::from("server")).is_ok();

    println!("Client-only? {client_only} Server-only? {server_only}");

    let mode = match (client_only, server_only) {
        (true, true) => panic!("only one of 'client' and 'server' arguments can be present. Provide no args to instead use a local simulation"),
        (true, false) => SimulationMode::ClientOnly,
        (false, true) => SimulationMode::ServerOnly,
        (false, false) => SimulationMode::ClientAndServer,
    };

    simulation::start(mode);
}
