use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{Implementation, ServerInfo};
use rmcp::schemars;
use rmcp::{tool, tool_handler, tool_router, ServerHandler, ServiceExt};
use serde::Deserialize;
use std::sync::Arc;

use crate::shell::{ShellManager, TerminalConfig};

#[derive(Clone)]
pub struct WtaMcpServer {
    shell_mgr: Arc<ShellManager>,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RunCommandParams {
    /// The command to run
    pub command: String,
    /// Arguments to pass to the command
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory (optional)
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateTerminalParams {
    /// The command to run in the terminal
    pub command: String,
    /// Arguments to pass to the command
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory (optional)
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TerminalIdParams {
    /// The terminal ID
    pub terminal_id: String,
}

// ── WT protocol tool params ─────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WindowIdParams {
    /// The window ID (from list_windows)
    pub window_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TabIdParams {
    /// The tab ID (from list_tabs)
    pub tab_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PaneIdParams {
    /// The pane ID
    pub pane_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReadPaneOutputParams {
    /// The pane ID to read output from
    pub pane_id: String,
    /// Maximum number of lines to return (optional)
    pub max_lines: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendInputParams {
    /// The pane ID to send input to
    pub pane_id: String,
    /// The text/keystrokes to send
    pub input: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateTabParams {
    /// Command to run in the new tab (optional, defaults to default shell)
    pub commandline: Option<String>,
    /// Working directory (optional)
    pub cwd: Option<String>,
    /// Tab title (optional)
    pub title: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SplitPaneParams {
    /// The pane ID to split
    pub pane_id: String,
    /// Command to run in the new pane (optional)
    pub commandline: Option<String>,
    /// Split direction: "horizontal" or "vertical" (optional, default: "auto")
    pub direction: Option<String>,
    /// Size fraction 0.0-1.0 (optional, default: 0.5)
    pub size: Option<f64>,
}

impl WtaMcpServer {
    pub fn new(shell_mgr: Arc<ShellManager>) -> Self {
        Self {
            shell_mgr,
            tool_router: Self::tool_router(),
        }
    }

    fn json_pretty(val: &serde_json::Value) -> String {
        serde_json::to_string_pretty(val).unwrap_or_else(|_| val.to_string())
    }
}

#[tool_router]
impl WtaMcpServer {
    // ── Local shell tools ───────────────────────────────────────────

    /// Run a command to completion and return its output.
    #[tool(description = "Run a command and return its output")]
    async fn run_command(&self, Parameters(params): Parameters<RunCommandParams>) -> String {
        let config = TerminalConfig {
            command: params.command,
            args: params.args,
            cwd: params.cwd,
            env: vec![],
        };

        let id = match self.shell_mgr.create_terminal(config).await {
            Ok(id) => id,
            Err(e) => return format!("Error creating terminal: {}", e),
        };

        let exit_code = match self.shell_mgr.wait_for_exit(&id).await {
            Ok(code) => code,
            Err(e) => return format!("Error waiting for exit: {}", e),
        };

        let output = match self.shell_mgr.get_output(&id).await {
            Ok(o) => o.data,
            Err(e) => return format!("Error getting output: {}", e),
        };

        let _ = self.shell_mgr.release(&id).await;

        format!("Exit code: {}\n\n{}", exit_code, output)
    }

    /// Create a persistent terminal session and return its ID.
    #[tool(description = "Create a persistent terminal session")]
    async fn create_terminal(
        &self,
        Parameters(params): Parameters<CreateTerminalParams>,
    ) -> String {
        let config = TerminalConfig {
            command: params.command,
            args: params.args,
            cwd: params.cwd,
            env: vec![],
        };

        match self.shell_mgr.create_terminal(config).await {
            Ok(id) => format!("Terminal created: {}", id),
            Err(e) => format!("Error creating terminal: {}", e),
        }
    }

    /// Get buffered output from a terminal session.
    #[tool(description = "Get output from a terminal session")]
    async fn get_terminal_output(
        &self,
        Parameters(params): Parameters<TerminalIdParams>,
    ) -> String {
        match self.shell_mgr.get_output(&params.terminal_id).await {
            Ok(output) => {
                let status = match output.exit_status {
                    Some(code) => format!(" (exited: {})", code),
                    None => " (running)".to_string(),
                };
                format!("[{}{}]\n{}", params.terminal_id, status, output.data)
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Wait for a terminal to exit and return the exit code.
    #[tool(description = "Wait for a terminal to exit")]
    async fn wait_for_terminal(
        &self,
        Parameters(params): Parameters<TerminalIdParams>,
    ) -> String {
        match self.shell_mgr.wait_for_exit(&params.terminal_id).await {
            Ok(code) => format!("Terminal {} exited with code {}", params.terminal_id, code),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Kill a terminal session.
    #[tool(description = "Kill a terminal session")]
    async fn kill_terminal(
        &self,
        Parameters(params): Parameters<TerminalIdParams>,
    ) -> String {
        match self.shell_mgr.kill(&params.terminal_id).await {
            Ok(()) => format!("Terminal {} killed", params.terminal_id),
            Err(e) => format!("Error: {}", e),
        }
    }

    // ── Windows Terminal state tools ────────────────────────────────

    /// List all Windows Terminal windows. Returns window IDs, titles, and tab counts.
    #[tool(description = "List all Windows Terminal windows with their IDs and titles")]
    async fn wt_list_windows(&self) -> String {
        match self.shell_mgr.wt_list_windows().await {
            Ok(val) => Self::json_pretty(&val),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// List all tabs in a Windows Terminal window.
    #[tool(description = "List all tabs in a Windows Terminal window. Use window_id from wt_list_windows.")]
    async fn wt_list_tabs(&self, Parameters(params): Parameters<WindowIdParams>) -> String {
        match self.shell_mgr.wt_list_tabs(&params.window_id).await {
            Ok(val) => Self::json_pretty(&val),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// List all panes in a tab.
    #[tool(description = "List all panes in a tab. Use tab_id from wt_list_tabs.")]
    async fn wt_list_panes(&self, Parameters(params): Parameters<TabIdParams>) -> String {
        match self.shell_mgr.wt_list_panes(&params.tab_id).await {
            Ok(val) => Self::json_pretty(&val),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Get info about the currently active/focused pane.
    #[tool(description = "Get the currently active pane's ID and info")]
    async fn wt_get_active_pane(&self) -> String {
        match self.shell_mgr.wt_get_active_pane().await {
            Ok(val) => Self::json_pretty(&val),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Read recent output from a terminal pane.
    #[tool(description = "Read recent output text from a terminal pane. Use pane_id from wt_list_panes or wt_get_active_pane.")]
    async fn wt_read_pane_output(
        &self,
        Parameters(params): Parameters<ReadPaneOutputParams>,
    ) -> String {
        match self
            .shell_mgr
            .wt_read_pane_output(&params.pane_id, params.max_lines)
            .await
        {
            Ok(val) => Self::json_pretty(&val),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Get the process status of a pane (running, exit code, etc.).
    #[tool(description = "Get the process status of a pane — whether it's running and its exit code")]
    async fn wt_get_process_status(
        &self,
        Parameters(params): Parameters<PaneIdParams>,
    ) -> String {
        match self
            .shell_mgr
            .wt_get_process_status(&params.pane_id)
            .await
        {
            Ok(val) => Self::json_pretty(&val),
            Err(e) => format!("Error: {}", e),
        }
    }

    // ── Windows Terminal control tools ──────────────────────────────

    /// Create a new tab in Windows Terminal.
    #[tool(description = "Create a new tab in Windows Terminal. Returns the new pane's ID.")]
    async fn wt_create_tab(&self, Parameters(params): Parameters<CreateTabParams>) -> String {
        match self
            .shell_mgr
            .wt_create_tab(
                params.commandline.as_deref(),
                params.cwd.as_deref(),
                params.title.as_deref(),
            )
            .await
        {
            Ok(val) => Self::json_pretty(&val),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Split an existing pane in Windows Terminal.
    #[tool(description = "Split a pane in Windows Terminal. Returns the new pane's ID.")]
    async fn wt_split_pane(&self, Parameters(params): Parameters<SplitPaneParams>) -> String {
        match self
            .shell_mgr
            .wt_split_pane(
                &params.pane_id,
                params.commandline.as_deref(),
                params.direction.as_deref(),
                params.size,
            )
            .await
        {
            Ok(val) => Self::json_pretty(&val),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Send text input to a terminal pane (keystrokes).
    #[tool(description = "Send text/keystrokes to a terminal pane. Use this to type commands into a pane.")]
    async fn wt_send_input(&self, Parameters(params): Parameters<SendInputParams>) -> String {
        match self
            .shell_mgr
            .wt_send_input(&params.pane_id, &params.input)
            .await
        {
            Ok(val) => Self::json_pretty(&val),
            Err(e) => format!("Error: {}", e),
        }
    }

    /// Close a terminal pane.
    #[tool(description = "Close a terminal pane")]
    async fn wt_close_pane(&self, Parameters(params): Parameters<PaneIdParams>) -> String {
        match self.shell_mgr.wt_close_pane(&params.pane_id).await {
            Ok(val) => Self::json_pretty(&val),
            Err(e) => format!("Error: {}", e),
        }
    }
}

#[tool_handler]
impl ServerHandler for WtaMcpServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.server_info = Implementation::from_build_env();
        info.instructions = Some(
            "WTA (Windows Terminal Agent) — provides shell tools and Windows Terminal integration. \
             Use the wt_* tools to inspect and control Windows Terminal windows, tabs, and panes. \
             Use wt_list_windows → wt_list_tabs → wt_list_panes to discover pane IDs, then \
             wt_read_pane_output to see what's on screen, wt_send_input to type commands, \
             and wt_create_tab / wt_split_pane to create new sessions."
                .into(),
        );
        info
    }
}

/// Run WTA as a headless MCP server over stdio.
pub async fn run_mcp_server(shell_mgr: Arc<ShellManager>) -> anyhow::Result<()> {
    let server = WtaMcpServer::new(shell_mgr);

    let service = server
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|e| anyhow::anyhow!("MCP server error: {:?}", e))?;

    service
        .waiting()
        .await
        .map_err(|e| anyhow::anyhow!("MCP server error: {:?}", e))?;

    Ok(())
}
