use serde::{Deserialize, Serialize, Serializer, Deserializer};
use tsid::TSID;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(i64);

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as TSID string
        let tsid = TSID::from(self.0 as u64);
        tsid.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Force deserialization as string (valid for JSON strings and Path segments)
        let s = String::deserialize(deserializer)?;
        let tsid = TSID::try_from(s.as_str())
            .map_err(|e| serde::de::Error::custom(format!("Invalid TSID: {:?}", e)))?;
        Ok(Id(tsid.number() as i64))
    }
}

impl Id {
    pub fn new() -> Self {
        Id(tsid::create_tsid().number() as i64)
    }

    pub fn to_i64(&self) -> i64 {
        self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Convert back to TSID for string representation
        let tsid = TSID::from(self.0 as u64);
        write!(f, "{}", tsid.to_string())
    }
}

impl FromStr for Id {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match TSID::try_from(s) {
            Ok(val) => Ok(Id(val.number() as i64)),
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

// SQLX traits removed

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_serialization_roundtrip() {
        let original_id = Id::new();
        // Test String serialization (JSON)
        let json = serde_json::to_string(&original_id).expect("Serialize failed");
        // Should be a string due to tsid serde_as_string
        assert!(json.starts_with("\""));
        assert!(json.ends_with("\""));
        
        let parsed_id: Id = serde_json::from_str(&json).expect("Deserialize failed");
        assert_eq!(original_id, parsed_id, "JSON roundtrip mismatch");

        // Test FromStr via string value (URL path usage)
        let id_str = json.trim_matches('"');
        let from_str_id = Id::from_str(id_str).expect("FromStr failed");
        assert_eq!(original_id, from_str_id, "FromStr roundtrip mismatch");
    }
}
