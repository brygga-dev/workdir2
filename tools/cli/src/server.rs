use crate::er::{self, FailExt, Result};
use crate::utils::{self, CliEnv};
use failure::format_err;
use failure::ResultExt;
use serde::{Deserialize, Serialize};
use std::io;
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub url: String,
    pub pem: String,
    pub instance_id: Option<String>,
    pub elastic_ip: Option<ElasticIp>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct ElasticIp {
    pub allocation_id: String,
    pub public_ip: String,
}
impl ServerConfig {
    pub fn pem_path(&self, env: &CliEnv) -> PathBuf {
        env.config_dirs
            .servers
            .filepath(&format!(".pem/{}", self.pem))
    }

    pub fn home_dir(&self) -> PathBuf {
        PathBuf::from("/home/ec2-user")
    }
    pub fn home_dir_and(&self, extra: &str) -> PathBuf {
        let mut home_dir = self.home_dir();
        home_dir.push(extra);
        home_dir
    }
}

pub fn has_config(env: &CliEnv, server: &str) -> bool {
    env.config_dirs.servers.has_file(server)
}

pub fn get_config(env: &CliEnv, server: &str) -> io::Result<ServerConfig> {
    let json_file = std::fs::File::open(env.config_dirs.servers.filepath(server))?;
    let buf_reader = io::BufReader::new(json_file);
    let config = serde_json::from_reader::<_, ServerConfig>(buf_reader)?;
    Ok(config)
}

pub fn get_servers(env: &CliEnv) -> io::Result<Vec<String>> {
    utils::files_in_dir(&env.config_dirs.servers.0)
}

pub fn select_server(env: &CliEnv) -> io::Result<ServerConfig> {
    let servers = get_servers(env)?;
    env.select("Select server", &servers, None)
        .and_then(|i| match servers.get(i) {
            Some(server_name) => get_config(env, server_name),
            None => utils::io_err("Error selecting server"),
        })
}

/// Manually input or edit a server
/// When running `provision`, a config will also be created
pub fn add_server(env: &CliEnv) -> io::Result<()> {
    // List current servers
    let current_files = utils::files_in_dir(&env.config_dirs.servers.0)?;
    if current_files.len() > 0 {
        println!("Current servers:");
        for file in current_files {
            println!("{}", file);
        }
    } else {
        println!("No existing servers");
    }

    let name = env.get_input("Internal server name", None)?;
    let current_config = if has_config(env, &name) {
        println!("Server config exists for: {}", &name);
        println!("Modifying entry.");
        Some(get_config(env, &name)?)
    } else {
        None
    };
    let url = env.get_input(
        "Ssh url",
        current_config.as_ref().map(|c| c.url.to_string()),
    )?;
    let pem = env.get_input("Pem filename", current_config.map(|c| c.pem))?;

    // If this is aws, we could allow to select instance by
    // describe_instance_status or similar
    let config = ServerConfig {
        name,
        url,
        pem,
        instance_id: None,
        elastic_ip: None,
    };
    write_config(env, config)
}

pub fn write_config(env: &CliEnv, config: ServerConfig) -> io::Result<()> {
    let content_str = match serde_json::to_string_pretty(&config) {
        Ok(content_str) => content_str,
        Err(_) => return Err(io::Error::from(io::ErrorKind::InvalidData)),
    };

    env.config_dirs.servers.write(&config.name, &content_str)
}

pub struct SshConn {
    pub tcp: TcpStream,
    pub session: ssh2::Session,
    pub tunnel: Option<SshTunnel>,
}
impl Drop for SshConn {
    fn drop(&mut self) {
        if let Some(tunnel) = self.tunnel.take() {
            match tunnel.close() {
                Ok(_) => println!("Closed tunnel"),
                Err(e) => eprintln!("Failed closing tunnel: {:?}", e),
            }
        }
    }
}
impl SshConn {
    /// Establish ssh connection to given server,
    /// currently based on config
    /// We want this to have security that makes sense
    pub fn connect(env: &CliEnv, server: &ServerConfig) -> Result<Self> {
        // http://api.libssh.org/master/libssh_tutorial.html
        println!("Connecting to {}", server.url);
        let tcp = TcpStream::connect(&server.url)?;
        let mut session = match ssh2::Session::new() {
            Some(session) => session,
            None => return Err(format_err!("Could not create session struct")),
        };
        println!("Connected");
        match session.handshake(&tcp) {
            Ok(_) => (),
            Err(e) => return er::Ssh::msg("Failed handshake", e).err(),
        }
        // Todo: Verify public key
        // Don't know how to get this in advance
        /*let known_hosts = match session.known_hosts() {
            Ok(known_hosts) => known_hosts,
            Err(e) => return utils::io_err(format!("Could not get known hosts: {:?}", e))
        };*/
        //known_hosts.
        /*
        println!(
            "Sha1 {:?}",
            session
                .host_key_hash(ssh2::HashType::Sha1)
                .map(String::from_utf8_lossy)
        );
        println!(
            "Md5 {:?}",
            session
                .host_key_hash(ssh2::HashType::Md5)
                .map(String::from_utf8_lossy)
        );
        println!(
            "{:?}",
            session.host_key().map(|(s, t)| (
                match t {
                    ssh2::HostKeyType::Dss => "dss",
                    ssh2::HostKeyType::Rsa => "rsa",
                    ssh2::HostKeyType::Unknown => "unknown",
                },
                String::from_utf8_lossy(s)
            ))
        );*/

        // Attempt authenticate
        let pem_file = server.pem_path(env);
        match session.userauth_pubkey_file("ec2-user", None, &pem_file, None) {
            Ok(_) => (),
            Err(e) => return er::Ssh::msg("Authentication failed", e).err(),
        }
        if !session.authenticated() {
            return Err(format_err!("Authenticated failed"));
        } else {
            println!("Authenticated to server: {}", server.name);
        }
        Ok(SshConn {
            tcp,
            session,
            tunnel: None,
        })
    }

    /// Connects to wp-cli ssh server
    pub fn connect_wp_cli(env: &CliEnv, port: u16, tunnel: Option<&ServerConfig>) -> Result<Self> {
        let (tunnel, port) = match tunnel {
            Some(server_config) => {
                // Setup tunnel
                // todo: Better port management,
                // now avoiding collision when wp-cli port is the same (2345)
                // on dev and prod
                let local_port = port + 1;
                let tunnel = SshTunnel::new(env, server_config, local_port, port)?;
                (Some(tunnel), local_port)
            }
            None => (None, port),
        };
        let url = format!("127.0.0.1:{}", port);
        println!("Connecting to {}", url);
        let tcp = TcpStream::connect(&url)?;
        let mut session = match ssh2::Session::new() {
            Some(session) => session,
            None => return Err(format_err!("Could not create session struct")),
        };
        println!("Connected");
        match session.handshake(&tcp) {
            Ok(_) => (),
            Err(e) => return er::Ssh::msg("Failed handshake", e).err(),
        }
        match session.userauth_password("www-data", "www-data") {
            Ok(_) => (),
            Err(e) => return er::Ssh::msg("Authentication failed", e).err(),
        }
        if !session.authenticated() {
            return Err(format_err!("Authenticated failed"));
        } else {
            println!("Authenticated to wp-cli");
        }
        Ok(SshConn {
            tcp,
            session,
            tunnel,
        })
    }

    pub fn channel(&self) -> Result<ssh2::Channel> {
        // These feels a little brittle, but needed here
        // Don't know if worth to keep own variable for it?
        self.session.set_blocking(true);
        match self.session.channel_session() {
            Ok(channel) => Ok(channel),
            Err(e) => er::Ssh::msg("Error opening channel", e).err(),
        }
    }

    fn update_pty_size(channel: &mut ssh2::Channel) {
        use terminal_size::{terminal_size, Height, Width};
        let size = terminal_size();
        if let Some((Width(width), Height(height))) = size {
            match channel.request_pty_size(width.into(), height.into(), None, None) {
                Ok(()) => (),
                Err(e) => eprintln!("Failed setting pty size: {:?}", e),
            }
        }
    }

    pub fn shell(&self) -> Result<()> {
        let mut channel = self.channel()?;
        // xterm should have more features, support colors etc
        // other options, vanilla, vt220, vt100 etc
        // Don't know if xterm could be bad for security
        // https://unix.stackexchange.com/questions/43945/whats-the-difference-between-various-term-variables
        // Mode at least cooked and raw. Cooked will process the input, for example
        // deleting a character when backspace is pressed
        // https://en.wikipedia.org/wiki/Terminal_mode
        // Had problems with both mode: Some("cooked") and Some("raw") on aws
        match channel.request_pty("xterm", None, None) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Could not request pty");
                return er::Ssh::msg("Could not request pty", e).err();
            }
        }
        Self::update_pty_size(&mut channel);
        match channel.shell() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Could not request shell");
                return er::Ssh::msg("Could not request shell", e).err();
            }
        }
        // Switch to raw mode for this stdin
        use termion::raw::IntoRawMode;
        // "raw_mode" will act as stdout, as well as keeping
        // state of the terminal and restore when dropped
        let mut raw_mode = match std::io::stdout().into_raw_mode() {
            Ok(restorer) => restorer,
            Err(e) => {
                eprintln!("Could not enter raw mode");
                return er::Io::msg("Could not enter raw mode", e).err();
            }
        };
        let mut inp = std::io::stdin();
        // Keep a thread to receive stdin
        let (tx, rx) = std::sync::mpsc::channel();
        let _thread = std::thread::spawn(move || {
            let mut inp_buf: [u8; 256] = [0; 256];
            use std::io::Read;
            loop {
                match inp.read(&mut inp_buf) {
                    Ok(num) => {
                        if num > 0 {
                            match tx.send(Vec::from(&inp_buf[0..num])) {
                                Ok(_) => (),
                                Err(e) => eprintln!("Failed sending input: {:?}", e),
                            }
                        } else {
                            println!("Received 0, breaking");
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Input read error, breaking, {:?}", e);
                        break;
                    }
                }
            }
        });
        let mut err = std::io::stderr();
        self.pipe_loop(&mut channel, &mut raw_mode, Some(rx), &mut err)?;
        // Todo: I haven't found solution to handling stdin well.
        // The stdin thread is blocking on read and no way to exit
        // Possibly a better solution could be some global handler tied
        // to env
        /*
        match thread.join() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to join thread: {:?}", e);
            }
        }*/
        Ok(())
    }

    /// Loops while piping channels stdout and stderr to
    /// respective fds on host system
    /// Also pipes input. It might make sense to do
    /// this as an option
    fn pipe_loop<W, E>(
        &self,
        channel: &mut ssh2::Channel,
        mut out: &mut W,
        inp: Option<std::sync::mpsc::Receiver<Vec<u8>>>,
        mut err: &mut E,
    ) -> Result<()>
    where
        W: std::io::Write,
        E: std::io::Write,
    {
        let mut acc_buf = Vec::with_capacity(2048);
        use std::io::Read;
        let mut read_buf: [u8; 2048] = [0; 2048];
        // If there is still something in stderr, channel should not eof
        use std::error::Error;
        while !channel.eof() {
            // Read stdout while there is bytes
            // Not sure how to do this well. Consideration is
            // to support "streaming"
            // Also, probably don't want to write in the middle
            // of certain bytes? Like utf8 chars etc?
            // This will block unless self.session.set_blocking is set to false
            self.session.set_blocking(false);
            loop {
                match channel.read(&mut read_buf) {
                    Ok(num) => {
                        if num > 0 {
                            acc_buf.extend_from_slice(&read_buf[0..num]);
                        } else {
                            break;
                        }
                    }
                    Err(e) => {
                        // Accept WouldBlock and Interrupted
                        match e.kind() {
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted => (),
                            std::io::ErrorKind::Other => {
                                use std::io::Error;
                                // todo: Better detection
                                if e.description() != "would block" {
                                    return er::Io::msg("Read failed", e).err();
                                }
                                break;
                            }
                            _ => {
                                return er::Io::msg("Read failed", e).err();
                            }
                        }
                    }
                }
            }
            if acc_buf.len() > 0 {
                match write!(&mut out, "{}", String::from_utf8_lossy(&acc_buf)) {
                    Ok(()) => (),
                    Err(e) => eprintln!("Failed to write output: {:?}", e),
                }
                match out.flush() {
                    Ok(_) => (),
                    Err(_) => (),
                }
                acc_buf.clear();
            }
            // Now check stderr
            let mut err_stream = channel.stderr();
            loop {
                match err_stream.read(&mut read_buf) {
                    Ok(num) => {
                        if num > 0 {
                            acc_buf.extend_from_slice(&read_buf[0..num]);
                        } else {
                            break;
                        }
                    }
                    Err(e) => {
                        // Accept "would block"
                        if e.description() != "would block" {
                            println!("Read error: {}, {:?}", e.description(), e.source());
                        }
                        break;
                    }
                }
            }
            drop(err_stream);
            if acc_buf.len() > 0 {
                match write!(&mut err, "{}", String::from_utf8_lossy(&acc_buf)) {
                    Ok(()) => (),
                    Err(e) => eprintln!("Failed to write output: {:?}", e),
                }
                match out.flush() {
                    Ok(_) => (),
                    Err(_) => (),
                }
                acc_buf.clear();
            }
            // Sleeping I think to not use too much cpu and
            // allow other threads some time
            std::thread::sleep(std::time::Duration::from_millis(20));
            if let Some(rx) = &inp {
                // Check stdin and send to channel
                loop {
                    match rx.try_recv() {
                        Ok(inp_buf) => {
                            use std::io::Write;
                            // Block while writing to ensure all is written
                            self.session.set_blocking(true);
                            match channel.write_all(&inp_buf) {
                                Ok(_) => (),
                                Err(e) => eprintln!("Error writing input to channel: {:?}", e),
                            }
                            self.session.set_blocking(false);
                        }
                        Err(_) => break,
                    }
                }
            }
        }
        // There may be something in out_buf if we got to .eof()
        // before reading Ok(0)
        /*
        if out_buf.len() > 0 {
            println!("{}", String::from_utf8_lossy(&out_buf));
            match writeln!(&mut out, "{}", String::from_utf8_lossy(&out_buf)) {
                Ok(()) => (),
                Err(e) => eprintln!("Failed to write output: {:?}", e),
            }
            out_buf.clear();
        }*/
        self.session.set_blocking(true);
        Ok(())
    }

    // Todo: Communicate error code better, possibly custom Result type
    /// Runs command, captures and returns output
    pub fn exec_capture<S: Into<String>, WD: Into<String>>(
        &self,
        cmd: S,
        working_dir: Option<WD>,
    ) -> Result<String> {
        // There could be better solutions for this,
        // somehow setting it on session
        // I think the problem is it would be harder to get
        // error code then.
        let cmd = match working_dir {
            Some(working_dir) => format!("cd {}; {}", working_dir.into(), cmd.into()),
            None => cmd.into()
        };
        let mut channel = self.channel()?;
        match channel.exec(&cmd) {
            Ok(_) => (),
            Err(e) => return er::Ssh::msg(format!("Error executing command: {}", cmd), e).err(),
        }
        let mut captured = String::with_capacity(128);
        use std::io::Read;
        channel.read_to_string(&mut captured)?;
        let mut stderr_capture = String::new();
        channel.stderr().read_to_string(&mut stderr_capture)?;
        if stderr_capture.len() > 0 {
            eprintln!("Stderr: {}", stderr_capture);
        }
        let exit_code = Self::finish_exec(channel)?;
        if exit_code == 0 {
            Ok(captured)
        } else {
            Err(format_err!(
                "Command with non-zero exit code: {}, {}",
                exit_code,
                cmd
            ))
        }
    }

    pub fn exec<S: Into<String>>(&self, cmd: S) -> Result<i32> {
        let cmd = cmd.into();
        println!("{}", console::style(&cmd).green());
        let mut channel = self.channel()?;
        match channel.exec(&cmd) {
            Ok(_) => (),
            Err(e) => return er::Ssh::msg(format!("Error executing command: {}", cmd), e).err(),
        }
        let mut out = std::io::stdout();
        let mut err = std::io::stderr();
        self.pipe_loop(&mut channel, &mut out, None, &mut err)?;
        Self::finish_exec(channel)
    }

    /// Internal helper to close exec channel and get status code
    fn finish_exec(mut channel: ssh2::Channel) -> Result<i32> {
        // Send signal to close
        match channel.close() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error closing channel: {:?}", e);
                return Err(format_err!("Error closing channel"));
            }
        }
        // Wait for remote channel to close
        match channel.wait_close() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error waiting for close: {:?}", e);
            }
        }
        match channel.exit_status() {
            Ok(status) => {
                if status != 0 {
                    eprintln!("Non-zero exit status: {}", status);
                }
                Ok(status)
            }
            Err(e) => er::Ssh::msg("Error getting exit status", e).err(),
        }
    }

    pub fn sftp(&self) -> Result<ssh2::Sftp> {
        let sftp = self
            .session
            .sftp()
            .map_err(|e| er::Ssh::msg("Failed to start sftp subsystem", e))?;
        Ok(sftp)
    }

    pub fn transfer_file(
        sftp: &ssh2::Sftp,
        abs_path: &Path,
        remote_path: &Path,
        bytes: u64,
        modified: Option<u64>,
        progress_bar: &indicatif::ProgressBar,
    ) -> Result<()> {
        progress_bar.set_message(&format!("{:?}", remote_path));
        progress_bar.set_length(bytes);
        let mut local_handle = std::fs::File::open(abs_path).map_err(er::Io::e)?;
        // As per ssh2::Sft::create(), using WRITE | TRUNCATE here to mean create
        println!("Opening {:?}", remote_path);
        let mut remote_handle = sftp
            .open_mode(
                &remote_path,
                ssh2::WRITE | ssh2::TRUNCATE,
                0o640,
                ssh2::OpenType::File,
            )
            .map_err(er::Ssh::e)?;
        Self::copy(&mut local_handle, &mut remote_handle, &progress_bar)?;
        drop(remote_handle);
        if let Some(modified) = modified {
            // Set modified time stat
            let stat_setter = ssh2::FileStat {
                size: None,
                uid: None,
                gid: None,
                perm: None,
                atime: None,
                mtime: Some(modified),
            };
            sftp.setstat(&remote_path, stat_setter)
                .map_err(er::Ssh::e)?;
        }
        use std::io::Write;
        let _ = std::io::stdout().flush();
        println!(
            "{:?}",
            console::style(remote_path.as_os_str().to_string_lossy()).green()
        );
        let _ = std::io::stdout().flush();
        Ok(())
    }

    /// Utility function to copy from a read to a write handle
    /// while displaying progress
    /// Based on io::copy
    fn copy<R, W>(
        reader: &mut R,
        writer: &mut W,
        progress_bar: &indicatif::ProgressBar,
    ) -> Result<u64>
    where
        R: std::io::Read,
        W: std::io::Write,
    {
        use std::time::{Duration, Instant};
        // 8 bytes buffer
        let mut buf: [u8; 8192] = [0; 8192];
        let mut written = 0;
        progress_bar.set_position(0);
        let progress_interval = Duration::from_millis(200);
        let mut last_progress = Instant::now();
        loop {
            let len = match reader.read(&mut buf) {
                Ok(0) => {
                    progress_bar.finish();
                    return Ok(written);
                }
                Ok(len) => {
                    if last_progress.elapsed() >= progress_interval {
                        progress_bar.set_position(written);
                        last_progress = Instant::now();
                    }
                    len
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(er::Io::e(e).into()),
            };
            writer.write_all(&buf[..len]).map_err(er::Io::e)?;
            written += len as u64;
        }
    }
}

trait SftpExt {
    fn exist_stat(&self, p: &Path) -> Result<Option<ssh2::FileStat>>;
    fn ensure_dir(&self, remote_dir: &Path) -> Result<()>;
}
impl<'a> SftpExt for ssh2::Sftp<'a> {
    /// Extend sftp stat to return None when file does not
    /// exist, and Some<FileStat> when it does
    fn exist_stat(&self, p: &Path) -> Result<Option<ssh2::FileStat>> {
        match self.stat(&p) {
            Ok(stat) => Ok(Some(stat)),
            Err(e) => {
                if e.code() == 2 {
                    Ok(None)
                } else {
                    return Err(failure::Error::from(er::Ssh::e(e)));
                }
            }
        }
    }
    fn ensure_dir(&self, remote_dir: &Path) -> Result<()> {
        let mut stack = Vec::new();
        let mut ancestors = remote_dir.ancestors();
        while let Some(ancestor) = ancestors.next() {
            match self.exist_stat(ancestor)? {
                Some(stat) => {
                    if stat.is_dir() {
                        break;
                    } else {
                        return Err(format_err!(
                            "Ensure dir: Expected dir, found file: {:?}",
                            ancestor
                        ));
                    }
                }
                None => {
                    stack.push(ancestor);
                }
            }
        }
        stack.reverse();
        for ancestor in stack.into_iter() {
            println!("Creating parent directory: {:?}", ancestor);
            self.mkdir(ancestor, 0o0700).map_err(er::Ssh::e)?;
        }
        Ok(())
    }
}

// Todo: Consider taking sftp as member (or go over
// argument positions for consistency)
/// Helper to sync as an archive, then decompress on server
pub struct SyncSet {
    local_base: PathBuf,
    server_base: PathBuf,
    entries: Vec<SyncSetEntry>,
}
enum SyncSetEntry {
    File {
        rel_path: PathBuf,
        abs_path: PathBuf,
        modified: Option<u64>,
        bytes: u64,
    },
    Dir {
        rel_path: PathBuf,
        abs_path: PathBuf,
        modified: Option<u64>,
    },
}
impl SyncSet {
    /// New SyncSet to "manually" call resolve(), or add
    /// functions on
    pub fn new(local_base: PathBuf, server_base: PathBuf) -> Self {
        SyncSet {
            local_base,
            server_base,
            entries: Vec::new(),
        }
    }
    // TODO: Change setup so we can transfer to different
    // named files/folders

    /// Sets up a SyncSet, mapping the file's parent
    /// folder to given remote folder and resolves
    /// whether the file is newer
    pub fn from_file(
        file_path: PathBuf,
        remote_folder: PathBuf,
        sftp: &ssh2::Sftp,
        force: bool,
    ) -> Result<Self> {
        // There should always be a parent for local, at least `/`
        match file_path.parent() {
            Some(parent) => {
                let mut sync_set = Self::new(parent.to_path_buf(), remote_folder);
                sync_set.resolve(&file_path, sftp, force)?;
                Ok(sync_set)
            }
            None => Err(format_err!("Could not get parent folder of file")),
        }
    }

    /// Sets up a SyncSet mapping a local to a remote folder,
    /// and resolves which files in it are newer than
    /// on the server
    pub fn from_dir(
        local: PathBuf,
        remote: PathBuf,
        sftp: &ssh2::Sftp,
        force: bool,
    ) -> Result<Self> {
        // When given a folder, there should always be a parent
        // We could use directly local/remote, but this aligns with
        // file setup, and dedups calls to ensure_dir,
        // since given directory will be part of DirWalker.
        match (local.parent(), remote.parent()) {
            (Some(local_parent), Some(remote_parent)) => {
                let mut sync_set =
                    Self::new(local_parent.to_path_buf(), remote_parent.to_path_buf());
                sync_set.resolve(&local, sftp, force)?;
                Ok(sync_set)
            }
            _ => Err(format_err!("Could not get parent folders")),
        }
    }

    /// Attempts to get seconds since unix epoch of metadata
    /// Returns none if it fails somehow
    fn modified_timestamp(metadata: &std::fs::Metadata) -> Option<u64> {
        match metadata.modified() {
            Ok(modified) => match modified.duration_since(std::time::UNIX_EPOCH) {
                Ok(duration) => Some(duration.as_secs()),
                Err(e) => {
                    eprintln!("Could not read modified since unix epoch: {:?}", e);
                    None
                }
            },
            Err(e) => {
                eprintln!("Could not read modified: {:?}", e);
                None
            }
        }
    }
    /// Expects absolute path. Walks through a directory, or single file
    /// and compares modified times with possible server file.
    /// Will add unless a server file exists with the same or higher
    /// modified time
    pub fn resolve(&mut self, local: &Path, sftp: &ssh2::Sftp, force: bool) -> Result<()> {
        let root_meta = local.metadata().map_err(er::Io::e)?;
        let root_rel_path = self.rel_from_abs(&local)?;
        let root_server_path = self.server_base.join(&root_rel_path);
        let root_server_meta = sftp.exist_stat(&root_server_path)?;
        let mut failed_mtime = false;
        if root_meta.is_file() {
            // This is a single file
            //println!("Single file: {:?}", local);
            let local_mtime = Self::modified_timestamp(&root_meta);
            // Do transfer unless we can confirm equal or
            // higher mtime on server
            let do_transfer = force
                || match (local_mtime, root_server_meta) {
                    (
                        Some(local),
                        Some(ssh2::FileStat {
                            mtime: Some(remote),
                            ..
                        }),
                    ) if remote >= local => false,
                    (None, _) => {
                        failed_mtime = true;
                        true
                    }
                    _ => true,
                };
            if do_transfer {
                self.entries.push(SyncSetEntry::File {
                    rel_path: root_rel_path,
                    abs_path: local.to_path_buf(),
                    modified: local_mtime,
                    bytes: root_meta.len(),
                });
            };
        } else if root_meta.is_dir() {
            //println!("Walking dir: {:?}", local);
            let root_exist = root_server_meta.is_some();
            for entry in WalkDir::new(local) {
                let entry = entry.map_err(er::Walkdir::e)?;
                let entry_path = entry.path();
                let rel_path = self.rel_from_abs(&entry_path)?;
                let local_meta = entry.metadata().map_err(er::Walkdir::e)?;
                let local_mtime = Self::modified_timestamp(&local_meta);
                if entry_path.is_file() {
                    let do_transfer = if !root_exist || force {
                        // Skip check if root does not exist or force
                        true
                    } else {
                        let server_path = self.server_base.join(&rel_path);
                        let server_meta = sftp.exist_stat(&server_path)?;
                        match (local_mtime, server_meta) {
                            (
                                Some(local),
                                Some(ssh2::FileStat {
                                    mtime: Some(remote),
                                    ..
                                }),
                            ) if remote >= local => false,
                            (None, _) => {
                                failed_mtime = true;
                                true
                            }
                            _ => true,
                        }
                    };
                    //println!("File: {:?}, do_transfer: {:?}", entry_path, do_transfer);
                    if do_transfer {
                        println!("Adding: {:?}", entry_path);
                        self.entries.push(SyncSetEntry::File {
                            rel_path,
                            abs_path: entry_path.to_path_buf(),
                            modified: local_mtime,
                            bytes: local_meta.len(),
                        });
                    };
                } else if entry_path.is_dir() {
                    //println!("Adding dir: {:?}", entry_path);
                    // Adding all dirs to be safe with zip for now
                    // todo: Possible optimization when syncing through sftp,
                    // to only add missing folders
                    self.entries.push(SyncSetEntry::Dir {
                        rel_path,
                        abs_path: local.to_path_buf(),
                        modified: local_mtime,
                    });
                } else {
                    return Err(format_err!("Unrecognized type: {:?}", entry_path));
                }
            }
        } else {
            return Err(format_err!("Only files and dirs supported: {:?}", local));
        }
        if failed_mtime {
            eprintln!("Notice: Failed reading local modified time on some entries");
        }
        Ok(())
    }
    #[inline]
    pub fn rel_from_abs(&self, path: &Path) -> Result<PathBuf> {
        path.strip_prefix(&self.local_base)
            .map_err(|_| {
                format_err!(
                    "Could not strip path, {:?} from: {:?}",
                    self.local_base,
                    path
                )
            })
            .map(|p| p.to_path_buf())
    }
    /// Note, these will not handle links currently
    pub fn sync_plain(&mut self, sftp: &ssh2::Sftp) -> Result<()> {
        sftp.ensure_dir(&self.server_base)?;
        // Progress bar
        let progress_bar = indicatif::ProgressBar::new(0);
        progress_bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{bar:25} {bytes}/{total_bytes} {msg}"),
        );
        for entry in &self.entries {
            match entry {
                SyncSetEntry::File {
                    rel_path,
                    abs_path,
                    modified,
                    bytes,
                } => {
                    let remote_path = self.server_base.join(rel_path);
                    SshConn::transfer_file(
                        &sftp,
                        &abs_path,
                        &remote_path,
                        *bytes,
                        *modified,
                        &progress_bar,
                    )?;
                }
                SyncSetEntry::Dir {
                    rel_path,
                    abs_path,
                    modified,
                } => {
                    println!("Creating directory: {:?}", rel_path);
                    let remote_path = self.server_base.join(&rel_path);
                    sftp.mkdir(&remote_path, 0o0700).map_err(er::Ssh::e)?;
                }
            }
        }
        Ok(())
    }
    /// Zips the registered files, transfers to server,
    /// unzips and deletes the zip-file
    pub fn sync_zipped(&mut self, ssh: &SshConn, sftp: &ssh2::Sftp) -> Result<()> {
        if self.entries.len() == 0 {
            // Todo: Better handling of added folders to zip
            println!("No files to sync");
            return Ok(());
        }
        let zip_file = self.local_base.join("to_sync.zip");
        print!("Compressing.. ");
        self.make_zip(&zip_file)?;
        println!("OK");
        let mut zip_set =
            SyncSet::from_file(zip_file.clone(), self.server_base.clone(), sftp, true)?;
        zip_set.sync_plain(sftp)?;
        // Decompress on the other side
        let server_zip_file = self.server_base.join("to_sync.zip");
        let server_zip_str = server_zip_file.to_string_lossy();
        // Todo: Pay attention to paths when implementing support for file.a -> file.b
        ssh.exec(format!(
            "unzip -o {} -d {}",
            server_zip_str,
            zip_set.server_base.to_string_lossy()
        ))?;
        // And remove zip file
        ssh.exec(format!("rm {}", server_zip_str))?;
        std::fs::remove_file(zip_file)?;
        Ok(())
    }
    fn make_zip(&self, zip_file: &Path) -> Result<()> {
        use std::fs;
        let to_sync_file = fs::File::create(zip_file)?;
        let mut zip_out = zip::ZipWriter::new(to_sync_file);
        for entry in &self.entries {
            match entry {
                SyncSetEntry::File {
                    abs_path,
                    rel_path,
                    modified,
                    ..
                } => {
                    // Deflate (default) compression is pure rust,
                    // while bzip should compresss more but slower
                    // The size difference is not huge, but could be worth it
                    // https://cran.r-project.org/web/packages/brotli/vignettes/brotli-2015-09-22.pdf
                    let mut options = zip::write::FileOptions::default();
                    if let Some(modified) = modified {
                        options = options.last_modified_time(Self::seconds_to_datetime(*modified)?);
                    }
                    zip_out
                        .start_file_from_path(&rel_path, options)
                        .map_err(|e| format_err!("Zip file error: {:?}", e))?;
                    let mut file = fs::File::open(abs_path)?;
                    io::copy(&mut file, &mut zip_out)?;
                }
                SyncSetEntry::Dir {
                    abs_path,
                    rel_path,
                    modified,
                } => {
                    let mut options = zip::write::FileOptions::default();
                    if let Some(modified) = modified {
                        options = options.last_modified_time(Self::seconds_to_datetime(*modified)?);
                    }
                    zip_out
                        .add_directory_from_path(&rel_path, options)
                        .map_err(|e| format_err!("Zip directory error: {:?}", e))?;
                }
            }
        }
        zip_out
            .finish()
            .map_err(|e| format_err!("Zip finish error: {:?}", e))?;
        Ok(())
    }
    /// Expects duration since unix epoch and returns a zip::DateTime
    fn seconds_to_datetime(secs: u64) -> Result<zip::DateTime> {
        use chrono::{Datelike, Timelike};
        use std::convert::TryInto;
        let secs: i64 = secs
            .try_into()
            .map_err(|_| format_err!("Failed to convert seconds to i64"))?;
        let m = chrono::NaiveDateTime::from_timestamp(secs, 0);
        let date = m.date();
        let time = m.time();
        // Adding 1 to seconds as there was a `1` mismatch after transfer,
        // and unlike chrono, zip::DateTime has seconds bound 0:60 (vs 0:59)
        let seconds: u8 = time
            .second()
            .try_into()
            .map_err(|_| format_err!("Failed to convert second"))?;
        zip::DateTime::from_date_and_time(
            m.year()
                .try_into()
                .map_err(|_| format_err!("Failed to convert year"))?,
            date.month()
                .try_into()
                .map_err(|_| format_err!("Failed to convert month"))?,
            date.day()
                .try_into()
                .map_err(|_| format_err!("Failed to convert day"))?,
            time.hour()
                .try_into()
                .map_err(|_| format_err!("Failed to convert hour"))?,
            time.minute()
                .try_into()
                .map_err(|_| format_err!("Failed to convert minute"))?,
            seconds + 1,
        )
        .map_err(|_| format_err!("Failed to convert to zip::DateTime"))
    }
}

/// Sets up remote server. Mainly docker
pub fn setup_server(env: &CliEnv, server: ServerConfig) -> Result<()> {
    // Could check instance status here
    let conn = SshConn::connect(env, &server)?;
    // https://gist.github.com/npearce/6f3c7826c7499587f00957fee62f8ee9
    conn.exec("sudo yum update")?;
    conn.exec("sudo amazon-linux-extras install docker")?;
    // Start docker and enable auto-start
    conn.exec("sudo systemctl enable --now docker.service")?;
    conn.exec("sudo usermod -a -G docker ec2-user")?;
    // Docker compose
    conn.exec("sudo curl -L https://github.com/docker/compose/releases/download/1.22.0/docker-compose-$(uname -s)-$(uname -m) -o /usr/local/bin/docker-compose")?;
    conn.exec("sudo chmod +x /usr/local/bin/docker-compose")?;
    // Can see error code if docker-compose successfully installed
    conn.exec("docker-compose version")?;
    Ok(())
}

/// Ssh shell
pub fn ssh(env: &CliEnv, server: ServerConfig) -> Result<()> {
    let conn = SshConn::connect(env, &server)?;
    conn.shell()
}
/// Wp-cli ssh shell
pub fn wp_cli_ssh(env: &CliEnv, port: u16, server: Option<&ServerConfig>) -> Result<()> {
    let conn = SshConn::connect_wp_cli(env, port, server)?;
    conn.shell()
}

// todo: Some of this would be phased out to dedicated images
// hosted in docker hub or otherwise, though some remain like
// compose files and custom images
// In any case handy for development of server setup
/// Syncs server base files, like Dockerfiles to server
pub fn sync_to_server(env: &CliEnv, server: ServerConfig) -> Result<()> {
    let conn = SshConn::connect(env, &server)?;
    let mut server_dir = env.workdir_dir.clone();
    server_dir.push("server");
    let remote_server_dir = server.home_dir_and("workdir/server");
    let sftp = conn.sftp()?;
    let mut sync_set = SyncSet::new(server_dir.clone(), remote_server_dir.clone());
    for subdir in ["base", "prod"].into_iter() {
        let mut local = server_dir.clone();
        local.push(subdir);
        sync_set.resolve(&local, &sftp, false)?;
    }
    sync_set.sync_zipped(&conn, &sftp)?;
    Ok(())
}

/// Read non-blocking from channel until
/// 0 read
fn read_until_zero<R: std::io::Read>(
    r: &mut R,
    read_buf: &mut [u8; 2048],
    acc_buf: &mut Vec<u8>,
) -> Result<()> {
    loop {
        match r.read(read_buf) {
            Ok(num) => {
                if num > 0 {
                    acc_buf.extend_from_slice(&read_buf[0..num]);
                } else {
                    break;
                }
            }
            Err(e) => {
                // Accept WouldBlock and Interrupted
                match e.kind() {
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted => {
                        break;
                    }
                    std::io::ErrorKind::Other => {
                        use std::error::Error;
                        // todo: Better detection
                        if e.description() != "would block" {
                            return er::Io::msg("Read failed", e).err();
                        }
                        break;
                    }
                    _ => {
                        return er::Io::msg("Read tunnel", e).err();
                    }
                }
            }
        }
    }
    Ok(())
}

pub struct SshTunnel {
    join_handle: std::thread::JoinHandle<Result<()>>,
    close_sender: std::sync::mpsc::SyncSender<bool>,
}
impl SshTunnel {
    /// Tunnels incoming requests to a port on
    /// server. Runs in a thread so it's possible
    /// to connect from other functions.
    /// Supports only one connection at the time
    pub fn new(
        env: &CliEnv,
        server: &ServerConfig,
        local_port: u16,
        remote_port: u16,
    ) -> Result<Self> {
        let conn = SshConn::connect(env, &server)?;
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let listener = TcpListener::bind(format!("127.0.0.1:{}", local_port)).map_err(er::Io::e)?;
        let handle = std::thread::spawn(move || -> Result<()> {
            listener.set_nonblocking(true)?;
            for stream in listener.incoming() {
                match stream {
                    Ok(mut socket) => {
                        let mut channel = conn
                            .session
                            .channel_direct_tcpip("127.0.0.1", remote_port, None)
                            .map_err(|e| er::Ssh::msg("Failed to connect on server", e))?;
                        println!("Opened tunnel");
                        conn.session.set_blocking(false);
                        socket.set_nonblocking(true).map_err(er::Io::e)?;
                        // Now pipe both ways
                        let mut acc_buf = Vec::with_capacity(2048);
                        use std::io::Write;
                        let mut read_buf: [u8; 2048] = [0; 2048];
                        // Loop until connection is closed
                        while !channel.eof() {
                            // Stdio
                            read_until_zero(&mut channel, &mut read_buf, &mut acc_buf)?;
                            // Write back to socket
                            if acc_buf.len() > 0 {
                                socket.write_all(&acc_buf).map_err(er::Io::e)?;
                                socket.flush().map_err(er::Io::e)?;
                                acc_buf.clear();
                            }
                            // Stderr
                            read_until_zero(&mut channel.stderr(), &mut read_buf, &mut acc_buf)?;
                            // Write back to socket
                            if acc_buf.len() > 0 {
                                socket.write_all(&acc_buf).map_err(er::Io::e)?;
                                socket.flush().map_err(er::Io::e)?;
                                acc_buf.clear();
                            }
                            // Read any data on socket and forward to tunneled
                            read_until_zero(&mut socket, &mut read_buf, &mut acc_buf)?;
                            // Write back to channel
                            if acc_buf.len() > 0 {
                                channel.write_all(&acc_buf).map_err(er::Io::e)?;
                                channel.flush().map_err(er::Io::e)?;
                                acc_buf.clear();
                            }
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                        println!("Channel closed");
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(200));
                    }
                    Err(e) => return er::Io::msg("Tunnel listener failed", e).err(),
                }
                // Check if we have close signal
                match rx.try_recv() {
                    Ok(_) => break,
                    Err(_) => (),
                }
            }
            println!("Done listener loop");
            Ok(())
        });
        Ok(SshTunnel {
            join_handle: handle,
            close_sender: tx,
        })
    }

    pub fn close(self) -> Result<()> {
        self.close_sender.send(true)?;
        let thread_result = self
            .join_handle
            .join()
            .map_err(|_| format_err!("Failed to join tunnel thread"))?;
        thread_result
    }
}
