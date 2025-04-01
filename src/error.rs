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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize)]
pub struct APIError {
    pub code: APIErrorCode,
    pub info: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
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
}
