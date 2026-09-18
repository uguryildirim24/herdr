use crate::api::schema::{EmptyParams, Method, Request, ServerLiveHandoffParams};

pub(super) fn run_server_command(args: &[String]) -> std::io::Result<Option<i32>> {
    let Some(subcommand) = args.first().map(|arg| arg.as_str()) else {
        return Ok(None);
    };

    match subcommand {
        "stop" => server_stop(&args[1..]).map(Some),
        "restart" => server_restart(&args[1..]).map(Some),
        "live-handoff" => server_live_handoff(&args[1..]).map(Some),
        "--handoff-import" => Ok(None),
        "reload-config" => server_reload_config(&args[1..]).map(Some),
        "agent-manifests" => server_agent_manifests(&args[1..]).map(Some),
        "update-agent-manifests" => server_update_agent_manifests(&args[1..]).map(Some),
        "reload-agent-manifests" => server_reload_agent_manifests(&args[1..]).map(Some),
        "help" | "--help" | "-h" => {
            print_server_help();
            Ok(Some(0))
        }
        _ => {
            print_server_help();
            Ok(Some(2))
        }
    }
}

fn server_stop(args: &[String]) -> std::io::Result<i32> {
    if !args.is_empty() {
        eprintln!("usage: herdr server stop");
        return Ok(2);
    }

    if super::target::is_remote() {
        return super::send_ok_request(Method::ServerStop(EmptyParams::default()));
    }

    match crate::session::stop_active_server() {
        Ok(()) => Ok(0),
        Err(err) => {
            eprintln!("{err}");
            Ok(1)
        }
    }
}

fn server_reload_config(args: &[String]) -> std::io::Result<i32> {
    if !args.is_empty() {
        eprintln!("usage: herdr server reload-config");
        return Ok(2);
    }

    super::print_response(&super::send_request(&Request {
        id: "cli:server:reload-config".into(),
        method: Method::ServerReloadConfig(EmptyParams::default()),
    })?)
}

fn server_agent_manifests(args: &[String]) -> std::io::Result<i32> {
    let json = match args {
        [] => false,
        [flag] if flag == "--json" => true,
        _ => {
            eprintln!("usage: herdr server agent-manifests [--json]");
            return Ok(2);
        }
    };

    let response = super::send_request(&Request {
        id: "cli:server:agent-manifests".into(),
        method: Method::ServerAgentManifests(EmptyParams::default()),
    })?;
    if json || response.get("error").is_some() {
        return super::print_response(&response);
    }

    print_agent_manifest_status(&response);
    Ok(0)
}

fn server_reload_agent_manifests(args: &[String]) -> std::io::Result<i32> {
    if !args.is_empty() {
        eprintln!("usage: herdr server reload-agent-manifests");
        return Ok(2);
    }

    super::print_response(&super::send_request(&Request {
        id: "cli:server:reload-agent-manifests".into(),
        method: Method::ServerReloadAgentManifests(EmptyParams::default()),
    })?)
}

fn server_update_agent_manifests(args: &[String]) -> std::io::Result<i32> {
    let json = match args {
        [] => false,
        [flag] if flag == "--json" => true,
        _ => {
            eprintln!("usage: herdr server update-agent-manifests [--json]");
            return Ok(2);
        }
    };

    let response = match update_agent_manifest_status(super::send_request, || {
        crate::detect::manifest_update::check_and_update().map(|_| ())
    })? {
        Ok(response) => response,
        Err(err) => {
            if json {
                return super::print_response(&agent_manifest_update_error_response(&err));
            }
            eprintln!("failed to update agent detection manifests: {err}");
            return Ok(1);
        }
    };
    if json || response.get("error").is_some() {
        return super::print_response(&response);
    }

    print_agent_manifest_status(&response);
    Ok(0)
}

fn update_agent_manifest_status(
    mut send_request: impl FnMut(&Request) -> std::io::Result<serde_json::Value>,
    update_manifests: impl FnOnce() -> Result<(), String>,
) -> std::io::Result<Result<serde_json::Value, String>> {
    if let Err(err) = update_manifests() {
        return Ok(Err(err));
    }

    let reload_response = send_request(&Request {
        id: "cli:server:reload-agent-manifests".into(),
        method: Method::ServerReloadAgentManifests(EmptyParams::default()),
    })?;
    if reload_response.get("error").is_some() {
        return Ok(Ok(reload_response));
    }

    send_request(&Request {
        id: "cli:server:agent-manifests".into(),
        method: Method::ServerAgentManifests(EmptyParams::default()),
    })
    .map(Ok)
}

fn agent_manifest_update_error_response(err: &str) -> serde_json::Value {
    serde_json::json!({
        "id": "cli:server:update-agent-manifests",
        "error": {
            "code": "agent_manifest_update_failed",
            "message": err,
        }
    })
}

fn print_agent_manifest_status(response: &serde_json::Value) {
    let result = &response["result"];
    let last_check = result["last_check_unix"]
        .as_u64()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "never".to_string());
    let last_result = result["last_result"].as_str().unwrap_or("not checked");
    println!("last check: {last_check}");
    println!("result: {last_result}");
    println!();

    let Some(manifests) = result["manifests"].as_array() else {
        return;
    };
    for manifest in manifests {
        let agent = manifest["agent"].as_str().unwrap_or("-");
        let source = manifest["source_kind"].as_str().unwrap_or("-");
        let active_version = manifest["active_version"].as_str().unwrap_or("-");
        let remote_version = manifest["cached_remote_version"].as_str().unwrap_or("-");
        let remote_result = manifest["remote_update_result"]
            .as_str()
            .unwrap_or("not checked");
        let local_override_shadowing_remote = manifest["local_override_shadowing_remote"]
            .as_bool()
            .unwrap_or(false);
        let marker = if local_override_shadowing_remote {
            "!"
        } else if manifest["remote_update_error"].as_str().is_some() {
            "x"
        } else {
            " "
        };
        println!(
            "{marker} {agent:<9} {source:<14} active {active_version:<14} remote {remote_version:<14} {remote_result}"
        );
        if let Some(error) = manifest["remote_update_error"].as_str() {
            println!("  {error}");
        } else if local_override_shadowing_remote {
            println!("  local override shadows cached remote rules");
        } else if let Some(warning) = manifest["warning"].as_str() {
            println!("  {warning}");
        }
    }
}

fn server_restart(args: &[String]) -> std::io::Result<i32> {
    let parsed = match parse_restart_args(args) {
        Ok(parsed) => parsed,
        Err(code) => return Ok(code),
    };

    if !parsed.force {
        if let Some(message) = config_preflight_error() {
            eprintln!("restart refused: {message} (pass --force to override)");
            return Ok(1);
        }
    }

    let import_exe = match parsed.exec {
        Some(exec) => match prepare_restart_exec(&exec) {
            Ok(path) => Some(path),
            Err(message) => {
                return super::print_response(&cli_json_error(
                    "cli:server:restart",
                    "invalid_exec",
                    message,
                ));
            }
        },
        None => {
            print_restart_version_line(std::env::current_exe().ok().as_deref());
            None
        }
    };

    let mut params = ServerLiveHandoffParams {
        force: parsed.force,
        ..ServerLiveHandoffParams::default()
    };
    if let Some(import_exe) = import_exe {
        params.import_exe = Some(import_exe.to_string_lossy().into_owned());
    }

    let response = super::send_request_unchecked(&Request {
        id: "cli:server:restart".into(),
        method: Method::ServerLiveHandoff(params),
    })?;
    if response.get("error").is_some() {
        return super::print_response(&response);
    }

    eprintln!(
        "server restart complete; server log: {}",
        crate::session::data_dir()
            .join("herdr-server.log")
            .display()
    );
    Ok(0)
}

#[derive(Debug, PartialEq, Eq)]
struct RestartArgs {
    exec: Option<String>,
    force: bool,
}

fn parse_restart_args(args: &[String]) -> Result<RestartArgs, i32> {
    let mut exec = None;
    let mut force = false;
    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        match arg.as_str() {
            "help" | "--help" | "-h" => {
                eprintln!("usage: herdr server restart [--exec <path>] [--force]");
                return Err(0);
            }
            "--force" => {
                force = true;
                idx += 1;
                continue;
            }
            _ => {}
        }
        let (flag, value) = if let Some((flag, value)) = arg.split_once('=') {
            (flag, Some(value.to_string()))
        } else {
            let value = args.get(idx + 1).cloned();
            idx += 1;
            (arg.as_str(), value)
        };
        let Some(value) = value else {
            eprintln!("usage: herdr server restart [--exec <path>] [--force]");
            return Err(2);
        };
        match flag {
            "--exec" => exec = Some(value),
            _ => {
                eprintln!("usage: herdr server restart [--exec <path>] [--force]");
                return Err(2);
            }
        }
        idx += 1;
    }
    Ok(RestartArgs { exec, force })
}

fn config_preflight_error() -> Option<String> {
    let errors = crate::config::Config::load()
        .diagnostics
        .into_iter()
        .filter(|diagnostic| {
            diagnostic.starts_with("config read error")
                || diagnostic.starts_with("config parse error")
        })
        .collect::<Vec<_>>();
    if errors.is_empty() {
        None
    } else {
        Some(errors.join("; "))
    }
}

fn prepare_restart_exec(path: &str) -> Result<std::path::PathBuf, String> {
    let canonical = validate_restart_exec(path)?;
    if let Some(path_herdr) = path_herdr() {
        let path_canonical = path_herdr.canonicalize().unwrap_or(path_herdr.clone());
        if path_canonical != canonical {
            eprintln!(
                "warning: --exec {} is not the herdr on PATH ({})",
                canonical.display(),
                path_herdr.display()
            );
        }
    }
    let new_version = herdr_version_output(&canonical)?;
    eprintln!(
        "herdr {} → herdr {new_version}",
        crate::build_info::version()
    );
    Ok(canonical)
}

fn print_restart_version_line(exe: Option<&std::path::Path>) {
    let Some(exe) = exe else {
        return;
    };
    if let Ok(new_version) = herdr_version_output(exe) {
        eprintln!(
            "herdr {} → herdr {new_version}",
            crate::build_info::version()
        );
    }
}

fn validate_restart_exec(path: &str) -> Result<std::path::PathBuf, String> {
    let raw = std::path::PathBuf::from(path);
    if !raw.is_absolute() {
        return Err(format!("--exec must be an absolute path: {path}"));
    }
    let canonical = raw
        .canonicalize()
        .map_err(|err| format!("--exec is not a usable file: {path}: {err}"))?;
    if !canonical.is_absolute() {
        return Err(format!(
            "--exec must resolve to an absolute path: {}",
            canonical.display()
        ));
    }
    let metadata = canonical.metadata().map_err(|err| {
        format!(
            "--exec is not a usable file: {}: {err}",
            canonical.display()
        )
    })?;
    if !metadata.is_file() {
        return Err(format!(
            "--exec is not a regular file: {}",
            canonical.display()
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(format!("--exec is not executable: {}", canonical.display()));
        }
    }
    Ok(canonical)
}

fn path_herdr() -> Option<std::path::PathBuf> {
    #[cfg(windows)]
    const HERDR_NAME: &str = "herdr.exe";
    #[cfg(not(windows))]
    const HERDR_NAME: &str = "herdr";
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(HERDR_NAME))
        .find(|candidate| candidate.is_file())
}

fn herdr_version_output(exe: &std::path::Path) -> Result<String, String> {
    let mut child = std::process::Command::new(exe)
        .arg("--version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|err| format!("failed to run {} --version: {err}", exe.display()))?;
    let stdout = child.stdout.take();
    let started = std::time::Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= std::time::Duration::from_secs(5) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("{} --version timed out", exe.display()));
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(20)),
            Err(err) => {
                return Err(format!(
                    "failed to wait for {} --version: {err}",
                    exe.display()
                ));
            }
        }
    };
    let mut buf = String::new();
    if let Some(mut out) = stdout {
        use std::io::Read;
        let _ = out.read_to_string(&mut buf);
    }
    if !status.success() {
        return Err(format!(
            "{} --version exited {}",
            exe.display(),
            status.code().unwrap_or(1)
        ));
    }
    let first = buf.lines().next().unwrap_or("").trim();
    let Some(version) = first.strip_prefix("herdr ") else {
        return Err(format!(
            "--exec --version must start with 'herdr ': {first}"
        ));
    };
    if version.is_empty() {
        return Err(format!(
            "--exec --version must start with 'herdr ': {first}"
        ));
    }
    Ok(version.to_string())
}

fn cli_json_error(id: &str, code: &str, message: impl Into<String>) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "error": {
            "code": code,
            "message": message.into(),
        }
    })
}

fn server_live_handoff(args: &[String]) -> std::io::Result<i32> {
    let Some(params) = parse_live_handoff_params(args) else {
        eprintln!(
            "usage: herdr server live-handoff [--import-exe <path>] [--expected-protocol <n>] [--expected-version <version>]"
        );
        return Ok(2);
    };

    // Live handoff is itself a protocol-mismatch recovery path, so it must
    // reach the running server without the normal CLI compatibility guard.
    let response = super::send_request_unchecked(&Request {
        id: "cli:server:live-handoff".into(),
        method: Method::ServerLiveHandoff(params),
    })?;
    if response.get("error").is_some() {
        let rendered = serde_json::to_string(&response).unwrap_or_else(|err| {
            format!(
                "{{\"error\":{{\"code\":\"render_failed\",\"message\":\"failed to render error response: {err}\"}}}}"
            )
        });
        eprintln!("{rendered}");
        return Ok(1);
    }

    eprintln!(
        "live handoff complete; server log: {}",
        crate::session::data_dir()
            .join("herdr-server.log")
            .display()
    );
    Ok(0)
}

fn parse_live_handoff_params(args: &[String]) -> Option<ServerLiveHandoffParams> {
    let mut params = ServerLiveHandoffParams::default();
    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        let (flag, value) = if let Some((flag, value)) = arg.split_once('=') {
            (flag, Some(value.to_string()))
        } else {
            let value = args.get(idx + 1).cloned();
            idx += 1;
            (arg.as_str(), value)
        };
        let value = value?;
        match flag {
            "--import-exe" => params.import_exe = Some(value),
            "--expected-protocol" => {
                params.expected_protocol = Some(value.parse().ok()?);
            }
            "--expected-version" => params.expected_version = Some(value),
            _ => return None,
        }
        idx += 1;
    }
    Some(params)
}

fn print_server_help() {
    eprintln!("herdr server commands:");
    eprintln!("  herdr server                run as headless server");
    eprintln!("  herdr server stop           stop the running server via the API socket");
    eprintln!(
        "  herdr server restart        replace the running local server and keep pane processes"
    );
    eprintln!("  herdr server reload-config  reload config.toml in the running server");
    eprintln!("  herdr server agent-manifests [--json]  show agent detection manifest status");
    eprintln!("  herdr server update-agent-manifests [--json]  fetch and reload agent detection manifests");
    eprintln!("  herdr server reload-agent-manifests  reload agent detection manifests in the running server");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_agent_manifest_status_fetches_reloads_then_reads_status() {
        let mut methods = Vec::new();
        let response = update_agent_manifest_status(
            |request| {
                methods.push(request.method.clone());
                match &request.method {
                    Method::ServerReloadAgentManifests(_) => Ok(serde_json::json!({
                        "id": request.id,
                        "result": { "type": "agent_manifest_reload", "manifests": [] }
                    })),
                    Method::ServerAgentManifests(_) => Ok(serde_json::json!({
                        "id": request.id,
                        "result": {
                            "type": "agent_manifest_status",
                            "last_result": "checked",
                            "manifests": []
                        }
                    })),
                    _ => panic!("unexpected request"),
                }
            },
            || Ok(()),
        )
        .unwrap()
        .unwrap();

        assert_eq!(response["result"]["type"], "agent_manifest_status");
        assert_eq!(
            methods,
            vec![
                Method::ServerReloadAgentManifests(EmptyParams::default()),
                Method::ServerAgentManifests(EmptyParams::default())
            ]
        );
    }

    #[test]
    fn update_agent_manifest_status_skips_server_when_fetch_fails() {
        let response = update_agent_manifest_status(
            |_request| panic!("server should not be called after fetch failure"),
            || Err("network unavailable".to_string()),
        )
        .unwrap();

        assert_eq!(response, Err("network unavailable".to_string()));
        assert_eq!(
            agent_manifest_update_error_response("network unavailable")["error"]["code"],
            "agent_manifest_update_failed"
        );
    }

    #[test]
    fn update_agent_manifest_status_stops_after_reload_error() {
        let mut methods = Vec::new();
        let response = update_agent_manifest_status(
            |request| {
                methods.push(request.method.clone());
                Ok(serde_json::json!({
                    "id": request.id,
                    "error": {
                        "code": "reload_failed",
                        "message": "reload failed"
                    }
                }))
            },
            || Ok(()),
        )
        .unwrap()
        .unwrap();

        assert_eq!(response["error"]["code"], "reload_failed");
        assert_eq!(
            methods,
            vec![Method::ServerReloadAgentManifests(EmptyParams::default())]
        );
    }

    #[test]
    fn live_handoff_params_parse_remote_update_fields() {
        let args = vec![
            "--import-exe".to_string(),
            "/home/me/.local/bin/herdr".to_string(),
            "--expected-protocol=9".to_string(),
            "--expected-version".to_string(),
            "0.6.2".to_string(),
        ];

        let params = parse_live_handoff_params(&args).expect("params");

        assert_eq!(
            params.import_exe.as_deref(),
            Some("/home/me/.local/bin/herdr")
        );
        assert_eq!(params.expected_protocol, Some(9));
        assert_eq!(params.expected_version.as_deref(), Some("0.6.2"));
        assert!(!params.force);
    }

    #[test]
    fn restart_args_parse_exec_and_force() {
        let args = vec![
            "--exec".to_string(),
            "/tmp/herdr".to_string(),
            "--force".to_string(),
        ];
        let parsed = parse_restart_args(&args).expect("params");
        assert_eq!(parsed.exec.as_deref(), Some("/tmp/herdr"));
        assert!(parsed.force);
    }

    #[test]
    fn restart_args_parse_equals_exec_without_expected_fields() {
        let parsed = parse_restart_args(&["--exec=/opt/herdr".to_string()]).expect("params");
        assert_eq!(parsed.exec.as_deref(), Some("/opt/herdr"));
        assert!(!parsed.force);
    }

    #[test]
    fn restart_args_reject_unknown_flags() {
        assert_eq!(
            parse_restart_args(&["--expected-protocol".to_string(), "22".to_string()]),
            Err(2)
        );
    }

    #[test]
    fn validate_restart_exec_rejects_relative_and_missing_paths() {
        assert!(validate_restart_exec("relative/herdr")
            .unwrap_err()
            .contains("absolute path"));
        assert!(validate_restart_exec("/nonexistent/herdr-missing")
            .unwrap_err()
            .contains("not a usable file"));
    }

    #[test]
    fn cli_json_error_uses_invalid_exec_code() {
        assert_eq!(
            cli_json_error("cli:server:restart", "invalid_exec", "missing")["error"]["code"],
            "invalid_exec"
        );
    }
}
