use {
    colored::*,
    core::panic,
    criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion},
    grug_app::{GasTracker, Instance, QuerierProvider, Shared, StorageProvider, Vm},
    grug_crypto::sha2_256,
    grug_tester_benchmarker::{ExecuteMsg, QueryMsg},
    grug_types::{
        from_json_slice, to_json_vec, Addr, BlockInfo, Coins, Context, Empty, GenericResult, Hash,
        Json, MockStorage, Timestamp, Uint128, Uint64,
    },
    grug_vm_wasm::{WasmInstance, WasmVm},
    std::time::Duration,
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
        sender: None,
        funds: None,
        simulate: None,
    };

    Ok((instance, ctx, gas_tracker, storage))
}

fn looping(c: &mut Criterion) {
    // Share one `WasmVm` across all benches, which caches the module, so we
    // don't need to rebuild it every time.
    let mut vm = WasmVm::new(100);

    for iterations in [200_000, 400_000, 600_000, 800_000, 1_000_000] {
        // The `criterion` library only benchmarks the time consumption, however
        // we additinally want to know the gas used, so that we can compute the
        // gas per second. So we record it separately here.
        let mut sum = 0;
        let mut repeats = 0;

        c.bench_with_input(
            BenchmarkId::new("looping", iterations),
            &iterations,
            |b, iterations| {
                // `Bencher::iter_with_setup` has been deprecated, in favor of
                // `Bencher::iter_batched` with `PerIteration`. See:
                // https://bheisler.github.io/criterion.rs/book/user_guide/timing_loops.html#deprecated-timing-loops
                b.iter_batched(
                    || -> anyhow::Result<_> {
                        let (istance, ctx, gas_tracker, _) = setup(&mut vm, None, None).unwrap();
                        let msg = to_json_vec(&QueryMsg::Loop {
                            iterations: *iterations,
                        })?;
                        let ok = to_json_vec(&GenericResult::Ok(Empty {}))?;

                        Ok((istance, ctx, msg, ok, gas_tracker))
                    },
                    |suite| {
                        let (instance, ctx, msg, ok, gas_tracker) = suite.unwrap();

                        // Call the `loop` query method
                        let output = instance.call_in_1_out_1("query", &ctx, &msg).unwrap();

                        // Make sure the contract didn't error
                        assert_eq!(output, ok);

                        // Record the gas consumed
                        sum += gas_tracker.used();
                        repeats += 1;
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        if repeats != 0 {
            println!(
                "Iterations per run = {}; points per run = {}\n",
                iterations,
                sum / repeats
            );
        }
    }
}

fn scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("sized");

    let mut vm = WasmVm::new(100);

    let mut setup_sized = |iterations: i32,
                           sized: bool|
     -> anyhow::Result<(WasmInstance, Context, GasTracker, Vec<u8>)> {
        let (instance, mut ctx, _, storage) = setup(&mut vm, None, None).unwrap();

        ctx.sender = Some(Addr::mock(3));
        ctx.funds = Some(Coins::default());
        ctx.simulate = Some(false);

        let data = (1..iterations + 1).fold(vec![], |mut buf, i| {
            buf.push((i.to_string(), Uint128::from(i as u128)));
            buf
        });

        let msg = to_json_vec(&ExecuteMsg::Populate { data }).unwrap();

        instance.call_in_1_out_1("execute", &ctx, &msg).unwrap();

        let query = to_json_vec(&QueryMsg::Data {
            min: None,
            max: None,
            order: grug_types::Order::Ascending,
            limit: iterations as u32,
            sized,
        })
        .unwrap();

        // rebuild instance because it has been moved during call_in_1_out_1
        let (instance, ctx, gas_tracker, _) = setup(&mut vm, Some(storage), None).unwrap();

        Ok((instance, ctx, gas_tracker, query))
    };

    for iterations in [200, 1000] {
        let mut output_non_sized: Option<Vec<u8>> = None;
        let mut sum = 0;
        let mut repeats = 0;
        group.bench_with_input(
            BenchmarkId::new("non_sized", iterations),
            &iterations,
            |b, iterations| {
                b.iter_batched(
                    || -> anyhow::Result<_> { setup_sized(*iterations, false) },
                    |suite| {
                        let (instance, ctx, gas_tracker, msg) = suite.unwrap();

                        // Call the `loop` query method
                        output_non_sized =
                            Some(instance.call_in_1_out_1("query", &ctx, &msg).unwrap());
                        // Record the gas consumed
                        sum += gas_tracker.used();
                        repeats += 1;
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        if repeats != 0 {
            println!(
                "Iterations per run = {}; points per run = {}\n",
                iterations,
                sum / repeats
            );
        }

        sum = 0;
        repeats = 0;

        let mut output_sized: Option<Vec<u8>> = None;
        group.bench_with_input(
            BenchmarkId::new("sized", iterations),
            &iterations,
            |b, iterations| {
                b.iter_batched(
                    || -> anyhow::Result<_> { setup_sized(*iterations, true) },
                    |suite| {
                        let (instance, ctx, gas_tracker, msg) = suite.unwrap();

                        // Call the `loop` query method
                        output_sized = Some(instance.call_in_1_out_1("query", &ctx, &msg).unwrap());

                        sum += gas_tracker.used();
                        repeats += 1;
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        if repeats != 0 {
            println!(
                "Iterations per run = {}; points per run = {}\n",
                iterations,
                sum / repeats
            );
        }

        let output_non_sized = from_json_slice::<GenericResult<Json>>(&output_non_sized.unwrap())
            .unwrap()
            .as_ok();
        let output_sized = from_json_slice::<GenericResult<Json>>(&output_sized.unwrap())
            .unwrap()
            .as_ok();

        match (&output_non_sized, &output_sized) {
            (Json::Array(non_sized), Json::Array(sized)) => {
                if non_sized != sized {
                    let clos = |comp: &[Json], with: &[Json], desc: &str| {
                        println!("{desc} - len: {}", comp.len());
                        for i in comp {
                            if !with.contains(i) {
                                print!("{}", format!("{i},").red());
                            } else {
                                print!("{}", format!("{i},").black());
                            }
                        }
                        println!();
                    };

                    clos(non_sized, sized, "non_sized");
                    clos(sized, non_sized, "sized");

                    println!("Warning: result as differents!");
                }
            },
            _ => panic!("unexpected output format"),
        }
    }
}

criterion_group!(
    name = wasmer_metering;
    config = Criterion::default().measurement_time(Duration::from_secs(40)).sample_size(200);
    targets = looping
);

criterion_group!(
    name = wasmer_scan;
    config = Criterion::default().measurement_time(Duration::from_secs(3)).sample_size(100);
    targets = scan
);

criterion_main!(wasmer_metering, wasmer_scan);

#[cfg(test)]
mod tests {}
