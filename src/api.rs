/*
 * This file is part of the IVMS Online.
 *
 * @copyright 2023 © by Rafał Wrzeszcz - Wrzasq.pl.
 */

use crate::runtime_error::RuntimeError;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// api contract

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorRequest {
    pub customer_id: Uuid,
    pub vessel_id: Uuid,
    pub issuer: String,
    pub audience: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorResponse {
    pub token: String,
}

impl GeneratorResponse {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

// downstream services API

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryListRequest {
    pub customer_id: Uuid,
    pub vessel_id: Uuid,
    pub page_token: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryFetchResponse {
    pub inventory_type: String,
    pub inventory_id: String,
    pub serial_number: Option<String>,
    pub aws_instance_id: Option<String>,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryListResponse {
    pub inventory: Vec<InventoryFetchResponse>,
    pub page_token: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LicensesListRequest {
    pub customer_id: Uuid,
    pub vessel_id: Uuid,
    pub page_token: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseFetchResponse {
    pub license_key: String,
    pub count: Option<u8>,
    pub expires_at: Option<DateTime<FixedOffset>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicensesListResponse {
    pub licenses: Vec<LicenseFetchResponse>,
    pub page_token: Option<String>,
}

// error response

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    RuntimeError(Box<RuntimeError>),
}

impl From<RuntimeError> for ApiError {
    fn from(error: RuntimeError) -> Self {
        ApiError::RuntimeError(error.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::api::{ApiError, GeneratorRequest, GeneratorResponse};
    use crate::runtime_error::RuntimeError;
    use serde_json::{from_str, to_string};
    use std::env::VarError;
    use uuid::{uuid, Uuid};

    const CUSTOMER_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000000");
    const VESSEL_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");
    const TOKEN: &str = "test0";
    const ISSUER: &str = "unit-test";
    const AUDIENCE: &str = "local";

    #[test]
    fn runtime_api_error() {
        match ApiError::from(RuntimeError::ClientConfigLoadingError(VarError::NotPresent)) {
            ApiError::RuntimeError(_) => {}
            _ => {
                panic!("Invalid error type.");
            }
        }
    }

    #[test]
    fn serialize_generate_response() {
        let output = to_string(&GeneratorResponse {
            token: String::from(TOKEN),
        })
        .unwrap();

        assert!(output.contains(&format!("test0")));
    }

    #[test]
    fn deserialize_trigger_request() {
        let input = format!("{{\"customerId\":\"{CUSTOMER_ID}\",\"vesselId\":\"{VESSEL_ID}\",\"issuer\":\"{ISSUER}\",\"audience\":\"{AUDIENCE}\"}}");
        let request: GeneratorRequest = from_str(&input).unwrap();

        assert_eq!(CUSTOMER_ID, request.customer_id);
        assert_eq!(VESSEL_ID, request.vessel_id);
        assert_eq!(ISSUER.to_string(), request.issuer);
        assert_eq!(AUDIENCE.to_string(), request.audience);
    }
}
