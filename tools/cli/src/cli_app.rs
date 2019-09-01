use crate::aws;
use crate::cli;
use crate::docker;
use crate::er;
use crate::git;
use crate::project;
use crate::server;
use crate::utils::{self, CliEnv};
use crate::workspace;
use crate::wp;
use failure::format_err;
use failure::Error;
use futures::future::lazy;

fn with_project<F, T>(env: &CliEnv, f: F) -> Result<T, failure::Error>
where
    F: FnOnce(project::ProjectConfig) -> Result<T, failure::Error>,
{
    match project::resolve_current_project_interactive(&env) {
        Ok(project) => f(project),
        Err(e) => {
            println!("Could not resolve project: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn with_server<F, T>(env: &CliEnv, f: F) -> Result<T, failure::Error>
where
    F: FnOnce(server::ServerConfig) -> Result<T, failure::Error>,
{
    match server::select_server(env) {
        Ok(server) => f(server),
        Err(e) => {
            eprintln!("Error selecting server: {:?}", e);
            std::process::exit(1);
        }
    }
}

pub fn run() -> Result<(), failure::Error> {
    let mut clap_app = cli::cli_app();
    let matches = clap_app.clone().get_matches();
    //println!("{:#?}", matches);
    let home_dir = match dirs::home_dir() {
        Some(home_dir) => home_dir,
        None => {
            println!("Couldn't resolve home directory when resolving projects dir");
            std::process::exit(1);
        }
    };
    // Workdir dir
    let mut workdir_dir = home_dir.clone();
    workdir_dir.push("workdir");
    // Projects dir
    let mut projects_dir = home_dir.clone();
    projects_dir.push("projects");
    let env = CliEnv::new(projects_dir, workdir_dir);
    match matches.subcommand() {
        ("init", Some(_sub_matches)) => {
            actix_rt::System::new("project-api")
                .block_on(lazy(|| project::init_cmd(&env)))
                .map_err(|e| Error::from(e))
            //env.display_result(res);
        }
        ("git-account", Some(_sub_matches)) => git::add_user(&env).map_err(|e| e.into()),
        ("server", Some(_sub_matches)) => server::add_server(&env).map_err(|e| e.into()),
        ("dev", Some(sub_matches)) => {
            let args = match sub_matches.values_of_lossy("dev-args") {
                Some(args) => args,
                None => Vec::new(),
            };
            with_project(&env, |project| {
                let current_process = utils::CurrentProcess::new();
                docker::dev_cmd(&env, current_process, project, args).map_err(|e| e.into())
            })
            .map(|_| ())
        }
        ("rebuild", Some(sub_matches)) => {
            let service = match sub_matches.value_of_lossy("service") {
                Some(service) => service.to_string(),
                None => {
                    eprint!("Service name is required");
                    std::process::exit(1);
                }
            };
            with_project(&env, |project| {
                let current_process = utils::CurrentProcess::new();
                docker::rebuild_container(&env, current_process, project, service)
                    .map_err(|e| e.into())
            })
            .map(|_| ())
        }
        ("sql", Some(sub_matches)) => {
            let args = match sub_matches.values_of_lossy("sql-args") {
                Some(args) => args,
                None => Vec::new(),
            };
            let sql = args.join(" ");
            with_project(&env, |project| {
                wp::sql_cli(&env, &sql)
            })
        }
        ("wp", Some(sub_matches)) => match sub_matches.subcommand() {
            ("cli", Some(sub_matches)) => {
                let args = match sub_matches.values_of_lossy("cli-args") {
                    Some(args) => args,
                    None => Vec::new(),
                };
                /*
                println!("Args: {:?}", &args);
                let (cmd, args) = match args.split_first() {
                    Some((cmd, args)) => (cmd, Vec::from(args)),
                    None => {
                        println!("Command not specified");
                        return;
                    }
                };*/
                with_project(&env, |project| {
                    let current_process = utils::CurrentProcess::new();
                    wp::wp_cli(&env, current_process, project, args, false).map_err(|e| e.into())
                })
                .map(|_| ())
            }
            ("install", Some(_sub_matches)) => with_project(&env, |project| {
                let current_process = utils::CurrentProcess::new();
                wp::wp_install(&env, project, current_process, false).map_err(|e| e.into())
            })
            .map(|_| ()),
            ("server-install", Some(_sub_matches)) => with_project(&env, |project| {
                let current_process = utils::CurrentProcess::new();
                wp::wp_install(&env, project, current_process, true).map_err(|e| e.into())
            })
            .map(|_| ()),
            ("sync-local", Some(_sub_matches)) => {
                with_project(&env, |project| wp::sync_local(&env, project, false))
            }
            ("server-sync-local", Some(_sub_matches)) => {
                with_project(&env, |project| wp::sync_local(&env, project, true))
            }
            ("clean", Some(_sub_matches)) => with_project(&env, |project| {
                let current_process = utils::CurrentProcess::new();
                wp::wp_clean(&env, project, current_process).map_err(|e| e.into())
            })
            .map(|_| ()),
            ("gen-docker-dev", Some(_sub_matches)) => with_project(&env, |project| {
                wp::create_wp_docker_yml(&env, project).map_err(|e| e.into())
            }),
            ("vscode-debug-config", Some(_sub_matches)) => with_project(&env, |project| {
                wp::gen_vscode_debug_config(&env, project).map_err(|e| e.into())
            }),
            (other, _) => return Err(format_err!("Unrecognized: {}", other)),
        },
        ("workspace", Some(sub_matches)) => match sub_matches.subcommand() {
            ("init-git", Some(_sub_matches)) => actix_rt::System::new("project-api")
                .block_on(lazy(|| workspace::init_git(&env)))
                .map_err(|e| Error::from(e)),
            ("push", Some(_sub_matches)) => workspace::push_workspace(&env).map_err(|e| e.into()),
            ("clone", Some(_sub_matches)) => workspace::clone_workspace(&env).map_err(|e| e.into()),
            // Could have push/pull here
            (other, _) => Err(format_err!("Unrecognized: {}", other)),
        },
        ("deploy", Some(_sub_matches)) => with_project(&env, |project| {
            with_server(&env, |server| {
                server::setup_server(&env, server).map_err(|e| e.into())
            })
        }),
        ("sync-server", Some(_)) => {
            with_server(&env, |server| server::sync_to_server(&env, server))
        }
        ("prod", Some(sub_matches)) => {
            let args = match sub_matches.values_of_lossy("prod-args") {
                Some(args) => args,
                None => Vec::new(),
            };
            with_project(&env, |project| project::prod(&env, &project, args))
        }
        ("ssh", Some(_sub_matches)) => with_server(&env, |server| server::ssh(&env, server)),
        ("wp-ssh", Some(_sub_matches)) => server::wp_cli_ssh(&env, 2345, None),
        ("server-wp-ssh", Some(_sub_matches)) => {
            with_server(&env, |server| server::wp_cli_ssh(&env, 2345, Some(&server)))
        }
        ("aws", Some(sub_matches)) => match sub_matches.subcommand() {
            ("provision", Some(_sub_matches)) => {
                aws::provision_server(&env, false).map_err(|e| e.into())
            }
            _ => {
                // Credentials config
                aws::aws_config(&env).map_err(|e| e.into())
            }
        },
        other => {
            env.error_msg(&format!("Command not recognized, {:?}", other));
            match clap_app.print_long_help() {
                Ok(_) => {
                    println!("");
                    Ok(())
                }
                Err(e) => Err(format_err!("Clap error: {:?}", e)),
            }
        }
    }
}
