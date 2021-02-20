# lambda-webhook
A small Rust program to handle GitHub webhooks using AWS Lambda and API Gateway

## How to Deploy
To deploy the webhooks handler as a Lambda function using the AWS CLI, you first need to build a deployment package (`lambda.zip`):
```sh
$ cargo build --release --target x86_64-unknown-linux-musl
$ cp ./target/x86_64-unknown-linux-musl/release/lambda-webhook ./bootstrap && zip lambda.zip bootstrap && rm bootstrap
```

Then, create a Lambda function using the AWS CLI:
```sh
$ aws lambda create-function --function-name rust-github-webhooks \
  --handler index.handler \
  --zip-file fileb://lambda.zip \
  --runtime provided \
  --role arn:aws:iam::XXXXXXXXXXXXX:role/your-lambda-execution-role \
  --environment Variables={RUST_BACKTRACE=1} \
  --tracing-config Mode=Active
```

Now you can test the function using the AWS CLI:
```sh
$ aws lambda invoke \
  --function-name rust-github-webhooks \
  --payload '{"first_name": "world"}' \
  output.json
$ cat output.json
```

## Reference
- [`lambda_runtime` crate [crates.io]](https://crates.io/crates/lambda_runtime)
