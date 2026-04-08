use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::PathBuf;

use clap::{ArgAction, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use commands::SlashCommand;
use runtime::PermissionMode;

const DEFAULT_MODEL: &str = "claude-opus-4-6";

pub type AllowedToolSet = BTreeSet<String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliAction {
    DumpManifests,
    BootstrapPlan,
    Agents {
        args: Option<String>,
    },
    Skills {
        args: Option<String>,
    },
    GenerateCompletions {
        shell: CompletionShellArg,
        output: Option<PathBuf>,
    },
    GenerateManpage {
        output: Option<PathBuf>,
    },
    PrintSystemPrompt {
        cwd: PathBuf,
        date: String,
    },
    Version,
    ResumeSession {
        session_path: PathBuf,
        commands: Vec<String>,
    },
    Prompt {
        prompt: String,
        model: String,
        output_format: CliOutputFormat,
        allowed_tools: Option<AllowedToolSet>,
        permission_mode: PermissionMode,
    },
    Pipe {
        model: String,
        output_format: CliOutputFormat,
        allowed_tools: Option<AllowedToolSet>,
        permission_mode: PermissionMode,
    },
    Login,
    Logout,
    Init,
    Serve {
        host: String,
        port: u16,
    },
    Repl {
        model: String,
        allowed_tools: Option<AllowedToolSet>,
        permission_mode: PermissionMode,
    },
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliOutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Parser, PartialEq, Eq)]
#[command(
    name = "claw",
    about = "Claw Code CLI",
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true
)]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    #[arg(short = 'h', long = "help", action = ArgAction::SetTrue)]
    help: bool,

    #[arg(short = 'V', long = "version", action = ArgAction::SetTrue)]
    version: bool,

    #[arg(long, default_value = DEFAULT_MODEL)]
    model: String,

    #[arg(long, value_enum, default_value_t = PermissionModeArg::DangerFullAccess)]
    permission_mode: PermissionModeArg,

    #[arg(long, value_enum, default_value_t = OutputFormatArg::Text)]
    output_format: OutputFormatArg,

    #[arg(long = "allowedTools", alias = "allowed-tools", action = ArgAction::Append)]
    allowed_tools: Vec<String>,

    #[arg(long = "dangerously-skip-permissions", action = ArgAction::SetTrue)]
    dangerously_skip_permissions: bool,

    #[arg(long = "print", action = ArgAction::SetTrue)]
    print: bool,

    #[arg(short = 'p', value_name = "PROMPT", num_args = 1.., allow_hyphen_values = true)]
    prompt: Option<Vec<String>>,

    #[arg(long = "pipe", action = ArgAction::SetTrue)]
    pipe: bool,

    #[arg(long = "resume", num_args = 1.., allow_hyphen_values = true)]
    resume: Option<Vec<String>>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
enum Command {
    DumpManifests,
    BootstrapPlan,
    Agents {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Skills {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Completions {
        #[arg(value_enum)]
        shell: CompletionShellArg,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    Manpage {
        #[arg(long)]
        output: Option<PathBuf>,
    },
    SystemPrompt {
        #[arg(long)]
        cwd: Option<PathBuf>,
        #[arg(long)]
        date: Option<String>,
    },
    Login,
    Logout,
    Init,
    Serve {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        #[arg(long, default_value_t = 4141)]
        port: u16,
    },
    Prompt {
        #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
        prompt: Vec<String>,
    },
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum PermissionModeArg {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum OutputFormatArg {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum CompletionShellArg {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

pub fn parse_cli_action<I, T, F>(args: I, normalize_allowed_tools: F) -> Result<CliAction, String>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
    F: Fn(&[String]) -> Result<Option<AllowedToolSet>, String>,
{
    let raw_args = args
        .into_iter()
        .map(Into::into)
        .map(|value: OsString| value.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    if raw_args.first().is_some_and(|arg| arg.starts_with('/')) {
        return parse_direct_slash_cli_action(&raw_args);
    }

    let cli = Cli::try_parse_from(
        std::iter::once(OsString::from("claw")).chain(raw_args.iter().cloned().map(OsString::from)),
    )
    .map_err(|error| error.to_string())?;

    if cli.help {
        return Ok(CliAction::Help);
    }
    if cli.version {
        return Ok(CliAction::Version);
    }
    if let Some(resume) = cli.resume {
        return parse_resume_args(&resume);
    }

    let model = resolve_model_alias(&cli.model).to_string();
    let permission_mode = if cli.dangerously_skip_permissions {
        PermissionMode::DangerFullAccess
    } else {
        cli.permission_mode.into()
    };
    let output_format = if cli.print {
        CliOutputFormat::Text
    } else {
        cli.output_format.into()
    };
    let allowed_tools = normalize_allowed_tools(&cli.allowed_tools)?;

    if let Some(prompt) = cli.prompt {
        return prompt_action(prompt, model, output_format, allowed_tools, permission_mode);
    }
    if cli.pipe {
        return Ok(CliAction::Pipe {
            model,
            output_format,
            allowed_tools,
            permission_mode,
        });
    }

    match cli.command {
        None => Ok(CliAction::Repl {
            model,
            allowed_tools,
            permission_mode,
        }),
        Some(Command::DumpManifests) => Ok(CliAction::DumpManifests),
        Some(Command::BootstrapPlan) => Ok(CliAction::BootstrapPlan),
        Some(Command::Agents { args }) => Ok(CliAction::Agents {
            args: join_optional_args(&args),
        }),
        Some(Command::Skills { args }) => Ok(CliAction::Skills {
            args: join_optional_args(&args),
        }),
        Some(Command::Completions { shell, output }) => {
            Ok(CliAction::GenerateCompletions { shell, output })
        }
        Some(Command::Manpage { output }) => Ok(CliAction::GenerateManpage { output }),
        Some(Command::SystemPrompt { cwd, date }) => Ok(CliAction::PrintSystemPrompt {
            cwd: cwd.unwrap_or_else(|| PathBuf::from(".")),
            date: date.unwrap_or_else(|| String::from("2026-03-31")),
        }),
        Some(Command::Login) => Ok(CliAction::Login),
        Some(Command::Logout) => Ok(CliAction::Logout),
        Some(Command::Init) => Ok(CliAction::Init),
        Some(Command::Serve { host, port }) => Ok(CliAction::Serve { host, port }),
        Some(Command::Prompt { prompt }) => {
            prompt_action(prompt, model, output_format, allowed_tools, permission_mode)
        }
        Some(Command::External(rest)) => {
            if rest.first().is_some_and(|first| first.starts_with('/')) {
                parse_direct_slash_cli_action(&rest)
            } else {
                prompt_action(rest, model, output_format, allowed_tools, permission_mode)
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn prompt_action(
    prompt_tokens: Vec<String>,
    model: String,
    output_format: CliOutputFormat,
    allowed_tools: Option<AllowedToolSet>,
    permission_mode: PermissionMode,
) -> Result<CliAction, String> {
    let prompt = prompt_tokens.join(" ");
    if prompt.trim().is_empty() {
        return Err("prompt subcommand requires a prompt string".to_string());
    }
    Ok(CliAction::Prompt {
        prompt,
        model,
        output_format,
        allowed_tools,
        permission_mode,
    })
}

fn join_optional_args(args: &[String]) -> Option<String> {
    (!args.is_empty()).then(|| args.join(" "))
}

fn parse_direct_slash_cli_action(rest: &[String]) -> Result<CliAction, String> {
    match SlashCommand::parse(&rest.join(" ")) {
        Some(SlashCommand::Help) => Ok(CliAction::Help),
        Some(SlashCommand::Agents { args }) => Ok(CliAction::Agents { args }),
        Some(SlashCommand::Skills { args }) => Ok(CliAction::Skills { args }),
        Some(other) => Err(format_direct_slash_command_error(
            &canonical_slash_name(&other),
            false,
        )),
        None => Err("slash command parsing failed".to_string()),
    }
}

fn parse_resume_args(args: &[String]) -> Result<CliAction, String> {
    let Some(session_path) = args.first().filter(|value| !value.trim().is_empty()) else {
        return Err("missing session path for --resume".to_string());
    };
    let commands = args.iter().skip(1).cloned().collect::<Vec<_>>();
    Ok(CliAction::ResumeSession {
        session_path: PathBuf::from(session_path),
        commands,
    })
}

fn format_direct_slash_command_error(command: &str, _is_unknown: bool) -> String {
    let mut lines = vec![
        format!("Direct slash command unavailable: /{command} requires the interactive REPL"),
        "supported direct slash commands: /help, /agents, /skills".to_string(),
    ];
    append_slash_command_suggestions(&mut lines, command);
    lines.join("\n")
}

fn append_slash_command_suggestions(lines: &mut Vec<String>, name: &str) {
    let suggestions = commands::suggest_slash_commands(name, 3);
    if suggestions.is_empty() {
        return;
    }
    lines.push("did you mean:".to_string());
    lines.extend(
        suggestions
            .into_iter()
            .map(|suggestion| format!("  {suggestion}")),
    );
}

fn canonical_slash_name(command: &SlashCommand) -> String {
    match command {
        SlashCommand::Help => "help".to_string(),
        SlashCommand::Status => "status".to_string(),
        SlashCommand::Context => "context".to_string(),
        SlashCommand::Compact => "compact".to_string(),
        SlashCommand::Branch { .. } => "branch".to_string(),
        SlashCommand::Bughunter { .. } => "bughunter".to_string(),
        SlashCommand::Worktree { .. } => "worktree".to_string(),
        SlashCommand::Commit => "commit".to_string(),
        SlashCommand::CommitPushPr { .. } => "commit-push-pr".to_string(),
        SlashCommand::Pr { .. } => "pr".to_string(),
        SlashCommand::Issue { .. } => "issue".to_string(),
        SlashCommand::Ultraplan { .. } => "ultraplan".to_string(),
        SlashCommand::Teleport { .. } => "teleport".to_string(),
        SlashCommand::DebugToolCall => "debug-tool-call".to_string(),
        SlashCommand::Model { .. } => "model".to_string(),
        SlashCommand::Permissions { .. } => "permissions".to_string(),
        SlashCommand::Clear { .. } => "clear".to_string(),
        SlashCommand::Cost => "cost".to_string(),
        SlashCommand::Resume { .. } => "resume".to_string(),
        SlashCommand::Config { .. } => "config".to_string(),
        SlashCommand::Memory => "memory".to_string(),
        SlashCommand::Init => "init".to_string(),
        SlashCommand::Diff => "diff".to_string(),
        SlashCommand::Version => "version".to_string(),
        SlashCommand::Export { .. } => "export".to_string(),
        SlashCommand::Summary => "summary".to_string(),
        SlashCommand::Stats => "stats".to_string(),
        SlashCommand::Share { .. } => "share".to_string(),
        SlashCommand::Rewind { .. } => "rewind".to_string(),
        SlashCommand::Tag { .. } => "tag".to_string(),
        SlashCommand::Session { .. } => "session".to_string(),
        SlashCommand::Plugins { .. } => "plugin".to_string(),
        SlashCommand::Hooks => "hooks".to_string(),
        SlashCommand::ReloadPlugins => "reload-plugins".to_string(),
        SlashCommand::Mcp { .. } => "mcp".to_string(),
        SlashCommand::Review { .. } => "review".to_string(),
        SlashCommand::SecurityReview { .. } => "security-review".to_string(),
        SlashCommand::Doctor => "doctor".to_string(),
        SlashCommand::Tasks { .. } => "tasks".to_string(),
        SlashCommand::Login => "login".to_string(),
        SlashCommand::Logout => "logout".to_string(),
        SlashCommand::Theme { .. } => "theme".to_string(),
        SlashCommand::SandboxToggle => "sandbox-toggle".to_string(),
        SlashCommand::Agents { .. } => "agents".to_string(),
        SlashCommand::Skills { .. } => "skills".to_string(),
        SlashCommand::AddDir { .. } => "add-dir".to_string(),
        SlashCommand::Voice { .. } => "voice".to_string(),
        SlashCommand::Stickers { .. } => "stickers".to_string(),
        SlashCommand::Unknown(name) => name.clone(),
    }
}

fn resolve_model_alias(model: &str) -> &str {
    match model {
        "opus" => "claude-opus-4-6",
        "sonnet" => "claude-sonnet-4-6",
        "haiku" => "claude-haiku-4-5-20251213",
        _ => model,
    }
}

impl From<PermissionModeArg> for PermissionMode {
    fn from(value: PermissionModeArg) -> Self {
        match value {
            PermissionModeArg::ReadOnly => Self::ReadOnly,
            PermissionModeArg::WorkspaceWrite => Self::WorkspaceWrite,
            PermissionModeArg::DangerFullAccess => Self::DangerFullAccess,
        }
    }
}

impl From<OutputFormatArg> for CliOutputFormat {
    fn from(value: OutputFormatArg) -> Self {
        match value {
            OutputFormatArg::Text => Self::Text,
            OutputFormatArg::Json => Self::Json,
        }
    }
}

impl From<CompletionShellArg> for Shell {
    fn from(value: CompletionShellArg) -> Self {
        match value {
            CompletionShellArg::Bash => Self::Bash,
            CompletionShellArg::Elvish => Self::Elvish,
            CompletionShellArg::Fish => Self::Fish,
            CompletionShellArg::PowerShell => Self::PowerShell,
            CompletionShellArg::Zsh => Self::Zsh,
        }
    }
}

#[must_use]
pub fn render_completion_script(shell: CompletionShellArg) -> String {
    let mut command = Cli::command();
    let mut buffer = Vec::new();
    let shell: Shell = shell.into();
    clap_complete::generate(shell, &mut command, "claw", &mut buffer);
    String::from_utf8(buffer).expect("completion script should be valid UTF-8")
}

#[must_use]
pub fn render_manpage() -> String {
    let command = Cli::command();
    let mut buffer = Vec::new();
    clap_mangen::Man::new(command)
        .render(&mut buffer)
        .expect("man page rendering should succeed");
    String::from_utf8(buffer).expect("man page should be valid UTF-8")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use runtime::PermissionMode;

    use super::{parse_cli_action, AllowedToolSet, CliAction, CliOutputFormat, CompletionShellArg};

    fn normalize_allowed_tools(values: &[String]) -> Result<Option<AllowedToolSet>, String> {
        if values.is_empty() {
            return Ok(None);
        }
        let mut allowed = BTreeSet::new();
        for value in values {
            for token in value
                .split(',')
                .map(str::trim)
                .filter(|token| !token.is_empty())
            {
                match token {
                    "read_file" | "write_file" | "glob_search" | "grep_search" => {
                        allowed.insert(token.to_string());
                    }
                    other => return Err(format!("unsupported tool in --allowedTools: {other}")),
                }
            }
        }
        Ok(Some(allowed))
    }

    #[test]
    fn defaults_to_repl_mode() {
        assert_eq!(
            parse_cli_action(Vec::<String>::new(), normalize_allowed_tools)
                .expect("args should parse"),
            CliAction::Repl {
                model: "claude-opus-4-6".to_string(),
                allowed_tools: None,
                permission_mode: PermissionMode::DangerFullAccess,
            }
        );
    }

    #[test]
    fn parses_prompt_subcommand_and_print_mode() {
        let args = vec![
            "--model".to_string(),
            "sonnet".to_string(),
            "--print".to_string(),
            "--allowedTools".to_string(),
            "read_file,write_file".to_string(),
            "prompt".to_string(),
            "hello".to_string(),
            "world".to_string(),
        ];

        assert_eq!(
            parse_cli_action(args, normalize_allowed_tools).expect("args should parse"),
            CliAction::Prompt {
                prompt: "hello world".to_string(),
                model: "claude-sonnet-4-6".to_string(),
                output_format: CliOutputFormat::Text,
                allowed_tools: Some(BTreeSet::from([
                    "read_file".to_string(),
                    "write_file".to_string(),
                ])),
                permission_mode: PermissionMode::DangerFullAccess,
            }
        );
    }

    #[test]
    fn parses_short_prompt_json_mode() {
        let args = vec![
            "--output-format".to_string(),
            "json".to_string(),
            "-p".to_string(),
            "summarize the diff".to_string(),
        ];

        assert_eq!(
            parse_cli_action(args, normalize_allowed_tools).expect("args should parse"),
            CliAction::Prompt {
                prompt: "summarize the diff".to_string(),
                model: "claude-opus-4-6".to_string(),
                output_format: CliOutputFormat::Json,
                allowed_tools: None,
                permission_mode: PermissionMode::DangerFullAccess,
            }
        );
    }

    #[test]
    fn parses_resume_flag_and_commands() {
        let args = vec![
            "--resume".to_string(),
            ".claw/sessions/session-123.json".to_string(),
            "/status".to_string(),
        ];

        assert_eq!(
            parse_cli_action(args, normalize_allowed_tools).expect("resume should parse"),
            CliAction::ResumeSession {
                session_path: PathBuf::from(".claw/sessions/session-123.json"),
                commands: vec!["/status".to_string()],
            }
        );
    }

    #[test]
    fn parses_help_version_and_subcommands() {
        assert_eq!(
            parse_cli_action(vec!["--help".to_string()], normalize_allowed_tools)
                .expect("help should parse"),
            CliAction::Help
        );
        assert_eq!(
            parse_cli_action(vec!["-V".to_string()], normalize_allowed_tools)
                .expect("version should parse"),
            CliAction::Version
        );
        assert_eq!(
            parse_cli_action(vec!["login".to_string()], normalize_allowed_tools)
                .expect("login should parse"),
            CliAction::Login
        );
        assert_eq!(
            parse_cli_action(vec!["logout".to_string()], normalize_allowed_tools)
                .expect("logout should parse"),
            CliAction::Logout
        );
        assert_eq!(
            parse_cli_action(vec!["init".to_string()], normalize_allowed_tools)
                .expect("init should parse"),
            CliAction::Init
        );
        assert_eq!(
            parse_cli_action(vec!["--pipe".to_string()], normalize_allowed_tools)
                .expect("pipe should parse"),
            CliAction::Pipe {
                model: "claude-opus-4-6".to_string(),
                output_format: CliOutputFormat::Text,
                allowed_tools: None,
                permission_mode: PermissionMode::DangerFullAccess,
            }
        );
        assert_eq!(
            parse_cli_action(
                vec![
                    "serve".to_string(),
                    "--host".to_string(),
                    "0.0.0.0".to_string(),
                    "--port".to_string(),
                    "8080".to_string(),
                ],
                normalize_allowed_tools,
            )
            .expect("serve should parse"),
            CliAction::Serve {
                host: "0.0.0.0".to_string(),
                port: 8080,
            }
        );
    }

    #[test]
    fn parses_system_prompt_agents_skills_and_direct_slash_aliases() {
        assert_eq!(
            parse_cli_action(
                vec![
                    "system-prompt".to_string(),
                    "--cwd".to_string(),
                    "repo".to_string(),
                    "--date".to_string(),
                    "2026-04-02".to_string(),
                ],
                normalize_allowed_tools,
            )
            .expect("system prompt should parse"),
            CliAction::PrintSystemPrompt {
                cwd: PathBuf::from("repo"),
                date: "2026-04-02".to_string(),
            }
        );
        assert_eq!(
            parse_cli_action(
                vec!["agents".to_string(), "--help".to_string()],
                normalize_allowed_tools,
            )
            .expect("agents should parse"),
            CliAction::Agents {
                args: Some("--help".to_string()),
            }
        );
        assert_eq!(
            parse_cli_action(vec!["/skills".to_string()], normalize_allowed_tools)
                .expect("/skills should parse"),
            CliAction::Skills { args: None }
        );
        assert_eq!(
            parse_cli_action(
                vec![
                    "completions".to_string(),
                    "bash".to_string(),
                    "--output".to_string(),
                    "dist/claw.bash".to_string(),
                ],
                normalize_allowed_tools,
            )
            .expect("completions should parse"),
            CliAction::GenerateCompletions {
                shell: CompletionShellArg::Bash,
                output: Some(PathBuf::from("dist/claw.bash")),
            }
        );
        assert_eq!(
            parse_cli_action(
                vec![
                    "manpage".to_string(),
                    "--output".to_string(),
                    "dist/claw.1".to_string(),
                ],
                normalize_allowed_tools,
            )
            .expect("manpage should parse"),
            CliAction::GenerateManpage {
                output: Some(PathBuf::from("dist/claw.1")),
            }
        );
    }

    #[test]
    fn rejects_unsupported_direct_slash_commands() {
        let error = parse_cli_action(vec!["/status".to_string()], normalize_allowed_tools)
            .expect_err("/status should require the repl");
        assert!(error.contains("Direct slash command unavailable"));
        assert!(error.contains("/help, /agents, /skills"));
    }
}
