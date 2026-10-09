//! `ascribe mcp`'s tools, resources, and prompts. The protocol is
//! `ascribe_mcp`'s; what the server offers is here, beside the commands,
//! because every tool returns what its command writes, and every resource
//! and prompt is what a command prints (`OFFERED` lists each with its
//! command).
//!
//! Every tool takes paths from the server's working directory, finds the
//! nearest `ascribe.toml` at or above them, and keeps the project loaded
//! for the next call ([`cache`]), so one server serves every project in a
//! repository. No tool writes a file.

mod cache;
mod tools;

use std::path::{Path, PathBuf};

use ascribe_check::Project;
use ascribe_core::path::{normalize, relative_path};
use ascribe_mcp::{
    Handler, Prompt, PromptArgument, PromptText, Resource, ResourceTemplate, ResourceText,
    ServerInfo, Tool, ToolResult,
};
use serde_json::{Map, Value};

use crate::agents::{prompts, skill, sync};
use crate::answer;
use crate::cli::{Color, Global};
use crate::commands::model::SUMMARY_BUDGET;

pub use cache::Cache;
pub use tools::TOOLS;

/// The resource with the directive reference.
const DIRECTIVES: &str = "ascribe://directives";
/// The start of a project's content model summary's URI.
const MODEL: &str = "ascribe://model/";
/// The start of a project's rules' URI.
const INSTRUCTIONS: &str = "ascribe://instructions/";

/// The most projects `resources/list` lists.
const MAX_PROJECTS: usize = 20;

/// Each resource (by URI or template) and prompt, with the command that
/// gives the same thing, which its description names. The tools name
/// theirs in [`TOOLS`].
pub const OFFERED: &[(&str, &str)] = &[
    (DIRECTIVES, "ascribe agents skill references/directives.md"),
    ("ascribe://model/{+project}", "ascribe model <project>"),
    (
        "ascribe://instructions/{+project}",
        "ascribe agents rules <project>",
    ),
    ("new-page", "ascribe agents prompt new-page"),
    ("fix", "ascribe agents prompt fix"),
    ("review", "ascribe agents prompt review"),
];

/// `description`, and the command [`OFFERED`] gives for `key`.
fn with_command(description: &str, key: &str) -> String {
    match OFFERED.iter().find(|(k, _)| *k == key) {
        Some((_, command)) => format!("{description} The command: `{command}`."),
        None => description.to_owned(),
    }
}

/// What the server says about itself to the model.
pub const INSTRUCTIONS_TEXT: &str = "Ascribe checks documentation written as code: Markdown \
    pages with `@` directives in a folder with an `ascribe.toml`. After you edit a page, call \
    `ascribe_check` with its path and fix what it reports; before you write frontmatter, a \
    directive, or a phrase, read `ascribe_model`. Paths are from the server's working \
    directory, and no tool writes a file. With a shell, the `ascribe` commands of the same \
    names give the same answers.";

/// The server's side of the protocol.
pub struct Server {
    cache: Cache,
    /// The working directory, which every path is read from.
    cwd: PathBuf,
}

impl Server {
    /// A server working in `cwd`.
    pub fn new(cwd: PathBuf) -> Server {
        Server {
            cache: Cache::default(),
            cwd,
        }
    }

    /// Who the server is.
    pub fn info() -> ServerInfo {
        ServerInfo {
            name: "ascribe".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            instructions: INSTRUCTIONS_TEXT.to_owned(),
        }
    }

    /// The projects worth listing: the one at or above the working
    /// directory, and those below it, at most [`MAX_PROJECTS`], each as its
    /// folder from the working directory.
    fn projects(&self) -> Vec<String> {
        let mut dirs = Vec::new();
        if let Some(config) = Project::find_config(&self.cwd)
            && let Some(dir) = config.parent()
        {
            dirs.push(dir.to_owned());
        }
        below(&self.cwd, &mut dirs);
        dirs.sort();
        dirs.dedup();
        dirs.into_iter()
            .take(MAX_PROJECTS)
            .filter_map(|dir| {
                let rel = relative_path(&normalize(&self.cwd), &normalize(&dir))?;
                Some(if rel.is_root() {
                    ".".to_owned()
                } else {
                    rel.to_string()
                })
            })
            .collect()
    }

    fn global(&self) -> Global {
        Global {
            config: None,
            color: Color::Never,
        }
    }

    /// A project's content model, summarized: what `ascribe model <project>`
    /// prints.
    fn model_summary(&self, project: &str) -> Result<String, String> {
        let config =
            answer::locate(&self.global(), Some(Path::new(project))).map_err(|e| e.to_string())?;
        let file = Project::load_model(&config)
            .map_err(|e| answer::Stop::Load(crate::context::Failure::Load(e)).message())?;
        Ok(ascribe_query::summary(&file.model, None, SUMMARY_BUDGET))
    }

    /// A project's rules: what `ascribe agents rules <project>` prints.
    fn rules(&self, project: &str) -> Result<String, String> {
        let loaded = answer::load(&self.cache, &self.global(), Some(Path::new(project)))
            .map_err(|f| answer::Stop::Load(f).message())?;
        sync::rules_beside(&loaded.project, loaded.index()).map_err(|e| e.to_string())
    }
}

/// Adds every folder below `dir` that holds an `ascribe.toml`, skipping
/// folders whose names start with `.`, `node_modules`, and `target`.
fn below(dir: &Path, found: &mut Vec<PathBuf>) {
    if found.len() > MAX_PROJECTS {
        return;
    }
    // Outside FileSystem: looking for projects, before there is one.
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            !name.starts_with('.') && name != "node_modules" && name != "target"
        })
        .map(|e| e.path())
        .collect();
    dirs.sort();
    for child in dirs {
        // Outside FileSystem: as above.
        if child.join(ascribe_check::MODEL_FILE).is_file() {
            found.push(child.clone());
        }
        below(&child, found);
    }
}

impl Handler for Server {
    fn tools(&self) -> Vec<Tool> {
        TOOLS.iter().map(tools::Spec::tool).collect()
    }

    fn call_tool(&mut self, name: &str, arguments: &Map<String, Value>) -> Option<ToolResult> {
        let spec = TOOLS.iter().find(|t| t.name == name)?;
        Some(spec.call(self, arguments))
    }

    fn resources(&mut self) -> Vec<Resource> {
        let mut out = vec![Resource {
            uri: DIRECTIVES.to_owned(),
            name: "directives".to_owned(),
            description: with_command("Each built-in directive's syntax.", DIRECTIVES),
            mime_type: "text/markdown".to_owned(),
        }];
        for project in self.projects() {
            let place = if project == "." {
                "the server's working directory".to_owned()
            } else {
                format!("`{project}`")
            };
            out.push(Resource {
                uri: format!("{MODEL}{project}"),
                name: format!("model {project}"),
                description: with_command(
                    &format!("What the content model of the project in {place} allows."),
                    "ascribe://model/{+project}",
                ),
                mime_type: "text/markdown".to_owned(),
            });
            out.push(Resource {
                uri: format!("{INSTRUCTIONS}{project}"),
                name: format!("instructions {project}"),
                description: with_command(
                    &format!("The rules for an agent writing pages in the project in {place}."),
                    "ascribe://instructions/{+project}",
                ),
                mime_type: "text/markdown".to_owned(),
            });
        }
        out
    }

    fn resource_templates(&self) -> Vec<ResourceTemplate> {
        vec![
            ResourceTemplate {
                uri_template: format!("{MODEL}{{+project}}"),
                name: "model".to_owned(),
                description: with_command(
                    "What a project's content model allows; `project` is a file or folder in \
                     it, from the server's working directory.",
                    "ascribe://model/{+project}",
                ),
                mime_type: "text/markdown".to_owned(),
            },
            ResourceTemplate {
                uri_template: format!("{INSTRUCTIONS}{{+project}}"),
                name: "instructions".to_owned(),
                description: with_command(
                    "The rules for an agent writing a project's pages; `project` is a file or \
                     folder in it, from the server's working directory.",
                    "ascribe://instructions/{+project}",
                ),
                mime_type: "text/markdown".to_owned(),
            },
        ]
    }

    fn read_resource(&mut self, uri: &str) -> Result<ResourceText, String> {
        let text = if uri == DIRECTIVES {
            skill::directives_md()
        } else if let Some(project) = uri.strip_prefix(MODEL) {
            self.model_summary(project)?
        } else if let Some(project) = uri.strip_prefix(INSTRUCTIONS) {
            self.rules(project)?
        } else {
            return Err(format!(
                "no resource {uri}; the resources are {DIRECTIVES}, {MODEL}<project>, and \
                 {INSTRUCTIONS}<project>"
            ));
        };
        Ok(ResourceText {
            mime_type: "text/markdown".to_owned(),
            text,
        })
    }

    fn prompts(&self) -> Vec<Prompt> {
        prompts::PROMPTS
            .iter()
            .map(|p| Prompt {
                name: p.name.to_owned(),
                title: p.title.to_owned(),
                description: with_command(p.description, p.name),
                arguments: p
                    .arguments
                    .iter()
                    .map(|(name, description, required)| PromptArgument {
                        name: (*name).to_owned(),
                        description: (*description).to_owned(),
                        required: *required,
                    })
                    .collect(),
            })
            .collect()
    }

    fn get_prompt(
        &mut self,
        name: &str,
        arguments: &Map<String, Value>,
    ) -> Option<Result<PromptText, String>> {
        let prompt = prompts::find(name)?;
        let mut given = prompts::Arguments::new();
        for (key, value) in arguments {
            match value {
                Value::String(s) => {
                    given.insert(key.clone(), s.clone());
                }
                Value::Null => {}
                _ => return Some(Err(format!("`{key}` is a string"))),
            }
        }
        Some(prompt.text(&self.cache, &given).map(|text| PromptText {
            description: prompt.description.to_owned(),
            text,
        }))
    }
}
