use wasm_bindgen::JsValue;

use crate::{
    Reflection, SemverVersion, StandardFunction, WalletAccount, WalletError, WalletResult,
};

/// The `standard:connect` struct containing a `version` and `callback`
/// within [StandardFunction] field
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Connect(StandardFunction);

impl Connect {
    /// Initialize a new `standard:connect` function by parsing a [JsValue]
    pub fn new(value: JsValue, version: SemverVersion) -> WalletResult<Self> {
        Ok(Self(StandardFunction::new(
            value, version, "connect", "standard",
        )?))
    }

    /// Connect to a wallet by calling the callback function
    pub(crate) async fn call_connect(&self) -> WalletResult<WalletAccount> {
        let outcome = self.0.callback.call0(&JsValue::from_bool(false))?;

        let outcome = js_sys::Promise::resolve(&outcome);

        match wasm_bindgen_futures::JsFuture::from(outcome).await {
            Ok(success) => {
                let get_accounts = Reflection::new(success)?.get_js_array("accounts")?;

                let wallet_account = get_accounts
                    .into_iter()
                    .map(|raw_account| WalletAccount::parse(Reflection::new(raw_account)?))
                    .collect::<WalletResult<Vec<WalletAccount>>>()
                    .map(|mut accounts| {
                        if accounts.is_empty() {
                            Err(WalletError::ConnectHasNoAccounts)
                        } else {
                            Ok(accounts.remove(0))
                        }
                    })??;

                Ok(wallet_account)
            }
            Err(error) => {
                let value: WalletError = error.into();

                Err(WalletError::WalletConnectError(value.to_string()))
            }
        }
    }
}
