// ZJ-TODO: refactor this class
// ideally, we have a sole Client resource that contains the RenetClient + state like this

use bevy::prelude::Resource;

#[derive(Default, Debug, Resource)]
pub struct ClientState {
    pub client_id: u64,
}
