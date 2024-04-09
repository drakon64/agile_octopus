data "aws_iam_policy_document" "scheduler_assume_role_policy" {
  statement {
    actions = ["sts:AssumeRole"]
    effect  = "Allow"

    principals {
      type        = "Service"
      identifiers = ["scheduler.amazonaws.com"]
    }
  }
}

data "aws_iam_policy_document" "scheduler" {
  statement {
    actions   = ["lambda:InvokeFunction"]
    effect    = "Allow"
    resources = [aws_lambda_function.agile_octopus.arn]
  }
}

resource "aws_iam_role" "scheduler" {
  name               = "AgileOctopusScheduler"
  assume_role_policy = data.aws_iam_policy_document.scheduler_assume_role_policy.json

  inline_policy {
    name   = "AgileOctopusScheduler"
    policy = data.aws_iam_policy_document.scheduler.json
  }
}

resource "aws_scheduler_schedule" "agile_octopus" {
  flexible_time_window {
    mode = "OFF"
  }

  schedule_expression = "cron(30 16 * * ? *)"

  target {
    arn      = aws_lambda_function.agile_octopus.arn
    role_arn = aws_iam_role.scheduler.arn

    retry_policy {
      maximum_retry_attempts = 0
    }
  }

  name                         = "AgileOctopus"
  schedule_expression_timezone = "Europe/London"
}
