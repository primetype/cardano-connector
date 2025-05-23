#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum APIErrorCode {
    #[error("Invalid inputs.")]
    InvalidRequest,
    #[error("An error occured during the execution of this API call.")]
    InternalError,
    #[error("The request was denied. The wallet may be disconnected.")]
    Refused,
    /// If this error happens we might need to re-authenticate.
    #[error("The account has changed.")]
    AccountChange,
    #[error("Unknown error code `{0}'")]
    Unknown(i64),
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error, serde::Deserialize,
)]
#[error("{code}. {info}.")]
pub struct APIError {
    pub code: APIErrorCode,
    pub info: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum DataSignErrorCode {
    #[error(
        "Wallet could not sign the data (e.g. does not have the secret key associated with the address)"
    )]
    ProofGeneration,
    #[error("Address was not a P2PK address and thus had no SK associated with it")]
    AddressNotPK,
    #[error("User declined to sign the data")]
    UserDeclined,
    #[error("Unknown error code `{0}'")]
    Unknown(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
#[error("{code}. {info}.")]
pub struct DataSignError {
    pub code: DataSignErrorCode,
    pub info: String,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error, serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
#[error("Pagination error")]
pub struct PaginateError {
    pub max_size: usize,
}

impl<'de> serde::Deserialize<'de> for APIErrorCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = APIErrorCode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "Expecting an integer APIErrorCode")
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    -1 => Ok(APIErrorCode::InvalidRequest),
                    -2 => Ok(APIErrorCode::InternalError),
                    -3 => Ok(APIErrorCode::Refused),
                    -4 => Ok(APIErrorCode::AccountChange),
                    unknown => Ok(APIErrorCode::Unknown(unknown)),
                }
            }
        }

        deserializer.deserialize_i64(Visitor)
    }
}

impl<'de> serde::Deserialize<'de> for DataSignErrorCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = DataSignErrorCode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "Expecting an integer DataSignErrorCode")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    1 => Ok(DataSignErrorCode::ProofGeneration),
                    2 => Ok(DataSignErrorCode::AddressNotPK),
                    3 => Ok(DataSignErrorCode::UserDeclined),
                    unknown => Ok(DataSignErrorCode::Unknown(unknown)),
                }
            }
        }

        deserializer.deserialize_u64(Visitor)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn api_error_code_json() {
        assert_eq!(
            serde_json::from_value::<APIErrorCode>(json! { -1 }).unwrap(),
            APIErrorCode::InvalidRequest
        );
        assert_eq!(
            serde_json::from_value::<APIErrorCode>(json! { -2 }).unwrap(),
            APIErrorCode::InternalError
        );
        assert_eq!(
            serde_json::from_value::<APIErrorCode>(json! { -3 }).unwrap(),
            APIErrorCode::Refused
        );
        assert_eq!(
            serde_json::from_value::<APIErrorCode>(json! { -4 }).unwrap(),
            APIErrorCode::AccountChange
        );
        assert_eq!(
            serde_json::from_value::<APIErrorCode>(json! { -42 }).unwrap(),
            APIErrorCode::Unknown(-42)
        );
    }

    #[test]
    fn api_error_json() {
        assert_eq!(
            serde_json::from_value::<APIError>(json! { {
                "code": -1,
                "info": "Parameter malformed.",
            }})
            .unwrap(),
            APIError {
                code: APIErrorCode::InvalidRequest,
                info: "Parameter malformed.".to_owned()
            }
        );

        assert_eq!(
            serde_json::from_value::<APIError>(json! { {
                "code": -2,
                "info": "Internal Error.",
            }})
            .unwrap(),
            APIError {
                code: APIErrorCode::InternalError,
                info: "Internal Error.".to_owned()
            }
        );

        assert_eq!(
            serde_json::from_value::<APIError>(json! { {
                "code": -3,
                "info": "Access Denied.",
            }})
            .unwrap(),
            APIError {
                code: APIErrorCode::Refused,
                info: "Access Denied.".to_owned()
            }
        );

        assert_eq!(
            serde_json::from_value::<APIError>(json! { {
                "code": -4,
                "info": "Account has changed.",
            }})
            .unwrap(),
            APIError {
                code: APIErrorCode::AccountChange,
                info: "Account has changed.".to_owned()
            }
        );
    }

    #[test]
    fn sign_data_error_code_json() {
        assert_eq!(
            serde_json::from_value::<DataSignErrorCode>(json! { 1 }).unwrap(),
            DataSignErrorCode::ProofGeneration
        );
        assert_eq!(
            serde_json::from_value::<DataSignErrorCode>(json! { 2 }).unwrap(),
            DataSignErrorCode::AddressNotPK
        );
        assert_eq!(
            serde_json::from_value::<DataSignErrorCode>(json! { 3 }).unwrap(),
            DataSignErrorCode::UserDeclined
        );
        assert_eq!(
            serde_json::from_value::<DataSignErrorCode>(json! { 42 }).unwrap(),
            DataSignErrorCode::Unknown(42)
        );
    }
}
