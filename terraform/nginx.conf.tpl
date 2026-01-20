
events {}

http {
  upstream servers_list {
    %{for server in servers ~}
    server ${server}:${app_port};
    %{endfor ~}
  }
  server {
        listen 80;    
        location / {
            proxy_pass http://servers_list;
        }
  }
}
