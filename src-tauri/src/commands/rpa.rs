use serde::Serialize;
use serde_json::Value;
use std::env;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::process::Command;
use tokio::time::timeout;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct LocalRpaScriptExecutionResult {
    pub value: String,
}

#[derive(Debug, serde::Deserialize)]
struct LocalRpaScriptEnvelope {
    ok: bool,
    value: Option<String>,
    error: Option<String>,
}

#[tauri::command]
pub async fn execute_local_rpa_script(
    script: String,
    variables: Value,
) -> std::result::Result<LocalRpaScriptExecutionResult, String> {
    ensure_node_runtime().await?;

    let script_path = build_temp_script_path();
    let script_source = build_runner_script(&script, &variables)?;

    fs::write(&script_path, script_source)
        .await
        .map_err(|_| "LOCAL_SCRIPT_PREPARE_FAILED".to_string())?;

    let mut command = Command::new("node");
    command.arg(&script_path);
    apply_hidden_window_flags(&mut command);

    let execution = timeout(Duration::from_secs(60), command.output()).await;

    let cleanup_result = fs::remove_file(&script_path).await;
    if cleanup_result.is_err() {
        let _ = cleanup_result;
    }

    let output = match execution {
        Ok(Ok(output)) => output,
        Ok(Err(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err("NODE_RUNTIME_NOT_FOUND".to_string());
        }
        Ok(Err(_)) => return Err("LOCAL_SCRIPT_EXECUTION_FAILED".to_string()),
        Err(_) => return Err("LOCAL_SCRIPT_TIMEOUT".to_string()),
    };

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if let Ok(envelope) = serde_json::from_str::<LocalRpaScriptEnvelope>(&stdout) {
        if envelope.ok {
            return Ok(LocalRpaScriptExecutionResult {
                value: envelope.value.unwrap_or_default(),
            });
        }

        return Err(envelope
            .error
            .filter(|message| !message.trim().is_empty())
            .unwrap_or_else(|| "LOCAL_SCRIPT_EXECUTION_FAILED".to_string()));
    }

    if !stderr.is_empty() {
        return Err(stderr);
    }

    if !stdout.is_empty() {
        return Err(stdout);
    }

    if !output.status.success() {
        return Err("LOCAL_SCRIPT_EXECUTION_FAILED".to_string());
    }

    Ok(LocalRpaScriptExecutionResult {
        value: String::new(),
    })
}

async fn ensure_node_runtime() -> std::result::Result<(), String> {
    let mut command = Command::new("node");
    command.arg("--version");
    apply_hidden_window_flags(&mut command);

    match command.output().await {
        Ok(output) if output.status.success() => Ok(()),
        Ok(_) => Err("NODE_RUNTIME_NOT_FOUND".to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Err("NODE_RUNTIME_NOT_FOUND".to_string())
        }
        Err(_) => Err("NODE_RUNTIME_NOT_FOUND".to_string()),
    }
}

fn build_temp_script_path() -> PathBuf {
    env::temp_dir().join(format!("simprint-rpa-script-{}.cjs", Uuid::new_v4()))
}

fn apply_hidden_window_flags(command: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
}

fn build_runner_script(script: &str, variables: &Value) -> std::result::Result<String, String> {
    let script_body =
        serde_json::to_string(script).map_err(|_| "LOCAL_SCRIPT_PREPARE_FAILED".to_string())?;
    let variables_json =
        serde_json::to_string(variables).map_err(|_| "LOCAL_SCRIPT_PREPARE_FAILED".to_string())?;

    Ok(format!(
        r#"const vars = {variables_json};
const AsyncFunction = Object.getPrototypeOf(async function () {{}}).constructor;

(async () => {{
  try {{
    const runner = new AsyncFunction('vars', 'require', 'fetch', {script_body});
    const localFetch =
      typeof globalThis.fetch === 'function' ? globalThis.fetch.bind(globalThis) : undefined;
    const result = await runner(vars, require, localFetch);

    let value = '';
    if (typeof result === 'string') {{
      value = result;
    }} else if (result == null) {{
      value = '';
    }} else if (typeof result === 'object') {{
      value = JSON.stringify(result);
    }} else {{
      value = String(result);
    }}

    process.stdout.write(JSON.stringify({{ ok: true, value }}));
  }} catch (error) {{
    process.stdout.write(
      JSON.stringify({{
        ok: false,
        error: error instanceof Error ? error.message : String(error ?? 'LOCAL_SCRIPT_EXECUTION_FAILED'),
      }})
    );
    process.exitCode = 1;
  }}
}})();
"#
    ))
}
