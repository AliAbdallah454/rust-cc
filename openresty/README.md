## NGINX Configuration Documentation
- This document provides an overview of the NGINX configuration and Dockerfile used in the OpenResty setup. The configuration is designed to handle HTTP requests, perform consistent hashing, and interact with a backend service.

## Overview
- The NGINX server is configured to listen on port 8080 and is set up to handle requests through Lua scripts. The primary functionalities include:

1. Consistent Hashing: The server retrieves a node based on a key provided in the request body.

2. Dynamic Proxying: It proxies requests to a backend service based on the node determined from the consistent hashing mechanism.

3. Environment Variables: The configuration utilizes environment variables to manage server IPs and ports.

## Locations
- Root Location (/):
1. Reads the request body and expects a JSON payload.
2. Extracts the key from the JSON and calls the get_node function to retrieve the corresponding node and hash.
3. Proxies the request to the determined node using the PUT method.