use serde::{Deserialize, Serialize};
use tsid::Tsid;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Id(#[serde(with = "tsid::serde::string")] Tsid);

impl Id {
    pub fn new() -> Self {
        Id(Tsid::new())
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
    type Err = tsid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Id(Tsid::from_str(s)?))
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
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
        Ok(Id(Tsid::from(val as u64)))
    }
}

impl utoipa::ToSchema for Id {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Id")
    }

    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::SchemaType::String)
            .format(Some(utoipa::openapi::schema::SchemaFormat::Custom(
                "tsid".to_string(),
            )))
            .description(Some("Time-Sorted Unique Identifier (TSID)"))
            .into()
    }
}

impl<'de> utoipa::IntoParams<'de> for Id {
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
