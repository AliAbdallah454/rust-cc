FROM openresty/openresty:centos

RUN /usr/local/openresty/luajit/bin/luarocks install lua-resty-http
COPY nginx.conf /usr/local/openresty/nginx/conf/nginx.conf

CMD ["openresty", "-g", "daemon off;"]