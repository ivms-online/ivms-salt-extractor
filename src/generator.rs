/*
 * This file is part of the IVMS Online.
 *
 * @copyright 2023 © by Rafał Wrzeszcz - Wrzasq.pl.
 */

use crate::api::{
    GeneratorRequest, InventoryFetchResponse, InventoryListRequest, InventoryListResponse, LicenseFetchResponse,
    LicensesListRequest, LicensesListResponse,
};
use crate::model::Claims;
use crate::runtime_error::RuntimeError;
use aws_sdk_lambda::Client;
use aws_smithy_types::Blob;
use hmac::digest::KeyInit;
use hmac::Hmac;
use jwt::SignWithKey;
use serde_json::{from_slice, to_string};
use sha2::Sha512;
use uuid::Uuid;

pub async fn load_inventory(
    client: &Client,
    lambda: &String,
    customer_id: &Uuid,
    vessel_id: &Uuid,
) -> Result<Vec<InventoryFetchResponse>, RuntimeError> {
    let mut request = InventoryListRequest {
        customer_id: *customer_id,
        vessel_id: *vessel_id,
        page_token: None,
    };
    let mut inventory = vec![];

    loop {
        if let Some(result) = client
            .invoke()
            .function_name(lambda)
            .payload(Blob::new(to_string(&request)?))
            .send()
            .await?
            .payload()
        {
            let response = from_slice::<InventoryListResponse>(result.as_ref())?;

            request.page_token = response.page_token;

            inventory.extend(response.inventory);
        }

        if request.page_token.is_none() {
            break;
        }
    }

    Ok(inventory)
}

pub async fn load_licenses(
    client: &Client,
    lambda: &String,
    customer_id: &Uuid,
    vessel_id: &Uuid,
) -> Result<Vec<LicenseFetchResponse>, RuntimeError> {
    let mut request = LicensesListRequest {
        customer_id: *customer_id,
        vessel_id: *vessel_id,
        page_token: None,
    };
    let mut licenses = vec![];

    loop {
        if let Some(result) = client
            .invoke()
            .function_name(lambda)
            .payload(Blob::new(to_string(&request)?))
            .send()
            .await?
            .payload()
        {
            let response = from_slice::<LicensesListResponse>(result.as_ref())?;

            request.page_token = response.page_token;

            licenses.extend(response.licenses);
        }

        if request.page_token.is_none() {
            break;
        }
    }

    Ok(licenses)
}

fn generate_key(_: Vec<InventoryFetchResponse>) -> String {
    // TODO: define list of keys required for JWT key
    String::from("<TODO>")
}

pub fn assemble_token(
    request: GeneratorRequest,
    inventory: Vec<InventoryFetchResponse>,
    licenses: Vec<LicenseFetchResponse>,
) -> Result<String, RuntimeError> {
    // key used for generating signature based on known hardware descriptors
    let key: Hmac<Sha512> = Hmac::new_from_slice(generate_key(inventory).as_bytes())?;

    // generate list of claims
    let claims = Claims::from_input(
        licenses,
        &request.customer_id,
        &request.vessel_id,
        request.issuer,
        request.audience,
    );

    claims.sign_with_key(&key).map_err(RuntimeError::from)
}
