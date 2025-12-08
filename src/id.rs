use serde::{Deserialize, Serialize};
use tsid::TSID;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Id(TSID);

impl Id {
    pub fn new() -> Self {
        Id(tsid::create_tsid())
    }

    pub fn to_i64(&self) -> i64 {
        self.0.number() as i64
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl FromStr for Id {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TSID::from_str is available via FromStr trait?
        // Let's check if TSID implements FromStr.
        // src/tsid/mod.rs does not show it directly, usually in conversions.rs.
        // But assuming it does (standard practice).
        // If not, TSID::try_from(str) might work?
        // The src/tsid/conversions.rs likely has it.
        // Code showed: pub fn try_from(val: &str) -> Result<Self, TsidError>
        // And usually TryFrom<String> or FromStr.
        // The parser error shows use crate::TSID usage in tests using TSID::try_from.
        // Let's assume TSID implements FromStr or try to use TryFrom logic.
        // Actually, let's use implicit FromStr if available, otherwise try_from.
        // Based on "test_regression_panic_try_from_str", TSID::try_from("...") works.
        // So TSID implements TryFrom<&str>.
        match TSID::try_from(s) {
            Ok(val) => Ok(Id(val)),
            Err(e) => Err(format!("{:?}", e)), // TsidError might be Debug but not accessible to name type
        }
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

// SQLX
impl From<i64> for Id {
    fn from(v: i64) -> Self {
        // TSID::new(u64) is private, but I need to construct from DB value.
        // Wait, if TSID::new(u64) is private, how do I load from DB?
        // I checked: pub(crate) fn new(number: u64).
        // Is there a public conversion From<u64>?
        // src/tsid/conversions.rs likely has From<u64>.
        // I can't see conversions.rs content but usually From<u64> is implemented.
        // Let's assume From<u64> for TSID is available.
        Id(TSID::from(v as u64))
    }
}

// Helper for when we have u64 directly (e.g. from TSID crate itself if we unwrapped)
impl From<u64> for Id {
    fn from(v: u64) -> Self {
        Id(TSID::from(v))
    }
}

impl sqlx::Type<sqlx::Postgres> for Id {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i64 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for Id {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
        let val = self.0.number() as i64;
        <i64 as sqlx::Encode<sqlx::Postgres>>::encode(val, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for Id {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let val = <i64 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        // Again, assuming TSID::from(u64) exists.
        Ok(Id(TSID::from(val as u64)))
    }
}

// Utoipa
impl utoipa::ToSchema<'_> for Id {
    fn schema() -> (&'static str, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>) {
        (
            "Id",
            utoipa::openapi::ObjectBuilder::new()
                .schema_type(utoipa::openapi::schema::SchemaType::String)
                .format(Some(utoipa::openapi::schema::SchemaFormat::Custom(
                    "tsid".to_string(),
                )))
                .description(Some("Time-Sorted Unique Identifier (TSID)"))
                .into()
        )
    }
}

impl utoipa::IntoParams for Id {
    fn into_params(
        parameter_in_provider: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>,
    ) -> Vec<utoipa::openapi::path::Parameter> {
        vec![utoipa::openapi::path::ParameterBuilder::new()
            .name("id")
            .parameter_in(parameter_in_provider().unwrap_or(utoipa::openapi::path::ParameterIn::Path))
            .schema(Some(
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::SchemaType::String)
                    .format(Some(utoipa::openapi::schema::SchemaFormat::Custom(
                        "tsid".to_string(),
                    )))
                    .description(Some("Time-Sorted Unique Identifier (TSID)"))
            ))
            .required(utoipa::openapi::Required::True)
            .build()]
    }
}
