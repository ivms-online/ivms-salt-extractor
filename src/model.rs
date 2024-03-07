/*
 * This file is part of the IVMS Online.
 *
 * @copyright 2023 - 2024 © by Rafał Wrzeszcz - Wrzasq.pl.
 */

use crate::api::LicenseFetchResponse;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

const MICROS_PER_TWO_YEARS: i64 = 62_208_000_000_000;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseClaim {
    pub count: Option<u8>,
    pub expires_at: Option<DateTime<FixedOffset>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Claims {
    #[serde(rename = "iss")]
    pub issuer: String,
    #[serde(rename = "sub")]
    pub user: String,
    #[serde(rename = "aud")]
    pub audience: String,
    #[serde(rename = "exp")]
    pub expires_at: i64,
    #[serde(rename = "iat")]
    pub issued_at: i64,
    #[serde(rename = "ivms:licenses")]
    pub licenses: HashMap<String, LicenseClaim>,
}

impl Claims {
    pub fn from_input(
        licenses: Vec<LicenseFetchResponse>,
        customer_id: &Uuid,
        vessel_id: &Uuid,
        issuer: String,
        audience: String,
    ) -> Self {
        let mut claims = HashMap::with_capacity(licenses.len());

        for license in licenses {
            claims.insert(
                license.license_key,
                LicenseClaim {
                    count: license.count,
                    expires_at: license.expires_at,
                },
            );
        }

        Self {
            issuer,
            user: format!("{customer_id}:{vessel_id}"),
            audience,
            // two years
            expires_at: (Utc::now() + Duration::microseconds(MICROS_PER_TWO_YEARS)).timestamp(),
            issued_at: Utc::now().timestamp(),
            licenses: claims,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api::LicenseFetchResponse;
    use crate::model::Claims;
    use chrono::{DateTime, FixedOffset, TimeZone, Utc};
    use uuid::{uuid, Uuid};

    const CUSTOMER_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000000");
    const VESSEL_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");
    const ISSUER: &str = "ivms-salt-executor";
    const AUDIENCE: &str = "test";
    const LICENSE_KEY_0: &str = "foo";
    const COUNT_0: u8 = 12;
    const LICENSE_KEY_1: &str = "bar";
    const LICENSE_KEY_2: &str = "baz";

    #[test]
    fn build_claims_from_licenses() {
        let expires_at_1: DateTime<FixedOffset> = Utc
            .with_ymd_and_hms(2011, 1, 30, 13, 58, 0)
            .unwrap()
            .with_timezone(&FixedOffset::east_opt(3600).unwrap());

        let before = Utc::now();

        let license0 = LicenseFetchResponse {
            license_key: LICENSE_KEY_0.to_string(),
            count: Some(COUNT_0),
            expires_at: None,
        };
        let license1 = LicenseFetchResponse {
            license_key: LICENSE_KEY_1.to_string(),
            count: None,
            expires_at: Some(expires_at_1),
        };
        let license2 = LicenseFetchResponse {
            license_key: LICENSE_KEY_2.to_string(),
            count: None,
            expires_at: None,
        };

        let claims = Claims::from_input(
            vec![license0, license1, license2],
            &CUSTOMER_ID,
            &VESSEL_ID,
            ISSUER.to_string(),
            AUDIENCE.to_string(),
        );

        let after = Utc::now();

        assert_eq!(ISSUER, claims.issuer);
        assert_eq!(format!("{CUSTOMER_ID}:{VESSEL_ID}"), claims.user);
        assert_eq!(AUDIENCE, claims.audience);
        assert!(claims.expires_at >= after.timestamp());
        assert!(claims.issued_at >= before.timestamp());
        assert!(claims.issued_at <= after.timestamp());
        assert_eq!(3, claims.licenses.len());

        let entry0 = claims.licenses.get(LICENSE_KEY_0);
        assert!(entry0.is_some());
        assert_eq!(Some(COUNT_0), entry0.unwrap().count);
        assert!(entry0.unwrap().expires_at.is_none());

        let entry1 = claims.licenses.get(LICENSE_KEY_1);
        assert!(entry1.is_some());
        assert!(entry1.unwrap().count.is_none());
        assert_eq!(Some(expires_at_1), entry1.unwrap().expires_at);

        let entry2 = claims.licenses.get(LICENSE_KEY_2);
        assert!(entry2.is_some());
        assert!(entry2.unwrap().count.is_none());
        assert!(entry2.unwrap().expires_at.is_none());
    }
}
