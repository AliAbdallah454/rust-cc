# Consistent Hashing Server

A Rust-based distributed system implementation using AWS ECS (Elastic Container Service) with consistent hashing capabilities.

## Overview

This project implements a distributed server system that manages ECS tasks and provides utilities for network communication and health checking. The system is designed to work with AWS Fargate for containerized deployments.

## Core Components

### Server Initialization

- When the Consistent Hashing Server is initialized, it calls the `get_ecs_task_private_ips` function to retrieve the private IPs of the tasks currently    running in the leaf service. This step is crucial as it allows the server to add these tasks to the consistent hashing ring before launching the HTTP server. This ensures that the server can effectively manage tasks, especially in scenarios where it is restarted.

### API Endpoints

- `/remove-node/<ip>` endpoint
    - When a task is gracefully shut down, it calls this endpoint to get the transactions that it needs to complete before shutting down.

- `/add-node` endpoint
    - When a task is spawning, it calls this endpoint to inform the server to add its `ip` into the consistent hashing ring.
    - It also handles the required transactions and the data that needs to be transferred from other nodes to the new node.

- `/get-node` endpoint
    - This endpoint is used to retrieve the node responsible for a given input key. It accepts a JSON payload containing the key, computes the hash, and returns the corresponding node and hash value. This is essential for ensuring that requests are routed to the correct service instance based on the consistent hashing algorithm.


### ECS Functions (`ecs_functions.rs`)

Collection of AWS ECS management functions:

#### Cluster Management
- `create_cluster(ecs: &aws_sdk_ecs::Client, cluster_name: &str)`
  - Creates a new ECS cluster with FARGATE capacity provider
  - Returns: `CreateClusterOutput`

#### Task Management
- `launch_task(ecs: &aws_sdk_ecs::Client, cluster_name: &str, task_name: &str)`
  - Launches a new Fargate task in the specified cluster
  - Configures networking with predefined subnets and security groups
  - Returns: `RunTaskOutput`

- `stop_task(ecs: &aws_sdk_ecs::Client, cluster_name: &str, ip: &str)`
  - Stops a specific task identified by its private IP
  - Returns: `StopTaskOutput`

#### Task Information
- `get_ecs_task_private_ips(ecs: &aws_sdk_ecs::Client, cluster_name: &str, service_name: &str)`
  - Retrieves private IPs of all running tasks in a service
  - Returns: Vector of IP addresses as strings

### Utilities (`utils.rs`)

Network utility functions:

- `get_private_ip()`
  - Retrieves the private IP address of the current instance
  - Returns: String containing the IP address

- `check_alive(ip: &str)`
  - Health check endpoint that verifies if a service is running at the specified IP
  - Sends GET request to `http://{ip}:7000/check-alive`
  - Returns: Boolean indicating if the service is responsive


## Notes

- All tasks are deployed using AWS Fargate
- The system maintains running task states and can manage task lifecycle