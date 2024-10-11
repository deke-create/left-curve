use {
    grug_testing::TestBuilder,
    grug_types::{Coins, Empty},
    grug_vm_rust::ContractBuilder,
};

mod replier {
    use grug_types::{Empty, MutableCtx, Response, StdResult};

    pub fn instantiate(_ctx: MutableCtx, _msg: Empty) -> StdResult<Response> {
        Ok(Response::new())
    }
}

#[test]
fn reply() {
    let (mut suite, mut accounts) = TestBuilder::new()
        .add_account("owner", Coins::new())
        .unwrap()
        .add_account("sender", Coins::new())
        .unwrap()
        .set_owner("owner")
        .unwrap()
        .build()
        .unwrap();

    let replier_code = ContractBuilder::new(Box::new(replier::instantiate)).build();

    let sender = accounts.get_mut("sender").unwrap();

    let (_, _replier_addr) = suite
        .upload_and_instantiate(sender, replier_code, "salt", &Empty {}, Coins::default())
        .unwrap();
}
