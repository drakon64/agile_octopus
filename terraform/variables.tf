variable "aws_region" {
  type = string
}

variable "phone_number" {
  type      = string
  sensitive = true
}
