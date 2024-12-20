use aws_sdk_ecs::{operation::{create_cluster::CreateClusterOutput, run_task::RunTaskOutput, stop_task::StopTaskOutput}, types::{AwsVpcConfiguration, DesiredStatus, NetworkConfiguration}};

type EcsError = aws_sdk_ecs::Error;

pub async fn create_cluster(ecs: &aws_sdk_ecs::Client, cluster_name: &str) -> Result<Option<CreateClusterOutput>, EcsError> {

    let create_cluster_response = ecs.create_cluster()
                            .capacity_providers("FARGATE")
                            .cluster_name(cluster_name)
                            .send().await?;
    return Ok(Some(create_cluster_response));
    // if let Some(cluster) = create_cluster_response.cluster() {
    //     println!("Cluster created successfully: {:?}", cluster.cluster_arn());
    // } else {
    //     println!("Failed to create cluster");
    // }
    //
}

pub async fn launch_task(ecs: &aws_sdk_ecs::Client, cluster_name: &str, task_name: &str) -> Result<Option<RunTaskOutput>, EcsError> {

    let subnets = vec![
        String::from("subnet-9ceab8d1"),
        String::from("subnet-7f0c0404"),
        String::from("subnet-b7a352df")
    ];

    let security_groups = vec!["sg-02841803d91e15204".to_string()];

    let aws_vpc_configuration = AwsVpcConfiguration::builder()
                                                    .set_subnets(Some(subnets))
                                                    .set_security_groups(Some(security_groups))
                                                    .assign_public_ip(aws_sdk_ecs::types::AssignPublicIp::Enabled)
                                                    .build()?;

    let network_configuration = NetworkConfiguration::builder()
                                                    .awsvpc_configuration(aws_vpc_configuration)
                                                    .build();

    println!("Launching task");

    let run_response = ecs.run_task()
    .task_definition(task_name)
    .cluster(cluster_name)
    .launch_type(aws_sdk_ecs::types::LaunchType::Fargate)
    .network_configuration(network_configuration)
    .count(1)
    .send()
    .await?;

    return Ok(Some(run_response));

}

pub async fn stop_task(ecs: &aws_sdk_ecs::Client, cluster_name: &str, ip: &str) ->Result<Option<StopTaskOutput>, EcsError> {

    let task_list = ecs.list_tasks()
                        .cluster(cluster_name)
                        .send().await?;

    let task_description = ecs.describe_tasks()
                        .cluster(cluster_name)
                        .set_tasks(Some(task_list.task_arns().to_vec()))
                        .send().await?;

    for task in task_description.tasks() {
        let attachments = task.attachments().to_vec();
        let task_arn = task.task_arn().expect("Task arn should never fail");
        for attachment in attachments {
            for kvp in attachment.details().iter() {
                if kvp.name() == Some("privateIPv4Address") && kvp.value() == Some(&ip){
                    println!("{:?} -> {:?}", kvp.name(), kvp.value());
                    let stop_response = ecs.stop_task()
                                            .cluster(cluster_name)
                                            .task(task_arn)
                                            .send().await?;
                    return Ok(Some(stop_response));
                }
                
            }
        }
    }

    return Ok(None);

}


// you must handle empty error
pub async fn get_ecs_task_private_ips(ecs: &aws_sdk_ecs::Client, cluster_name: &str, service_name: &str) -> Result<Vec<String>, EcsError> {

    let list_tasks_output = ecs
        .list_tasks()
        .cluster(cluster_name)
        .service_name(service_name)
        .desired_status(DesiredStatus::Running)
        .send()
        .await?;

    let task_arns = list_tasks_output.task_arns.unwrap_or_default();

    if task_arns.is_empty() {
        println!("No running tasks found.");
    }

    let describe_tasks_output = ecs
        .describe_tasks()
        .cluster(cluster_name)
        .set_tasks(Some(task_arns))
        .send()
        .await?;

    let mut ips = vec![];
    for task in describe_tasks_output.tasks() {
        println!("last status: {}", task.last_status().unwrap());
        if task.last_status() != Some("RUNNING") { continue; }
        let attachments = task.attachments().to_vec();
        let _task_arn = task.task_arn().expect("Task arn should never fail");
        for attachment in attachments {
            for kvp in attachment.details().iter() {
                if kvp.name() == Some("privateIPv4Address"){
                    let ip = kvp.value().unwrap();
                    ips.push(ip.to_string());
                }
            }
        }
    }
    
    return Ok(ips);

}

pub async fn get_specific_task(ecs: &aws_sdk_ecs::Client, cluster_name: &str, task_name: &str) -> Result<Vec<String>, EcsError> {

    let mut arns = vec![];

    let task_list = ecs.list_tasks()
                        .cluster(cluster_name)
                        .send().await?;

    let task_description = ecs.describe_tasks()
                        .cluster(cluster_name)
                        .set_tasks(Some(task_list.task_arns().to_vec()))
                        .send().await?;

    for task in task_description.tasks() {
        let task_definition_arn = task.task_definition_arn().unwrap();
        let family = task_definition_arn
                        .split(':')
                        .nth(5)
                        .and_then(|s| s.split('/').skip(1).next())
                        .unwrap();
        if family == task_name {
            arns.push(task_definition_arn.to_string());
        }
        // println!("{}", family);
    }
    return Ok(arns);

}