# Consistent Hashing Server

A Rust-based distributed system implementation using AWS ECS (Elastic Container Service) with consistent hashing capabilities.

## Overview

This project implements a distributed server system that manages ECS tasks and provides utilities for network communication and health checking. The system is designed to work with AWS Fargate for containerized deployments.

## Core Components

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

- `get_specific_task(ecs: &aws_sdk_ecs::Client, cluster_name: &str, task_name: &str)`
  - Retrieves task definition ARNs for tasks matching a specific name
  - Returns: Vector of task definition ARNs

### Utilities (`utils.rs`)

Network utility functions:

- `get_private_ip()`
  - Retrieves the private IP address of the current instance
  - Returns: String containing the IP address

- `check_alive(ip: &str)`
  - Health check endpoint that verifies if a service is running at the specified IP
  - Sends GET request to `http://{ip}:7000/check-alive`
  - Returns: Boolean indicating if the service is responsive

## Network Configuration

The system uses the following AWS networking configuration:
- Subnets: 
  - subnet-9ceab8d1
  - subnet-7f0c0404
  - subnet-b7a352df
- Security Group: sg-02841803d91e15204
- Public IP assignment: Enabled

## API Endpoints

- Health Check: `GET http://{ip}:7000/check-alive`
  - Returns: 200 OK if service is running

## Dependencies

- aws-sdk-ecs: AWS ECS SDK for Rust
- reqwest: HTTP client for health checks
- std::net: UDP socket operations

## Error Handling

The system implements comprehensive error handling using:
- AWS SDK specific error types (`EcsError`)
- Standard Rust `Result` types
- Custom error handling for network operations

## Notes

- All tasks are deployed using AWS Fargate
- The system maintains running task states and can manage task lifecycle
- Network health checks are performed asynchronously