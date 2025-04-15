use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub pusd_manager: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ChainSetting {
    pub job_id: String,
}

pub const STATE: Item<State> = Item::new("state");
pub const CHAIN_SETTINGS: Map<String, ChainSetting> = Map::new("chain_settings");
