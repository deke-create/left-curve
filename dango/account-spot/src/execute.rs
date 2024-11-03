use {
    anyhow::ensure,
    dango_auth::authenticate_tx,
    dango_types::{account::InstantiateMsg, config::ACCOUNT_FACTORY_KEY},
    grug::{Addr, AuthCtx, AuthResponse, MutableCtx, Response, StdResult, Tx},
};

/// The `instantiate` function initializes the Spot account contract.
/// It ensures that only the account factory can create new accounts.
#[cfg_attr(not(feature = "library"), grug::export)]
pub fn instantiate(ctx: MutableCtx, _msg: InstantiateMsg) -> anyhow::Result<Response> {
    let account_factory: Addr = ctx.querier.query_app_config(ACCOUNT_FACTORY_KEY)?;

    // Only the account factory can create new accounts.
    ensure!(
        ctx.sender == account_factory,
        "you don't have the right, O you don't have the right"
    );

    Ok(Response::new())
}

/// The `authenticate` function authenticates a transaction for the Spot account.
/// It uses the `authenticate_tx` function from the `dango_auth` crate.
#[cfg_attr(not(feature = "library"), grug::export)]
pub fn authenticate(ctx: AuthCtx, tx: Tx) -> anyhow::Result<AuthResponse> {
    authenticate_tx(ctx, tx, None, None)?;

    Ok(AuthResponse::new().request_backrun(false))
}

/// The `receive` function is a placeholder for handling received funds.
/// Currently, it does nothing and accepts all transfers.
#[cfg_attr(not(feature = "library"), grug::export)]
pub fn receive(_ctx: MutableCtx) -> StdResult<Response> {
    // Do nothing, accept all transfers.
    Ok(Response::new())
}
