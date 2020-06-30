resource "aws_dynamodb_table" "locker" {
  name = "poor-locker-${var.stage}-lock-table"
  hash_key = "hash_key"

  read_capacity  = 1
  write_capacity = 1

  attribute {
    name = "hash_key"
    type = "S"
  }
}
