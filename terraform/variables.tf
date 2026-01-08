

variable "mysql_user" {
  type    = string
  default = "admin"
}
variable "mysql_password" {
  type      = string
  default   = "admin123"
  sensitive = true
}
variable "mysql_root_pwd" {
  type      = string
  sensitive = true
}
variable "mysql_port" {
  type    = number
  default = 3306
}
variable "mysql_database" {
  type    = string
  default = "todo"
}
variable "server_port" {
  type    = number
  default = 8080
}
