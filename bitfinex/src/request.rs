use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct DepositAddressRequest<'a> {
    pub(crate) wallet: &'a str,
    pub(crate) method: &'a str,
    pub(crate) op_renew: i32,
}
