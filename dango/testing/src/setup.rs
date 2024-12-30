use {
    crate::{Accounts, TestAccount},
    dango_app::ProposalPreparer,
    dango_genesis::{
        build_genesis, build_rust_codes, read_wasm_files, Codes, Contracts, GenesisUser,
    },
    grug::{
        btree_map, coins, Binary, BlockInfo, Coin, Coins, ContractWrapper, Duration, HashExt,
        NumberConst, Timestamp, Udec128, GENESIS_BLOCK_HASH, GENESIS_BLOCK_HEIGHT,
    },
    grug_app::{AppError, Db, Indexer, NaiveProposalPreparer, NullIndexer, Vm},
    grug_db_disk::{DiskDb, TempDataDir},
    grug_db_memory::MemDb,
    grug_vm_hybrid::HybridVm,
    grug_vm_rust::RustVm,
    grug_vm_wasm::WasmVm,
    hex_literal::hex,
    std::{path::PathBuf, sync::LazyLock},
};

pub const MOCK_CHAIN_ID: &str = "mock-1";
pub const MOCK_GENESIS_TIMESTAMP: Timestamp = Timestamp::from_days(365);

pub const OWNER_PRIVATE_KEY: [u8; 32] =
    hex!("8a8b0ab692eb223f6a2927ad56e63c2ae22a8bc9a5bdfeb1d8127819ddcce177");
pub const USER1_PRIVATE_KEY: [u8; 32] =
    hex!("a5122c0729c1fae8587e3cc07ae952cb77dfccc049efd5be1d2168cbe946ca18");
pub const USER2_PRIVATE_KEY: [u8; 32] =
    hex!("cac7b4ced59cf0bfb14c373272dfb3d4447c7cd5aea732ea6ff69e19f85d34c4");
pub const USER3_PRIVATE_KEY: [u8; 32] =
    hex!("cf6bb15648a3a24976e2eeffaae6201bc3e945335286d273bb491873ac7c3141");
pub const USER4_PRIVATE_KEY: [u8; 32] =
    hex!("126b714bfe7ace5aac396aa63ff5c92c89a2d894debe699576006202c63a9cf6");

pub static TOKEN_FACTORY_CREATION_FEE: LazyLock<Coin> =
    LazyLock::new(|| Coin::new("uusdc", 10_000_000).unwrap());

pub type TestSuite<PP = ProposalPreparer, DB = MemDb, VM = RustVm, ID = NullIndexer> =
    grug::TestSuite<DB, VM, PP, ID>;

/// Set up a `TestSuite` with `MemDb`, `RustVm`, `ProposalPreparer`, and
/// `ContractWrapper` codes.
///
/// Used for running regular tests.
pub fn setup_test() -> (TestSuite, Accounts, Codes<ContractWrapper>, Contracts) {
    let codes = build_rust_codes();

    setup_suite_with_db_and_vm(
        MemDb::new(),
        RustVm::new(),
        codes,
        ProposalPreparer::new(),
        NullIndexer,
    )
}

/// Set up a `TestSuite` with `MemDb`, `RustVm`, `NaiveProposalPreparer`, and
/// `ContractWrapper` codes.
///
/// Used for running tests that don't require an oracle feed. For such cases, we
/// avoid adding the proposal preparer that will pull price feeds from Pyth API.
pub fn setup_test_naive() -> (
    TestSuite<NaiveProposalPreparer>,
    Accounts,
    Codes<ContractWrapper>,
    Contracts,
) {
    let codes = build_rust_codes();

    setup_suite_with_db_and_vm(
        MemDb::new(),
        RustVm::new(),
        codes,
        NaiveProposalPreparer,
        NullIndexer,
    )
}

/// Set up a `TestSuite` with `DiskDb`, `HybridVm`, `NaiveProposalPreparer`, and
/// `ContractWrapper` codes.
///
/// Used for running benchmarks with the hybrid VM.
pub fn setup_benchmark_hybrid(
    dir: &TempDataDir,
    wasm_cache_size: usize,
) -> (
    TestSuite<NaiveProposalPreparer, DiskDb, HybridVm, NullIndexer>,
    Accounts,
    Codes<ContractWrapper>,
    Contracts,
) {
    let codes = build_rust_codes();
    let db = DiskDb::open(dir).unwrap();
    let vm = HybridVm::new(wasm_cache_size, [
        codes.account_factory.to_bytes().hash256(),
        codes.account_margin.to_bytes().hash256(),
        codes.account_safe.to_bytes().hash256(),
        codes.account_spot.to_bytes().hash256(),
        codes.amm.to_bytes().hash256(),
        codes.bank.to_bytes().hash256(),
        codes.ibc_transfer.to_bytes().hash256(),
        codes.lending.to_bytes().hash256(),
        codes.oracle.to_bytes().hash256(),
        codes.taxman.to_bytes().hash256(),
        codes.token_factory.to_bytes().hash256(),
        codes.vesting.to_bytes().hash256(),
    ]);

    setup_suite_with_db_and_vm(db, vm, codes, NaiveProposalPreparer, NullIndexer)
}

/// Set up a `TestSuite` with `DiskDb`, `WasmVm`, `NaiveProposalPreparer`, and
/// `Vec<u8>` codes.
///
/// Used for running benchmarks with the Wasm VM.
pub fn setup_benchmark_wasm(
    dir: &TempDataDir,
    wasm_cache_size: usize,
) -> (
    TestSuite<NaiveProposalPreparer, DiskDb, WasmVm, NullIndexer>,
    Accounts,
    Codes<Vec<u8>>,
    Contracts,
) {
    let codes = read_wasm_files(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../artifacts"))
        .unwrap();
    let db = DiskDb::open(dir).unwrap();
    let vm = WasmVm::new(wasm_cache_size);

    setup_suite_with_db_and_vm(db, vm, codes, NaiveProposalPreparer, NullIndexer)
}

fn setup_suite_with_db_and_vm<DB, VM, T, PP, ID>(
    db: DB,
    vm: VM,
    codes: Codes<T>,
    pp: PP,
    indexer: ID,
) -> (TestSuite<PP, DB, VM, ID>, Accounts, Codes<T>, Contracts)
where
    T: Clone + Into<Binary>,
    DB: Db,
    VM: Vm + Clone + 'static,
    ID: Indexer,
    PP: grug_app::ProposalPreparer,
    AppError: From<DB::Error> + From<VM::Error> + From<PP::Error> + From<ID::Error>,
{
    let owner = TestAccount::new_from_private_key("owner", OWNER_PRIVATE_KEY);
    let user1 = TestAccount::new_from_private_key("user1", USER1_PRIVATE_KEY);
    let user2 = TestAccount::new_from_private_key("user2", USER2_PRIVATE_KEY);
    let user3 = TestAccount::new_from_private_key("user3", USER3_PRIVATE_KEY);
    let user4 = TestAccount::new_from_private_key("user4", USER3_PRIVATE_KEY);

    let (genesis_state, contracts, addresses) = build_genesis(
        codes.clone(),
        btree_map! {
            owner.username.clone() => GenesisUser {
                key: owner.key(),
                key_hash: owner.key_hash(),
                // Some of the tests depend on the number of tokens, so careful
                // when changing these. They may break tests...
                balances: coins! {
                    "udng"  => 100_000_000_000_000,
                    "uusdc" => 100_000_000_000,
                },
            },
            user1.username.clone() => GenesisUser {
                key: user1.key(),
                key_hash: user1.key_hash(),
                balances: coins! {
                    "udng"  => 100_000_000_000_000,
                    "uusdc" => 100_000_000_000_000,
                    "uatom" => 100_000_000_000_000,
                    "uosmo" => 100_000_000_000_000,
                },
            },
            user2.username.clone() => GenesisUser {
                key: user2.key(),
                key_hash: user2.key_hash(),
                balances: coins! {
                    "udng"  => 100_000_000_000_000,
                    "uusdc" => 100_000_000_000_000,
                },
            },
            user3.username.clone() => GenesisUser {
                key: user3.key(),
                key_hash: user3.key_hash(),
                balances: coins! {
                    "udng"  => 100_000_000_000_000,
                    "uusdc" => 100_000_000_000_000,
                },
            },
            user4.username.clone() => GenesisUser {
                key: user4.key(),
                key_hash: user4.key_hash(),
                balances: coins! {
                    "udng"  => 100_000_000_000_000,
                    "uusdc" => 100_000_000_000_000,
                },
            },
        },
        &owner.username,
        TOKEN_FACTORY_CREATION_FEE.denom.to_string(),
        Udec128::ZERO,
        Some(TOKEN_FACTORY_CREATION_FEE.amount),
        Duration::from_seconds(7 * 24 * 60 * 60),
    )
    .unwrap();

    let suite = grug::TestSuite::new_with_db_vm_indexer_and_pp(
        db,
        vm,
        pp,
        indexer,
        MOCK_CHAIN_ID.to_string(),
        Duration::from_millis(250),
        1_000_000,
        BlockInfo {
            hash: GENESIS_BLOCK_HASH,
            height: GENESIS_BLOCK_HEIGHT,
            timestamp: MOCK_GENESIS_TIMESTAMP,
        },
        genesis_state,
    );

    let accounts = Accounts {
        owner: owner.set_address(&addresses),
        user1: user1.set_address(&addresses),
        user2: user2.set_address(&addresses),
        user3: user3.set_address(&addresses),
        user4: user4.set_address(&addresses),
    };

    (suite, accounts, codes, contracts)
}
