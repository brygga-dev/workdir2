use crate::project::ProjectConfig;
use crate::utils::{self, CliEnv};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;
use std::process;

#[derive(Serialize, Debug)]
pub struct ComposeYml {
    pub version: &'static str,
    pub services: BTreeMap<String, ComposeService>,
}

#[derive(Serialize, Debug)]
pub struct ComposeService {
    pub volumes: Vec<String>,
    pub environment: BTreeMap<String, String>,
}

pub fn rebuild_container(
    env: &CliEnv,
    current_process: utils::CurrentProcess,
    project: ProjectConfig,
    service: String,
) -> io::Result<utils::CurrentProcess> {
    println!("Rebuilding and restarting service: {}", service);
    // Todo: Should do the following only if dev is running
    // Todo: Option to remove volumes?
    let p = dev_cmds(
        env,
        current_process,
        project,
        vec![
            vec![
                "rm".to_string(),
                "-s".to_string(),
                "-f".to_string(),
                service.clone(),
            ],
            vec!["build".to_string(), service.clone()],
            vec!["up".to_string(), "-d".to_string(), service.clone()],
        ],
    )?;
    Ok(p)
}

/// Convencience for single command
#[inline]
pub fn dev_cmd(
    env: &CliEnv,
    current_process: utils::CurrentProcess,
    project: ProjectConfig,
    user_args: Vec<String>,
) -> io::Result<utils::CurrentProcess> {
    dev_cmds(env, current_process, project, vec![user_args])
}

/// Allows multiple commands
// todo: Bit verbose to take String at times
pub fn dev_cmds(
    env: &CliEnv,
    mut current_process: utils::CurrentProcess,
    project: ProjectConfig,
    cmds: Vec<Vec<String>>,
) -> io::Result<utils::CurrentProcess> {
    // Generating local docker
    // It would be nice to detect changes beforehand
    // Also it may be a little out of place with wp
    // specifics here. Some module system would be cool
    crate::wp::create_wp_docker_yml(env, project.clone())?;

    let project_dir = project.dir(env);
    std::env::set_current_dir(project_dir)?;
    // Add base compose files
    let mut args: Vec<String> = [
        "-f",
        "../../workdir/server/base/docker-compose.yml",
        "-f",
        "../../workdir/server/dev/docker-compose.dev.yml",
        "-f",
        "../../workdir/server/base/docker-reimage.yml",
    ]
    .into_iter()
    .map(|i| i.to_string())
    .collect();
    // Then add local compose file(s?)
    args.push("-f".into());
    args.push("docker/dev.yml".into());
    for mut user_args in cmds {
        let mut args = args.clone();
        if user_args.len() > 0 {
            args.append(&mut user_args);
        } else {
            args.push("up".into());
        }
        // Run command
        // By default, the command inherits stdin, out, err
        // when used with .spawn()
        let mut cmd = process::Command::new("docker-compose");
        cmd.args(args);
        current_process = current_process.spawn_and_wait(cmd, false)?;
    }
    Ok(current_process)
}
