import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import * as awsx from "@pulumi/awsx";

// 1. VPC Configuration
// Create a VPC for our Lambda and ElastiCache to reside in.
const vpc = new awsx.ec2.Vpc("coding-quiz-vpc", {
    numberOfAvailabilityZones: 2,
    natGateways: { strategy: "Single" }, // Save cost for dev
    tags: { Name: "coding-quiz-vpc" },
});

// 2. Security Groups
const lambdaSg = new aws.ec2.SecurityGroup("lambda-sg", {
    vpcId: vpc.vpcId,
    egress: [{ protocol: "-1", fromPort: 0, toPort: 0, cidrBlocks: ["0.0.0.0/0"] }],
    tags: { Name: "coding-quiz-lambda-sg" },
});

const redisSg = new aws.ec2.SecurityGroup("redis-sg", {
    vpcId: vpc.vpcId,
    ingress: [{
        protocol: "tcp",
        fromPort: 6379,
        toPort: 6379,
        securityGroups: [lambdaSg.id]
    }],
    tags: { Name: "coding-quiz-redis-sg" },
});

// 3. ElastiCache (Redis)
const subnetGroup = new aws.elasticache.SubnetGroup("redis-subnet-group", {
    subnetIds: vpc.privateSubnetIds,
});

const redisCluster = new aws.elasticache.ReplicationGroup("coding-quiz-redis", {
    replicationGroupId: "coding-quiz-redis",
    description: "Redis cluster for Coding Quiz API",
    engine: "redis",
    nodeType: "cache.t4g.micro", // Free tier eligible-ish
    numCacheClusters: 1,
    parameterGroupName: "default.redis7",
    subnetGroupName: subnetGroup.name,
    securityGroupIds: [redisSg.id],
    port: 6379,
});

// 4. IAM Role for Lambda
const lambdaRole = new aws.iam.Role("lambda-role", {
    assumeRolePolicy: aws.iam.assumeRolePolicyForPrincipal({ Service: "lambda.amazonaws.com" }),
});

new aws.iam.RolePolicyAttachment("lambda-basic-execution", {
    role: lambdaRole,
    policyArn: "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole",
});

new aws.iam.RolePolicyAttachment("lambda-vpc-access", {
    role: lambdaRole,
    policyArn: "arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole",
});

// 5. Lambda Functions
// Assumes binaries are built with `cargo lambda build --release`
// Paths: 
//   - api: ../target/lambda/coding-quiz-api/bootstrap
//   - authorizer: ../target/lambda/authorizer/bootstrap

const authorizerLambda = new aws.lambda.Function("authorizer-lambda", {
    code: new pulumi.asset.FileArchive("../target/lambda/authorizer"),
    handler: "bootstrap",
    role: lambdaRole.arn,
    runtime: "provided.al2023",
    architectures: ["arm64"], // Assuming cargo lambda build --arm64
    timeout: 5,
});

const apiLambda = new aws.lambda.Function("api-lambda", {
    code: new pulumi.asset.FileArchive("../target/lambda/coding-quiz-api"),
    handler: "bootstrap",
    role: lambdaRole.arn,
    runtime: "provided.al2023",
    architectures: ["arm64"],
    timeout: 30,
    vpcConfig: {
        subnetIds: vpc.privateSubnetIds,
        securityGroupIds: [lambdaSg.id],
    },
    environment: {
        variables: {
            // "REDIS_URL": redisCluster.primaryEndpointAddress.apply(addr => `redis://${addr}:6379`),
            // Removed RUST_LOG env var to let default env_logger logic work or set explicitly
            "RUST_LOG": "info",
        },
    },
});

// 6. API Gateway (HTTP)
const api = new aws.apigatewayv2.Api("coding-quiz-gateway", {
    protocolType: "HTTP",
});

// Custom Authorizer
const authorizer = new aws.apigatewayv2.Authorizer("jwt-authorizer", {
    apiId: api.id,
    authorizerType: "REQUEST",
    authorizerUri: authorizerLambda.invokeArn,
    identitySources: ["$request.header.Authorization"],
    name: "jwt-authorizer",
    authorizerPayloadFormatVersion: "2.0",
    enableSimpleResponses: true, // Simplified response format for Lambda Authorizers
});

// Permission for API Gateway to invoke Authorizer
new aws.lambda.Permission("api-gateway-invoke-authorizer", {
    action: "lambda:InvokeFunction",
    function: authorizerLambda.name,
    principal: "apigateway.amazonaws.com",
    sourceArn: pulumi.interpolate`${api.executionArn}/authorizers/${authorizer.id}`,
});

// Permission for API Gateway to invoke API Lambda
new aws.lambda.Permission("api-gateway-invoke-api", {
    action: "lambda:InvokeFunction",
    function: apiLambda.name,
    principal: "apigateway.amazonaws.com",
    sourceArn: pulumi.interpolate`${api.executionArn}/*/*`,
});

// Routes
const integration = new aws.apigatewayv2.Integration("api-integration", {
    apiId: api.id,
    integrationType: "AWS_PROXY",
    integrationUri: apiLambda.invokeArn,
    payloadFormatVersion: "2.0",
});

// Public Routes (No Auth)
new aws.apigatewayv2.Route("public-route", {
    apiId: api.id,
    routeKey: "GET /health",
    target: pulumi.interpolate`integrations/${integration.id}`,
});
new aws.apigatewayv2.Route("swaggger-route", {
    apiId: api.id,
    routeKey: "GET /swagger-ui/{proxy+}",
    target: pulumi.interpolate`integrations/${integration.id}`,
});

// Protected Routes (With Auth)
// Note: We might want everything under specific prefixes to be auth'd
new aws.apigatewayv2.Route("api-route", {
    apiId: api.id,
    routeKey: "ANY /{proxy+}",
    target: pulumi.interpolate`integrations/${integration.id}`,
    authorizationType: "CUSTOM",
    authorizerId: authorizer.id,
});

const stage = new aws.apigatewayv2.Stage("dev-stage", {
    apiId: api.id,
    name: "$default",
    autoDeploy: true,
});

// Exports
export const url = api.apiEndpoint;
export const redisEndpoint = redisCluster.primaryEndpointAddress;
