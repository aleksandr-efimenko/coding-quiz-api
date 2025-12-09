import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as elasticache from 'aws-cdk-lib/aws-elasticache';
import * as apigwv2 from 'aws-cdk-lib/aws-apigatewayv2';
import { HttpLambdaIntegration } from 'aws-cdk-lib/aws-apigatewayv2-integrations';
import { HttpLambdaAuthorizer, HttpLambdaResponseType } from 'aws-cdk-lib/aws-apigatewayv2-authorizers';
import * as path from 'path';

export class InfraStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // 1. VPC Configuration
    // Lambda needs to be in VPC to access Redis.
    // We use NAT Gateways = 0 to reduce costs; Lambdas will be in isolated subnets (no internet access).
    const vpc = new ec2.Vpc(this, 'CodingQuizVpc', {
      maxAzs: 2,
      natGateways: 0,
      subnetConfiguration: [
        {
          cidrMask: 24,
          name: 'Isolated',
          subnetType: ec2.SubnetType.PRIVATE_ISOLATED,
        },
      ],
    });

    // 2. Security Groups
    const lambdaSg = new ec2.SecurityGroup(this, 'LambdaSg', {
      vpc,
      description: 'Security Group for Lambda Functions',
      allowAllOutbound: true, // Allow outbound to Redis
    });

    const redisSg = new ec2.SecurityGroup(this, 'RedisSg', {
      vpc,
      description: 'Security Group for Redis Cluster',
    });

    // Allow Redis ingress from Lambda
    redisSg.addIngressRule(
      lambdaSg,
      ec2.Port.tcp(6379),
      'Allow connection from Lambda'
    );

    // 3. ElastiCache (Redis)
    const subnetGroup = new elasticache.CfnSubnetGroup(this, 'RedisSubnetGroup', {
      description: 'Subnet group for Redis',
      subnetIds: vpc.isolatedSubnets.map((subnet) => subnet.subnetId),
    });

    const redisCluster = new elasticache.CfnReplicationGroup(this, 'CodingQuizRedis', {
      replicationGroupDescription: 'Redis cluster for Coding Quiz API',
      engine: 'redis',
      cacheNodeType: 'cache.t4g.micro',
      numCacheClusters: 1,
      cacheSubnetGroupName: subnetGroup.ref,
      securityGroupIds: [redisSg.securityGroupId],
      port: 6379,
    });

    // 4. Lambda Functions
    const lambdaProps: lambda.FunctionProps = {
      runtime: lambda.Runtime.PROVIDED_AL2023,
      architecture: lambda.Architecture.ARM_64,
      handler: 'bootstrap',
      timeout: cdk.Duration.seconds(30),
      code: lambda.Code.fromAsset(path.join(__dirname, '../../target/lambda/authorizer')), // Placeholder, overridden below
    };

    const authorizerFn = new lambda.Function(this, 'AuthorizerFn', {
      ...lambdaProps,
      code: lambda.Code.fromAsset(path.join(__dirname, '../../target/lambda/authorizer')),
      // Authorizer doesn't necessarily need VPC if it just does stateless JWT validation
    });

    const apiFn = new lambda.Function(this, 'ApiFn', {
      ...lambdaProps,
      code: lambda.Code.fromAsset(path.join(__dirname, '../../target/lambda/coding-quiz-api')),
      vpc,
      vpcSubnets: { subnetType: ec2.SubnetType.PRIVATE_ISOLATED },
      securityGroups: [lambdaSg],
      environment: {
        RUST_LOG: 'info',
        // REDIS_URL: `redis://${redisCluster.attrPrimaryEndPointAddress}:6379` // Can un-comment when app supports it
      },
    });

    // 5. API Gateway (HTTP API)
    const authorizer = new HttpLambdaAuthorizer('JwtAuthorizer', authorizerFn, {
      responseTypes: [HttpLambdaResponseType.SIMPLE], // Returns { isAuthorized: true/false, context }
      resultsCacheTtl: cdk.Duration.minutes(5),
      identitySource: ['$request.header.Authorization'],
    });

    const apiIntegration = new HttpLambdaIntegration('ApiIntegration', apiFn);

    const httpApi = new apigwv2.HttpApi(this, 'CodingQuizApi', {
      apiName: 'Coding Quiz API',
      defaultAuthorizer: authorizer, // Protect everything by default
    });

    // Public Routes (Bypass Auth)
    httpApi.addRoutes({
      path: '/health',
      methods: [apigwv2.HttpMethod.GET],
      integration: apiIntegration,
      authorizer: new apigwv2.HttpNoneAuthorizer(), // No Auth
    });

    httpApi.addRoutes({
      path: '/swagger-ui/{proxy+}',
      methods: [apigwv2.HttpMethod.GET],
      integration: apiIntegration,
      authorizer: new apigwv2.HttpNoneAuthorizer(),
    });

    // Default catch-all for protected routes
    httpApi.addRoutes({
      path: '/{proxy+}',
      methods: [apigwv2.HttpMethod.ANY],
      integration: apiIntegration,
      // Uses default authorizer
    });

    // Exports
    new cdk.CfnOutput(this, 'ApiUrl', {
      value: httpApi.url ?? 'Something went wrong',
    });

    new cdk.CfnOutput(this, 'RedisEndpoint', {
      value: redisCluster.attrPrimaryEndPointAddress,
    });
  }
}
