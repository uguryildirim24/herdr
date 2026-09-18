#![cfg(unix)]

pub mod support;

use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use support::{
    cleanup_test_base, client_shell_handshake, register_runtime_dir, register_spawned_herdr_pid,
    send_client_shell_shift_enter, unregister_spawned_herdr_pid, wait_for_client_shell_bootstrap,
    wait_for_message_variant, wait_for_socket, SERVER_MESSAGE_ENDPOINT_CONTROL,
    SERVER_MESSAGE_SERVER_SHUTDOWN,
};

struct SpawnedHerdr {
    _master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
}

struct RequestError {
    retryable: bool,
    message: String,
}

impl Drop for SpawnedHerdr {
    fn drop(&mut self) {
        let pid = self.child.process_id();
        let _ = self.child.kill();
        unregister_spawned_herdr_pid(pid);
    }
}

fn test_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn unique_test_dir() -> PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    PathBuf::from(format!("/tmp/hlh-{}-{n}", std::process::id()))
}

fn spawn_server(config_home: &Path, runtime_dir: &Path, api_socket: &Path) -> SpawnedHerdr {
    spawn_server_with_env(config_home, runtime_dir, api_socket, &[])
}

fn spawn_server_with_env(
    config_home: &Path,
    runtime_dir: &Path,
    api_socket: &Path,
    extra_env: &[(&str, &str)],
) -> SpawnedHerdr {
    fs::create_dir_all(config_home.join("herdr")).unwrap();
    fs::create_dir_all(runtime_dir).unwrap();
    fs::write(
        config_home.join("herdr/config.toml"),
        "onboarding = false\n",
    )
    .unwrap();

    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_herdr"));
    cmd.arg("server");
    cmd.env("XDG_CONFIG_HOME", config_home);
    cmd.env("XDG_RUNTIME_DIR", runtime_dir);
    cmd.env("HERDR_SOCKET_PATH", api_socket);
    cmd.env(
        "HERDR_CLIENT_SOCKET_PATH",
        runtime_dir.join("herdr-client.sock"),
    );
    cmd.env("SHELL", "/bin/sh");
    for (key, value) in extra_env {
        cmd.env(key, value);
    }

    let child = pair.slave.spawn_command(cmd).unwrap();
    register_spawned_herdr_pid(child.process_id());
    SpawnedHerdr {
        _master: pair.master,
        child,
    }
}

fn spawn_named_session_server(
    config_home: &Path,
    runtime_dir: &Path,
    session_name: &str,
) -> SpawnedHerdr {
    fs::create_dir_all(config_home.join("herdr-dev")).unwrap();
    fs::create_dir_all(runtime_dir).unwrap();
    fs::write(
        config_home.join("herdr-dev/config.toml"),
        "onboarding = false\n",
    )
    .unwrap();

    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_herdr"));
    cmd.arg("server");
    cmd.env("XDG_CONFIG_HOME", config_home);
    cmd.env("XDG_RUNTIME_DIR", runtime_dir);
    cmd.env("HERDR_SESSION", session_name);
    cmd.env_remove("HERDR_SOCKET_PATH");
    cmd.env_remove("HERDR_CLIENT_SOCKET_PATH");
    cmd.env("SHELL", "/bin/sh");

    let child = pair.slave.spawn_command(cmd).unwrap();
    register_spawned_herdr_pid(child.process_id());
    SpawnedHerdr {
        _master: pair.master,
        child,
    }
}

fn spawn_named_session_server_with(
    config_home: &Path,
    runtime_dir: &Path,
    session_name: &str,
    config_toml: &str,
    extra_env: &[(&str, &str)],
) -> SpawnedHerdr {
    fs::create_dir_all(config_home.join("herdr-dev")).unwrap();
    fs::create_dir_all(runtime_dir).unwrap();
    fs::write(config_home.join("herdr-dev/config.toml"), config_toml).unwrap();

    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_herdr"));
    cmd.arg("server");
    cmd.env("XDG_CONFIG_HOME", config_home);
    cmd.env("XDG_RUNTIME_DIR", runtime_dir);
    cmd.env("HERDR_SESSION", session_name);
    cmd.env_remove("HERDR_SOCKET_PATH");
    cmd.env_remove("HERDR_CLIENT_SOCKET_PATH");
    cmd.env("SHELL", "/bin/sh");
    for (key, value) in extra_env {
        cmd.env(*key, *value);
    }

    let child = pair.slave.spawn_command(cmd).unwrap();
    register_spawned_herdr_pid(child.process_id());
    SpawnedHerdr {
        _master: pair.master,
        child,
    }
}

fn spawn_default_session_server(config_home: &Path, runtime_dir: &Path) -> SpawnedHerdr {
    fs::create_dir_all(config_home.join("herdr-dev")).unwrap();
    fs::create_dir_all(runtime_dir).unwrap();
    fs::write(
        config_home.join("herdr-dev/config.toml"),
        "onboarding = false\n",
    )
    .unwrap();

    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_herdr"));
    cmd.arg("server");
    cmd.env("XDG_CONFIG_HOME", config_home);
    cmd.env("XDG_RUNTIME_DIR", runtime_dir);
    cmd.env("XDG_STATE_HOME", runtime_dir.join("state"));
    cmd.env_remove("HERDR_SESSION");
    cmd.env_remove("HERDR_SOCKET_PATH");
    cmd.env_remove("HERDR_CLIENT_SOCKET_PATH");
    cmd.env("SHELL", "/bin/sh");

    let child = pair.slave.spawn_command(cmd).unwrap();
    register_spawned_herdr_pid(child.process_id());
    SpawnedHerdr {
        _master: pair.master,
        child,
    }
}

fn spawn_server_with_args_and_socket_env(
    config_home: &Path,
    runtime_dir: &Path,
    session_name: Option<&str>,
    api_socket_env: Option<&Path>,
    client_socket_env: Option<&Path>,
) -> SpawnedHerdr {
    fs::create_dir_all(config_home.join("herdr-dev")).unwrap();
    fs::create_dir_all(runtime_dir).unwrap();
    fs::write(
        config_home.join("herdr-dev/config.toml"),
        "onboarding = false\n",
    )
    .unwrap();

    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_herdr"));
    if let Some(session_name) = session_name {
        cmd.arg("--session");
        cmd.arg(session_name);
    }
    cmd.arg("server");
    cmd.env("XDG_CONFIG_HOME", config_home);
    cmd.env("XDG_RUNTIME_DIR", runtime_dir);
    cmd.env_remove("HERDR_SESSION");
    if let Some(api_socket_env) = api_socket_env {
        cmd.env("HERDR_SOCKET_PATH", api_socket_env);
    } else {
        cmd.env_remove("HERDR_SOCKET_PATH");
    }
    if let Some(client_socket_env) = client_socket_env {
        cmd.env("HERDR_CLIENT_SOCKET_PATH", client_socket_env);
    } else {
        cmd.env_remove("HERDR_CLIENT_SOCKET_PATH");
    }
    cmd.env("SHELL", "/bin/sh");

    let child = pair.slave.spawn_command(cmd).unwrap();
    register_spawned_herdr_pid(child.process_id());
    SpawnedHerdr {
        _master: pair.master,
        child,
    }
}

fn try_request(
    socket_path: &Path,
    request: serde_json::Value,
) -> Result<serde_json::Value, RequestError> {
    let mut stream = UnixStream::connect(socket_path).map_err(|err| RequestError {
        retryable: true,
        message: format!("connect {}: {err}", socket_path.display()),
    })?;
    let request_text = request.to_string();
    stream
        .write_all(request_text.as_bytes())
        .map_err(|err| RequestError {
            retryable: true,
            message: format!("write request to {}: {err}", socket_path.display()),
        })?;
    stream.write_all(b"\n").map_err(|err| RequestError {
        retryable: true,
        message: format!("write newline to {}: {err}", socket_path.display()),
    })?;
    stream.flush().map_err(|err| RequestError {
        retryable: true,
        message: format!("flush request to {}: {err}", socket_path.display()),
    })?;
    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .map_err(|err| RequestError {
            retryable: true,
            message: format!("read response from {}: {err}", socket_path.display()),
        })?;
    if line.is_empty() {
        return Err(RequestError {
            retryable: true,
            message: format!(
                "empty response from {} for request {request_text}",
                socket_path.display()
            ),
        });
    }
    serde_json::from_str(&line).map_err(|err| RequestError {
        retryable: false,
        message: format!(
            "parse response from {} for request {request_text}: {err}; response was {line:?}",
            socket_path.display()
        ),
    })
}

fn request(socket_path: &Path, request: serde_json::Value) -> serde_json::Value {
    try_request(socket_path, request).unwrap_or_else(|err| panic!("{}", err.message))
}

fn response_error_code(value: &serde_json::Value) -> Option<&str> {
    value
        .get("error")
        .and_then(|error| error.get("code"))
        .and_then(serde_json::Value::as_str)
}

fn request_until_ok(
    socket_path: &Path,
    body: serde_json::Value,
    timeout: Duration,
) -> serde_json::Value {
    let deadline = Instant::now() + timeout;
    let mut last = String::new();
    while Instant::now() < deadline {
        match try_request(socket_path, body.clone()) {
            Ok(response) if response.get("result").is_some() => return response,
            Ok(response) => last = response.to_string(),
            Err(err) if err.retryable => last = err.message,
            Err(err) => panic!("{}", err.message),
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!(
        "request did not succeed at {}; last: {last}",
        socket_path.display()
    );
}

fn assert_ok(response: serde_json::Value) {
    assert!(
        response.get("result").is_some(),
        "api request failed: {response}"
    );
}

fn wait_for_socket_gone(path: &Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("socket did not go away at {}", path.display());
}

fn wait_for_api(socket_path: &Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    let mut last_error = String::new();
    while Instant::now() < deadline {
        match try_request(
            socket_path,
            serde_json::json!({"id":"test:ping","method":"ping","params":{}}),
        ) {
            Ok(response) if response.get("result").is_some() => return,
            Ok(response)
                if matches!(
                    response_error_code(&response),
                    Some("server_unavailable" | "server_handed_off")
                ) =>
            {
                last_error = response.to_string();
            }
            Ok(response) => panic!("api ping returned non-success response: {response}"),
            Err(err) if !err.retryable => panic!("{}", err.message),
            Err(err) => {
                last_error = err.message;
            }
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!(
        "api did not become ready at {}; last error: {last_error}",
        socket_path.display()
    );
}

fn write_plugin_manifest(root: &Path, plugin_id: &str) {
    fs::create_dir_all(root).unwrap();
    fs::write(
        root.join("herdr-plugin.toml"),
        format!(
            r#"id = "{plugin_id}"
name = "Live handoff test"
version = "0.1.0"
min_herdr_version = "0.6.10"
platforms = ["linux", "macos", "windows"]
"#
        ),
    )
    .unwrap();
}

fn link_plugin(socket_path: &Path, root: &Path) {
    assert_ok(request(
        socket_path,
        serde_json::json!({
            "id": "test:plugin:link",
            "method": "plugin.link",
            "params": {"path": root, "enabled": true}
        }),
    ));
}

fn listed_plugin_ids(socket_path: &Path) -> Vec<String> {
    let response = request(
        socket_path,
        serde_json::json!({"id":"test:plugin:list","method":"plugin.list","params":{}}),
    );
    assert_ok(response.clone());
    response["result"]["plugins"]
        .as_array()
        .unwrap()
        .iter()
        .map(|plugin| plugin["plugin_id"].as_str().unwrap().to_string())
        .collect()
}

fn saved_plugin_ids(registry_path: &Path) -> Vec<String> {
    let mut ids =
        serde_json::from_str::<Vec<serde_json::Value>>(&fs::read_to_string(registry_path).unwrap())
            .unwrap()
            .into_iter()
            .map(|plugin| plugin["plugin_id"].as_str().unwrap().to_string())
            .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn wait_for_output(socket_path: &Path, pane_id: &str, needle: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut last_text = String::new();
    let mut last_response = serde_json::Value::Null;
    while Instant::now() < deadline {
        let response = request(
            socket_path,
            serde_json::json!({
                "id": "test:pane:read",
                "method": "pane.read",
                "params": {
                    "pane_id": pane_id,
                    "source": "visible",
                    "lines": 20,
                    "format": "text",
                    "strip_ansi": true
                }
            }),
        );
        last_response = response.clone();
        let text = response["result"]["read"]["text"]
            .as_str()
            .unwrap_or_default();
        last_text = text.to_string();
        if text.contains(needle) {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!(
        "pane output did not contain {needle:?}; last text was {last_text:?}; last response was {last_response}"
    );
}

fn wait_for_file_contains(path: &Path, needle: &str, timeout: Duration) -> String {
    let deadline = Instant::now() + timeout;
    let mut last_text = String::new();
    while Instant::now() < deadline {
        if let Ok(text) = fs::read_to_string(path) {
            last_text = text;
            if last_text.contains(needle) {
                return last_text;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!(
        "{} did not contain {needle:?}; last text was {last_text:?}",
        path.display()
    );
}

#[cfg(target_os = "linux")]
fn server_ptmx_fd_count(pid: u32) -> usize {
    let Ok(entries) = fs::read_dir(format!("/proc/{pid}/fd")) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| fs::read_link(entry.path()).ok())
        // ptmx master node: /dev/ptmx or /dev/pts/ptmx (devpts); slaves /dev/pts/<N> excluded.
        .filter(|target| target == Path::new("/dev/ptmx") || target == Path::new("/dev/pts/ptmx"))
        .count()
}

#[cfg(target_os = "macos")]
fn server_ptmx_fd_count(pid: u32) -> usize {
    let Ok(output) = std::process::Command::new("lsof")
        .args(["-nP", "-p", &pid.to_string()])
        .output()
    else {
        return 0;
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.contains("/dev/ptmx"))
        .count()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn wait_for_server_ptmx_fd_count(pid: u32, expected: usize, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    let mut last_count = 0;
    while Instant::now() < deadline {
        last_count = server_ptmx_fd_count(pid);
        if last_count == expected {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("server pid {pid} had {last_count} ptmx master fds; expected {expected}");
}

#[cfg(target_os = "linux")]
fn wait_for_replacement_server_pid(runtime_dir: &Path, old_pid: u32, timeout: Duration) -> u32 {
    let deadline = Instant::now() + timeout;
    let mut last_pids = Vec::new();
    while Instant::now() < deadline {
        last_pids = support::herdr_server_pids_for_runtime_dir(runtime_dir).unwrap_or_default();
        if let Some(pid) = last_pids.iter().copied().find(|pid| *pid != old_pid) {
            return pid;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!(
        "replacement server for {} did not appear; last pids: {:?}",
        runtime_dir.display(),
        last_pids
    );
}

#[cfg(target_os = "macos")]
fn wait_for_replacement_server_pid(_runtime_dir: &Path, old_pid: u32, timeout: Duration) -> u32 {
    let handoff_socket_pattern = format!("herdr-handoff-{old_pid}.sock");
    let deadline = Instant::now() + timeout;
    let mut last_stdout = String::new();
    while Instant::now() < deadline {
        if let Ok(output) = std::process::Command::new("pgrep")
            .args(["-af", &handoff_socket_pattern])
            .output()
        {
            last_stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            for line in last_stdout.lines() {
                let Some(pid_text) = line.split_whitespace().next() else {
                    continue;
                };
                let Ok(pid) = pid_text.parse::<u32>() else {
                    continue;
                };
                if pid != old_pid {
                    return pid;
                }
            }
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!(
        "replacement server for {} did not appear; last pgrep output: {}",
        _runtime_dir.display(),
        last_stdout
    );
}

fn unused_local_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn wait_for_http_contains(port: u16, needle: &str, timeout: Duration) -> String {
    let deadline = Instant::now() + timeout;
    let mut last_response = String::new();
    while Instant::now() < deadline {
        if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
            let _ =
                stream.write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
            let mut response = String::new();
            let _ = stream.read_to_string(&mut response);
            last_response = response;
            if last_response.contains(needle) {
                return last_response;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!(
        "http server on port {port} did not return {needle:?}; last response was {last_response:?}"
    );
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn live_server_holds_one_pty_master_fd_per_pane() {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let api_socket = runtime_dir.join("herdr.sock");

    let spawned = spawn_server(&config_home, &runtime_dir, &api_socket);
    wait_for_socket(&api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);
    let server_pid = spawned
        .child
        .process_id()
        .expect("test server should expose pid");
    wait_for_server_ptmx_fd_count(server_pid, 0, Duration::from_secs(5));

    let created = request(
        &api_socket,
        serde_json::json!({
            "id": "test:workspace:create",
            "method": "workspace.create",
            "params": {"cwd": "/tmp", "focus": true}
        }),
    );
    let pane_id = created["result"]["root_pane"]["pane_id"]
        .as_str()
        .unwrap()
        .to_string();
    wait_for_server_ptmx_fd_count(server_pid, 1, Duration::from_secs(5));

    let second = request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:split-second",
            "method": "pane.split",
            "params": {
                "target_pane_id": pane_id,
                "direction": "right",
                "focus": true
            }
        }),
    );
    assert_ok(second.clone());
    let second_pane_id = second["result"]["pane"]["pane_id"]
        .as_str()
        .unwrap()
        .to_string();
    wait_for_server_ptmx_fd_count(server_pid, 2, Duration::from_secs(5));

    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:split-third",
            "method": "pane.split",
            "params": {
                "target_pane_id": second_pane_id,
                "direction": "down",
                "focus": true
            }
        }),
    ));
    wait_for_server_ptmx_fd_count(server_pid, 3, Duration::from_secs(5));

    assert_ok(request(
        &api_socket,
        serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{}}),
    ));
    let replacement_pid =
        wait_for_replacement_server_pid(&runtime_dir, server_pid, Duration::from_secs(10));
    wait_for_api(&api_socket, Duration::from_secs(10));
    wait_for_server_ptmx_fd_count(replacement_pid, 3, Duration::from_secs(5));

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    );
    drop(spawned);
    cleanup_test_base(&base);
}

#[cfg(target_os = "linux")]
#[test]
fn live_handoff_unknown_pane_exit_preserves_session_on_shutdown() {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let api_socket = runtime_dir.join("herdr.sock");

    let spawned = spawn_server(&config_home, &runtime_dir, &api_socket);
    wait_for_socket(&api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);

    let created = request(
        &api_socket,
        serde_json::json!({
            "id": "test:workspace:create",
            "method": "workspace.create",
            "params": {"cwd": "/tmp", "focus": true}
        }),
    );
    let pane_id = created["result"]["root_pane"]["pane_id"]
        .as_str()
        .expect("root pane id")
        .to_string();
    let old_pid = spawned.child.process_id().expect("old server pid");

    assert_ok(request(
        &api_socket,
        serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{}}),
    ));
    let replacement_pid =
        wait_for_replacement_server_pid(&runtime_dir, old_pid, Duration::from_secs(10));
    drop(spawned);
    wait_for_api(&api_socket, Duration::from_secs(10));

    let process_info = request(
        &api_socket,
        serde_json::json!({
            "id": "test:process-info",
            "method": "pane.process_info",
            "params": {"pane_id": pane_id}
        }),
    );
    let shell_pid = process_info["result"]["process_info"]["shell_pid"]
        .as_u64()
        .expect("shell pid") as libc::pid_t;
    assert_eq!(unsafe { libc::kill(shell_pid, libc::SIGHUP) }, 0);

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let panes = request(
            &api_socket,
            serde_json::json!({"id":"test:panes","method":"pane.list","params":{}}),
        );
        if panes["result"]["panes"]
            .as_array()
            .is_some_and(Vec::is_empty)
        {
            break;
        }
        assert!(Instant::now() < deadline, "handoff pane was not removed");
        thread::sleep(Duration::from_millis(20));
    }

    assert_ok(request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    ));
    let deadline = Instant::now() + Duration::from_secs(5);
    while Path::new(&format!("/proc/{replacement_pid}")).exists() {
        assert!(Instant::now() < deadline, "replacement server did not stop");
        thread::sleep(Duration::from_millis(20));
    }

    let session: serde_json::Value = serde_json::from_slice(
        &fs::read(config_home.join("herdr-dev/session.json")).expect("saved session"),
    )
    .expect("valid session json");
    assert_eq!(session["workspaces"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        session["workspaces"][0]["tabs"][0]["panes"]
            .as_object()
            .map(serde_json::Map::len),
        Some(1)
    );

    cleanup_test_base(&base);
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn live_handoff_carries_more_panes_than_one_scm_rights_message() {
    const PANES: usize = 70;

    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let api_socket = runtime_dir.join("herdr.sock");

    let spawned = spawn_server(&config_home, &runtime_dir, &api_socket);
    wait_for_socket(&api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);
    let server_pid = spawned
        .child
        .process_id()
        .expect("test server should expose pid");

    let created = request(
        &api_socket,
        serde_json::json!({
            "id": "test:workspace:create",
            "method": "workspace.create",
            "params": {"cwd": "/tmp", "focus": true}
        }),
    );
    let workspace_id = created["result"]["workspace"]["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();

    // One pane per tab keeps the layout shallow, so this exercises the fd
    // transfer rather than the depth of a single split tree.
    for index in 1..PANES {
        assert_ok(request(
            &api_socket,
            serde_json::json!({
                "id": format!("test:tab:create-{index}"),
                "method": "tab.create",
                "params": {"workspace_id": workspace_id, "focus": false}
            }),
        ));
    }
    wait_for_server_ptmx_fd_count(server_pid, PANES, Duration::from_secs(60));

    assert_ok(request(
        &api_socket,
        serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{}}),
    ));
    let replacement_pid =
        wait_for_replacement_server_pid(&runtime_dir, server_pid, Duration::from_secs(30));
    wait_for_api(&api_socket, Duration::from_secs(30));
    wait_for_server_ptmx_fd_count(replacement_pid, PANES, Duration::from_secs(30));

    let panes = request(
        &api_socket,
        serde_json::json!({"id":"test:pane:list","method":"pane.list","params":{}}),
    );
    assert_eq!(
        panes["result"]["panes"].as_array().map(Vec::len),
        Some(PANES),
        "replacement server should report every pane after handoff"
    );

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    );
    drop(spawned);
    cleanup_test_base(&base);
}

#[test]
fn live_handoff_preserves_named_session_socket_paths() {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let session_dir = config_home.join("herdr-dev/sessions/work");
    let api_socket = session_dir.join("herdr.sock");
    let client_socket = session_dir.join("herdr-client.sock");

    let spawned = spawn_named_session_server(&config_home, &runtime_dir, "work");
    wait_for_socket(&api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);

    assert_ok(request(
        &api_socket,
        serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{}}),
    ));
    drop(spawned);
    wait_for_api(&api_socket, Duration::from_secs(10));
    wait_for_socket(&client_socket, Duration::from_secs(5));
    assert!(
        !config_home.join("herdr-dev/herdr.sock").exists(),
        "named handoff unexpectedly bound the default session API socket"
    );

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    );
    cleanup_test_base(&base);
}

#[test]
fn live_handoff_ignores_leaked_default_socket_env_for_named_session() {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let default_session_dir = config_home.join("herdr-dev");
    let default_api_socket = default_session_dir.join("herdr.sock");
    let default_client_socket = default_session_dir.join("herdr-client.sock");
    let work_session_dir = config_home.join("herdr-dev/sessions/work");
    let work_api_socket = work_session_dir.join("herdr.sock");
    let work_client_socket = work_session_dir.join("herdr-client.sock");

    let default_spawned = spawn_default_session_server(&config_home, &runtime_dir);
    wait_for_socket(&default_api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);

    let work_spawned = spawn_server_with_args_and_socket_env(
        &config_home,
        &runtime_dir,
        Some("work"),
        Some(&default_api_socket),
        Some(&default_client_socket),
    );
    wait_for_socket(&work_api_socket, Duration::from_secs(10));

    assert_ok(request(
        &work_api_socket,
        serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{}}),
    ));
    drop(work_spawned);
    wait_for_api(&default_api_socket, Duration::from_secs(10));
    wait_for_api(&work_api_socket, Duration::from_secs(10));
    wait_for_socket(&work_client_socket, Duration::from_secs(5));

    let _ = request(
        &work_api_socket,
        serde_json::json!({"id":"test:stop-work","method":"server.stop","params":{}}),
    );
    let _ = request(
        &default_api_socket,
        serde_json::json!({"id":"test:stop-default","method":"server.stop","params":{}}),
    );
    drop(default_spawned);
    cleanup_test_base(&base);
}

#[test]
fn live_handoff_preserves_client_socket_env_without_api_socket_env() {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let api_socket = config_home.join("herdr-dev/herdr.sock");
    let client_socket = runtime_dir.join("custom-client.sock");

    let spawned = spawn_server_with_args_and_socket_env(
        &config_home,
        &runtime_dir,
        None,
        None,
        Some(&client_socket),
    );
    wait_for_socket(&api_socket, Duration::from_secs(10));
    wait_for_socket(&client_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);

    assert_ok(request(
        &api_socket,
        serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{}}),
    ));
    drop(spawned);
    wait_for_api(&api_socket, Duration::from_secs(10));
    wait_for_socket(&client_socket, Duration::from_secs(5));

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    );
    cleanup_test_base(&base);
}

#[test]
fn live_handoff_preserves_installed_plugins() {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let api_socket = config_home.join("herdr-dev/herdr.sock");
    let registry_path = config_home.join("herdr-dev/plugins.json");
    let existing_plugin = base.join("plugins/existing");
    let added_plugin = base.join("plugins/added");
    write_plugin_manifest(&existing_plugin, "test.live-handoff-existing");
    write_plugin_manifest(&added_plugin, "test.live-handoff-added");

    let spawned = spawn_default_session_server(&config_home, &runtime_dir);
    wait_for_socket(&api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);

    link_plugin(&api_socket, &existing_plugin);
    assert_eq!(
        listed_plugin_ids(&api_socket),
        ["test.live-handoff-existing"]
    );

    assert_ok(request(
        &api_socket,
        serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{}}),
    ));
    drop(spawned);
    wait_for_api(&api_socket, Duration::from_secs(10));

    assert_eq!(
        listed_plugin_ids(&api_socket),
        ["test.live-handoff-existing"]
    );
    link_plugin(&api_socket, &added_plugin);
    assert_eq!(
        saved_plugin_ids(&registry_path),
        ["test.live-handoff-added", "test.live-handoff-existing"]
    );

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    );
    cleanup_test_base(&base);
}

#[test]
fn live_handoff_preserves_pane_process_io() {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let api_socket = runtime_dir.join("herdr.sock");
    let client_socket = runtime_dir.join("herdr-client.sock");
    let marker = base.join("child.pid");
    let second_marker = base.join("second-child.pid");
    let hup_marker = base.join("hup");
    let second_hup_marker = base.join("second-hup");
    let received_marker = base.join("received");
    let second_received_marker = base.join("second-received");

    let spawned = spawn_server(&config_home, &runtime_dir, &api_socket);
    wait_for_socket(&api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);

    let created = request(
        &api_socket,
        serde_json::json!({
            "id": "test:workspace:create",
            "method": "workspace.create",
            "params": {"cwd": "/tmp", "focus": true}
        }),
    );
    let pane_id = created["result"]["root_pane"]["pane_id"]
        .as_str()
        .unwrap()
        .to_string();
    let split = request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:split",
            "method": "pane.split",
            "params": {
                "target_pane_id": pane_id,
                "direction": "right",
                "focus": false
            }
        }),
    );
    assert_ok(split.clone());
    let second_pane_id = split["result"]["pane"]["pane_id"]
        .as_str()
        .unwrap()
        .to_string();

    let command = format!(
        "sh -c 'echo READY $$ > {}; trap \"echo HUP >> {}\" HUP; while read line; do echo got:$line; echo got:$line >> {}; done'",
        marker.display(),
        hup_marker.display(),
        received_marker.display()
    );
    let second_command = format!(
        "sh -c 'echo SECOND_READY $$ > {}; trap \"echo HUP >> {}\" HUP; while read line; do echo second:$line; echo second:$line >> {}; done'",
        second_marker.display(),
        second_hup_marker.display(),
        second_received_marker.display()
    );
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:run",
            "method": "pane.send_input",
            "params": {"pane_id": pane_id, "text": command, "keys": ["Enter"]}
        }),
    ));
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:second-pane:run",
            "method": "pane.send_input",
            "params": {"pane_id": second_pane_id, "text": second_command, "keys": ["Enter"]}
        }),
    ));
    support::wait_for_file(&marker, Duration::from_secs(5));
    support::wait_for_file(&second_marker, Duration::from_secs(5));
    let pid_text = fs::read_to_string(&marker).unwrap();
    let child_pid: u32 = pid_text.split_whitespace().last().unwrap().parse().unwrap();
    let second_pid_text = fs::read_to_string(&second_marker).unwrap();
    let second_child_pid: u32 = second_pid_text
        .split_whitespace()
        .last()
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(unsafe { libc::kill(child_pid as libc::pid_t, 0) }, 0);
    assert_eq!(unsafe { libc::kill(second_child_pid as libc::pid_t, 0) }, 0);

    let endpoint_generation = support::CURRENT_ENDPOINT_PROTOCOL_GENERATION;
    let mut client_stream = UnixStream::connect(&client_socket).unwrap();
    let (server_generation, error) =
        client_shell_handshake(&mut client_stream, endpoint_generation, 54, 23).unwrap();
    assert_eq!(server_generation, endpoint_generation);
    assert!(error.is_none(), "client shell handshake failed: {error:?}");
    assert!(
        wait_for_message_variant(
            &mut client_stream,
            Duration::from_secs(5),
            SERVER_MESSAGE_ENDPOINT_CONTROL,
        )
        .unwrap(),
        "client shell should receive a complete snapshot before handoff"
    );

    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:before-log",
            "method": "pane.send_input",
            "params": {"pane_id": pane_id, "text": "before_replay", "keys": ["Enter"]}
        }),
    ));
    wait_for_output(&api_socket, &pane_id, "got:before_replay");

    assert_ok(request(
        &api_socket,
        serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{}}),
    ));
    assert!(
        wait_for_message_variant(
            &mut client_stream,
            Duration::from_secs(5),
            SERVER_MESSAGE_SERVER_SHUTDOWN,
        )
        .unwrap(),
        "connected client shell should receive live-handoff shutdown"
    );
    drop(spawned);
    thread::sleep(Duration::from_millis(300));
    wait_for_api(&api_socket, Duration::from_secs(10));
    wait_for_socket(&client_socket, Duration::from_secs(5));
    assert_eq!(unsafe { libc::kill(child_pid as libc::pid_t, 0) }, 0);
    assert_eq!(unsafe { libc::kill(second_child_pid as libc::pid_t, 0) }, 0);
    assert!(
        !hup_marker.exists(),
        "pane process received HUP during handoff"
    );
    assert!(
        !second_hup_marker.exists(),
        "second pane process received HUP during handoff"
    );
    wait_for_output(&api_socket, &pane_id, "got:before_replay");

    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:send",
            "method": "pane.send_input",
            "params": {"pane_id": pane_id, "text": "after-handoff", "keys": ["Enter"]}
        }),
    ));
    wait_for_file_contains(
        &received_marker,
        "got:after-handoff",
        Duration::from_secs(5),
    );
    wait_for_output(&api_socket, &pane_id, "got:after-handoff");
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:second-pane:send",
            "method": "pane.send_input",
            "params": {"pane_id": second_pane_id, "text": "after-handoff-second", "keys": ["Enter"]}
        }),
    ));
    wait_for_file_contains(
        &second_received_marker,
        "second:after-handoff-second",
        Duration::from_secs(5),
    );
    wait_for_output(&api_socket, &second_pane_id, "second:after-handoff-sec");

    let mut reattached_shell = UnixStream::connect(&client_socket).unwrap();
    let (server_generation, error) = client_shell_handshake(
        &mut reattached_shell,
        support::CURRENT_ENDPOINT_PROTOCOL_GENERATION,
        54,
        23,
    )
    .unwrap();
    assert_eq!(
        server_generation,
        support::CURRENT_ENDPOINT_PROTOCOL_GENERATION
    );
    assert!(error.is_none(), "reattached client shell failed: {error:?}");
    wait_for_client_shell_bootstrap(&mut reattached_shell, Duration::from_secs(5))
        .expect("fresh client shell should receive restored snapshot before pane content");

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    );
    let _ = client_socket;
    cleanup_test_base(&base);
}

#[test]
fn live_handoff_preserves_keyboard_protocol_for_client_input() {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let api_socket = runtime_dir.join("herdr.sock");
    let client_socket = runtime_dir.join("herdr-client.sock");
    let script = base.join("read-raw.py");
    let ready_marker = base.join("keyboard-ready");
    let received_marker = base.join("keyboard-received");

    fs::create_dir_all(&base).unwrap();
    fs::write(
        &script,
        format!(
            r#"import os
import pathlib
import select
import sys
import tty

sys.stdout.buffer.write(b"\x1b[>5u")
sys.stdout.flush()
pathlib.Path({ready:?}).write_text("ready")
tty.setraw(sys.stdin.fileno())
ready_fds, _, _ = select.select([sys.stdin.fileno()], [], [], 5)
data = os.read(sys.stdin.fileno(), 32) if ready_fds else b""
pathlib.Path({received:?}).write_text(data.hex())
"#,
            ready = ready_marker.display().to_string(),
            received = received_marker.display().to_string()
        ),
    )
    .unwrap();

    let spawned = spawn_server(&config_home, &runtime_dir, &api_socket);
    wait_for_socket(&api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);

    let created = request(
        &api_socket,
        serde_json::json!({
            "id": "test:workspace:create",
            "method": "workspace.create",
            "params": {"cwd": "/tmp", "focus": true}
        }),
    );
    let pane_id = created["result"]["root_pane"]["pane_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:run",
            "method": "pane.send_input",
            "params": {"pane_id": pane_id, "text": format!("python3 {}", script.display()), "keys": ["Enter"]}
        }),
    ));
    support::wait_for_file(&ready_marker, Duration::from_secs(5));

    assert_ok(request(
        &api_socket,
        serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{}}),
    ));
    drop(spawned);
    wait_for_api(&api_socket, Duration::from_secs(10));
    wait_for_socket(&client_socket, Duration::from_secs(5));

    let mut client_stream = UnixStream::connect(&client_socket).unwrap();
    let (server_generation, error) = client_shell_handshake(
        &mut client_stream,
        support::CURRENT_ENDPOINT_PROTOCOL_GENERATION,
        54,
        23,
    )
    .unwrap();
    assert_eq!(
        server_generation,
        support::CURRENT_ENDPOINT_PROTOCOL_GENERATION
    );
    assert!(error.is_none(), "client shell handshake failed: {error:?}");
    wait_for_client_shell_bootstrap(&mut client_stream, Duration::from_secs(5))
        .expect("client shell should receive restored state before sending input");
    send_client_shell_shift_enter(&mut client_stream, &pane_id).unwrap();

    wait_for_file_contains(&received_marker, "1b5b31333b3275", Duration::from_secs(5));

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    );
    cleanup_test_base(&base);
}

#[test]
fn live_handoff_preserves_modify_other_keys_for_client_input() {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let api_socket = runtime_dir.join("herdr.sock");
    let client_socket = runtime_dir.join("herdr-client.sock");
    let script = base.join("read-raw.py");
    let ready_marker = base.join("modify-ready");
    let received_marker = base.join("modify-received");

    fs::create_dir_all(&base).unwrap();
    fs::write(
        &script,
        format!(
            r#"import os
import pathlib
import select
import sys
import tty

sys.stdout.buffer.write(b"\x1b[>4;2m")
sys.stdout.flush()
pathlib.Path({ready:?}).write_text("ready")
tty.setraw(sys.stdin.fileno())
ready_fds, _, _ = select.select([sys.stdin.fileno()], [], [], 5)
data = os.read(sys.stdin.fileno(), 32) if ready_fds else b""
pathlib.Path({received:?}).write_text(data.hex())
"#,
            ready = ready_marker.display().to_string(),
            received = received_marker.display().to_string()
        ),
    )
    .unwrap();

    let spawned = spawn_server(&config_home, &runtime_dir, &api_socket);
    wait_for_socket(&api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);

    let created = request(
        &api_socket,
        serde_json::json!({
            "id": "test:workspace:create",
            "method": "workspace.create",
            "params": {"cwd": "/tmp", "focus": true}
        }),
    );
    let pane_id = created["result"]["root_pane"]["pane_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:run",
            "method": "pane.send_input",
            "params": {"pane_id": pane_id, "text": format!("python3 {}", script.display()), "keys": ["Enter"]}
        }),
    ));
    support::wait_for_file(&ready_marker, Duration::from_secs(5));

    assert_ok(request(
        &api_socket,
        serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{}}),
    ));
    drop(spawned);
    wait_for_api(&api_socket, Duration::from_secs(10));
    wait_for_socket(&client_socket, Duration::from_secs(5));

    let mut client_stream = UnixStream::connect(&client_socket).unwrap();
    let (server_generation, error) = client_shell_handshake(
        &mut client_stream,
        support::CURRENT_ENDPOINT_PROTOCOL_GENERATION,
        54,
        23,
    )
    .unwrap();
    assert_eq!(
        server_generation,
        support::CURRENT_ENDPOINT_PROTOCOL_GENERATION
    );
    assert!(error.is_none(), "client shell handshake failed: {error:?}");
    wait_for_client_shell_bootstrap(&mut client_stream, Duration::from_secs(5))
        .expect("client shell should receive restored state before sending input");
    send_client_shell_shift_enter(&mut client_stream, &pane_id).unwrap();

    wait_for_file_contains(
        &received_marker,
        "1b5b32373b323b31337e",
        Duration::from_secs(5),
    );

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    );
    cleanup_test_base(&base);
}

#[test]
fn live_handoff_accepts_canonical_pane_id_from_child_env() {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let api_socket = runtime_dir.join("herdr.sock");
    let pane_id_marker = base.join("pane-id");

    let spawned = spawn_server(&config_home, &runtime_dir, &api_socket);
    wait_for_socket(&api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);

    let created = request(
        &api_socket,
        serde_json::json!({
            "id": "test:workspace:create",
            "method": "workspace.create",
            "params": {"cwd": "/tmp", "focus": true}
        }),
    );
    let pane_id = created["result"]["root_pane"]["pane_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:print-id",
            "method": "pane.send_input",
            "params": {"pane_id": pane_id, "text": format!("printf '%s' \"$HERDR_PANE_ID\" > {}", pane_id_marker.display()), "keys": ["Enter"]}
        }),
    ));
    let old_pane_id = wait_for_file_contains(&pane_id_marker, &pane_id, Duration::from_secs(5));
    assert!(
        old_pane_id == pane_id,
        "unexpected pane id from env: {old_pane_id:?}"
    );

    assert_ok(request(
        &api_socket,
        serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{}}),
    ));
    drop(spawned);
    wait_for_api(&api_socket, Duration::from_secs(10));

    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:old-pane-report",
            "method": "pane.report_agent",
            "params": {
                "pane_id": old_pane_id,
                "source": "handoff-test",
                "agent": "pi",
                "state": "working"
            }
        }),
    ));
    let agents = request(
        &api_socket,
        serde_json::json!({"id":"test:agent-list","method":"agent.list","params":{}}),
    );
    let found = agents["result"]["agents"]
        .as_array()
        .unwrap()
        .iter()
        .any(|agent| {
            agent["agent"].as_str() == Some("pi")
                && agent["agent_status"].as_str() == Some("working")
        });
    assert!(
        found,
        "old pane id report did not update restored pane: {agents}"
    );

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    );
    cleanup_test_base(&base);
}

#[test]
fn live_handoff_keeps_unmanaged_agent_name_bound_to_saved_session() {
    use std::os::unix::fs::PermissionsExt;

    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let api_socket = runtime_dir.join("herdr.sock");
    let old_session = base.join("old-session.jsonl");
    let new_session = base.join("new-session.jsonl");
    let started_marker = base.join("agent-started");
    let fake_pi = base.join("pi");
    fs::create_dir_all(&base).unwrap();
    fs::write(
        &fake_pi,
        format!(
            "#!/bin/sh\nexport HERDR_AGENT=pi\necho started > {}\n/bin/sleep 30\n:\n",
            started_marker.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&fake_pi, fs::Permissions::from_mode(0o755)).unwrap();

    let spawned = spawn_server(&config_home, &runtime_dir, &api_socket);
    wait_for_socket(&api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);
    let created = request(
        &api_socket,
        serde_json::json!({
            "id": "test:workspace:create",
            "method": "workspace.create",
            "params": {"cwd": "/tmp", "focus": true}
        }),
    );
    let pane_id = created["result"]["root_pane"]["pane_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:start-agent",
            "method": "pane.send_input",
            "params": {"pane_id": pane_id, "text": fake_pi, "keys": ["Enter"]}
        }),
    ));
    support::wait_for_file(&started_marker, Duration::from_secs(5));
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:agent:session",
            "method": "pane.report_agent_session",
            "params": {
                "pane_id": pane_id,
                "source": "herdr:pi",
                "agent": "pi",
                "seq": 1,
                "agent_session_path": old_session,
                "session_start_source": "startup"
            }
        }),
    ));
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:agent:report",
            "method": "pane.report_agent",
            "params": {
                "pane_id": pane_id,
                "source": "herdr:pi",
                "agent": "pi",
                "state": "idle",
                "seq": 2,
                "agent_session_path": old_session
            }
        }),
    ));
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let response = request(
            &api_socket,
            serde_json::json!({
                "id": "test:agent:wait-for-process",
                "method": "agent.get",
                "params": {"target": pane_id}
            }),
        );
        if response.get("result").is_some() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "agent process was not detected: {response}"
        );
        thread::sleep(Duration::from_millis(25));
    }
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:agent:rename",
            "method": "agent.rename",
            "params": {"target": pane_id, "name": "reviewer"}
        }),
    ));

    assert_ok(request(
        &api_socket,
        serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{}}),
    ));
    drop(spawned);
    wait_for_api(&api_socket, Duration::from_secs(10));

    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:agent:new-session",
            "method": "pane.report_agent_session",
            "params": {
                "pane_id": pane_id,
                "source": "herdr:pi",
                "agent": "pi",
                "seq": 3,
                "agent_session_path": new_session,
                "session_start_source": "new"
            }
        }),
    ));
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let old_name = request(
            &api_socket,
            serde_json::json!({
                "id": "test:agent:get-old-name",
                "method": "agent.get",
                "params": {"target": "reviewer"}
            }),
        );
        if old_name["error"]["code"] == "agent_not_found" {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "old session alias was not cleared: {old_name}"
        );
        thread::sleep(Duration::from_millis(25));
    }

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    );
    cleanup_test_base(&base);
}

#[test]
fn live_handoff_keeps_agent_started_pane_after_agent_exits() {
    use std::os::unix::fs::PermissionsExt;

    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let api_socket = runtime_dir.join("herdr.sock");
    let started_marker = base.join("agent-started");
    let exited_marker = base.join("agent-exited");
    let ready_marker = base.join("shell-ready");
    let shell_marker = base.join("shell-after-agent");
    let bin = base.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let delayed_shell = bin.join("delayed-shell");
    fs::write(&delayed_shell, "#!/bin/sh\n/bin/sleep 0.4\nexec /bin/sh\n").unwrap();
    fs::set_permissions(&delayed_shell, fs::Permissions::from_mode(0o755)).unwrap();
    let fake_pi = bin.join("pi");
    fs::write(
        &fake_pi,
        format!(
            "#!/bin/sh\nexport HERDR_AGENT=pi\necho started > {}\n/bin/sleep 1\necho exited > {}\n",
            started_marker.display(),
            exited_marker.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&fake_pi, fs::Permissions::from_mode(0o755)).unwrap();
    let path = format!("{}:/bin:/usr/bin", bin.display());

    let spawned = spawn_server_with_env(
        &config_home,
        &runtime_dir,
        &api_socket,
        &[
            ("PATH", path.as_str()),
            ("SHELL", delayed_shell.to_str().unwrap()),
        ],
    );
    wait_for_socket(&api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);
    let workspace = request(
        &api_socket,
        serde_json::json!({
            "id": "test:workspace-create",
            "method": "workspace.create",
            "params": { "cwd": "/tmp", "focus": false }
        }),
    );
    assert_ok(workspace.clone());
    let pane_id = workspace["result"]["root_pane"]["pane_id"]
        .as_str()
        .unwrap()
        .to_string();

    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:shell-ready",
            "method": "pane.send_input",
            "params": {
                "pane_id": pane_id,
                "text": format!("printf ready > {}", ready_marker.display()),
                "keys": ["Enter"]
            }
        }),
    ));
    // Creation acknowledges the PTY, not an idle interactive shell. A real
    // shell command must execute before this raw agent.start request.
    support::wait_for_file(&ready_marker, Duration::from_secs(5));

    let started = request(
        &api_socket,
        serde_json::json!({
            "id": "test:agent-start",
            "method": "agent.start",
            "params": {
                "name": "handoff-agent",
                "kind": "pi",
                "pane_id": pane_id,
                "timeout_ms": 5000
            }
        }),
    );
    assert_ok(started);
    support::wait_for_file(&started_marker, Duration::from_secs(5));

    assert_ok(request(
        &api_socket,
        serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{"force":true}}),
    ));
    drop(spawned);
    wait_for_api(&api_socket, Duration::from_secs(10));
    support::wait_for_file(&exited_marker, Duration::from_secs(5));
    thread::sleep(Duration::from_millis(300));

    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:shell-after-agent",
            "method": "pane.send_input",
            "params": {"pane_id": pane_id, "text": format!("echo alive > {}", shell_marker.display()), "keys": ["Enter"]}
        }),
    ));
    support::wait_for_file(&shell_marker, Duration::from_secs(5));

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    );
    cleanup_test_base(&base);
}

#[test]
fn live_handoff_keeps_shell_pane_after_foreground_process_exits() {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let api_socket = runtime_dir.join("herdr.sock");
    let started_marker = base.join("foreground-started");
    let exited_marker = base.join("foreground-exited");
    let shell_marker = base.join("shell-after-foreground");

    let spawned = spawn_server(&config_home, &runtime_dir, &api_socket);
    wait_for_socket(&api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);

    let created = request(
        &api_socket,
        serde_json::json!({
            "id": "test:workspace:create",
            "method": "workspace.create",
            "params": {"cwd": "/tmp", "focus": true}
        }),
    );
    let pane_id = created["result"]["root_pane"]["pane_id"]
        .as_str()
        .unwrap()
        .to_string();
    let command = format!(
        "sh -c 'echo started > {}; sleep 1; echo exited > {}'",
        started_marker.display(),
        exited_marker.display()
    );
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:run-foreground",
            "method": "pane.send_input",
            "params": {"pane_id": pane_id, "text": command, "keys": ["Enter"]}
        }),
    ));
    support::wait_for_file(&started_marker, Duration::from_secs(5));

    assert_ok(request(
        &api_socket,
        serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{}}),
    ));
    drop(spawned);
    wait_for_api(&api_socket, Duration::from_secs(10));
    support::wait_for_file(&exited_marker, Duration::from_secs(5));

    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:shell-after-foreground",
            "method": "pane.send_input",
            "params": {"pane_id": pane_id, "text": format!("echo alive > {}", shell_marker.display()), "keys": ["Enter"]}
        }),
    ));
    support::wait_for_file(&shell_marker, Duration::from_secs(5));

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    );
    cleanup_test_base(&base);
}

#[test]
fn live_handoff_preserves_python_http_server() {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let api_socket = runtime_dir.join("herdr.sock");
    let client_socket = runtime_dir.join("herdr-client.sock");
    let web_root = base.join("web");
    fs::create_dir_all(&web_root).unwrap();
    fs::write(
        web_root.join("index.html"),
        "hello-from-python-before-and-after",
    )
    .unwrap();
    let port = unused_local_port();

    let spawned = spawn_server(&config_home, &runtime_dir, &api_socket);
    wait_for_socket(&api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);

    let created = request(
        &api_socket,
        serde_json::json!({
            "id": "test:workspace:create",
            "method": "workspace.create",
            "params": {"cwd": web_root, "focus": true}
        }),
    );
    let pane_id = created["result"]["root_pane"]["pane_id"]
        .as_str()
        .unwrap()
        .to_string();

    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:run-python",
            "method": "pane.send_input",
            "params": {
                "pane_id": pane_id,
                "text": format!("python3 -m http.server {port} --bind 127.0.0.1"),
                "keys": ["Enter"]
            }
        }),
    ));
    wait_for_http_contains(
        port,
        "hello-from-python-before-and-after",
        Duration::from_secs(10),
    );

    assert_ok(request(
        &api_socket,
        serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{}}),
    ));
    drop(spawned);
    wait_for_api(&api_socket, Duration::from_secs(10));
    wait_for_http_contains(
        port,
        "hello-from-python-before-and-after",
        Duration::from_secs(10),
    );

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    );
    let _ = client_socket;
    cleanup_test_base(&base);
}

#[test]
fn live_handoff_preserves_http_servers_across_multiple_sessions() {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let sessions = [
        (None, config_home.join("herdr-dev/herdr.sock")),
        (
            Some("work"),
            config_home.join("herdr-dev/sessions/work/herdr.sock"),
        ),
    ];
    let mut spawned = Vec::new();
    let mut ports = Vec::new();

    for (session_name, api_socket) in &sessions {
        let web_root = base.join(format!("web-{}", session_name.unwrap_or("default")));
        fs::create_dir_all(&web_root).unwrap();
        fs::write(
            web_root.join("index.html"),
            format!("hello-from-{}", session_name.unwrap_or("default")),
        )
        .unwrap();
        let port = unused_local_port();
        let server = if let Some(session_name) = session_name {
            spawn_named_session_server(&config_home, &runtime_dir, session_name)
        } else {
            spawn_default_session_server(&config_home, &runtime_dir)
        };
        wait_for_socket(api_socket, Duration::from_secs(10));
        let created = request(
            api_socket,
            serde_json::json!({
                "id": "test:workspace:create",
                "method": "workspace.create",
                "params": {"cwd": web_root, "focus": true}
            }),
        );
        let pane_id = created["result"]["root_pane"]["pane_id"]
            .as_str()
            .unwrap()
            .to_string();
        assert_ok(request(
            api_socket,
            serde_json::json!({
                "id": "test:pane:run-python",
                "method": "pane.send_input",
                "params": {
                    "pane_id": pane_id,
                    "text": format!("python3 -m http.server {port} --bind 127.0.0.1"),
                    "keys": ["Enter"]
                }
            }),
        ));
        wait_for_http_contains(
            port,
            &format!("hello-from-{}", session_name.unwrap_or("default")),
            Duration::from_secs(10),
        );
        spawned.push(server);
        ports.push((port, session_name.unwrap_or("default").to_string()));
    }
    register_runtime_dir(&runtime_dir);

    for (_session_name, api_socket) in &sessions {
        assert_ok(request(
            api_socket,
            serde_json::json!({"id":"test:handoff","method":"server.live_handoff","params":{}}),
        ));
    }
    drop(spawned);

    for (_session_name, api_socket) in &sessions {
        wait_for_api(api_socket, Duration::from_secs(10));
    }
    for (port, label) in ports {
        wait_for_http_contains(
            port,
            &format!("hello-from-{label}"),
            Duration::from_secs(10),
        );
    }

    for (_session_name, api_socket) in &sessions {
        let _ = request(
            api_socket,
            serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
        );
    }
    cleanup_test_base(&base);
}

#[test]
fn live_handoff_bad_expected_protocol_rolls_back_old_server() {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let api_socket = runtime_dir.join("herdr.sock");
    let marker = base.join("child.pid");
    let received_marker = base.join("received");

    let spawned = spawn_server(&config_home, &runtime_dir, &api_socket);
    wait_for_socket(&api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);

    let created = request(
        &api_socket,
        serde_json::json!({
            "id": "test:workspace:create",
            "method": "workspace.create",
            "params": {"cwd": "/tmp", "focus": true}
        }),
    );
    let pane_id = created["result"]["root_pane"]["pane_id"]
        .as_str()
        .unwrap()
        .to_string();
    let command = format!(
        "sh -c 'echo READY $$ > {}; while read line; do echo got:$line; echo got:$line >> {}; done'",
        marker.display(),
        received_marker.display()
    );
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:run",
            "method": "pane.send_input",
            "params": {"pane_id": pane_id, "text": command, "keys": ["Enter"]}
        }),
    ));
    support::wait_for_file(&marker, Duration::from_secs(5));
    let pid_text = fs::read_to_string(&marker).unwrap();
    let child_pid: u32 = pid_text.split_whitespace().last().unwrap().parse().unwrap();

    let failed = request(
        &api_socket,
        serde_json::json!({
            "id": "test:bad-handoff",
            "method": "server.live_handoff",
            "params": {"expected_protocol": 999999}
        }),
    );
    assert!(
        failed.get("error").is_some(),
        "bad protocol handoff should fail: {failed}"
    );
    wait_for_api(&api_socket, Duration::from_secs(5));
    assert_eq!(unsafe { libc::kill(child_pid as libc::pid_t, 0) }, 0);

    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:send-after-failed-handoff",
            "method": "pane.send_input",
            "params": {"pane_id": pane_id, "text": "after-failed-handoff", "keys": ["Enter"]}
        }),
    ));
    wait_for_file_contains(
        &received_marker,
        "got:after-failed-handoff",
        Duration::from_secs(5),
    );
    wait_for_output(&api_socket, &pane_id, "got:after-failed-handoff");

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    );
    drop(spawned);
    cleanup_test_base(&base);
}

fn live_handoff_import_failure_rolls_back_old_server_at(failure_point: &str) {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let api_socket = runtime_dir.join("herdr.sock");
    let client_socket = runtime_dir.join("herdr-client.sock");
    let marker = base.join("child.pid");
    let received_marker = base.join("received");

    let spawned = spawn_server_with_env(
        &config_home,
        &runtime_dir,
        &api_socket,
        &[("HERDR_TEST_HANDOFF_IMPORT_FAIL", failure_point)],
    );
    wait_for_socket(&api_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);

    let created = request(
        &api_socket,
        serde_json::json!({
            "id": "test:workspace:create",
            "method": "workspace.create",
            "params": {"cwd": "/tmp", "focus": true}
        }),
    );
    let pane_id = created["result"]["root_pane"]["pane_id"]
        .as_str()
        .unwrap()
        .to_string();
    let command = format!(
        "sh -c 'echo READY $$ > {}; while read line; do echo got:$line; echo got:$line >> {}; done'",
        marker.display(),
        received_marker.display()
    );
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:run",
            "method": "pane.send_input",
            "params": {"pane_id": pane_id, "text": command, "keys": ["Enter"]}
        }),
    ));
    support::wait_for_file(&marker, Duration::from_secs(5));
    let pid_text = fs::read_to_string(&marker).unwrap();
    let child_pid: u32 = pid_text.split_whitespace().last().unwrap().parse().unwrap();

    let failed = request(
        &api_socket,
        serde_json::json!({"id":"test:handoff-fail","method":"server.live_handoff","params":{}}),
    );
    assert!(
        failed.get("error").is_some(),
        "{failure_point} handoff should fail: {failed}"
    );
    wait_for_api(&api_socket, Duration::from_secs(10));
    wait_for_socket(&client_socket, Duration::from_secs(5));
    assert_eq!(unsafe { libc::kill(child_pid as libc::pid_t, 0) }, 0);

    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "test:pane:send-after-import-failure",
            "method": "pane.send_input",
            "params": {"pane_id": pane_id, "text": failure_point, "keys": ["Enter"]}
        }),
    ));
    wait_for_file_contains(
        &received_marker,
        &format!("got:{failure_point}"),
        Duration::from_secs(5),
    );

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"test:stop","method":"server.stop","params":{}}),
    );
    drop(spawned);
    cleanup_test_base(&base);
}

#[test]
fn live_handoff_after_restored_failure_rolls_back_old_server() {
    live_handoff_import_failure_rolls_back_old_server_at("after_restored");
}

const DRILL_SESSION_CONFIG: &str = r#"onboarding = false
[experimental]
agent_parent_notify = true
[session]
resume_agents_on_restore = true
"#;

fn write_executable(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn drill_command(config_home: &Path, runtime_dir: &Path, session: &str, path: &str) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_herdr"));
    cmd.env("XDG_CONFIG_HOME", config_home);
    cmd.env("XDG_RUNTIME_DIR", runtime_dir);
    cmd.env("HERDR_SESSION", session);
    cmd.env("PATH", path);
    cmd.env_remove("HERDR_SOCKET_PATH");
    cmd.env_remove("HERDR_CLIENT_SOCKET_PATH");
    cmd
}

fn pane_send(api_socket: &Path, pane_id: &str, text: &str) {
    assert_ok(request(
        api_socket,
        serde_json::json!({
            "id": "drill:pane:send",
            "method": "pane.send_input",
            "params": {"pane_id": pane_id, "text": text, "keys": ["Enter"]}
        }),
    ));
}

fn pane_send_with_path(api_socket: &Path, pane_id: &str, path: &str, text: &str) {
    pane_send(api_socket, pane_id, &format!("PATH={path} {text}"));
}

fn wait_for_agent(api_socket: &Path, target: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let response = request(
            api_socket,
            serde_json::json!({
                "id": "drill:agent:wait",
                "method": "agent.get",
                "params": {"target": target}
            }),
        );
        if response.get("result").is_some() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "agent was not detected on {target}: {response}"
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn wait_for_agent_kind(api_socket: &Path, target: &str, kind: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut last = String::new();
    while Instant::now() < deadline {
        match try_request(
            api_socket,
            serde_json::json!({
                "id": "drill:agent:kind",
                "method": "agent.get",
                "params": {"target": target}
            }),
        ) {
            Ok(response) if response["result"]["agent"]["agent"].as_str() == Some(kind) => {
                return;
            }
            Ok(response) => last = response.to_string(),
            Err(err) if err.retryable => last = err.message,
            Err(err) => panic!("{}", err.message),
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("agent {target} was not kind {kind}; last: {last}");
}

fn report_unmanaged_agent(api_socket: &Path, pane_id: &str, agent: &str, session_id: Option<&str>) {
    let source = format!("herdr:{agent}");
    if let Some(session_id) = session_id {
        assert_ok(request(
            api_socket,
            serde_json::json!({
                "id": "drill:agent:session",
                "method": "pane.report_agent_session",
                "params": {
                    "pane_id": pane_id,
                    "source": source.as_str(),
                    "agent": agent,
                    "seq": 1,
                    "agent_session_id": session_id,
                    "session_start_source": "startup"
                }
            }),
        ));
    }
    assert_ok(request(
        api_socket,
        serde_json::json!({
            "id": "drill:agent:report",
            "method": "pane.report_agent",
            "params": {
                "pane_id": pane_id,
                "source": source.as_str(),
                "agent": agent,
                "state": "idle",
                "seq": 2
            }
        }),
    ));
    wait_for_agent(api_socket, pane_id);
}

fn agent_list(api_socket: &Path) -> serde_json::Value {
    let response = request_until_ok(
        api_socket,
        serde_json::json!({"id":"drill:agent:list","method":"agent.list","params":{}}),
        Duration::from_secs(15),
    );
    assert_ok(response.clone());
    response
}

fn agent_names(api_socket: &Path) -> Vec<String> {
    agent_list(api_socket)["result"]["agents"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|agent| agent["name"].as_str().map(str::to_string))
        .collect()
}

fn pane_list_ids(api_socket: &Path) -> Vec<String> {
    let response = request(
        api_socket,
        serde_json::json!({"id":"drill:pane:list","method":"pane.list","params":{}}),
    );
    assert_ok(response.clone());
    response["result"]["panes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|pane| pane["pane_id"].as_str().map(str::to_string))
        .collect()
}

fn wait_for_pane_ids(api_socket: &Path, min_count: usize, timeout: Duration) -> Vec<String> {
    let deadline = Instant::now() + timeout;
    let mut last = Vec::new();
    while Instant::now() < deadline {
        last = pane_list_ids(api_socket);
        if last.len() >= min_count {
            return last;
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("expected at least {min_count} panes; last={last:?}");
}

fn pane_pids(api_socket: &Path, pane_id: &str) -> Vec<i32> {
    let response = request(
        api_socket,
        serde_json::json!({
            "id": "drill:process-info",
            "method": "pane.process_info",
            "params": {"pane_id": pane_id}
        }),
    );
    assert_ok(response.clone());
    let info = &response["result"]["process_info"];
    let mut pids = Vec::new();
    if let Some(pid) = info["shell_pid"].as_u64() {
        pids.push(pid as i32);
    }
    if let Some(processes) = info["foreground_processes"].as_array() {
        for process in processes {
            if let Some(pid) = process["pid"].as_u64() {
                pids.push(pid as i32);
            }
        }
    }
    pids.sort_unstable();
    pids.dedup();
    pids
}

fn pids_alive(pids: &[i32]) {
    for pid in pids {
        assert_eq!(unsafe { libc::kill(*pid, 0) }, 0, "pid {pid} is not alive");
    }
}

fn wait_marker(path: &Path) {
    support::wait_for_file(path, Duration::from_secs(15));
}

fn wait_shell_ready(api_socket: &Path, pane_id: &str, marker: &Path) {
    let _ = fs::remove_file(marker);
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut last = String::new();
    while Instant::now() < deadline {
        match try_request(
            api_socket,
            serde_json::json!({
                "id": "drill:pane:send",
                "method": "pane.send_input",
                "params": {"pane_id": pane_id, "text": format!("printf ready > {}", marker.display()), "keys": ["Enter"]}
            }),
        ) {
            Ok(response) if response.get("result").is_some() => {
                wait_marker(marker);
                return;
            }
            Ok(response) => last = response.to_string(),
            Err(err) if err.retryable => last = err.message,
            Err(err) => panic!("{}", err.message),
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("pane {pane_id} did not accept input; last: {last}");
}

fn wait_for_cli_exit_with_logs(
    child: &mut std::process::Child,
    timeout: Duration,
    logs: &[PathBuf],
) -> i32 {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait().unwrap() {
            Some(status) => return status.code().unwrap_or(1),
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                let mut dump = String::new();
                for path in logs {
                    let body = fs::read_to_string(path).unwrap_or_default();
                    dump.push_str(&format!(
                        "\n--- {} ---\n{body}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ));
                }
                panic!("cli child did not exit in {timeout:?}{dump}");
            }
            None => thread::sleep(Duration::from_millis(50)),
        }
    }
}

fn split_n_panes(api_socket: &Path, root: &str, count: usize) -> Vec<String> {
    let mut ids = vec![root.to_string()];
    let mut current = root.to_string();
    while ids.len() < count {
        let split = request(
            api_socket,
            serde_json::json!({
                "id": format!("drill:split:{}", ids.len()),
                "method": "pane.split",
                "params": {
                    "target_pane_id": current,
                    "direction": if ids.len() % 2 == 0 { "right" } else { "down" },
                    "focus": true
                }
            }),
        );
        assert_ok(split.clone());
        current = split["result"]["pane"]["pane_id"]
            .as_str()
            .unwrap()
            .to_string();
        ids.push(current.clone());
    }
    ids
}

#[test]
fn live_restart_keeps_lane_tree() {
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let bin = base.join("bin");
    let session = format!("restart-core-{}", std::process::id());
    let api_socket = config_home.join(format!("herdr-dev/sessions/{session}/herdr.sock"));
    let client_socket = config_home.join(format!("herdr-dev/sessions/{session}/herdr-client.sock"));
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(&base).unwrap();

    let claude_started = base.join("claude-started");
    let cursor_started = base.join("cursor-started");
    let cursor_argv = base.join("cursor-argv");
    let pi_started = base.join("pi-started");
    let pi_session = base.join("lane-a.jsonl");
    write_executable(
        &bin.join("claude"),
        &format!(
            "#!/bin/sh\nexport HERDR_AGENT=claude\necho started\necho started > {}\n/bin/sleep 600\n:\n",
            claude_started.display()
        ),
    );
    write_executable(
        &bin.join("cursor-agent"),
        &format!(
            "#!/bin/sh\nexport HERDR_AGENT=cursor\nprintf '%s\\n' \"$0\" \"$@\" > {}\necho started > {}\n/bin/sleep 600\n:\n",
            cursor_argv.display(),
            cursor_started.display()
        ),
    );
    write_executable(
        &bin.join("pi"),
        &format!(
            "#!/bin/sh\nexport HERDR_AGENT=pi\necho started > {}\n/bin/sleep 600\n:\n",
            pi_started.display()
        ),
    );
    write_executable(
        &bin.join("codex"),
        "#!/bin/sh\nexport HERDR_AGENT=codex\n/bin/sleep 600\n:\n",
    );
    let path = format!("{}:/bin:/usr/bin:/usr/local/bin", bin.display());

    let spawned = spawn_named_session_server_with(
        &config_home,
        &runtime_dir,
        &session,
        DRILL_SESSION_CONFIG,
        &[("PATH", path.as_str())],
    );
    wait_for_socket(&api_socket, Duration::from_secs(15));
    wait_for_socket(&client_socket, Duration::from_secs(10));
    register_runtime_dir(&runtime_dir);
    let old_server_pid = spawned.child.process_id().expect("old server pid") as i32;

    let created = request(
        &api_socket,
        serde_json::json!({
            "id": "drill:workspace:create",
            "method": "workspace.create",
            "params": {"cwd": base.to_string_lossy(), "focus": true, "env": {"PATH": path}}
        }),
    );
    assert_ok(created.clone());
    let root = created["result"]["root_pane"]["pane_id"]
        .as_str()
        .unwrap()
        .to_string();
    let workspace_id = created["result"]["workspace"]["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();
    let panes = split_n_panes(&api_socket, &root, 16);
    assert_eq!(panes.len(), 16);
    let p1 = &panes[0];
    let p2 = &panes[1];
    let p3 = &panes[2];
    let p16 = &panes[15];
    let extra = panes[4].clone();
    let ghost = &panes[3];

    wait_shell_ready(&api_socket, p1, &base.join("p1-ready"));
    pane_send_with_path(
        &api_socket,
        p1,
        &path,
        &bin.join("claude").display().to_string(),
    );
    wait_marker(&claude_started);
    report_unmanaged_agent(&api_socket, p1, "claude", Some("hcoord-session"));
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "drill:p1:rename",
            "method": "agent.rename",
            "params": {"target": p1, "name": "hcoord"}
        }),
    ));

    wait_shell_ready(&api_socket, p2, &base.join("p2-ready"));
    pane_send(&api_socket, p2, "sleep 3600 &");
    thread::sleep(Duration::from_millis(200));

    wait_shell_ready(&api_socket, p3, &base.join("p3-ready"));
    pane_send(&api_socket, p3, &format!("export PATH={path}"));
    let started = request(
        &api_socket,
        serde_json::json!({
            "id": "drill:p3:start",
            "method": "agent.start",
            "params": {
                "name": "lane-a",
                "kind": "pi",
                "pane_id": p3,
                "args": ["--force", "--model", "m"],
                "timeout_ms": 15000
            }
        }),
    );
    assert_ok(started);
    wait_marker(&pi_started);
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "drill:p3:report",
            "method": "pane.report_agent",
            "params": {
                "pane_id": p3,
                "source": "herdr:pi",
                "agent": "pi",
                "state": "idle",
                "seq": 1,
                "agent_session_path": pi_session.to_string_lossy()
            }
        }),
    ));
    thread::sleep(Duration::from_secs(4));
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "drill:p3:tokens",
            "method": "pane.report_metadata",
            "params": {
                "pane_id": p3,
                "source": "drill",
                "tokens": {"lane": "lane-a", "done": "1", "parent": p1}
            }
        }),
    ));
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "drill:ws:tokens",
            "method": "workspace.report_metadata",
            "params": {
                "workspace_id": workspace_id,
                "source": "drill",
                "tokens": {"round": "r1"}
            }
        }),
    ));

    wait_shell_ready(&api_socket, ghost, &base.join("ghost-ready"));
    pane_send_with_path(
        &api_socket,
        ghost,
        &path,
        &bin.join("claude").display().to_string(),
    );
    wait_for_output(&api_socket, ghost, "started");
    report_unmanaged_agent(&api_socket, ghost, "claude", Some("ghost-session"));
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "drill:ghost:rename",
            "method": "agent.rename",
            "params": {"target": ghost, "name": "ghost"}
        }),
    ));

    for extra in &panes[4..15] {
        pane_send(&api_socket, extra, "sleep 600");
    }

    // Row 8: bad --exec must not move the server pid or disconnect a TUI.
    let endpoint_generation = support::CURRENT_ENDPOINT_PROTOCOL_GENERATION;
    let mut client_stream = UnixStream::connect(&client_socket).unwrap();
    let (server_generation, error) =
        client_shell_handshake(&mut client_stream, endpoint_generation, 54, 23).unwrap();
    assert_eq!(server_generation, endpoint_generation);
    assert!(error.is_none(), "client shell handshake failed: {error:?}");
    assert!(
        wait_for_message_variant(
            &mut client_stream,
            Duration::from_secs(5),
            SERVER_MESSAGE_ENDPOINT_CONTROL,
        )
        .unwrap(),
        "client shell should receive a snapshot before the refused restart"
    );
    let bad_exec = drill_command(&config_home, &runtime_dir, &session, &path)
        .args(["server", "restart", "--exec", "/nonexistent/herdr"])
        .output()
        .unwrap();
    let bad_text = format!(
        "{}{}",
        String::from_utf8_lossy(&bad_exec.stdout),
        String::from_utf8_lossy(&bad_exec.stderr)
    );
    assert!(
        bad_text.contains("invalid_exec"),
        "row 8 should return invalid_exec: {bad_text}"
    );
    assert_eq!(
        spawned
            .child
            .process_id()
            .expect("server pid after bad exec") as i32,
        old_server_pid
    );
    assert!(
        !wait_for_message_variant(
            &mut client_stream,
            Duration::from_millis(400),
            SERVER_MESSAGE_SERVER_SHUTDOWN,
        )
        .unwrap(),
        "row 8 must not disconnect the TUI"
    );
    drop(client_stream);

    // Row 9: pending agent start refuses restart without --force.
    wait_shell_ready(&api_socket, p16, &base.join("p16-ready"));
    pane_send(&api_socket, p16, &format!("export PATH={path}"));
    let busy_pane = p16.clone();
    let busy_socket = api_socket.clone();
    let busy = thread::spawn(move || {
        request(
            &busy_socket,
            serde_json::json!({
                "id": "drill:busy-start",
                "method": "agent.start",
                "params": {
                    "name": "busy-lane",
                    "kind": "codex",
                    "pane_id": busy_pane,
                    "timeout_ms": 15000
                }
            }),
        )
    });
    thread::sleep(Duration::from_millis(250));
    let busy_restart = drill_command(&config_home, &runtime_dir, &session, &path)
        .args(["server", "restart"])
        .output()
        .unwrap();
    let busy_text = format!(
        "{}{}",
        String::from_utf8_lossy(&busy_restart.stdout),
        String::from_utf8_lossy(&busy_restart.stderr)
    );
    assert!(
        busy_text.contains("restart_busy"),
        "row 9 should return restart_busy: {busy_text}"
    );
    let _ = busy.join();
    thread::sleep(Duration::from_secs(4));
    wait_for_agent(&api_socket, p16);

    let agent_wait_out = fs::File::create(base.join("agent-wait.out")).unwrap();
    let agent_wait_err = fs::File::create(base.join("agent-wait.err")).unwrap();
    let mut agent_wait = drill_command(&config_home, &runtime_dir, &session, &path)
        .args([
            "agent",
            "wait",
            "lane-a",
            "--until",
            "blocked",
            "--timeout",
            "60000",
        ])
        .stdout(Stdio::from(agent_wait_out))
        .stderr(Stdio::from(agent_wait_err))
        .spawn()
        .unwrap();
    let pane_wait_out = fs::File::create(base.join("pane-wait.out")).unwrap();
    let pane_wait_err = fs::File::create(base.join("pane-wait.err")).unwrap();
    let mut pane_wait = drill_command(&config_home, &runtime_dir, &session, &path)
        .args([
            "pane",
            "wait-output",
            p2,
            "--match",
            "DRILL_PANE_WAIT_TOKEN",
            "--timeout",
            "60000",
        ])
        .stdout(Stdio::from(pane_wait_out))
        .stderr(Stdio::from(pane_wait_err))
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_millis(200));

    let ids_before = pane_list_ids(&api_socket);
    let mut pids_before = Vec::new();
    for pane_id in &panes {
        pids_before.extend(pane_pids(&api_socket, pane_id));
    }
    pids_before.sort_unstable();
    pids_before.dedup();
    pids_alive(&pids_before);

    let herdr = env!("CARGO_BIN_EXE_herdr");
    let restart_out = base.join("restart.out");
    pane_send(
        &api_socket,
        p2,
        &format!(
            "XDG_CONFIG_HOME='{config}' XDG_RUNTIME_DIR='{runtime}' HERDR_SESSION='{session}' env -u HERDR_SOCKET_PATH -u HERDR_CLIENT_SOCKET_PATH '{herdr}' server restart > '{out}' 2>&1; echo EXIT:$? >> '{out}'",
            config = config_home.display(),
            runtime = runtime_dir.display(),
            session = session,
            herdr = herdr,
            out = restart_out.display()
        ),
    );
    wait_for_file_contains(&restart_out, "EXIT:0", Duration::from_secs(30));
    wait_for_api(&api_socket, Duration::from_secs(15));

    // Rows 1-3, 7.
    pids_alive(&pids_before);
    let listed = request_until_ok(
        &api_socket,
        serde_json::json!({"id":"drill:pane:list","method":"pane.list","params":{}}),
        Duration::from_secs(20),
    );
    assert_ok(listed.clone());
    let mut ids_after: Vec<String> = listed["result"]["panes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|pane| pane["pane_id"].as_str().map(str::to_string))
        .collect();
    let mut ids_before_sorted = ids_before.clone();
    ids_before_sorted.sort();
    ids_after.sort();
    assert_eq!(ids_after, ids_before_sorted, "row 2 public ids");
    let names_response = request_until_ok(
        &api_socket,
        serde_json::json!({"id":"drill:agent:list","method":"agent.list","params":{}}),
        Duration::from_secs(20),
    );
    assert_ok(names_response.clone());
    let names: Vec<String> = names_response["result"]["agents"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|agent| agent["name"].as_str().map(str::to_string))
        .collect();
    assert!(
        names.contains(&"hcoord".to_string()),
        "row 3 missing hcoord: {names:?}"
    );
    assert!(
        names.contains(&"lane-a".to_string()),
        "row 3 missing lane-a: {names:?}"
    );

    let list_out = base.join("agent-list.out");
    pane_send(
        &api_socket,
        p2,
        &format!(
            "$HERDR_BIN_PATH agent list > {} 2>&1; echo EXIT:$? >> {}",
            list_out.display(),
            list_out.display()
        ),
    );
    wait_for_file_contains(&list_out, "EXIT:0", Duration::from_secs(10));

    wait_for_agent_kind(&api_socket, "lane-a", "pi");
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "drill:lane-a:session",
            "method": "pane.report_agent_session",
            "params": {
                "pane_id": p3,
                "source": "herdr:pi",
                "agent": "pi",
                "seq": 50,
                "agent_session_path": pi_session.to_string_lossy(),
                "session_start_source": "new"
            }
        }),
    ));
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "drill:lane-a:blocked",
            "method": "pane.report_agent",
            "params": {
                "pane_id": p3,
                "source": "herdr:pi",
                "agent": "pi",
                "state": "blocked",
                "seq": 51,
                "agent_session_path": pi_session.to_string_lossy()
            }
        }),
    ));
    let blocked = request(
        &api_socket,
        serde_json::json!({
            "id": "drill:lane-a:get",
            "method": "agent.get",
            "params": {"target": "lane-a"}
        }),
    );
    assert_eq!(
        blocked["result"]["agent"]["agent_status"].as_str(),
        Some("blocked"),
        "lane-a should be blocked after report: {blocked}"
    );
    pane_send(&api_socket, p2, "echo DRILL_PANE_WAIT_TOKEN");
    assert_eq!(
        wait_for_cli_exit_with_logs(
            &mut agent_wait,
            Duration::from_secs(20),
            &[
                base.join("agent-wait.out"),
                base.join("agent-wait.err"),
                base.join("restart.out")
            ]
        ),
        0
    );
    assert_eq!(
        wait_for_cli_exit_with_logs(
            &mut pane_wait,
            Duration::from_secs(20),
            &[base.join("pane-wait.out"), base.join("pane-wait.err")]
        ),
        0
    );

    // Row 12: a named fake with no resume plan comes back unnamed after cold restore.
    let _ = request(
        &api_socket,
        serde_json::json!({"id":"drill:stop","method":"server.stop","params":{}}),
    );
    drop(spawned);
    wait_for_socket_gone(&api_socket, Duration::from_secs(10));
    wait_for_socket_gone(&client_socket, Duration::from_secs(5));
    let restored = spawn_named_session_server_with(
        &config_home,
        &runtime_dir,
        &session,
        DRILL_SESSION_CONFIG,
        &[("PATH", path.as_str())],
    );
    wait_for_socket(&api_socket, Duration::from_secs(15));
    wait_for_api(&api_socket, Duration::from_secs(15));
    let cold_ids = wait_for_pane_ids(&api_socket, 16, Duration::from_secs(15));
    let ghost = if cold_ids.iter().any(|id| id == &extra) {
        extra
    } else {
        cold_ids[4].clone()
    };
    let cold_names = agent_names(&api_socket);
    assert!(
        !cold_names.iter().any(|name| name == "ghost"),
        "row 12 ghost name should be dropped on cold restore: {cold_names:?}"
    );
    let _ = fs::remove_file(&pi_started);
    wait_shell_ready(&api_socket, &ghost, &base.join("ghost-cold-ready"));
    pane_send(&api_socket, &ghost, &format!("export PATH={path}"));
    let ghost_start = request(
        &api_socket,
        serde_json::json!({
            "id": "drill:ghost:start",
            "method": "agent.start",
            "params": {
                "name": "ghost",
                "kind": "pi",
                "pane_id": ghost,
                "timeout_ms": 15000
            }
        }),
    );
    assert_ok(ghost_start);
    wait_marker(&pi_started);

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"drill:stop-final","method":"server.stop","params":{}}),
    );
    drop(restored);
    cleanup_test_base(&base);
}

#[test]
fn live_restart_keeps_lane_tree_lineage() {
    // Rows 4, 6, 10, 11, and 13: token persist, GONE notify,
    // managed_agent_args replay, cold name restore.
    let _lock = test_lock();
    let base = unique_test_dir();
    let config_home = base.join("config");
    let runtime_dir = base.join("runtime");
    let bin = base.join("bin");
    let session = format!("restart-core-lineage-{}", std::process::id());
    let api_socket = config_home.join(format!("herdr-dev/sessions/{session}/herdr.sock"));
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(&base).unwrap();
    let cursor_started = base.join("cursor-started");
    let cursor_argv = base.join("cursor-argv");
    let claude_started = base.join("claude-started");
    write_executable(
        &bin.join("claude"),
        &format!(
            "#!/bin/sh\nexport HERDR_AGENT=claude\necho started\necho started > {}\n/bin/sleep 600\n:\n",
            claude_started.display()
        ),
    );
    write_executable(
        &bin.join("cursor-agent"),
        &format!(
            "#!/bin/sh\nexport HERDR_AGENT=cursor\nprintf '%s\\n' \"$0\" \"$@\" > {}\necho started > {}\n/bin/sleep 600\n:\n",
            cursor_argv.display(),
            cursor_started.display()
        ),
    );
    let path = format!("{}:/bin:/usr/bin:/usr/local/bin", bin.display());
    let spawned = spawn_named_session_server_with(
        &config_home,
        &runtime_dir,
        &session,
        DRILL_SESSION_CONFIG,
        &[("PATH", path.as_str())],
    );
    wait_for_socket(&api_socket, Duration::from_secs(15));
    register_runtime_dir(&runtime_dir);
    let created = request(
        &api_socket,
        serde_json::json!({
            "id": "lineage:workspace:create",
            "method": "workspace.create",
            "params": {"cwd": base.to_string_lossy(), "focus": true, "env": {"PATH": path}}
        }),
    );
    assert_ok(created.clone());
    let root = created["result"]["root_pane"]["pane_id"]
        .as_str()
        .unwrap()
        .to_string();
    let workspace_id = created["result"]["workspace"]["workspace_id"]
        .as_str()
        .unwrap()
        .to_string();
    let panes = split_n_panes(&api_socket, &root, 16);
    let p1 = &panes[0];
    let p2 = &panes[1];
    let p3 = &panes[2];
    wait_shell_ready(&api_socket, p1, &base.join("p1-ready"));
    pane_send_with_path(
        &api_socket,
        p1,
        &path,
        &bin.join("claude").display().to_string(),
    );
    wait_marker(&claude_started);
    report_unmanaged_agent(&api_socket, p1, "claude", Some("hcoord-session"));
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "lineage:p1:rename",
            "method": "agent.rename",
            "params": {"target": p1, "name": "hcoord"}
        }),
    ));
    pane_send(&api_socket, p2, "sleep 3600 &");
    wait_shell_ready(&api_socket, p3, &base.join("p3-ready"));
    pane_send(&api_socket, p3, &format!("export PATH={path}"));
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "lineage:p3:start",
            "method": "agent.start",
            "params": {
                "name": "lane-a",
                "kind": "cursor",
                "pane_id": p3,
                "args": ["--force", "--model", "m"],
                "timeout_ms": 15000
            }
        }),
    ));
    wait_marker(&cursor_started);
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "lineage:p3:tokens",
            "method": "pane.report_metadata",
            "params": {
                "pane_id": p3,
                "source": "drill",
                "tokens": {"lane": "lane-a", "done": "1", "parent": p1}
            }
        }),
    ));
    assert_ok(request(
        &api_socket,
        serde_json::json!({
            "id": "lineage:ws:tokens",
            "method": "workspace.report_metadata",
            "params": {
                "workspace_id": workspace_id,
                "source": "drill",
                "tokens": {"round": "r1"}
            }
        }),
    ));
    let restart = drill_command(&config_home, &runtime_dir, &session, &path)
        .args(["server", "restart"])
        .output()
        .unwrap();
    assert!(restart.status.success(), "restart failed: {restart:?}");
    drop(spawned);
    wait_for_api(&api_socket, Duration::from_secs(15));

    let agents = agent_list(&api_socket);
    let lane = agents["result"]["agents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|agent| agent["name"].as_str() == Some("lane-a"))
        .expect("lane-a");
    // Row 4: live handoff keeps pane tokens and the workspace round token.
    assert_eq!(lane["tokens"]["parent"], *p1);
    assert_eq!(lane["tokens"]["lane"], "lane-a");
    assert_eq!(lane["tokens"]["done"], "1");
    let workspace = request(
        &api_socket,
        serde_json::json!({
            "id": "lineage:workspace:get",
            "method": "workspace.get",
            "params": {"workspace_id": workspace_id}
        }),
    );
    assert_eq!(workspace["result"]["workspace"]["tokens"]["round"], "r1");

    // Row 6: killing the child fake after restart notifies the parent.
    let child_pids = pane_pids(&api_socket, p3);
    for pid in child_pids {
        if pid > 1 {
            let _ = unsafe { libc::kill(pid, libc::SIGKILL) };
        }
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut saw_gone = false;
    while Instant::now() < deadline {
        let read = request(
            &api_socket,
            serde_json::json!({
                "id": "lineage:p1:read",
                "method": "pane.read",
                "params": {
                    "pane_id": p1,
                    "source": "recent",
                    "lines": 40,
                    "format": "text",
                    "strip_ansi": true
                }
            }),
        );
        let text = read["result"]["read"]["text"].as_str().unwrap_or_default();
        if text.contains("GONE") && text.contains("lane-a") {
            saw_gone = true;
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    assert!(saw_gone, "row 6 parent pane should show GONE lane-a");

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"lineage:stop","method":"server.stop","params":{}}),
    );
    thread::sleep(Duration::from_millis(300));
    let _cursor_argv_before = fs::read_to_string(&cursor_argv).unwrap_or_default();
    let _ = fs::remove_file(&cursor_argv);
    let restored = spawn_named_session_server_with(
        &config_home,
        &runtime_dir,
        &session,
        DRILL_SESSION_CONFIG,
        &[("PATH", path.as_str())],
    );
    wait_for_socket(&api_socket, Duration::from_secs(15));
    wait_for_api(&api_socket, Duration::from_secs(15));
    wait_marker(&cursor_started);
    let argv = wait_for_file_contains(&cursor_argv, "--resume", Duration::from_secs(10));
    // Row 10: cold resume replays --resume <id> --force --model m.
    assert!(argv.contains("--resume"), "{argv}");
    assert!(argv.contains("--force"), "{argv}");
    assert!(argv.contains("--model"), "{argv}");
    let cold_names = agent_names(&api_socket);
    // Row 11: hcoord has a typed plan, so the name returns on cold restore.
    assert!(
        cold_names.contains(&"hcoord".to_string()),
        "row 11 missing hcoord: {cold_names:?}"
    );
    let cold_agents = agent_list(&api_socket);
    let cold_lane = cold_agents["result"]["agents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|agent| {
            agent["name"].as_str() == Some("lane-a") || agent["pane_id"].as_str() == Some(p3)
        });
    if let Some(cold_lane) = cold_lane {
        // Row 13: parent and lane restore; done is TTL-like and must not.
        assert_eq!(cold_lane["tokens"]["parent"], *p1);
        assert_eq!(cold_lane["tokens"]["lane"], "lane-a");
        assert!(
            cold_lane["tokens"].get("done").is_none()
                || cold_lane["tokens"]["done"].as_str() != Some("1"),
            "row 13 must not restore done"
        );
    } else {
        panic!("row 13 missing restored lane-a pane");
    }

    let _ = request(
        &api_socket,
        serde_json::json!({"id":"lineage:stop-final","method":"server.stop","params":{}}),
    );
    drop(restored);
    cleanup_test_base(&base);
}
