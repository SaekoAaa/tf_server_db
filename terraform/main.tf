


resource "docker_image" "mysql" {
  name         = "mysql:latest"
  keep_locally = true
}
resource "docker_image" "nginx" {
  name         = "nginx:latest"
  keep_locally = true
}
data "docker_image" "server" {
  name = "db_server:1.0.0"
}

locals {
  db_servers = [
    for c in docker_container.server :
    c.name
  ]
}

resource "local_file" "nginx_config" {
  filename = "${abspath(path.module)}/nginx.conf"
  content = templatefile("${abspath(path.module)}/nginx.conf.tpl", {
    servers  = local.db_servers,
    app_port = var.server_port
  })
}

resource "docker_network" "main_network" {
  name = "main_network"
}

resource "docker_container" "mysql" {
  name  = "mysql_db"
  image = docker_image.mysql.image_id
  env = ["MYSQL_USER=${var.mysql_user}",
    "MYSQL_PASSWORD=${var.mysql_password}",
    "MYSQL_ROOT_PASSWORD=${var.mysql_root_pwd}",
    "MYSQL_DATABASE=${var.mysql_database}"
  ]
  networks_advanced {
    name = docker_network.main_network.name
  }
}

resource "docker_container" "server" {
  count = 1
  name  = "db_server_${count.index}"
  image = data.docker_image.server.name
  depends_on = [
    docker_container.mysql
  ]
  command = [
    "sh", "-c",
    "/bin/server || echo SERVER_CRASHED; sleep 300"
  ]
  env = [
    "DB_USER=${var.mysql_user}",
    "DB_PASSWORD=${var.mysql_password}",
    "DB_DATABASE=${var.mysql_database}",
    "DB_ADDRESS=${docker_container.mysql.name}",
    "DB_PORT=${var.mysql_port}",
    "PORT=${var.server_port}"
  ]
  networks_advanced {
    name = docker_network.main_network.name
  }
}

resource "docker_container" "nginx" {
  name  = "nginx"
  image = docker_image.nginx.image_id
  depends_on = [
    docker_container.server
  ]
  networks_advanced {
    name = docker_network.main_network.name
  }
  volumes {
    host_path      = "${abspath(path.module)}/nginx.conf"
    container_path = "/etc/nginx/nginx.conf"
    read_only      = true
  }

}
