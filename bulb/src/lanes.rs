use crate::tunic::gate;
use nectar::msg::Msg;
use nectar::spsc::Lane;
use tunic::Verb;

pub const LABEL_ECHO: u32 = 7;
pub const LABEL_LEAF: u32 = 13;
pub const LABEL_BLOOM: u32 = 42;

pub const ECHO_CAP: usize = 4;
const LANE_CAP: usize = 8;

static ECHO: Lane<ECHO_CAP> = Lane::new();
static BLOOM_LANE: Lane<LANE_CAP> = Lane::new();
static LEAF_LANE: Lane<LANE_CAP> = Lane::new();

pub fn ping(msg: Msg) -> bool {
    if !gate(tunic::LEAF, Verb::Send, LABEL_ECHO) {
        return false;
    }
    ECHO.try_send(msg).is_ok()
}

pub fn listen() -> Option<Msg> {
    if !gate(tunic::LEAF, Verb::Recv, LABEL_ECHO) {
        return None;
    }
    ECHO.try_recv().ok()
}

pub fn bloom_say(msg: Msg) -> bool {
    if !gate(tunic::BLOOM, Verb::Send, LABEL_BLOOM) {
        return false;
    }
    BLOOM_LANE.try_send(msg).is_ok()
}

pub fn from_bloom() -> Option<Msg> {
    if !gate(tunic::LEAF, Verb::Recv, LABEL_BLOOM) {
        return None;
    }
    BLOOM_LANE.try_recv().ok()
}

pub fn leaf_say(msg: Msg) -> bool {
    if !gate(tunic::LEAF, Verb::Send, LABEL_LEAF) {
        return false;
    }
    LEAF_LANE.try_send(msg).is_ok()
}

pub fn from_leaf() -> Option<Msg> {
    if !gate(tunic::BLOOM, Verb::Recv, LABEL_LEAF) {
        return None;
    }
    LEAF_LANE.try_recv().ok()
}
