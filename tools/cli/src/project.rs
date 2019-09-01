use crate::er::{self, Result};
use crate::git;
use crate::server;
use crate::utils::{self, CliEnv};
use failure::{format_err, Error};
use futures::{
    future::{self, Either},
    Future,
};
use serde::{Deserialize, Serialize};
use server::SshConn;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectConfig {
    pub name: String,
    pub git_repo_uri: String,
    pub git_user: String,
    pub server_name: String,
}

impl ProjectConfig {
    pub fn dir(&self, env: &CliEnv) -> PathBuf {
        project_dir(env, &self.name)
    }

    pub fn dir_and(&self, env: &CliEnv, extra: &str) -> PathBuf {
        let mut path = project_dir(env, &self.name);
        path.push(extra);
        path
    }

    pub fn get_server(&self, env: &CliEnv) -> Option<server::ServerConfig> {
        match server::get_config(env, &self.server_name) {
            Ok(server_config) => Some(server_config),
            Err(_) => None,
        }
    }

    /// Writes to a file given a path relative to
    /// project root
    pub fn write_file(&self, env: &CliEnv, file: &str, content: &str) -> io::Result<()> {
        let mut file_path = self.dir(env);
        file_path.push(file);
        utils::ensure_parent_dir(&file_path)?;
        fs::write(file_path, content)
    }
}

pub fn has_config(env: &CliEnv, project: &str) -> bool {
    env.config_dirs.projects.has_file(project)
}

/// Resolves current project interactive, either
/// by figuring it out from the current directory,
/// or asking the user to select one
// todo: Possibly make ProjEnv over CliEnv
// (not so composy)
// or something lessen boilerplate a little
// shorter name
pub fn resolve_current_project_interactive(env: &CliEnv) -> io::Result<ProjectConfig> {
    match resolve_current_project(env) {
        Ok(project_confir) => Ok(project_confir),
        Err(_e) => {
            // Todo: Could allow init here if in appropriate directory
            let projects = get_projects(env)?;
            // A little speculative, but for now, auto select
            // project if there is only one
            if projects.len() == 1 {
                println!("Only one project, selecting {}!", projects[0]);
                get_config(env, &projects[0])
            } else {
                env.select("Select project", &projects, None)
                    .and_then(|i| match projects.get(i) {
                        Some(project_name) => get_config(env, project_name),
                        None => utils::io_err("Error selecting project"),
                    })
            }
        }
    }
}

/// Resolve project from current directory, or error
pub fn resolve_current_project(env: &CliEnv) -> io::Result<ProjectConfig> {
    let cd = std::env::current_dir()?;
    let cd = cd
        .strip_prefix(&env.projects_dir)
        .map_err(|e| utils::io_error(format!("{:?}", e)))?;
    // Then use first component
    match cd.components().next() {
        Some(std::path::Component::Normal(os_str)) => {
            match get_config(env, &os_str.to_string_lossy()) {
                Ok(project_config) => Ok(project_config),
                Err(e) => utils::io_err(format!("Could not resolve project config: {:?}", e)),
            }
        }
        _ => utils::io_err("Could not resolve project dir"),
    }
}

pub fn get_config(env: &CliEnv, project: &str) -> io::Result<ProjectConfig> {
    let json_file = std::fs::File::open(env.config_dirs.projects.filepath(project))?;
    let buf_reader = io::BufReader::new(json_file);
    let config = serde_json::from_reader::<_, ProjectConfig>(buf_reader)?;
    Ok(config)
}

/// Returns names of projects
pub fn get_projects(env: &CliEnv) -> io::Result<Vec<String>> {
    utils::files_in_dir(&env.config_dirs.projects.0)
}

pub fn project_dir(env: &CliEnv, project: &str) -> PathBuf {
    env.get_project_path(project)
}

fn init_project_config(env: &CliEnv) -> io::Result<(ProjectConfig, git::InspectGit)> {
    let name = env.get_input("Project name", None)?;
    let current_config = if has_config(env, &name) {
        println!("Project config exists for: {}", &name);
        println!("Modifying entry.");
        Some(get_config(env, &name)?)
    } else {
        println!(
            "Project config does not exist, collecting data for: {}",
            &name
        );
        None
    };
    // Git account
    let git_account =
        git::select_account(env, current_config.as_ref().map(|c| c.git_user.to_owned()))?;
    let git_user = git_account.user.clone();
    // Git repo
    let project_git = git::inspect_git(project_dir(env, &name))?;
    let git_repo_uri = match project_git.origin_url.as_ref() {
        Some(origin_url) => {
            println!(
                "Get repo uri (from existing origin): {}",
                console::style(origin_url).magenta()
            );
            origin_url.to_owned()
        }
        None => {
            let user_uri = format!("https://github.com/{}", &git_user);
            let mut repo_type_options = vec![
                format!(
                    "User repo ({})",
                    console::style(user_uri.clone() + "/..").dim()
                ),
                "Repo uri (user or existing)".to_string(),
            ];
            let default = match current_config.as_ref() {
                Some(current_config) => {
                    repo_type_options.push(format!("Current: {}", &current_config.git_repo_uri));
                    Some(2)
                }
                None => None,
            };
            let repo_type = env.select("Repo uri", &repo_type_options, default)?;
            // todo: Remove after select?
            match repo_type {
                0 => {
                    // User repo
                    let repo_name = env.get_input(&format!("User repo {}/", &user_uri), None)?;
                    format!("{}/{}", &user_uri, &repo_name)
                }
                1 => {
                    // Full repo uri
                    env.get_input("Repo uri", None)?
                }
                _ => return utils::io_err("Unrecognized select"),
            }
        }
    };
    // Server
    let servers = server::get_servers(&env)?;
    let server_name = env
        .select(
            "Server",
            &servers,
            current_config
                .as_ref()
                .and_then(|c| servers.iter().position(|e| *e == c.server_name)),
        )
        .and_then(|i| {
            servers
                .get(i)
                .ok_or_else(|| utils::io_error("Could not resolve server"))
        })?
        .to_owned();

    let config = ProjectConfig {
        name,
        git_repo_uri,
        git_user,
        server_name,
    };
    println!("{:?}", &config);

    let content_str = serde_json::to_string_pretty(&config)?;

    // Todo: Consider keeping some in <project>/.project
    // Possibly this should be done after git setup after
    // this function, but might also be advantages saving the
    // data. The logic should handle it if re-running
    env.config_dirs.projects.write(&config.name, &content_str)?;
    Ok((config, project_git))
}

// Not ideally, but split initially with init_project_config
// to avoid type complexity with future and io:Result
pub fn init_cmd<'a>(env: &'a CliEnv) -> impl Future<Item = (), Error = Error> + 'a {
    let (config, project_git) = match init_project_config(env) {
        Ok(config) => config,
        Err(io_err) => return Either::A(future::err(Error::from(io_err))),
    };
    // Get projects git account
    let git_config = match git::get_config(env, &config.git_user) {
        Ok(git_config) => git_config,
        Err(io_err) => return Either::A(future::err(Error::from(io_err))),
    };
    Either::B(git::setup_git_dir(
        env,
        project_git,
        git_config,
        config.git_repo_uri.clone(),
    ))
}

/// Prod container command through ssh
pub fn prod(env: &CliEnv, project: &ProjectConfig, mut user_args: Vec<String>) -> Result<()> {
    // Sync docker file
    crate::wp::create_docker_prod_yml(env, &project)?;
    let server = match project.get_server(env) {
        Some(server) => server,
        None => {
            return Err(format_err!(
                "Missing server in project config, required for prod"
            ))
        }
    };
    let conn = SshConn::connect(env, &server)?;
    let sftp = conn.sftp()?;
    crate::server::SyncSet::from_file(
        project.dir_and(env, "docker/prod.yml"),
        server.home_dir_and(&format!("projects/{}/docker", project.name)),
        &sftp,
        false,
    )?
    .sync_plain(&sftp)?;
    drop(sftp);
    // Start docker
    // todo: Consider some better path handling
    // Currently expecting to be called from project dir,
    // Could consider absolute paths also
    let mut args: Vec<String> = [
        "../../workdir/server/base/docker-compose.yml",
        "../../workdir/server/prod/docker-compose.prod.yml",
        "../../workdir/server/base/docker-reimage.yml",
        "docker/prod.yml",
    ]
    .into_iter()
    .flat_map(|e| vec!["-f", e])
    .map(|e| String::from(e))
    .collect();
    // Apply user_args or default to "up"
    if user_args.len() > 0 {
        args.append(&mut user_args);
    } else {
        args.push("up".into());
    }
    let cd = format!(
        "cd {}/projects/{}",
        server.home_dir().to_string_lossy(),
        project.name
    );
    conn.exec(format!("{}; docker-compose {}", cd, args.join(" ")))?;
    Ok(())
}
