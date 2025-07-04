# PUSD Connector Contract - Security Audit Documentation

## Overview

The PUSD Connector Contract is a CosmWasm smart contract that facilitates cross-chain PUSD (Pegged USD) transfers and management. It acts as a bridge between different blockchain networks, allowing users to send and withdraw PUSD tokens across chains through the Paloma network.

**Contract Name**: `crates.io:pusd-connector-cw`  
**Version**: `0.1.0`  
**Author**: Volume Finance

## Architecture

The contract consists of the following main components:
- **State Management**: Global state and chain-specific settings
- **Execute Functions**: Core business logic for cross-chain operations
- **Query Functions**: Read-only operations for state inspection
- **Message Handlers**: Entry points for contract interactions

## State Management

### Global State
```rust
pub struct State {
    pub owner: Addr,           // Contract owner with administrative privileges
    pub pusd_manager: Addr,    // PUSD manager contract address
}
```

### Chain Settings
```rust
pub struct ChainSetting {
    pub job_id: String,        // Paloma job ID for cross-chain operations
}
```

## Entry Point Functions

### 1. `instantiate`
**Purpose**: Initializes the contract with owner and PUSD manager addresses.

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `_env: Env` - Contract environment
- `info: MessageInfo` - Message sender information
- `msg: InstantiateMsg` - Initialization message containing `pusd_manager` address

**Security Considerations**:
- Only callable once during contract deployment
- Sets the initial owner to the message sender
- No validation of `pusd_manager` address format

**Example Usage**:
```json
{
  "instantiate": {
    "pusd_manager": "cosmos1..."
  }
}
```

### 2. `migrate`
**Purpose**: Handles contract migrations and version updates.

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `_env: Env` - Contract environment
- `_msg: MigrateMsg` - Migration message (currently empty)

**Security Considerations**:
- Updates contract version information
- No state modifications in current implementation

### 3. `execute`
**Purpose**: Main entry point for all contract operations.

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `_env: Env` - Contract environment
- `info: MessageInfo` - Message sender information
- `msg: ExecuteMsg` - Execution message containing the operation to perform

**Security Considerations**:
- Routes to specific execute functions based on message type
- All operations require proper authorization checks

## Execute Functions

### 1. `register_chain`
**Purpose**: Registers a new blockchain network for PUSD operations.

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `info: MessageInfo` - Message sender information
- `chain_id: String` - Unique identifier for the blockchain
- `chain_setting: ChainSetting` - Configuration for the chain

**Security Considerations**:
- **Authorization**: Only the contract owner can register chains
- **Validation**: No validation of `chain_id` format or uniqueness
- **Storage**: Saves chain settings to persistent storage

**Example Usage**:
```json
{
  "register_chain": {
    "chain_id": "ethereum",
    "chain_setting": {
      "job_id": "job_123"
    }
  }
}
```

### 2. `send_pusd`
**Purpose**: Initiates a cross-chain PUSD transfer to a specified address.

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `info: MessageInfo` - Message sender information
- `chain_id: String` - Target blockchain identifier
- `to: String` - Destination address on target chain
- `amount: Uint128` - Amount of PUSD to transfer
- `nonce: Uint128` - Unique transaction identifier

**Security Considerations**:
- **Authorization**: Only the contract owner can send PUSD
- **Validation**: No validation of destination address format
- **Cross-chain**: Creates a Paloma Skyway message for cross-chain transfer
- **Token Construction**: Dynamically constructs PUSD token denomination

**Example Usage**:
```json
{
  "send_pusd": {
    "chain_id": "ethereum",
    "to": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
    "amount": "1000000",
    "nonce": "12345"
  }
}
```

### 3. `withdraw_pusd`
**Purpose**: Withdraws PUSD tokens from the contract to a specified recipient.

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `info: MessageInfo` - Message sender information
- `chain_id: String` - Source blockchain identifier
- `recipient: String` - Recipient address
- `amount: Uint128` - Amount of PUSD to withdraw

**Security Considerations**:
- **Authorization**: Only the contract owner can withdraw PUSD
- **External Call**: Executes a message to the PUSD manager contract
- **Token Transfer**: Sends actual PUSD tokens to the recipient
- **Validation**: No validation of recipient address format

**Example Usage**:
```json
{
  "withdraw_pusd": {
    "chain_id": "ethereum",
    "recipient": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
    "amount": "500000"
  }
}
```

### 4. `cancel_tx`
**Purpose**: Cancels a pending cross-chain transaction.

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `info: MessageInfo` - Message sender information
- `transaction_id: u64` - Unique transaction identifier to cancel

**Security Considerations**:
- **Authorization**: Only the contract owner can cancel transactions
- **Assertion**: Uses `assert!` macro instead of proper error handling
- **Cross-chain**: Sends cancellation message through Paloma Skyway

**Example Usage**:
```json
{
  "cancel_tx": {
    "transaction_id": 12345
  }
}
```

### 5. `change_config`
**Purpose**: Updates the contract's global configuration (owner and PUSD manager).

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `info: MessageInfo` - Message sender information
- `owner: Option<Addr>` - New owner address (optional)
- `pusd_manager: Option<Addr>` - New PUSD manager address (optional)

**Security Considerations**:
- **Authorization**: Only the current owner can change configuration
- **Partial Updates**: Allows updating only specific parameters
- **State Mutation**: Modifies global contract state
- **Validation**: No validation of new address formats

**Example Usage**:
```json
{
  "change_config": {
    "owner": "cosmos1...",
    "pusd_manager": "cosmos1..."
  }
}
```

### 6. `set_paloma`
**Purpose**: Sets the Paloma address for a specific blockchain.

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `info: MessageInfo` - Message sender information
- `chain_id: String` - Target blockchain identifier

**Security Considerations**:
- **Authorization**: Only the contract owner can set Paloma addresses
- **Assertion**: Uses `assert!` macro instead of proper error handling
- **Cross-chain**: Executes through Paloma Scheduler with encoded function call
- **Validation**: No validation of chain_id existence

**Example Usage**:
```json
{
  "set_paloma": {
    "chain_id": "ethereum"
  }
}
```

### 7. `update_withdraw_limit`
**Purpose**: Updates the withdrawal limit for a specific blockchain.

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `info: MessageInfo` - Message sender information
- `chain_id: String` - Target blockchain identifier
- `new_withdraw_limit: Uint256` - New withdrawal limit amount

**Security Considerations**:
- **Authorization**: Only the contract owner can update limits
- **Assertion**: Uses `assert!` macro instead of proper error handling
- **Cross-chain**: Executes through Paloma Scheduler with encoded function call
- **Data Conversion**: Converts Uint256 to Ethereum Uint format

**Example Usage**:
```json
{
  "update_withdraw_limit": {
    "chain_id": "ethereum",
    "new_withdraw_limit": "1000000000000000000000000"
  }
}
```

### 8. `update_pusd`
**Purpose**: Updates the PUSD contract address for a specific blockchain.

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `info: MessageInfo` - Message sender information
- `chain_id: String` - Target blockchain identifier
- `new_pusd: String` - New PUSD contract address

**Security Considerations**:
- **Authorization**: Only the contract owner can update PUSD addresses
- **Assertion**: Uses `assert!` macro instead of proper error handling
- **Address Validation**: Converts string to Ethereum Address format
- **Cross-chain**: Executes through Paloma Scheduler with encoded function call

**Example Usage**:
```json
{
  "update_pusd": {
    "chain_id": "ethereum",
    "new_pusd": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6"
  }
}
```

### 9. `update_pusd_manager`
**Purpose**: Updates the PUSD manager address for a specific blockchain.

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `info: MessageInfo` - Message sender information
- `chain_id: String` - Target blockchain identifier
- `new_pusd_manager: String` - New PUSD manager address

**Security Considerations**:
- **Authorization**: Only the contract owner can update manager addresses
- **Assertion**: Uses `assert!` macro instead of proper error handling
- **Address Validation**: Converts string to Ethereum Address format
- **Cross-chain**: Executes through Paloma Scheduler with encoded function call

**Example Usage**:
```json
{
  "update_pusd_manager": {
    "chain_id": "ethereum",
    "new_pusd_manager": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6"
  }
}
```

### 10. `update_refund_wallet`
**Purpose**: Updates the refund wallet address for a specific blockchain.

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `info: MessageInfo` - Message sender information
- `chain_id: String` - Target blockchain identifier
- `new_refund_wallet: String` - New refund wallet address

**Security Considerations**:
- **Authorization**: Only the contract owner can update refund wallets
- **Assertion**: Uses `assert!` macro instead of proper error handling
- **Address Validation**: Converts string to Ethereum Address format
- **Cross-chain**: Executes through Paloma Scheduler with encoded function call

**Example Usage**:
```json
{
  "update_refund_wallet": {
    "chain_id": "ethereum",
    "new_refund_wallet": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6"
  }
}
```

### 11. `update_gas_fee`
**Purpose**: Updates the gas fee for a specific blockchain.

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `info: MessageInfo` - Message sender information
- `chain_id: String` - Target blockchain identifier
- `new_gas_fee: Uint256` - New gas fee amount

**Security Considerations**:
- **Authorization**: Only the contract owner can update gas fees
- **Assertion**: Uses `assert!` macro instead of proper error handling
- **Data Conversion**: Converts Uint256 to Ethereum Uint format
- **Cross-chain**: Executes through Paloma Scheduler with encoded function call

**Example Usage**:
```json
{
  "update_gas_fee": {
    "chain_id": "ethereum",
    "new_gas_fee": "50000000000000000"
  }
}
```

### 12. `update_service_fee_collector`
**Purpose**: Updates the service fee collector address for a specific blockchain.

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `info: MessageInfo` - Message sender information
- `chain_id: String` - Target blockchain identifier
- `new_service_fee_collector: String` - New service fee collector address

**Security Considerations**:
- **Authorization**: Only the contract owner can update fee collectors
- **Assertion**: Uses `assert!` macro instead of proper error handling
- **Address Validation**: Converts string to Ethereum Address format
- **Cross-chain**: Executes through Paloma Scheduler with encoded function call

**Example Usage**:
```json
{
  "update_service_fee_collector": {
    "chain_id": "ethereum",
    "new_service_fee_collector": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6"
  }
}
```

### 13. `update_service_fee`
**Purpose**: Updates the service fee amount for a specific blockchain.

**Parameters**:
- `deps: DepsMut` - Contract dependencies
- `info: MessageInfo` - Message sender information
- `chain_id: String` - Target blockchain identifier
- `new_service_fee: Uint256` - New service fee amount

**Security Considerations**:
- **Authorization**: Only the contract owner can update service fees
- **Assertion**: Uses `assert!` macro instead of proper error handling
- **Data Conversion**: Converts Uint256 to Ethereum Uint format
- **Cross-chain**: Executes through Paloma Scheduler with encoded function call

**Example Usage**:
```json
{
  "update_service_fee": {
    "chain_id": "ethereum",
    "new_service_fee": "1000000000000000000"
  }
}
```

## Query Functions

### 1. `query`
**Purpose**: Main entry point for all read-only operations.

**Parameters**:
- `deps: Deps` - Contract dependencies
- `_env: Env` - Contract environment
- `msg: QueryMsg` - Query message containing the operation to perform

### 2. `GetState`
**Purpose**: Returns the current contract state.

**Response**: `State` struct containing owner and PUSD manager addresses

**Example Usage**:
```json
{
  "get_state": {}
}
```

### 3. `GetChainSettings`
**Purpose**: Returns all registered chain settings.

**Response**: Array of `ChainSettingInfo` containing chain IDs and job IDs

**Example Usage**:
```json
{
  "get_chain_settings": {}
}
```

## Security Considerations

### Critical Issues
1. **Inconsistent Authorization**: Some functions use `assert!` macro while others use proper error handling
2. **No Input Validation**: Address formats and chain IDs are not validated
3. **Unsafe External Calls**: Direct calls to PUSD manager without validation
4. **No Reentrancy Protection**: Functions that make external calls lack reentrancy guards

### Access Control
- All administrative functions require owner authorization
- No role-based access control beyond owner
- No multi-signature or timelock mechanisms

### Data Validation
- No validation of Ethereum address formats
- No validation of chain ID uniqueness
- No bounds checking on numeric parameters

### Cross-chain Security
- Relies on Paloma network for cross-chain operations
- No validation of cross-chain message integrity
- No timeout mechanisms for pending transactions

## Dependencies

- `cosmwasm-std`: Core CosmWasm functionality
- `ethabi`: Ethereum ABI encoding/decoding
- `cw-storage-plus`: Enhanced storage utilities
- `thiserror`: Error handling utilities

## Testing Recommendations

1. **Unit Tests**: Test each function with valid and invalid inputs
2. **Integration Tests**: Test cross-chain message flows
3. **Security Tests**: Test authorization bypass attempts
4. **Fuzzing**: Test with malformed input data
5. **Reentrancy Tests**: Test external call scenarios

## Deployment Considerations

1. **Initial Configuration**: Ensure proper owner and PUSD manager addresses
2. **Chain Registration**: Register all supported chains before use
3. **Parameter Validation**: Validate all configuration parameters
4. **Monitoring**: Monitor cross-chain transaction status
5. **Upgrade Path**: Plan for contract upgrades and migrations