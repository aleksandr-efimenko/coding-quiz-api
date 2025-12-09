# Deploying Coding Quiz API to AWS

This project uses **AWS CDK** for infrastructure and **Cargo Lambda** for building Rust binaries.

## Prerequisites

1.  **AWS Credentials**: Ensure you have AWS credentials configured (`~/.aws/credentials` or env vars).
2.  **Node.js**: Installed.
3.  **Cargo Lambda**: Installed.
    ```bash
    brew tap cargo-lambda/cargo-lambda
    brew install cargo-lambda
    ```

## Deployment Steps

1.  **Build Lambda Binaries**
    Build the Rust crates for AWS Lambda (ARM64 architecture).
    ```bash
    cargo lambda build --release --arm64 --workspace
    ```
    *This creates artifacts in `target/lambda/coding-quiz-api` and `target/lambda/authorizer`.*

2.  **Deploy Infrastructure**
    Go to the infrastructure directory and run CDK.
    ```bash
    cd infra
    npm install  # First time only
    npx cdk deploy
    ```

3.  **Verify & Use**
    CDK will output the `ApiUrl` of your API Gateway.
    
    -   **Health Check**: `curl <url>/health`
    -   **Swagger UI**: Open `<url>/swagger-ui/` in your browser.

## Clean Up
To destroy all resources:
```bash
cd infra
npx cdk destroy
```
