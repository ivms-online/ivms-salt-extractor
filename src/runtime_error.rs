/*
 * This file is part of the IVMS Online.
 *
 * @copyright 2023 © by Rafał Wrzeszcz - Wrzasq.pl.
 */

use aws_sdk_lambda::operation::invoke::InvokeError;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::SdkError;
use hmac::digest::InvalidLength;
use http::Response;
use jwt::Error as JwtError;
use serde_json::Error as SerializationError;
use std::env::VarError;
use std::fmt::{Debug, Display, Formatter, Result};
use thiserror::Error;
use uuid::Error as UuidError;

#[derive(Error, Debug)]
pub enum RuntimeError {
    ClientConfigLoadingError(VarError),
    LambdaInvokeError(#[from] SdkError<InvokeError, Response<SdkBody>>),
    InvalidKey(#[from] InvalidLength),
    JwtError(#[from] JwtError),
    SerializationError(#[from] SerializationError),
    UuidError(#[from] UuidError),
}

impl Display for RuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{self:?}")
    }
}
