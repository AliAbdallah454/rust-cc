FROM openresty/openresty:centos

RUN /usr/local/openresty/luajit/bin/luarocks install lua-resty-http
RUN /usr/local/openresty/luajit/bin/luarocks install lua-resty-env

COPY nginx.conf /usr/local/openresty/nginx/conf/nginx.conf

EXPOSE 8080

CMD ["openresty", "-g", "daemon off;"]