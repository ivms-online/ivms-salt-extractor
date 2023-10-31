/*
 * This file is part of the IVMS Online.
 *
 * @copyright 2023 © by Rafał Wrzeszcz - Wrzasq.pl.
 */

#![feature(async_closure, future_join)]

use aws_config::load_from_env;
use aws_sdk_lambda::Client as LambdaClient;
use aws_smithy_types::Blob;
use cucumber::{given, then, when, World};
use futures::future::join_all;
use hmac::digest::KeyInit;
use hmac::Hmac;
use jwt::{Claims, VerifyWithKey};
use serde_json::{from_slice, json, to_vec};
use sha2::Sha512;
use std::collections::HashMap;
use std::env::{var, VarError};
use std::future::join;
use tokio::main as tokio_main;

// TODO: temporary fixed key
const TEMP_KEY: &str = "<TODO>";

macro_rules! serialize_blob {
    ($($data:tt)+) => {
        Blob::new(
            to_vec(&json!($($data)+)).unwrap()
        )
    };
}

#[derive(World, Debug)]
#[world(init = Self::new)]
struct TestWorld {
    // initialization scope
    generator_lambda: String,
    inventory_creator_lambda: String,
    inventory_deleter_lambda: String,
    license_creator_lambda: String,
    license_deleter_lambda: String,
    lambda: LambdaClient,
    // test run scope
    cleanup_inventories: Vec<(String, String, String, String)>,
    cleanup_licenses: Vec<(String, String, String)>,
    token_claims: Option<Claims>,
}

impl TestWorld {
    async fn new() -> Result<Self, VarError> {
        let config = &load_from_env().await;

        Ok(Self {
            generator_lambda: var("GENERATOR_LAMBDA")?,
            inventory_creator_lambda: var("INVENTORY_CREATOR_LAMBDA")?,
            inventory_deleter_lambda: var("INVENTORY_DELETER_LAMBDA")?,
            license_creator_lambda: var("LICENSES_CREATOR_LAMBDA")?,
            license_deleter_lambda: var("LICENSES_DELETER_LAMBDA")?,
            lambda: LambdaClient::new(config),
            cleanup_inventories: vec![],
            cleanup_licenses: vec![],
            token_claims: None,
        })
    }
}

async fn delete_inventory(
    world: &TestWorld,
    customer_id: &Option<String>,
    vessel_id: &Option<String>,
    inventory_type: &Option<String>,
    inventory_id: &Option<String>,
) {
    if let (Some(customer_id), Some(vessel_id), Some(inventory_type), Some(inventory_id)) =
        (customer_id, vessel_id, inventory_type, inventory_id)
    {
        world
            .lambda
            .invoke()
            .function_name(world.inventory_deleter_lambda.as_str())
            .payload(serialize_blob!({
                "customer_id": customer_id,
                "vessel_id": vessel_id,
                "inventory_type": inventory_type,
                "inventory_id": inventory_id,
            }))
            .send()
            .await
            .unwrap();
    }
}

async fn delete_license(
    world: &TestWorld,
    customer_id: &Option<String>,
    vessel_id: &Option<String>,
    license_key: &Option<String>,
) {
    if let (Some(customer_id), Some(vessel_id), Some(license_key)) = (customer_id, vessel_id, license_key) {
        world
            .lambda
            .invoke()
            .function_name(world.license_deleter_lambda.as_str())
            .payload(serialize_blob!({
                "customer_id": customer_id,
                "vessel_id": vessel_id,
                "license_key": license_key,
            }))
            .send()
            .await
            .unwrap();
    }
}

#[tokio_main]
async fn main() {
    TestWorld::cucumber()
        .after(|_feature, _rule, _scenario, _finished, world| {
            Box::pin(async move {
                if let Some(&mut ref cleanup) = world {
                    let inventory_tasks = cleanup.cleanup_inventories.iter().map(async move |key| {
                        delete_inventory(
                            &cleanup,
                            &Some(key.0.clone()),
                            &Some(key.1.clone()),
                            &Some(key.2.clone()),
                            &Some(key.3.clone()),
                        )
                        .await
                    });

                    let license_tasks = cleanup.cleanup_licenses.iter().map(async move |key| {
                        delete_license(
                            &cleanup,
                            &Some(key.0.clone()),
                            &Some(key.1.clone()),
                            &Some(key.2.clone()),
                        )
                        .await
                    });

                    join!(join_all(inventory_tasks), join_all(license_tasks),).await;
                }
            })
        })
        .run_and_exit("tests/features")
        .await;
}

// Given …

#[given(
    expr = "There is an inventory {string} of type {string} for vessel {string} of customer {string} with serial number {string}"
)]
async fn there_is_an_inventory(
    world: &mut TestWorld,
    inventory_id: String,
    inventory_type: String,
    vessel_id: String,
    customer_id: String,
    serial_number: String,
) {
    world.cleanup_inventories.push((
        customer_id.clone(),
        vessel_id.clone(),
        inventory_type.clone(),
        inventory_id.clone(),
    ));

    world
        .lambda
        .invoke()
        .function_name(world.inventory_creator_lambda.as_str())
        .payload(serialize_blob!({
            "customer_id": customer_id,
            "vessel_id": vessel_id,
            "inventory_type": inventory_type,
            "inventory_id": inventory_id,
            "serial_number": serial_number,
        }))
        .send()
        .await
        .unwrap();
}

#[given(
    expr = "There is a license {string} for vessel {string} of customer {string} with count {int} and expiration date {string}"
)]
async fn there_is_a_license(
    world: &mut TestWorld,
    license_key: String,
    vessel_id: String,
    customer_id: String,
    count: usize,
    expires_at: String,
) {
    world
        .cleanup_licenses
        .push((customer_id.clone(), vessel_id.clone(), license_key.clone()));

    world
        .lambda
        .invoke()
        .function_name(world.license_creator_lambda.as_str())
        .payload(serialize_blob!({
            "customer_id": customer_id,
            "vessel_id": vessel_id,
            "license_key": license_key,
            "count": count,
            "expires_at": expires_at,
        }))
        .send()
        .await
        .unwrap();
}

// When …

#[when(
    expr = "I request JWT token for vessel {string} of customer {string} with {string} issuer for {string} audience"
)]
async fn i_request_jwt_token(
    world: &mut TestWorld,
    customer_id: String,
    vessel_id: String,
    issuer: String,
    audience: String,
) {
    let key: Hmac<Sha512> = Hmac::new_from_slice(TEMP_KEY.as_bytes()).unwrap();

    world.token_claims = from_slice::<HashMap<String, String>>(
        world
            .lambda
            .invoke()
            .function_name(world.generator_lambda.as_str())
            .payload(serialize_blob!({
                "customerId": customer_id,
                "vesselId": vessel_id,
                "issuer": issuer,
                "audience": audience,
            }))
            .send()
            .await
            .ok()
            .as_ref()
            .and_then(|response| response.payload())
            .unwrap()
            .as_ref(),
    )
    .ok()
    .as_ref()
    .and_then(|response| response.get("token"))
    .and_then(|token| token.verify_with_key(&key).ok());
}

// Then …

#[then(expr = "I have JWT token with {string} issuer claim")]
async fn i_have_jwt_with_issuer(world: &mut TestWorld, issuer: String) {
    assert_eq!(
        issuer,
        world
            .token_claims
            .as_ref()
            .and_then(|claims| claims.registered.issuer.clone())
            .unwrap()
    );
}

#[then(expr = "I have JWT token for {string} audience claim")]
async fn i_have_jwt_with_audience(world: &mut TestWorld, audience: String) {
    assert_eq!(
        audience,
        world
            .token_claims
            .as_ref()
            .and_then(|claims| claims.registered.audience.clone())
            .unwrap()
    );
}

#[then(expr = "I have JWT token for {string} sub user claim")]
async fn i_have_jwt_with_sub(world: &mut TestWorld, user: String) {
    assert_eq!(
        user,
        world
            .token_claims
            .as_ref()
            .and_then(|claims| claims.registered.subject.clone())
            .unwrap()
    );
}

#[then(expr = "I can find license {string} with count {int} and expiration date {string} in JWT claims")]
async fn i_can_find_license(world: &mut TestWorld, license_key: String, count: usize, expires_at: String) {
    let entry = world
        .token_claims
        .as_ref()
        .and_then(|claims| claims.private["ivms:licenses"].as_object())
        .and_then(|licenses| licenses.get(&license_key))
        .and_then(|license| license.as_object())
        .unwrap();

    assert_eq!(Some(count as u64), entry.get("count").and_then(|value| value.as_u64()));
    assert_eq!(
        Some(expires_at),
        entry
            .get("expiresAt")
            .and_then(|value| value.as_str().map(String::from))
    );
}

#[then(expr = "I can not find license {string} in JWT claims")]
async fn i_can_not_find_license(world: &mut TestWorld, license_key: String) {
    assert!(!world
        .token_claims
        .as_ref()
        .and_then(|claims| claims.private["ivms:licenses"].as_object())
        .unwrap()
        .contains_key(&license_key));
}
