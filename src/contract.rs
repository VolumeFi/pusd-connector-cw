#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ChainSettingInfo, ExecuteMsg, InstantiateMsg, MigrateMsg, PalomaMsg, QueryMsg};
use crate::state::{State, CHAIN_SETTINGS, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:pusd-connector-cw";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        pusd_manager: msg.pusd_manager.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("pusd_manager", msg.pusd_manager))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<PalomaMsg>, ContractError> {
    match msg {
        ExecuteMsg::RegisterChain {
            chain_id,
            chain_setting,
        } => execute::register_chain(deps, info, chain_id, chain_setting),
        ExecuteMsg::SendPusd {
            chain_id,
            to,
            amount,
            nonce,
        } => execute::send_pusd(deps, info, chain_id, to, amount, nonce),
        ExecuteMsg::WithdrawPusd {
            chain_id,
            recipient,
            amount,
        } => execute::withdraw_pusd(deps, info, chain_id, recipient, amount),
        ExecuteMsg::ChangeConfig {
            owner,
            pusd_manager,
        } => execute::change_config(deps, info, owner, pusd_manager),
        ExecuteMsg::SetPaloma { chain_id } => execute::set_paloma(deps, info, chain_id),
        ExecuteMsg::UpdateWithdrawLimit {
            chain_id,
            new_withdraw_limit,
        } => execute::update_withdraw_limit(deps, info, chain_id, new_withdraw_limit),
        ExecuteMsg::UpdatePusd { chain_id, new_pusd } => {
            execute::update_pusd(deps, info, chain_id, new_pusd)
        }
        ExecuteMsg::UpdatePusdManager {
            chain_id,
            new_pusd_manager,
        } => execute::update_pusd_manager(deps, info, chain_id, new_pusd_manager),
        ExecuteMsg::UpdateRefundWallet {
            chain_id,
            new_refund_wallet,
        } => execute::update_refund_wallet(deps, info, chain_id, new_refund_wallet),
        ExecuteMsg::UpdateGasFee {
            chain_id,
            new_gas_fee,
        } => execute::update_gas_fee(deps, info, chain_id, new_gas_fee),
        ExecuteMsg::UpdateServiceFeeCollector {
            chain_id,
            new_service_fee_collector,
        } => execute::update_service_fee_collector(deps, info, chain_id, new_service_fee_collector),
        ExecuteMsg::UpdateServiceFee {
            chain_id,
            new_service_fee,
        } => execute::update_service_fee(deps, info, chain_id, new_service_fee),
    }
}

pub mod execute {
    use std::collections::BTreeMap;

    use cosmwasm_std::{Addr, Coin, CosmosMsg, Uint128, Uint256, WasmMsg};
    use ethabi::{Address, Contract, Function, Param, ParamType, StateMutability, Token, Uint};

    use super::*;
    use crate::{
        msg::{ExecuteJob, ExternalExecuteMsg, PalomaMsg, SendTx},
        state::{ChainSetting, CHAIN_SETTINGS},
    };
    use std::str::FromStr;

    pub fn register_chain(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
        chain_setting: ChainSetting,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        CHAIN_SETTINGS.save(deps.storage, chain_id.clone(), &chain_setting)?;
        Ok(Response::new()
            .add_attribute("action", "register_chain")
            .add_attribute("chain_id", chain_id))
    }

    pub fn send_pusd(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
        to: String,
        amount: Uint128,
        nonce: Uint128,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        let amount: String =
            amount.to_string() + "factory/" + state.pusd_manager.as_str() + "/upusd";

        let response = Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SkywayMsg {
                send_tx: Some(SendTx {
                    remote_chain_destination_address: to.clone(),
                    amount: amount.clone(),
                    chain_reference_id: chain_id.clone(),
                }),
                cancel_tx: None,
            }))
            .add_attribute("action", "send_pusd")
            .add_attribute("chain_id", chain_id)
            .add_attribute("to", to)
            .add_attribute("amount", amount.to_string())
            .add_attribute("nonce", nonce.to_string());

        Ok(response)
    }

    pub fn withdraw_pusd(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        let pusd_manager = state.pusd_manager;
        let pusd_denom: String = "factory/".to_string() + pusd_manager.as_str() + "/upusd";
        Ok(Response::new()
            .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: pusd_manager.to_string(),
                msg: to_json_binary(&ExternalExecuteMsg::Withdraw {
                    chain_id,
                    recipient,
                })?,
                funds: vec![Coin {
                    denom: pusd_denom,
                    amount,
                }],
            }))
            .add_attribute("action", "withdraw_pusd"))
    }

    pub fn change_config(
        deps: DepsMut,
        info: MessageInfo,
        owner: Option<Addr>,
        pusd_manager: Option<Addr>,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let mut state = STATE.load(deps.storage)?;
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        let mut response: Response<PalomaMsg> =
            Response::new().add_attribute("action", "change_config");

        if let Some(owner) = owner {
            state.owner = owner.clone();
            response = response.add_attribute("new_owner", owner.to_string());
        }
        if let Some(pusd_manager) = pusd_manager {
            state.pusd_manager = pusd_manager.clone();
            response = response.add_attribute("new_pusd_manager", pusd_manager.to_string());
        }
        STATE.save(deps.storage, &state)?;
        Ok(response)
    }

    pub fn set_paloma(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        // ACTION: Implement SetPaloma
        let state = STATE.load(deps.storage)?;
        assert!(info.sender == state.owner, "Unauthorized");

        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "set_paloma".to_string(),
                vec![Function {
                    name: "set_paloma".to_string(),
                    inputs: vec![],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: CHAIN_SETTINGS.load(deps.storage, chain_id.clone())?.job_id,
                    payload: Binary::new(
                        contract
                            .function("set_paloma")
                            .unwrap()
                            .encode_input(&[])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "set_paloma"))
    }

    pub fn update_withdraw_limit(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
        new_withdraw_limit: Uint256,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        // ACTION: Implement UpdateWithdrawLimit
        let state = STATE.load(deps.storage)?;
        assert!(info.sender == state.owner, "Unauthorized");
        let new_withdraw_limit: Uint = Uint::from_big_endian(&new_withdraw_limit.to_be_bytes());
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_withdraw_limit".to_string(),
                vec![Function {
                    name: "update_withdraw_limit".to_string(),
                    inputs: vec![Param {
                        name: "new_withdraw_limit".to_string(),
                        kind: ParamType::Uint(256),
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: CHAIN_SETTINGS.load(deps.storage, chain_id.clone())?.job_id,
                    payload: Binary::new(
                        contract
                            .function("update_withdraw_limit")
                            .unwrap()
                            .encode_input(&[Token::Uint(new_withdraw_limit)])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_withdraw_limit"))
    }

    pub fn update_pusd(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
        new_pusd: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        // ACTION: Implement UpdatePusd
        let state = STATE.load(deps.storage)?;
        assert!(info.sender == state.owner, "Unauthorized");
        let new_pusd_address: Address = Address::from_str(new_pusd.as_str()).unwrap();
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_pusd".to_string(),
                vec![Function {
                    name: "update_pusd".to_string(),
                    inputs: vec![Param {
                        name: "new_pusd".to_string(),
                        kind: ParamType::Address,
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: CHAIN_SETTINGS.load(deps.storage, chain_id.clone())?.job_id,
                    payload: Binary::new(
                        contract
                            .function("update_pusd")
                            .unwrap()
                            .encode_input(&[Token::Address(new_pusd_address)])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_pusd"))
    }

    pub fn update_pusd_manager(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
        new_pusd_manager: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        // ACTION: Implement UpdatePusdManager
        let state = STATE.load(deps.storage)?;
        assert!(info.sender == state.owner, "Unauthorized");
        let new_pusd_manager_address: Address =
            Address::from_str(new_pusd_manager.as_str()).unwrap();
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_pusd_manager".to_string(),
                vec![Function {
                    name: "update_pusd_manager".to_string(),
                    inputs: vec![Param {
                        name: "new_pusd_manager".to_string(),
                        kind: ParamType::Address,
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: CHAIN_SETTINGS.load(deps.storage, chain_id.clone())?.job_id,
                    payload: Binary::new(
                        contract
                            .function("update_pusd_manager")
                            .unwrap()
                            .encode_input(&[Token::Address(new_pusd_manager_address)])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_pusd_manager"))
    }

    pub fn update_refund_wallet(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
        new_refund_wallet: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        let state = STATE.load(deps.storage)?;
        assert!(state.owner == info.sender, "Unauthorized");
        let new_refund_wallet_address: Address =
            Address::from_str(new_refund_wallet.as_str()).unwrap();
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_refund_wallet".to_string(),
                vec![Function {
                    name: "update_refund_wallet".to_string(),
                    inputs: vec![Param {
                        name: "new_refund_wallet".to_string(),
                        kind: ParamType::Address,
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: CHAIN_SETTINGS.load(deps.storage, chain_id.clone())?.job_id,
                    payload: Binary::new(
                        contract
                            .function("update_refund_wallet")
                            .unwrap()
                            .encode_input(&[Token::Address(new_refund_wallet_address)])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_refund_wallet"))
    }
    pub fn update_gas_fee(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
        new_gas_fee: Uint256,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        // ACTION: Implement UpdateGasFee
        let state = STATE.load(deps.storage)?;
        assert!(info.sender == state.owner, "Unauthorized");
        let new_gas_fee: Uint = Uint::from_big_endian(&new_gas_fee.to_be_bytes());
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_gas_fee".to_string(),
                vec![Function {
                    name: "update_gas_fee".to_string(),
                    inputs: vec![Param {
                        name: "new_gas_fee".to_string(),
                        kind: ParamType::Uint(256),
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: CHAIN_SETTINGS.load(deps.storage, chain_id.clone())?.job_id,
                    payload: Binary::new(
                        contract
                            .function("update_gas_fee")
                            .unwrap()
                            .encode_input(&[Token::Uint(new_gas_fee)])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_gas_fee"))
    }
    pub fn update_service_fee_collector(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
        new_service_fee_collector: String,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        // ACTION: Implement UpdateServiceFeeCollector
        let state = STATE.load(deps.storage)?;
        assert!(info.sender == state.owner, "Unauthorized");
        let new_service_fee_collector: Address =
            Address::from_str(new_service_fee_collector.as_str()).unwrap();
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_service_fee_collector".to_string(),
                vec![Function {
                    name: "update_service_fee_collector".to_string(),
                    inputs: vec![Param {
                        name: "new_service_fee_collector".to_string(),
                        kind: ParamType::Address,
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: CHAIN_SETTINGS.load(deps.storage, chain_id.clone())?.job_id,
                    payload: Binary::new(
                        contract
                            .function("update_service_fee_collector")
                            .unwrap()
                            .encode_input(&[Token::Address(new_service_fee_collector)])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_service_fee_collector"))
    }
    pub fn update_service_fee(
        deps: DepsMut,
        info: MessageInfo,
        chain_id: String,
        new_service_fee: Uint256,
    ) -> Result<Response<PalomaMsg>, ContractError> {
        // ACTION: Implement UpdateServiceFee
        let state = STATE.load(deps.storage)?;
        assert!(info.sender == state.owner, "Unauthorized");
        let new_service_fee: Uint = Uint::from_big_endian(&new_service_fee.to_be_bytes());
        #[allow(deprecated)]
        let contract: Contract = Contract {
            constructor: None,
            functions: BTreeMap::from_iter(vec![(
                "update_service_fee".to_string(),
                vec![Function {
                    name: "update_service_fee".to_string(),
                    inputs: vec![Param {
                        name: "new_service_fee".to_string(),
                        kind: ParamType::Uint(256),
                        internal_type: None,
                    }],
                    outputs: Vec::new(),
                    constant: None,
                    state_mutability: StateMutability::NonPayable,
                }],
            )]),
            events: BTreeMap::new(),
            errors: BTreeMap::new(),
            receive: false,
            fallback: false,
        };
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg::SchedulerMsg {
                execute_job: ExecuteJob {
                    job_id: CHAIN_SETTINGS.load(deps.storage, chain_id.clone())?.job_id,
                    payload: Binary::new(
                        contract
                            .function("update_service_fee")
                            .unwrap()
                            .encode_input(&[Token::Uint(new_service_fee)])
                            .unwrap(),
                    ),
                },
            }))
            .add_attribute("action", "update_service_fee"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => {
            let state = STATE.load(deps.storage)?;
            to_json_binary(&state)
        }
        QueryMsg::GetChainSettings {} => {
            let mut chain_setting_info: Vec<ChainSettingInfo> = Vec::new();
            CHAIN_SETTINGS
                .range(deps.storage, None, None, Order::Ascending)
                .for_each(|item| {
                    let item = item.unwrap();
                    chain_setting_info.push(ChainSettingInfo {
                        chain_id: item.clone().0,
                        job_id: item.1.job_id.clone(),
                    });
                });
            to_json_binary(&chain_setting_info)
        }
    }
}
