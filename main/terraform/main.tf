provider "aws" {
  region = var.aws_region
}

provider "archive" {}
data "archive_file" "zip" {
  type        = "zip"
  source_file = "../target/bootstrap"
  output_path = "walle_poc.zip"
}

data "aws_iam_policy_document" "policy" {
  statement {
    sid    = ""
    effect = "Allow"
    principals {
      identifiers = ["lambda.amazonaws.com"]
      type        = "Service"
    }
    actions = ["sts:AssumeRole"]
  }
}

resource "aws_iam_policy" "logs" {
  name        = "logs-policy"
  description = "Write and read logs policy."

  policy = <<EOF
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "logs:CreateLogGroup",
                "logs:CreateLogStream",
                "logs:PutLogEvents"
            ],
            "Resource": "*"
        }
    ]
  }
EOF
}

resource "aws_iam_role" "iam_for_lambda" {
  name               = "iam_for_lambda"
  assume_role_policy = data.aws_iam_policy_document.policy.json
}

resource "aws_iam_role_policy_attachment" "attach_logs" {
  role       = aws_iam_role.iam_for_lambda.name
  policy_arn = aws_iam_policy.logs.arn
}

resource "aws_lambda_function" "lambda" {
  function_name    = "walle_poc"
  filename         = data.archive_file.zip.output_path
  source_code_hash = data.archive_file.zip.output_base64sha256
  role             = aws_iam_role.iam_for_lambda.arn
  handler          = "walle_poc.lambda_handler"
  runtime          = "provided"
  environment {
    variables = {
      "DISCORD_PUBLIC_KEY" = local.envs["DISCORD_PUBLIC_KEY"]
    }
  }
}

resource "aws_lambda_function_url" "url1" {
  function_name      = aws_lambda_function.lambda.function_name
  authorization_type = "NONE"

  cors {
    allow_origins  = ["*"]
    allow_methods  = ["*"]
    allow_headers  = ["date", "keep-alive"]
    expose_headers = ["keep-alive", "date"]
    max_age        = 86400
  }
}

resource "aws_apigatewayv2_api" "lambda_api" {
  name          = "discord_gw"
  protocol_type = "HTTP"
}

resource "aws_apigatewayv2_stage" "lambda_default" {
  name        = "$default"
  api_id      = aws_apigatewayv2_api.lambda_api.id
  auto_deploy = true
}

resource "aws_apigatewayv2_integration" "gateway_to_lambda" {
  api_id                 = aws_apigatewayv2_api.lambda_api.id
  integration_type       = "AWS_PROXY"
  connection_type        = "INTERNET"
  integration_method     = "POST"
  integration_uri        = aws_lambda_function.lambda.arn
  payload_format_version = "2.0"
}

resource "aws_apigatewayv2_route" "route" {
  api_id    = aws_apigatewayv2_api.lambda_api.id
  route_key = "POST /discord"
  target    = "integrations/${aws_apigatewayv2_integration.gateway_to_lambda.id}"
}

resource "aws_lambda_permission" "execution_lambda_from_gateway" {
  statement_id  = "AllowExecutionFromAPIGateway"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.lambda.function_name
  principal     = "apigateway.amazonaws.com"
}
