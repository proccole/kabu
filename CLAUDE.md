# Kabu MEV Bot - Claude Development Guide

This guide helps Claude instances understand and work with the Kabu MEV bot codebase effectively.

## Quick Start Commands

```bash
# Run tests
make test

# Pre-release checks
make pre-release

# Clean unused dependencies
make udeps

# Type checking and linting
make lint
make check

# Build the project
cargo build --release

# Run the main binary
cargo run -p kabu
```

## Architecture Overview

Kabu is an MEV (Maximum Extractable Value) bot built with Rust, using an actor-based architecture for concurrent processing of blockchain data and arbitrage opportunities.

### Core Design Principles

1. **Actor-Based Concurrency**: Each component runs as an independent actor with message-passing communication
2. **Type Safety**: Extensive use of Rust's type system for correctness
3. **Modular Design**: Clear separation between data types, business logic, and infrastructure
4. **Performance First**: Optimized for low-latency arbitrage detection and execution

### Key Components

1. **Actor System** (`crates/core/actors/`)
   - Message-passing concurrency model using tokio
   - Actors communicate via channels (Broadcaster/Consumer/Producer traits)
   - ActorManager orchestrates actor lifecycle
   - Key actors:
     - StateChangeArbSearcher: Finds arbitrage in state changes
     - SwapRouter: Routes and encodes swap transactions
     - TxSigners: Signs transactions with managed keys
     - Broadcast actors: Submit to network/flashbots

2. **Type System** (split into three crates for modularity)
   - `crates/types/entities/` - Core blockchain entities
     - Block, Transaction, Account structures
     - Strategy configurations
     - Pool configurations
   - `crates/types/market/` - Market-related types
     - Token: ERC20 token with price data
     - Pool trait and implementations
     - Market: Pool and token registry
     - SwapDirection, SwapPath, SwapPathBuilder
   - `crates/types/swap/` - Swap execution types
     - Swap: Executable arbitrage transaction
     - SwapLine: Path with calculated amounts
     - SwapStep: Individual pool interaction

3. **Strategy Layer** (`crates/strategy/`)
   - `backrun/`: State change arbitrage detection
     - BackrunConfig: Strategy configuration
     - StateChangeArbSearcher: Main arbitrage finder
     - SwapCalculator: Profit calculation logic
   - `merger/`: Transaction optimization
     - SamePathMerger: Combines similar paths
     - DiffPathMerger: Merges different opportunities
     - ArbSwapPathMerger: Multi-path arbitrage

4. **Execution Layer** (`crates/execution/`)
   - `estimator/`: EVM simulation for gas and success estimation
   - `multicaller/`: Custom contract for batch operations
     - Deploy logic for multicaller contract
     - Encoding for complex swap sequences

5. **DeFi Integrations** (`crates/defi/`)
   - `pools/`: Protocol implementations
     - UniswapV2Pool, UniswapV3Pool
     - CurvePool with multiple implementations
     - MaverickPool, PancakeV3Pool
   - `market/`: Market state management
   - `preloader/`: Initial state loading
   - `health_monitor/`: Pool reliability tracking

6. **Node Interaction** (`crates/node/`)
   - WebSocket subscription management
   - Block and transaction streaming
   - State diff calculation

7. **Database Layer** (`crates/evm/db/`)
   - In-memory state cache
   - Fork database for simulations
   - State diff application

## Actor Communication Pattern

```rust
// Typical actor setup
let mut actor = StateChangeArbSearcherActor::new(config);
actor
    .access(market.clone())           // Shared state access
    .consume(state_updates.clone())   // Input channel
    .produce(swaps.clone())          // Output channel
    .start()?;
```

## Type System Organization

### types/entities
Core blockchain and configuration types:
- `Block`, `Transaction`, `Account`
- `StrategyConfig` trait and implementations
- Pool loading configurations

### types/market  
Market structure and routing:
- `Token`: ERC20 with decimals, price, categories
- `Pool` trait: Unified pool interface
- `Market`: Registry of tokens and pools
- `SwapDirection`: Token pair direction in pool
- `SwapPath`: Route through multiple pools
- `PoolId`: Various pool identification methods

### types/swap
Execution and profit tracking:
- `Swap`: Final transaction ready for execution
- `SwapLine`: Path with amounts and gas costs
- `SwapStep`: Single pool interaction
- Calculation utilities for profit estimation

## Common Tasks

### Adding a New Pool Protocol

1. Create pool implementation:
```rust
// In crates/defi/pools/src/mynewpool.rs
#[derive(Clone, Debug)]
pub struct MyNewPool {
    address: Address,
    token0: Address,
    token1: Address,
    // ... protocol specific fields
}

impl Pool for MyNewPool {
    // Implement required methods
}
```

2. Add to pool loader:
```rust
// In crates/defi/pools/src/loaders/mod.rs
// Add detection and loading logic
```

3. Update PoolClass enum if needed:
```rust
// In crates/types/market/src/pool_class.rs
pub enum PoolClass {
    // ... existing variants
    MyNewProtocol,
}
```

### Creating a New Actor

1. Define actor struct:
```rust
#[derive(Accessor, Consumer, Producer)]
pub struct MyActor {
    #[accessor]
    market: Option<SharedState<Market>>,
    #[consumer]
    input_rx: Option<Broadcaster<InputMessage>>,
    #[producer]
    output_tx: Option<Broadcaster<OutputMessage>>,
}
```

2. Implement Actor trait:
```rust
impl Actor for MyActor {
    fn start(&self) -> ActorResult {
        let task = tokio::task::spawn(my_actor_worker(
            self.market.clone().unwrap(),
            self.input_rx.clone().unwrap(),
            self.output_tx.clone().unwrap(),
        ));
        Ok(vec![task])
    }
    
    fn name(&self) -> &'static str {
        "MyActor"
    }
}
```

### Working with Arbitrage Detection

1. **State Change Detection**: 
   - Monitor `StateUpdateEvent` for pool changes
   - Build affected swap paths
   - Calculate profitability with SwapCalculator

2. **Swap Optimization**:
   - Use merger actors to combine opportunities
   - Estimate gas with EvmEstimator
   - Encode with multicaller for efficiency

3. **Execution Flow**:
   ```
   StateChange → ArbSearcher → SwapCompose → Estimator → 
   Router → Signer → Broadcaster
   ```

## Testing Infrastructure

### Unit Tests
- Located alongside source files
- Mock implementations in test modules
- Run: `cargo test -p <package-name>`

### Integration Testing (`testing/`)

**backtest-runner**: Historical simulation
- Replays blocks with state
- Validates arbitrage detection
- Measures performance metrics

**gasbench**: Gas optimization
- Profiles contract interactions
- Compares encoding strategies

**nodebench**: Infrastructure testing
- Measures node latency
- Tests concurrent connections

**replayer**: Transaction analysis
- Replays specific transactions
- Debugs execution failures

## Performance Optimization

### Parallel Processing
```rust
// State change searcher uses thread pool
thread_pool.install(|| {
    swap_paths.into_par_iter().for_each_with(context, |ctx, path| {
        // Parallel calculation
    });
});
```

### Channel Sizing
- Size channels based on expected throughput
- Monitor channel depths in production
- Use bounded channels to prevent memory issues

### State Management
- Minimize lock contention with read-write locks
- Clone cheaply with Arc for immutable data
- Batch state updates when possible

## Common Patterns

### Error Handling
```rust
use eyre::{Result, WrapErr};

fn risky_operation() -> Result<Value> {
    external_call()
        .wrap_err("Failed to call external service")?;
    Ok(value)
}
```

### Message Passing
```rust
// Send with error handling
if let Err(e) = channel.send(Message::new(data)) {
    error!("Failed to send: {}", e);
}

// Receive with timeout
tokio::select! {
    msg = receiver.recv() => handle_message(msg?),
    _ = tokio::time::sleep(timeout) => handle_timeout(),
}
```

### Shared State Access
```rust
// Minimize lock duration
let data = {
    let guard = shared_state.read().await;
    guard.expensive_clone()
}; // Lock released here
process_data(data);
```

## Debugging Tips

1. **Enable Tracing**: 
   ```bash
   RUST_LOG=debug cargo run
   ```

2. **Actor Communication**: 
   - Log message sends/receives
   - Monitor channel depths
   - Track actor lifecycle

3. **Arbitrage Calculation**:
   - Log intermediate amounts
   - Verify pool states
   - Check gas estimations

4. **State Differences**:
   - Compare before/after states
   - Validate state transitions
   - Monitor pool updates

## Security Considerations

1. **Private Key Management**:
   - Never log private keys
   - Use secure key derivation
   - Rotate keys regularly

2. **Transaction Security**:
   - Validate all calculations
   - Simulate before submission
   - Monitor for reverts

3. **MEV Protection**:
   - Use flashbots when possible
   - Implement commit-reveal if needed
   - Monitor mempool exposure

## Configuration

### Strategy Configuration
Located in strategy configs, typically:
```toml
[backrun_strategy]
eoa = "0x..." # Optional execution address
smart = true  # Enable optimization
```

### Pool Loading
Configure which pools to load:
```rust
PoolsLoadingConfig {
    load_uni_v2: true,
    load_uni_v3: true,
    load_curve: true,
    // ...
}
```

## Maintenance Notes

1. **Dependency Updates**: 
   - Run `make udeps` to clean unused deps
   - Check compatibility before updating
   - Test thoroughly after updates

2. **Performance Monitoring**:
   - Profile regularly with flamegraph
   - Monitor actor performance
   - Track arbitrage success rates

3. **Code Organization**:
   - Keep actors focused and single-purpose
   - Maintain clear module boundaries
   - Document complex algorithms

## Build and Development Commands

### Essential Commands
```bash
# Build the project
make build              # Debug build
make release           # Release build  
make maxperf           # Optimized release build with native CPU optimizations

# Run tests
make test              # Run all tests (excludes some integration tests)
cargo test -p <package-name> --lib  # Run tests for specific package

# Code quality
make fmt               # Format code
make clippy            # Run linter
make pre-release       # Run all checks (fmt, clippy, taplo, udeps)
make udeps             # Check for unused dependencies

# Backtest specific swap scenarios
make swap-test FILE=./testing/backtest-runner/test_18498188.toml
make swap-test-all     # Run all swap tests

# Run replayer
make replayer
```