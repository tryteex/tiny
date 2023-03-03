# tiny
"tiny" is a small web framework that allows you to write a Laravel-style frontend in Rust language.

Demo: [https://tiny.com.ua/](https://tiny.com.ua) 

It works as a FastCGI server paired with nginx.

## Compilation

### For windows
1. Install rust.
2. Make sure to check that "C++ tools" and "Windows 10 SDK" are installed for the linker.
2. Clone repository.
3. Go to tiny folder.
4. Type in console: 
```
cargo build --release
```

### For Linux (Ubuntu)
1. Install rust:
```
curl https://sh.rustup.rs -sSf | sh
```
2. Install pkg-config:
```
sudo apt install pkg-config
```
3. Clone repository.
4. Go to tiny folder.
5. Delete .cargo folder.
6. Type in console: 
```
cargo build --release
```

## Installation

### For windows

### For Linux (Ubuntu)
User home folder: /home/user/
1. Create folders: log, web, web/bin, web/www 
- /home/user/log
- /home/user/web/bin
- /home/user/web/www
2. Copy www folder from repository to /home/user/web/www
3. Copy app folder from repository to /home/user/web/bin
4. Copy tiny.sample.conf file from repository to /home/user/web/bin
5. Copy compiled file tiny from targer/release folder to /home/user/web/bin
6. Rename tiny.sample.conf to tiny.conf
7. Edit tiny.conf file
```
"log": "/home/user/log/tiny.log",
"salt": "write secured salt",
```
8. Config you nginx

nginx.conf
```nginx.conf
...
upstream fcgi_backend {
        server 127.0.0.1:12501 max_conns=100;
        keepalive 100;
}
server {
        listen 443 ssl http2;
        listen [::]:443 ssl http2;
        server_name example.com;

        access_log /home/user/log/nginx/access.log;
        error_log /home/user/log/nginx/error.log;

        ssl_certificate /etc/letsencrypt/live/example.com/fullchain.pem;
        ssl_certificate_key /etc/letsencrypt/live/example.com/privkey.pem;
        ssl_trusted_certificate /etc/letsencrypt/live/example.com/fullchain.pem;

        root /home/user/web/www;
        location / {
                autoindex off;

                location ~* ^.+\.(?:css|cur|js|jpg|gif|ico|png|xml|otf|ttf|eot|woff|woff2|svg)$ {
                        break;
                }

                location ~\.(ini|html)$ {
                        rewrite ^(.*)$ //$1/ last;
                }


                location ~ ^/$ {
                        rewrite ^(.*)$ // last;
                }

                location ~ ^// {
                        fastcgi_connect_timeout 1;
                        fastcgi_next_upstream timeout;
                        fastcgi_pass fcgi_backend;
                        fastcgi_read_timeout 5d;
                        fastcgi_param REDIRECT_URL $request_uri;
                        include fastcgi_params;
                        fastcgi_keep_conn on;
                        fastcgi_buffering off;
                        fastcgi_socket_keepalive on;
                        break;
                }

                if (!-f $request_filename) {
                        rewrite ^(.*)$ //$1 last;
                }
        }
}
```

# Run
1. Restart nginx
2. Type in console
```
/home/user/web/bin/tiny start
```