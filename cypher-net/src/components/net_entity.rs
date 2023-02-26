use bevy::prelude::Component;
use std::sync::Mutex;

pub type NetEntityT = u64;

static NEXT_NET_ENTITY_ID: Mutex<NetEntityT> = Mutex::new(0);

/// Gets the next NetEntity to use.
fn next() -> NetEntityT {
    let mut next = NEXT_NET_ENTITY_ID.lock().unwrap();
    *next += 1;
    *next
}

#[derive(Component)]
pub struct NetEntity {
    pub id: NetEntityT,
}

impl Default for NetEntity {
    fn default() -> Self {
        Self { id: next() }
    }
}
