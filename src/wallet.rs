use crate::{
    ConnectedWallet,
    error::{APIError, APIErrorCode},
    ffi,
};
use wasm_bindgen::JsValue;

#[derive(Clone, PartialEq)]
pub struct Wallet {
    cip30_wallet: ffi::Cip30Wallet,
}

/// attempt to find the wallet by name
///
/// This function is equivalent to
///
/// `wallets().into_iter().find(|wallet| wallet.name() == name)`
///
pub fn wallet(name: &str) -> Option<Wallet> {
    let wallets = wallets();

    wallets.into_iter().find(|wallet| wallet.name() == name)
}

pub fn lace() -> Option<Wallet> {
    ffi::cip30::LACE.with(|opt| opt.clone().map(Wallet::new))
}

/// List the wallets that may be available.
///
/// If the list is empty it means we didn't detect any wallets that we support
/// and support CIP30. However it is possible we are simply missing this wallet
/// and wallets are welcomed to add support.
///
///
/// Alternatively, it is also possible the wallets were not injected yet by the
/// extensions. You should make sure the page is loaded fully before calling this
/// function (or do refresh the value from time to time).
///
pub fn wallets() -> Vec<Wallet> {
    ffi::cip30::WALLETS.with(|wallets| {
        let mut vec = Vec::new();

        // Try to refresh the wallets from window.cardano in case they were loaded after initial check
        let fresh_wallets = js_sys::Reflect::get(
            &js_sys::global(),
            &wasm_bindgen::JsValue::from_str("window"),
        )
        .ok()
        .and_then(|win| {
            js_sys::Reflect::get(&win, &wasm_bindgen::JsValue::from_str("cardano")).ok()
        })
        .unwrap_or_else(|| wallets.clone().into());

        if fresh_wallets.is_object() {
            let fresh_wallets_obj: js_sys::Object = fresh_wallets.into();
            for element in js_sys::Object::values(&fresh_wallets_obj) {
                if !looks_like_cip30_wallet(&element) {
                    continue;
                }
                let cip30_wallet = ffi::Cip30Wallet::from(element);
                let wallet = Wallet { cip30_wallet };

                vec.push(wallet)
            }
        }

        vec
    })
}

fn looks_like_cip30_wallet(value: &JsValue) -> bool {
    if !value.is_object() {
        return false;
    }

    let has_string_property = |prop: &str| {
        js_sys::Reflect::get(value, &JsValue::from_str(prop))
            .ok()
            .and_then(|v| v.as_string())
            .is_some()
    };

    let has_function_property = |prop: &str| {
        js_sys::Reflect::get(value, &JsValue::from_str(prop))
            .ok()
            .map(|v| v.is_function())
            .unwrap_or(false)
    };

    has_string_property("name")
        && has_string_property("apiVersion")
        && has_string_property("icon")
        && has_function_property("enable")
        && has_function_property("isEnabled")
}

impl Wallet {
    fn new(cip30_wallet: ffi::Cip30Wallet) -> Self {
        Wallet { cip30_wallet }
    }

    /// get the name of the wallet connector application
    ///
    /// This can be `"lace"` for example.
    pub fn name(&self) -> String {
        self.cip30_wallet.name()
    }

    /// get the version of the wallet connector application
    ///
    /// Can be `"0.1.0"`
    pub fn version(&self) -> String {
        self.cip30_wallet.version()
    }

    /// get the HTML ready icon for this wallet connector application
    ///
    pub fn icon(&self) -> String {
        self.cip30_wallet.icon()
    }

    /// list the extensions supported by this wallet connector application.
    pub fn supported_extensions(&self) -> Vec<ffi::Extension> {
        self.cip30_wallet.supported_extensions()
    }

    /// Check if the wallet is already connected or not: i.e. if the users have
    /// already approved for the webapp to use connect with the wallet.
    ///
    /// If this returns `true` then calling [`Wallet::enable`] will returns the
    /// [`ConnectedWallet`] without prompting the user.
    ///
    pub async fn enabled(&self) -> Result<bool, APIError> {
        match self.cip30_wallet.enabled().await {
            Ok(obj) => {
                if let Some(boolean) = obj.as_bool() {
                    Ok(boolean)
                } else {
                    Err(APIError {
                        code: APIErrorCode::InternalError,
                        info: format!("Unexpected returned JSON Object: {obj:?}"),
                    })
                }
            }
            Err(error) => serde_wasm_bindgen::from_value(error)
                .map_err(|decode_error| APIError {
                    code: APIErrorCode::InternalError,
                    info: format!("Couldn't decode the error content: {decode_error}"),
                })
                .and_then(Err),
        }
    }

    /// Returns the [`ConnectedWallet`] after prompting the user to authorise your
    /// webapp. If the application is already authorised calling this function will
    /// return the [`ConnectedWallet`] without prompting the user.
    ///
    pub async fn enable(&self) -> Result<ConnectedWallet, APIError> {
        match self.cip30_wallet.enable(JsValue::undefined()).await {
            Ok(cip30_api) => Ok(ConnectedWallet::new(self.clone(), cip30_api)),
            Err(error) => serde_wasm_bindgen::from_value(error)
                .map_err(|decode_error| APIError {
                    code: APIErrorCode::InternalError,
                    info: format!("Couldn't decode the error content: {decode_error}"),
                })
                .and_then(Err),
        }
    }
}
