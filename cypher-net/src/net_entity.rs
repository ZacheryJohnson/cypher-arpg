use std::sync::Mutex;

pub type NetEntityT = u64;

static NEXT_NET_ENTITY_ID: Mutex<NetEntityT> = Mutex::new(0);

/// Gets the next NetEntity to use.
pub fn next() -> NetEntityT {
    let mut next = NEXT_NET_ENTITY_ID.lock().unwrap();
    *next += 1;
    *next
}
