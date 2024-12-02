use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_ecs::{operation::{create_cluster::CreateClusterOutput, run_task::RunTaskOutput, stop_task::StopTaskOutput}, types::{AwsVpcConfiguration, NetworkConfiguration}};

type EcsError = aws_sdk_ecs::Error;

pub async fn launch_task(ecs: &aws_sdk_ecs::Client, cluster_name: &String, task_name: &String) -> Result<Option<RunTaskOutput>, EcsError> {

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