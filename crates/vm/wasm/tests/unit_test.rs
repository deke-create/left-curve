use {
    grug_app::{GasTracker, Instance, QuerierProvider, Shared, StorageProvider, Vm},
    grug_crypto::sha2_256,
    grug_tester_benchmarker::{ExecuteMsg, QueryMsg},
    grug_types::{
        from_json_slice, to_json_vec, Addr, BlockInfo, Coins, Context, GenericResult, Hash, Json,
        MockStorage, Timestamp, Uint128, Uint64,
    },
    grug_vm_wasm::WasmVm,
    tracing::debug,
};

const MOCK_CHAIN_ID: &str = "dev-1";

const MOCK_CONTRACT: Addr = Addr::mock(1);

const MOCK_BLOCK: BlockInfo = BlockInfo {
    height: Uint64::new(1),
    timestamp: Timestamp::from_seconds(100),
    hash: Hash::ZERO,
};

static BENCHMARKER_CODE: &[u8] = include_bytes!("../testdata/grug_tester_benchmarker.wasm");

fn setup(
    vm: &mut WasmVm,
    storage: Option<Shared<MockStorage>>,
    gas_tracker: Option<GasTracker>,
) -> anyhow::Result<(
    grug_vm_wasm::WasmInstance,
    Context,
    GasTracker,
    Shared<MockStorage>,
)> {
    let storage = storage.unwrap_or_default();
    let gas_tracker = gas_tracker.unwrap_or_else(GasTracker::new_limitless);

    let querier = QuerierProvider::new(
        vm.clone(),
        Box::new(storage.clone()),
        gas_tracker.clone(),
        MOCK_BLOCK,
    );
    let storage_provider = StorageProvider::new(Box::new(storage.clone()), &[&MOCK_CONTRACT]);

    let instance = vm.build_instance(
        BENCHMARKER_CODE,
        &Hash::from_slice(sha2_256(BENCHMARKER_CODE)),
        storage_provider,
        false,
        querier,
        gas_tracker.clone(),
    )?;

    let ctx = Context {
        chain_id: MOCK_CHAIN_ID.to_string(),
        block: MOCK_BLOCK,
        contract: MOCK_CONTRACT,
        sender: Some(Addr::mock(3)),
        funds: Some(Coins::default()),
        simulate: Some(false),
    };

    Ok((instance, ctx, gas_tracker, storage))
}

const SIZED: bool = true;
const LIMIT: u32 = 1000;

// The purpose of this test is to check the performance of scan_sized vs scan.
// The benchmark results are quite strange, with a big advantage for the scan method.
// However, this simple test shows how the scan_sized method is about 3 times faster.
// Need to understand why the benchmark gives opposite results, especially on long iterations.
#[test]
fn try_execute() {
    let mut vm = WasmVm::new(10000);

    let (instance, ctx, _, storage) = setup(&mut vm, None, None).unwrap();

    let data = (1..LIMIT + 1).fold(vec![], |mut buf, i| {
        buf.push((i.to_string(), Uint128::from(i as u128)));
        buf
    });

    let msg = to_json_vec(&ExecuteMsg::Populate { data }).unwrap();

    let res = instance.call_in_1_out_1("execute", &ctx, &msg).unwrap();

    let res: GenericResult<Json> = from_json_slice(res).unwrap();

    debug!("{:?}", res);

    let (instance, ctx, ..) = setup(&mut vm, Some(storage), None).unwrap();

    let query = to_json_vec(&QueryMsg::Data {
        min: None,
        max: None,
        order: grug_types::Order::Ascending,
        limit: LIMIT,
        sized: SIZED,
    })
    .unwrap();

    let now = std::time::Instant::now();
    let res = instance.call_in_1_out_1("query", &ctx, &query).unwrap();
    println!("{:?}", now.elapsed());

    let res = from_json_slice::<GenericResult<Json>>(res).unwrap().as_ok();
    println!("{res:?}")
}

#[test]
fn des() {
    let b_18 = "18".to_string();
    let b_180 = "180".to_string();
    let b_19 = "19".to_string();

    let bytes_18 = to_json_vec(&b_18).unwrap();
    let bytes_180 = to_json_vec(&b_180).unwrap();
    let bytes_19 = to_json_vec(&b_19).unwrap();
    println!("{:?}", bytes_18);
    println!("{:?}", bytes_180);
    println!("{:?}", bytes_19);
}
