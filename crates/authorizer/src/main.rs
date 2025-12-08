use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use authorizer::validate_token;

#[derive(Deserialize)]
struct CustomAuthorizerRequest {
    #[serde(rename = "authorizationToken")]
    authorization_token: Option<String>,
    #[serde(rename = "methodArn")]
    method_arn: String,
}

#[derive(Serialize)]
struct CustomAuthorizerResponse {
    principalId: String,
    policyDocument: PolicyDocument,
    context: Option<Value>,
}

#[derive(Serialize)]
struct PolicyDocument {
    Version: String,
    Statement: Vec<Statement>,
}

#[derive(Serialize)]
struct Statement {
    Action: String,
    Effect: String,
    Resource: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn handler(event: LambdaEvent<CustomAuthorizerRequest>) -> Result<CustomAuthorizerResponse, Error> {
    let (event, _) = event.into_parts();
    
    // 1. Extract Token
    let token = match event.authorization_token {
        Some(t) => t.replace("Bearer ", ""),
        None => return Ok(generate_policy("user", "Deny", &event.method_arn)),
    };

    // 2. Validate Token
    match validate_token(&token) {
        Ok(claims) => {
            // 3. Allow Access
            // In a real scenario, you might restrict resource access based on role
            log::info!("User {} authorized", claims.sub);
            Ok(generate_policy(&claims.sub, "Allow", &event.method_arn))
        },
        Err(_) => {
            // 4. Deny Access
            log::info!("Token validation failed");
            Ok(generate_policy("user", "Deny", &event.method_arn))
        }
    }
}

fn generate_policy(principal_id: &str, effect: &str, resource: &str) -> CustomAuthorizerResponse {
    CustomAuthorizerResponse {
        principalId: principal_id.to_string(),
        policyDocument: PolicyDocument {
            Version: "2012-10-17".to_string(),
            Statement: vec![Statement {
                Action: "execute-api:Invoke".to_string(),
                Effect: effect.to_string(),
                Resource: resource.to_string(), 
                // Note: 'resource' here is the method ARN. In production you might want to wildcard this 
                // to avoid cached policy issues if the ARN changes slightly (e.g. diff paths)
            }],
        },
        context: None,
    }
}
