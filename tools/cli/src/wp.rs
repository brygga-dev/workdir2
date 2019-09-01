use crate::docker;
use crate::er::{self, Result};
use crate::project::ProjectConfig;
use crate::project_path::ProjectItemPaths;
use crate::server::{self, SshConn, SyncSet};
use crate::utils::{self, CliEnv};
use failure::format_err;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};

/// While wp-specific, dev setup might be a different category
pub fn gen_vscode_debug_config(env: &CliEnv, project: ProjectConfig) -> io::Result<()> {
    // Using hjson here to preserve comments in json file
    // edit: unfortunately doesn't preserve comments, at least
    // parses it
    use serde_hjson::{Map, Value};
    // Collect mapping for plugins and themes
    let mut mappings = Map::new();
    let site_local = get_local_site_data(env, &project)?;
    for (_name, plugin) in site_local.plugins {
        mappings.insert(
            plugin.paths.server_path.string(),
            Value::String(format!(
                "${{workspaceRoot}}/{}",
                plugin.paths.from_project.cow()
            )),
        );
    }
    for (_name, theme) in site_local.themes {
        mappings.insert(
            theme.paths.server_path.string(),
            Value::String(format!(
                "${{workspaceRoot}}/{}",
                theme.paths.from_project.cow()
            )),
        );
    }
    // Lastly, add fallback to wp root for all other files
    mappings.insert(
        "/var/www/html".to_string(),
        Value::String("${workspaceRoot}/wp".to_string()),
    );
    // Assemble entry
    let our_debug_key = "Wop";
    use std::iter::FromIterator;
    let config_entry = Value::Object(Map::from_iter(vec![
        ("name".to_string(), Value::String(our_debug_key.to_string())),
        ("request".to_string(), Value::String("launch".to_string())),
        ("type".to_string(), Value::String("php".to_string())),
        ("pathMappings".to_string(), Value::Object(mappings)),
        ("port".to_string(), Value::I64(9002)),
    ]));
    let mut conf_path = project.dir(env);
    conf_path.push(".vscode");
    conf_path.push("launch.json");
    let json_value = if conf_path.is_file() {
        // Existing json file, replace wop entry
        let conf_str = std::fs::read_to_string(&conf_path)?;
        // Deserializing to generic json value to ease
        // dealing with potientially unknown values
        let mut json_val = serde_hjson::from_str::<serde_hjson::Value>(&conf_str)
            .map_err(|e| utils::io_error(format!("Json decode error: {:?}", e)))?;
        // If there is existing entry, replace, otherwise append
        // Ugh, next time investigate api. There are many methods for stuff like this.
        match &mut json_val {
            Value::Object(ref mut map) => {
                match map.get_mut("configurations") {
                    Some(ref mut configurations) => match configurations {
                        Value::Array(ref mut arr) => {
                            // Check for name = "Wop"
                            let index = arr.iter().position(|entry| match entry {
                                Value::Object(map) => match map.get("name") {
                                    Some(Value::String(name_val)) => name_val == our_debug_key,
                                    _ => false,
                                },
                                _ => false,
                            });
                            match index {
                                Some(index) => {
                                    // Replace
                                    arr[index] = config_entry;
                                }
                                None => {
                                    // Push
                                    arr.push(config_entry);
                                }
                            }
                        }
                        _ => {
                            return utils::io_err(
                                "Expected configurations array in .vscode/launch.json",
                            )
                        }
                    },
                    None => {
                        return utils::io_err("Expected configurations key in .vscode/launch.json")
                    }
                }
            }
            _ => return utils::io_err("Expected object in .vscode/launch.json"),
        }
        json_val
    } else {
        // New json file
        Value::Object(Map::from_iter(vec![
            ("version".to_string(), Value::String("0.2.0".to_string())),
            (
                "configurations".to_string(),
                Value::Array(vec![config_entry]),
            ),
        ]))
    };
    let json_str = serde_json::to_string_pretty(&to_json_val(json_value))
        .map_err(|e| utils::io_error(format!("Serialize error: {:?}", e)))?;
    utils::ensure_parent_dir(&conf_path)?;
    std::fs::write(conf_path, json_str)
}

// Ugh. Got type error with hjson
fn to_json_val(v: serde_hjson::Value) -> serde_json::Value {
    match v {
        serde_hjson::Value::Array(v) => {
            serde_json::Value::Array(v.into_iter().map(|e| to_json_val(e)).collect())
        }
        serde_hjson::Value::Bool(v) => serde_json::Value::Bool(v),
        serde_hjson::Value::F64(v) => serde_json::Value::Number(serde_json::Number::from(
            serde_json::de::ParserNumber::F64(v),
        )),
        serde_hjson::Value::I64(v) => serde_json::Value::Number(serde_json::Number::from(
            serde_json::de::ParserNumber::I64(v),
        )),
        serde_hjson::Value::Null => serde_json::Value::Null,
        serde_hjson::Value::Object(m) => {
            serde_json::Value::Object(m.into_iter().map(|(k, v)| (k, to_json_val(v))).collect())
        }
        serde_hjson::Value::String(s) => serde_json::Value::String(s),
        serde_hjson::Value::U64(v) => serde_json::Value::Number(serde_json::Number::from(
            serde_json::de::ParserNumber::U64(v),
        )),
    }
}

pub fn create_docker_prod_yml(env: &CliEnv, project: &ProjectConfig) -> Result<()> {
    use crate::docker::{ComposeService, ComposeYml};
    // Set environment variable for external url
    let server = server::get_config(env, &project.server_name).map_err(er::Io::e)?;
    let mut proxy_env = BTreeMap::new();
    let elastic_ip = match server.elastic_ip {
        Some(elastic_ip) => elastic_ip,
        None => {
            eprintln!("Elastic ip is required for prod.yml");
            return Err(format_err!("Elastic ip is required for prod.yml"));
        }
    };
    proxy_env.insert(
        "EXTERNAL".to_string(),
        format!("http://{}", elastic_ip.public_ip),
    );
    let proxy = ComposeService {
        volumes: Vec::new(),
        environment: proxy_env,
    };
    let mut services = BTreeMap::new();
    services.insert("proxy".to_string(), proxy);
    let yml = ComposeYml {
        version: "3.3",
        services,
    };
    let yml_str = match serde_yaml::to_string::<ComposeYml>(&yml) {
        Ok(yml_str) => yml_str,
        Err(e) => return Err(format_err!("{:?}", e)),
    };
    println!("{}", &yml_str);
    project.write_file(env, "docker/prod.yml", &yml_str)?;
    println!("Wrote prox.yml");
    Ok(())
}

/// Create mount entries for directories in
/// plugins/ and themes/ folders
pub fn create_wp_docker_yml(env: &CliEnv, project: ProjectConfig) -> io::Result<()> {
    use crate::docker::{ComposeService, ComposeYml};
    // Iterate plugins and themes and collect mounts
    let mut mounts = Vec::new();
    let local_site = get_local_site_data(env, &project)?;

    // Plugin mounts
    for (_name, plugin) in local_site.plugins {
        mounts.push((
            plugin.paths.full_path.string(),
            plugin.paths.server_path.string(),
        ));
    }
    // Theme mounts
    for (_name, theme) in local_site.themes {
        mounts.push((
            theme.paths.full_path.string(),
            theme.paths.server_path.string(),
        ));
    }
    let mut services = BTreeMap::new();
    // Add mounts to wordpress-container and wp-cli
    let mut volumes = mounts
        .into_iter()
        .map(|(source, dst)| format!("{}:{}", source, dst))
        .collect::<Vec<_>>();

    // Add mounts relevent to static files to proxy service
    /*
    services.insert(
        "proxy".into(),
        ComposeService {
            volumes: volumes.clone(),
        },
    );*/
    // And mount wp root onto project for easy interaction
    // in development
    let mut local_wp = project.dir(env);
    local_wp.push("wp");
    volumes.push(format!("{}:/var/www/html", local_wp.to_string_lossy()));

    // todo: Investigate global volume drivers to
    // see if we can mount onto them with particular drivers
    services.insert(
        "wordpress-container".into(),
        ComposeService {
            volumes: volumes.clone(),
            environment: BTreeMap::new(),
        },
    );
    services.insert(
        "wp-cli".into(),
        ComposeService {
            volumes,
            environment: BTreeMap::new(),
        },
    );
    let yml = ComposeYml {
        version: "3.3",
        services,
    };
    let yml_str = match serde_yaml::to_string::<ComposeYml>(&yml) {
        Ok(yml_str) => yml_str,
        Err(e) => return utils::io_err(format!("{:?}", e)),
    };
    println!("{}", &yml_str);
    project.write_file(env, "docker/dev.yml", &yml_str)?;
    println!("Wrote dev.yml");
    Ok(())
}

/// Returns a connection to either dev server,
/// or piped on prod server
pub fn wp_cli_conn(env: &CliEnv, project: &ProjectConfig, on_server: bool) -> Result<SshConn> {
    let server_config = if on_server {
        // Todo: Could allow to choose, or return error
        match project.get_server(&env) {
            Some(server_config) => Some(server_config),
            None => return Err(format_err!("Could not resolve server")),
        }
    } else {
        None
    };
    let conn = if on_server {
        match server_config {
            Some(server_config) => {
                server::SshConn::connect_wp_cli(env, 2345, Some(&server_config))?
            }
            None => return Err(format_err!("No server config")),
        }
    } else {
        server::SshConn::connect_wp_cli(env, 2345, None)?
    };
    Ok(conn)
}

/// Wp_cli invokation cli command entry point
pub fn wp_cli(
    env: &CliEnv,
    current_process: utils::CurrentProcess,
    project: ProjectConfig,
    args: Vec<String>,
    on_server: bool,
) -> Result<utils::CurrentProcess> {
    let cmd = format!("wp {}", args.join(" "));
    println!("{}", console::style(&cmd).green());
    let conn = wp_cli_conn(env, &project, on_server)?;
    let output = conn.exec_capture(cmd, Some("/var/www/html"))?;
    println!("{}", output);
    Ok(current_process)
}

pub fn wp_clean(
    env: &CliEnv,
    project: ProjectConfig,
    current_process: utils::CurrentProcess,
) -> io::Result<utils::CurrentProcess> {
    // Running docker-compose down including
    // volumes
    docker::dev_cmd(
        &env,
        current_process,
        project.clone(),
        vec!["down".to_string(), "--volumes".to_string()],
    )
}

/// Run wp installation process
pub fn wp_install(
    env: &CliEnv,
    project: ProjectConfig,
    current_process: utils::CurrentProcess,
    on_server: bool,
) -> Result<utils::CurrentProcess> {
    // Collect needed install info,
    // then run wp-cli command
    let title = env.get_input("Title", Some("Site name".into()))?;
    let admin_user = env.get_input("Admin user", Some("admin".into()))?;
    // todo: Some security
    // Maybe some easy way to login, command or otherwise
    let admin_pass = env.get_input("Admin pass", Some("pass".into()))?;
    let admin_email = env.get_input("Admin email", Some("wp@example.com".into()))?;

    let args = vec![
        "core".to_string(),
        "install".to_string(),
        "--url=wordpress-container".to_string(),
        format!("--title=\"{}\"", title),
        format!("--admin_user={}", admin_user),
        // Would be preferable to use the option to read from file
        format!("--admin_password={}", admin_pass),
        format!("--admin_email={}", admin_email),
        "--skip-email".to_string(),
    ];

    // Download is not necessary currently
    // as it is done in the docker entrypoint script

    //wp_cli(env, project.clone(), "core", Some(vec!["download".into()]))?;
    let current_process = wp_cli(env, current_process, project.clone(), args, on_server)?;
    sync_local(env, project, on_server)?;
    Ok(current_process)
}

pub fn install_plugin(cli_conn: &SshConn, plugin: &str, activate: bool) -> Result<()> {
    let mut args = vec!["plugin", "install", plugin];
    if activate {
        args.push("--activate");
    }
    match cli_conn.exec(format!("wp {}", args.join(" "))) {
        Ok(_) => {
            println!("Plugin installed and activated: {}", plugin);
            Ok(())
        }
        Err(e) => Err(format_err!("Couldn't install: {}, {:?}", plugin, e)),
    }
}

pub fn activate_plugin(cli_conn: &SshConn, plugin: &str) -> Result<()> {
    match cli_conn.exec(format!("wp plugin activate {}", plugin)) {
        Ok(_) => {
            println!("Plugin activated: {}", plugin);
            Ok(())
        }
        Err(e) => Err(format_err!("Couldn't activate: {}, {:?}", plugin, e)),
    }
}

pub fn activate_theme(cli_conn: &SshConn, theme: &str) -> Result<()> {
    match cli_conn.exec(format!("wp theme activate {}", theme)) {
        Ok(_) => {
            println!("Theme activated: {}", theme);
            Ok(())
        }
        Err(e) => Err(format_err!("Couldn't activate theme: {}, {:?}", theme, e)),
    }
}

// todo: it would be nice with "plugin" architecture for subsystems
/// Copy project files to container
pub fn sync_files_to_prod(cli_conn: &SshConn, site_local: &WpLocalSiteData) -> Result<()> {
    let sftp = cli_conn.sftp()?;
    // Make sync set
    // todo: This barely works, but it would be
    // be nice to combine sync_sets for example
    let mut sync_set = SyncSet::new(
        site_local.project_dir.clone(),
        PathBuf::from("/var/www/html/wp-content"),
    );
    for (_name, plugin) in &site_local.plugins {
        sync_set.resolve(&plugin.paths.full_path.0, &sftp, false)?;
    }
    for (_name, theme) in &site_local.themes {
        sync_set.resolve(&theme.paths.full_path.0, &sftp, false)?;
    }
    sync_set.sync_zipped(cli_conn, &sftp)?;
    // Copy to docker volume
    // In this case, plugins and themes folders should be present,
    // but note that `cp` does not create parent folders
    // Specifying `-` for either source or destination will
    // accept a tar file from stdin, or export one to stdout
    // Using a tar file is a trick to handling file owner.
    Ok(())
}

pub fn sync_content_to_prod(env: &CliEnv, project: &ProjectConfig) -> Result<()> {
    Ok(())
}

/// Syncs plugins, themes, other site data between local and install
/// on dev or server
pub fn sync_local(env: &CliEnv, project: ProjectConfig, on_server: bool) -> Result<()> {
    let local_data = get_local_site_data(env, &project)?;
    let cli_conn = wp_cli_conn(env, &project, on_server)?;
    if on_server {
        sync_files_to_prod(&cli_conn, &local_data)?;
    }
    let install_data = match wp_install_data(&cli_conn) {
        Ok(install_data) => install_data,
        Err(e) => return Err(format_err!("Install data error: {}", e)),
    };
    // Ensure local plugins, themes and their dependencies are activated

    // First do deps, ideally this should be a bigger dependency graph,
    // so deps of deps are installed first.
    // also could consider running wp-cli without loading plugins
    for dep in local_data.deps {
        match install_data.plugins.get(&dep) {
            Some(plugin_data) => {
                // Plugin is installed, check for activated
                if plugin_data.status != "active" {
                    activate_plugin(&cli_conn, &plugin_data.name)?;
                }
            }
            None => {
                install_plugin(&cli_conn, &dep, true)?;
            }
        }
    }
    // Activate local plugins
    // Todo: Could verify requirements (plugin.php?) first
    for (plugin_name, _local_plugin) in local_data.plugins {
        match install_data.plugins.get(&plugin_name) {
            Some(plugin_data) => {
                if plugin_data.status != "active" {
                    activate_plugin(&cli_conn, &plugin_name)?;
                } else {
                    println!("Already active: {}", plugin_name);
                }
            }
            None => {
                return Err(format_err!(
                    "Local plugin not found as installed on site, {}",
                    plugin_name
                ));
            }
        }
    }
    // If there is one theme locally, we currently activate this
    // Otherwise, could present a select to activate
    // Todo: Could verify requirements (functions.php and style.css?)
    if local_data.themes.len() == 1 {
        match local_data.themes.into_iter().next() {
            Some((theme_name, _local_theme)) => match install_data.themes.get(&theme_name) {
                Some(site_theme) => {
                    if site_theme.status != "active" {
                        activate_theme(&cli_conn, &theme_name)?;
                    }
                }
                None => {
                    return Err(format_err!(
                        "Local theme not found as installed on site, {}",
                        theme_name
                    ));
                }
            },
            None => (),
        }
    }
    Ok(())
}
#[derive(Debug)]
pub struct WpPlugin {
    pub name: String,
    pub paths: ProjectItemPaths,
}

#[derive(Debug)]
pub struct WpTheme {
    pub name: String,
    pub paths: ProjectItemPaths,
}

#[derive(Debug)]
pub struct WpLocalSiteData {
    pub project_dir: PathBuf,
    pub plugins: HashMap<String, WpPlugin>,
    pub themes: HashMap<String, WpTheme>,
    pub deps: HashSet<String>,
}

/// Plugin conf from plugin.json in plugin dir
#[derive(Deserialize)]
pub struct PluginConf {
    deps: Vec<String>,
}

// Data from project, ie local
pub fn get_local_site_data(env: &CliEnv, project: &ProjectConfig) -> io::Result<WpLocalSiteData> {
    let project_dir = project.dir(env);
    let mut site_data = WpLocalSiteData {
        project_dir: project_dir.clone(),
        plugins: HashMap::new(),
        themes: HashMap::new(),
        deps: HashSet::new(),
    };
    // Plugins
    let mut plugins_dir = project_dir.clone();
    plugins_dir.push("plugins");
    if plugins_dir.is_dir() {
        for plugin_path in utils::entries_in_dir(&plugins_dir)? {
            if plugin_path.is_dir() {
                let plugin_name = utils::file_name_string(&plugin_path)?;
                let from_project = plugin_path
                    .strip_prefix(&project_dir)
                    .map_err(|e| utils::io_error(format!("Strip path error: {:?}", e)))?;
                let server_path = Path::new("/var/www/html/wp-content").join(from_project);
                let mut plugin_conf_file = plugin_path.clone();
                site_data.plugins.insert(
                    plugin_name.clone(),
                    WpPlugin {
                        name: plugin_name,
                        paths: ProjectItemPaths::new(
                            from_project.to_path_buf(),
                            plugin_path,
                            server_path,
                        ),
                    },
                );
                plugin_conf_file.push("plugin.json");
                if plugin_conf_file.is_file() {
                    let plugin_conf_str = std::fs::read_to_string(&plugin_conf_file)?;
                    match serde_json::from_str::<PluginConf>(&plugin_conf_str) {
                        Ok(plugin_conf) => {
                            // Local plugin.json config
                            for dep in plugin_conf.deps {
                                // Could have something like "plugin_name:https://github.com/plugin"
                                // to expand capabilities (or something else)
                                site_data.deps.insert(dep);
                            }
                        }
                        Err(e) => println!("Deserialize error {:?}: {:?}", plugin_conf_file, e),
                    }
                }
            }
        }
    }
    let mut themes_dir = project_dir.clone();
    themes_dir.push("themes");
    if themes_dir.is_dir() {
        for theme_path in utils::entries_in_dir(&themes_dir)? {
            if theme_path.is_dir() {
                let theme_name = utils::file_name_string(&theme_path)?;
                let from_project = theme_path
                    .strip_prefix(&project_dir)
                    .map_err(|e| utils::io_error(format!("Strip path error: {:?}", e)))?;
                let server_path = Path::new("/var/www/html/wp-content").join(from_project);
                site_data.themes.insert(
                    theme_name.clone(),
                    WpTheme {
                        name: theme_name,
                        paths: ProjectItemPaths::new(
                            from_project.to_path_buf(),
                            theme_path,
                            server_path,
                        ),
                    },
                );
            }
        }
    }
    Ok(site_data)
}

// Various info from wp installation
#[derive(Deserialize, Debug)]
pub struct WpInstallPlugin {
    pub name: String,
    pub status: String,
    pub update: String,
    pub version: String,
}
#[derive(Deserialize, Debug)]
pub struct WpInstallTheme {
    pub name: String,
    pub status: String,
    pub update: String,
    pub version: String,
}
#[derive(Deserialize, Debug)]
pub struct WpInstallData {
    pub plugins: HashMap<String, WpInstallPlugin>,
    pub themes: HashMap<String, WpInstallTheme>,
}
pub fn wp_install_data(cli_conn: &SshConn) -> Result<WpInstallData> {
    let plugins_output = match cli_conn.exec_capture("wp plugin list --format=json", Some("/var/www/html")) {
        Ok(output) => output,
        Err(e) => return Err(format_err!("Plugin list failed: {:?}", e)),
    };
    let themes_output = match cli_conn.exec_capture("wp theme list --format=json", Some("/var/www/html")) {
        Ok(output) => output,
        Err(e) => return Err(format_err!("Theme list failed: {:?}", e)),
    };
    let plugins = match serde_json::from_str::<Vec<WpInstallPlugin>>(&plugins_output) {
        Ok(plugins) => plugins,
        Err(e) => {
            return Err(format_err!("Failed deserialize plugins: {:?}", e));
        }
    };
    let themes = match serde_json::from_str::<Vec<WpInstallTheme>>(&themes_output) {
        Ok(themes) => themes,
        Err(e) => {
            return Err(format_err!("Failed deserialize themes: {:?}", e));
        }
    };
    let data = WpInstallData {
        plugins: plugins.into_iter().fold(HashMap::new(), |mut hm, p| {
            hm.insert(p.name.clone(), p);
            hm
        }),
        themes: themes.into_iter().fold(HashMap::new(), |mut hm, t| {
            hm.insert(t.name.clone(), t);
            hm
        }),
    };
    println!("WpData: {:#?}", data);
    Ok(data)
}

pub fn sql_cli(env: &CliEnv, sql: &str) -> Result<()> {
    use mysql_utils::Db;
    let mut db = Db::new("127.0.0.1", 3307, "wordpress", "wordpress", "wordpress")?;
    db.print_query(sql)?;
    Ok(())
}