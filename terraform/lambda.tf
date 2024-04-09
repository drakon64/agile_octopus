data "aws_iam_policy_document" "lambda_assume_role_policy" {
  statement {
    actions = ["sts:AssumeRole"]
    effect  = "Allow"

    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

data "aws_iam_policy_document" "lambda" {
  statement {
    actions   = ["sns:Publish"]
    effect    = "Allow"
    resources = ["*"]
  }
}

resource "aws_iam_role" "agile_octopus" {
  name               = "AgileOctopus"
  assume_role_policy = data.aws_iam_policy_document.lambda_assume_role_policy.json

  inline_policy {
    name   = "AgileOctopus"
    policy = data.aws_iam_policy_document.lambda.json
  }

  managed_policy_arns = ["arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"]
}

data "local_file" "agile_octopus" {
  filename = "${path.module}/../target/lambda/agile_octopus/bootstrap.zip"
}

resource "aws_lambda_function" "agile_octopus" {
  function_name = "agile_octopus"
  role          = aws_iam_role.agile_octopus.arn

  architectures = ["arm64"]

  environment {
    variables = {
      PHONE_NUMBER = var.phone_number
    }
  }

  filename         = data.local_file.agile_octopus.filename
  handler          = "rust.handler"
  memory_size      = 128
  package_type     = "Zip"
  runtime          = "provided.al2023"
  source_code_hash = data.local_file.agile_octopus.content_base64sha256
  timeout          = 3
}
