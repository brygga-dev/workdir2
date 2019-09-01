use clap::{self, App, Arg, SubCommand};

pub fn cli_app() -> App<'static, 'static> {
    App::new("Project-cli")
        .version("0.1")
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize a project")
                .arg(Arg::with_name("name")),
        )
        .subcommand(
            SubCommand::with_name("dev")
                .about("For a given project, creates and starts dev containers")
                .setting(clap::AppSettings::TrailingVarArg)
                .arg(
                    Arg::with_name("dev-args")
                        .multiple(true)
                        .help("Arguments passed to docker-compose"),
                ),
        )
        .subcommand(
            SubCommand::with_name("sql")
                .about("Wp sql command")
                .setting(clap::AppSettings::TrailingVarArg)
                .arg(
                    Arg::with_name("sql-args")
                        .multiple(true)
                        .help("Sql command"),
                ),
        )
        .subcommand(
            SubCommand::with_name("rebuild")
                .about("Rebuilds a given service/container")
                .arg(Arg::with_name("service").help("Container to rebuild and restart")),
        )
        .subcommand(
            SubCommand::with_name("wp")
                .about("Wordpress specific commands")
                .subcommand(
                    SubCommand::with_name("cli")
                        .about("Runs wp-cli commands")
                        .setting(clap::AppSettings::TrailingVarArg)
                        .arg(Arg::with_name("cli-args").multiple(true)),
                )
                .subcommand(
                    SubCommand::with_name("install")
                        .about("Runs wp installation process on dev server"),
                )
                .subcommand(
                    SubCommand::with_name("server-install")
                        .about("Runs wp installation process on prod server"),
                )
                .subcommand(
                    SubCommand::with_name("sync-local")
                        .about("Install deps and activates local plugins and themes"),
                )
                .subcommand(
                    SubCommand::with_name("server-sync-local")
                        .about("Install deps and activates local plugins and themes"),
                )
                .subcommand(
                    SubCommand::with_name("clean")
                        .about("DANGER: Shuts down containers and removes volumes"),
                )
                .subcommand(
                    SubCommand::with_name("gen-docker-dev").about("Creates docker dev mounts yml"),
                )
                .subcommand(
                    SubCommand::with_name("vscode-debug-config")
                        .about("Creates vscode debug config"),
                ),
        )
        .subcommand(SubCommand::with_name("git-account").about("Adds or modifies a git account"))
        .subcommand(SubCommand::with_name("server").about("Adds or modifies server config"))
        .subcommand(
            SubCommand::with_name("deploy").about("For a given project, pushes updates to prod"),
        )
        .subcommand(
            SubCommand::with_name("sync-server")
                .about("Syncs base files like Dockerfiles to server"),
        )
        .subcommand(SubCommand::with_name("ssh").about("For a server, enter shell through ssh"))
        .subcommand(SubCommand::with_name("wp-ssh").about("Wp-cli shell through ssh"))
        .subcommand(
            SubCommand::with_name("server-wp-ssh")
                .about("Wp-cli shell through ssh and tunnel from server"),
        )
        .subcommand(
            SubCommand::with_name("prod")
                .about("For a given project, updates, starts prod containers")
                .setting(clap::AppSettings::TrailingVarArg)
                .arg(
                    Arg::with_name("prod-args")
                        .multiple(true)
                        .help("Arguments passed to docker-compose"),
                ),
        )
        .subcommand(
            SubCommand::with_name("aws")
                .about("Configures aws credentials")
                .subcommand(SubCommand::with_name("provision").about("Provisions an ec2 instance")),
        )
        .subcommand(
            SubCommand::with_name("workspace")
                .about("Subcommands to init git repository, or clone")
                .subcommand(
                    SubCommand::with_name("init-git")
                        .about("Inits and creates a git repo given a registered git account"),
                )
                .subcommand(
                    SubCommand::with_name("push")
                        .about("Pushes workspace repository to origin master"),
                )
                .subcommand(
                    SubCommand::with_name("clone")
                        .about("Clones a given git repository into workspace location"),
                ),
        )
}
