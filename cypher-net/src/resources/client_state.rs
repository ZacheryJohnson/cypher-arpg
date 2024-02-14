// ZJ-TODO: refactor this class
// ideally, we have a sole Client resource that contains the RenetClient + state like this

use bevy::prelude::Resource;
use bevy_renet::renet::ClientId;

#[derive(Debug, Resource)]
pub struct ClientState {
    pub client_id: ClientId,
}
