/*
 * This file is part of the IVMS Online.
 *
 * @copyright 2023 © by Rafał Wrzeszcz - Wrzasq.pl.
 */

#![feature(fn_traits)]
#![feature(future_join)]
#![feature(unboxed_closures)]

mod api;
mod generator;
mod model;
mod runtime_error;

use crate::api::{ApiError, GeneratorRequest, GeneratorResponse};
use crate::generator::{assemble_token, load_inventory, load_licenses};
use crate::runtime_error::RuntimeError;
use aws_config::load_from_env;
use aws_sdk_lambda::Client as LambdaClient;
use lambda_runtime::{Error, LambdaEvent};
use std::env::var;
use std::future::{join, Future};
use std::rc::Rc;
use tokio::main as tokio_main;
use wrzasqpl_commons_aws::{run_lambda, LambdaError};

fn generate_license_file(
    lambda: Rc<LambdaClient>,
    inventory_lister: Rc<String>,
    licenses_lister: Rc<String>,
) -> impl Fn<(LambdaEvent<GeneratorRequest>,), Output = impl Future<Output = Result<GeneratorResponse, ApiError>>> {
    move |event: LambdaEvent<GeneratorRequest>| {
        let lambda = lambda.clone();
        let inventory_lister = inventory_lister.clone();
        let licenses_lister = licenses_lister.clone();

        async move {
            let customer_id = event.payload.customer_id;
            let vessel_id = event.payload.vessel_id;

            let (inventory, licenses) = join!(
                load_inventory(lambda.as_ref(), inventory_lister.as_ref(), &customer_id, &vessel_id),
                load_licenses(lambda.as_ref(), licenses_lister.as_ref(), &customer_id, &vessel_id)
            )
            .await;

            let token = assemble_token(event.payload, inventory?, licenses?)?;

            Ok(GeneratorResponse::new(token))
        }
    }
}

#[tokio_main]
async fn main() -> Result<(), Error> {
    let config = &load_from_env().await;

    run_lambda!(
        "extractor:generate": generate_license_file(
            Rc::new(LambdaClient::new(config)),
            Rc::new(var("INVENTORY_LISTER").map_err(RuntimeError::ClientConfigLoadingError)?),
            Rc::new(var("LICENSES_LISTER").map_err(RuntimeError::ClientConfigLoadingError)?),
        ),
    )
}
