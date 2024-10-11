use std::str::FromStr;

use grug::{Addr, Coins, ContractBuilder, Empty, Hash256, ResultExt, Salt, TestBuilder};

mod query_maker {
    use grug::{
        Addr, Empty, Hash256, ImmutableCtx, Json, JsonSerExt, MutableCtx, Response, StdResult,
    };

    #[grug::derive(Serde, Borsh, QueryRequest)]
    pub enum QueryMsg {
        #[returns(String)]
        Foo { bar: u64 },
        #[returns(Addr)]
        Fuzz(u8),
        #[returns(Hash256)]
        Buzz,
    }

    pub fn instantiate(_ctx: MutableCtx, _msg: Empty) -> StdResult<Response> {
        Ok(Response::new())
    }

    pub fn query(_ctx: ImmutableCtx, msg: QueryMsg) -> StdResult<Json> {
        match msg {
            QueryMsg::Foo { bar } => {
                let bar = bar.to_string();
                bar.to_json_value()
            },
            QueryMsg::Fuzz(fuzz) => {
                let fuzz = Addr::mock(fuzz);
                fuzz.to_json_value()
            },
            QueryMsg::Buzz => {
                let buzz = Hash256::from_array([1; 32]);
                buzz.to_json_value()
            },
        }
    }
}

#[test]
fn query_super_smart() {
    let (mut suite, mut accounts) = TestBuilder::new()
        .add_account("larry", Coins::one("uusdc", 123).unwrap())
        .unwrap()
        .set_chain_id("kebab")
        .set_owner("larry")
        .unwrap()
        .build()
        .unwrap();

    let code = ContractBuilder::new(Box::new(query_maker::instantiate))
        .with_query(Box::new(query_maker::query))
        .build();

    let (_, contract) = suite
        .upload_and_instantiate(
            accounts.get_mut("larry").unwrap(),
            code,
            Salt::from_str("contract").unwrap(),
            &Empty {},
            Coins::new(),
        )
        .unwrap();

    // Here, the compiler should be able to infer the type of the response as
    // `String` based on the request type `QueryFooRequest`.
    suite
        .query_wasm_smart(contract, query_maker::QueryFooRequest { bar: 12345 })
        .should_succeed_and_equal(12345.to_string());

    // Similarly, for unnamed variant `Fuzz`.
    suite
        .query_wasm_smart(contract, query_maker::QueryFuzzRequest(123))
        .should_succeed_and_equal(Addr::mock(123));

    // Similarly, for unit variant `Buzz`.
    suite
        .query_wasm_smart(contract, query_maker::QueryBuzzRequest)
        .should_succeed_and_equal(Hash256::from_array([1; 32]));
}
