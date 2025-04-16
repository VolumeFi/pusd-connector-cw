use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, CustomMsg, Uint128, Uint256};

use crate::state::{ChainSetting, State};

#[cw_serde]
pub struct InstantiateMsg {
    pub pusd_manager: Addr,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    // Register Jobs in hash map with chain_id as key and job_id as value
    RegisterChain {
        chain_id: String,
        chain_setting: ChainSetting,
    },
    SendPusd {
        chain_id: String,
        to: String,
        amount: Uint128,
        nonce: Uint128,
    },
    WithdrawPusd {
        chain_id: String,
        recipient: String,
        amount: Uint128,
    },
    ChangeConfig {
        owner: Option<Addr>,
        pusd_manager: Option<Addr>,
    },
    UpdateWithdrawLimit {
        chain_id: String,
        new_withdraw_limit: Uint256,
    },
    // Set Paloma address of a chain
    SetPaloma {
        chain_id: String,
    },
    // Update Refund Wallet
    UpdateRefundWallet {
        chain_id: String,
        new_refund_wallet: String,
    },
    // Update Gas Fee
    UpdateGasFee {
        chain_id: String,
        new_gas_fee: Uint256,
    },
    // Update Service Fee Collector
    UpdateServiceFeeCollector {
        chain_id: String,
        new_service_fee_collector: String,
    },
    // Update Service Fee
    UpdateServiceFee {
        chain_id: String,
        new_service_fee: Uint256,
    },
    // Update Pusd
    UpdatePusd {
        chain_id: String,
        new_pusd: String,
    },
    // Update Pusd Manager
    UpdatePusdManager {
        chain_id: String,
        new_pusd_manager: String,
    },
}

#[cw_serde]
pub enum ExternalExecuteMsg {
    Withdraw { chain_id: String, recipient: String },
    // ReWithdraw PUSD by nonce
    ReWithdraw { nonce: u64 },
    // Cancel Withdraw by nonce
    CancelWithdraw { nonce: u64 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(State)]
    GetState {},

    #[returns(Vec<ChainSettingInfo>)]
    GetChainSettings {},
}

#[cw_serde]
pub enum PalomaMsg {
    /// Message struct for tokenfactory calls.
    SkywayMsg {
        send_tx: Option<SendTx>,
        cancel_tx: Option<CancelTx>,
    },
    /// Message struct for cross-chain calls.
    SchedulerMsg { execute_job: ExecuteJob },
}

#[cw_serde]
pub struct ExecuteJob {
    pub job_id: String,
    pub payload: Binary,
}

#[cw_serde]
pub struct SendTx {
    pub remote_chain_destination_address: String,
    pub amount: String,
    pub chain_reference_id: String,
}

#[cw_serde]
pub struct CancelTx {
    pub transaction_id: u64,
}

#[cw_serde]
pub struct ChainSettingInfo {
    pub chain_id: String,
    pub job_id: String,
}

impl CustomMsg for PalomaMsg {}
