//! The hook and MCP entries `ascribe agents sync --with-hook` writes into a
//! project's agent settings: Claude Code's `.claude/settings.json` and
//! `.mcp.json`, Codex's `.codex/hooks.json`, and Copilot's
//! `.github/hooks/ascribe.json`.
//!
//! A shared file is merged: Ascribe's entries (those whose command runs
//! `ascribe agents hook`, and the `ascribe` MCP server) are updated in
//! place, keeping what else a user set on them, such as a timeout, and
//! everything else is kept, in its order. Copilot's file is Ascribe's own.

use std::fmt;

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use serde_json::Value;

/// How long a harness lets the edit hook run, in seconds: longer than the
/// hook's own limit, so the hook always answers first.
const EDIT_TIMEOUT: u64 = 10;

/// How long a harness lets the stop hook run, in seconds, for the same
/// reason.
pub const STOP_TIMEOUT: u64 = 60;

/// The MCP server's name in `.mcp.json`.
const MCP_SERVER: &str = "ascribe";

/// The tools after which Claude Code runs the edit hook: `MultiEdit` is
/// gone from its current tools, but older versions have it.
const CLAUDE_EDITS: &str = "Write|Edit|MultiEdit";

/// The MCP server's entry, `ascribe mcp`, as `.mcp.json` holds it.
pub fn mcp_servers(ascribe: &str) -> Value {
    serde_json::json!({ "mcpServers": { MCP_SERVER: { "command": ascribe, "args": ["mcp"] } } })
}

/// What's in each command: how the project runs `ascribe`. A harness may
/// run its hooks from a subfolder, so a pinned binary is found from the
/// repository's root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ascribe {
    /// For a POSIX shell (Codex's hooks, and Copilot's `bash`): `ascribe`,
    /// or the pinned binary under `git rev-parse --show-toplevel`.
    pub shell: String,
    /// For Claude Code, whose hooks run from wherever its session is:
    /// the pinned path under `$CLAUDE_PROJECT_DIR`.
    pub claude: String,
    /// For PowerShell (Copilot's `powershell`): the `.cmd` npm writes
    /// beside the pinned binary, run with `&`.
    pub powershell: String,
    /// Where no shell can find the repository's root on every system:
    /// `.mcp.json`, and Codex on Windows, whose shell isn't documented.
    /// Always `ascribe`, from the path.
    pub plain: String,
}

impl Ascribe {
    /// `ascribe`, found on the path.
    pub fn on_path() -> Ascribe {
        Ascribe {
            shell: "ascribe".to_owned(),
            claude: "ascribe".to_owned(),
            powershell: "ascribe".to_owned(),
            plain: "ascribe".to_owned(),
        }
    }

    /// The binary at `path`, from the repository's root,
    /// `/`-separated: `node_modules/.bin/ascribe`.
    pub fn pinned(path: &str) -> Ascribe {
        Ascribe {
            shell: format!("\"$(git rev-parse --show-toplevel)/{path}\""),
            claude: format!("\"$CLAUDE_PROJECT_DIR\"/{path}"),
            powershell: format!("& \"$(git rev-parse --show-toplevel)/{path}.cmd\""),
            plain: "ascribe".to_owned(),
        }
    }
}

/// The words every Ascribe hook command has, by which it's known.
const HOOK: &str = "agents hook";

/// `.claude/settings.json`, from its text now: the edit hook after `Write`
/// and `Edit`, and the stop hook.
pub fn claude_settings(old: Option<&str>, ascribe: &Ascribe) -> Result<String, String> {
    let command =
        |event: &str| format!("{} agents hook claude-code --event {event}", ascribe.claude);
    let mut settings = parse(old)?;
    let hooks = settings.object("hooks")?;
    hooks.set_groups(
        "PostToolUse",
        Some(CLAUDE_EDITS),
        &[("command", command("edit"))],
        EDIT_TIMEOUT,
    )?;
    hooks.set_groups("Stop", None, &[("command", command("stop"))], STOP_TIMEOUT)?;
    Ok(pretty(&settings))
}

/// `.codex/hooks.json`: the same as Claude Code's hooks, which Codex
/// shares, without a matcher: Codex names its edits `apply_patch`, and the
/// hook passes over a tool that writes no file. `commandWindows` runs
/// `ascribe` from the path: Codex doesn't say which shell runs it.
pub fn codex_hooks(old: Option<&str>, ascribe: &Ascribe) -> Result<String, String> {
    let command =
        |ascribe: &str, event: &str| format!("{ascribe} agents hook codex --event {event}");
    let commands = |event: &str| {
        [
            ("command", command(&ascribe.shell, event)),
            ("commandWindows", command(&ascribe.plain, event)),
        ]
    };
    let mut settings = parse(old)?;
    let hooks = settings.object("hooks")?;
    hooks.set_groups("PostToolUse", None, &commands("edit"), EDIT_TIMEOUT)?;
    hooks.set_groups("Stop", None, &commands("stop"), STOP_TIMEOUT)?;
    Ok(pretty(&settings))
}

/// `.mcp.json`, with the `ascribe` MCP server: its command and arguments
/// set, and what else a user set on it kept. `None` when the file has no
/// `ascribe` server and `add` isn't set: a user who removed it keeps it
/// removed.
pub fn mcp_json(old: Option<&str>, ascribe: &Ascribe, add: bool) -> Result<Option<String>, String> {
    let mut settings = parse(old)?;
    let servers = settings.object("mcpServers")?;
    let ours = mcp_servers(&ascribe.plain)["mcpServers"][MCP_SERVER].clone();
    match servers.get_mut(MCP_SERVER) {
        Some(Json::Object(entry)) => {
            for key in ["command", "args"] {
                let value = Json::from(&ours[key]);
                match entry.iter_mut().find(|(k, _)| k == key) {
                    Some((_, old)) => *old = value,
                    None => entry.push((key.to_owned(), value)),
                }
            }
        }
        Some(_) => return Err(format!("its `mcpServers.{MCP_SERVER}` isn't an object")),
        None if add => servers.set(MCP_SERVER, Json::from(&ours)),
        None => return Ok(None),
    }
    Ok(Some(pretty(&settings)))
}

/// Whether `old` and `new` mean the same JSON, whatever their layout and
/// the order of their keys.
pub fn same_meaning(old: &str, new: &str) -> bool {
    match (
        serde_json::from_str::<Value>(old),
        serde_json::from_str::<Value>(new),
    ) {
        (Ok(old), Ok(new)) => old == new,
        _ => false,
    }
}

/// `.github/hooks/ascribe.json`, Copilot's: wholly Ascribe's.
pub fn copilot_hooks(ascribe: &Ascribe) -> String {
    let entry = |event: &str, timeout: u64| {
        serde_json::json!({
            "type": "command",
            "bash": format!("{} agents hook copilot --event {event}", ascribe.shell),
            "powershell": format!("{} agents hook copilot --event {event}", ascribe.powershell),
            "timeoutSec": timeout,
        })
    };
    let hooks = serde_json::json!({
        "version": 1,
        "hooks": {
            "postToolUse": [entry("edit", EDIT_TIMEOUT)],
            "agentStop": [entry("stop", STOP_TIMEOUT)],
        }
    });
    pretty(&Json::from(&hooks))
}

fn parse(old: Option<&str>) -> Result<Json, String> {
    match old {
        None => Ok(Json::Object(Vec::new())),
        Some(text) if text.trim().is_empty() => Ok(Json::Object(Vec::new())),
        Some(text) => match serde_json::from_str::<Json>(text) {
            Ok(json @ Json::Object(_)) => Ok(json),
            Ok(_) => Err("it isn't a JSON object".to_owned()),
            Err(e) => Err(format!("it isn't JSON: {e}")),
        },
    }
}

fn pretty(json: &Json) -> String {
    serde_json::to_string_pretty(json).unwrap_or_default() + "\n"
}

/// JSON that keeps its objects' keys in order, so a merge changes only
/// what it means to.
#[derive(Clone, Debug, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    /// The object at `key` in this object, made when there's none.
    fn object(&mut self, key: &str) -> Result<&mut Json, String> {
        let Json::Object(entries) = self else {
            return Err("it isn't a JSON object".to_owned());
        };
        if !entries.iter().any(|(k, _)| k == key) {
            entries.push((key.to_owned(), Json::Object(Vec::new())));
        }
        match entries.iter_mut().find(|(k, _)| k == key) {
            Some((_, value @ Json::Object(_))) => Ok(value),
            _ => Err(format!("its `{key}` isn't an object")),
        }
    }

    /// Sets `key` in this object, in its place when it's there.
    fn set(&mut self, key: &str, value: Json) {
        if let Json::Object(entries) = self {
            match entries.iter_mut().find(|(k, _)| k == key) {
                Some((_, old)) => *old = value,
                None => entries.push((key.to_owned(), value)),
            }
        }
    }

    /// The value at `key` in this object.
    fn get_mut(&mut self, key: &str) -> Option<&mut Json> {
        let Json::Object(entries) = self else {
            return None;
        };
        entries.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// In a `hooks` object, Ascribe's hook for `event`, with `commands`.
    /// The first Ascribe hook there is updated in place, keeping its other
    /// fields (a timeout the user changed) and its group, whose matcher is
    /// set only when the group holds nothing else; any other Ascribe hook
    /// is taken out, and a group left empty by that dropped. With none, a
    /// group with the hook is added at the end.
    fn set_groups(
        &mut self,
        event: &str,
        matcher: Option<&str>,
        commands: &[(&str, String)],
        timeout: u64,
    ) -> Result<(), String> {
        let Json::Object(entries) = self else {
            return Err("its `hooks` isn't an object".to_owned());
        };
        let mut groups = match entries.iter().find(|(k, _)| k == event) {
            Some((_, Json::Array(groups))) => groups.clone(),
            Some(_) => return Err(format!("its `hooks.{event}` isn't a list")),
            None => Vec::new(),
        };
        let mut found = false;
        groups.retain_mut(|group| {
            let Json::Object(fields) = group else {
                return true;
            };
            let Some((_, Json::Array(hooks))) = fields.iter_mut().find(|(k, _)| k == "hooks")
            else {
                return true;
            };
            hooks.retain_mut(|hook| {
                if !hook.is_ascribe_hook() {
                    return true;
                }
                if found {
                    return false;
                }
                found = true;
                for (key, command) in commands {
                    hook.set(key, Json::String(command.clone()));
                }
                true
            });
            if hooks.is_empty() {
                return false;
            }
            if hooks.iter().all(Json::is_ascribe_hook) {
                match matcher {
                    Some(matcher) => {
                        let matcher = Json::String(matcher.to_owned());
                        match fields.iter_mut().find(|(k, _)| k == "matcher") {
                            Some((_, old)) => *old = matcher,
                            None => fields.insert(0, ("matcher".to_owned(), matcher)),
                        }
                    }
                    None => fields.retain(|(k, _)| k != "matcher"),
                }
            }
            true
        });
        if !found {
            let mut hook = vec![("type".to_owned(), Json::String("command".to_owned()))];
            for (key, command) in commands {
                hook.push(((*key).to_owned(), Json::String(command.clone())));
            }
            hook.push(("timeout".to_owned(), Json::Number(timeout.into())));
            let mut group = Vec::new();
            if let Some(matcher) = matcher {
                group.push(("matcher".to_owned(), Json::String(matcher.to_owned())));
            }
            group.push(("hooks".to_owned(), Json::Array(vec![Json::Object(hook)])));
            groups.push(Json::Object(group));
        }
        self.set(event, Json::Array(groups));
        Ok(())
    }

    fn is_ascribe_hook(&self) -> bool {
        let Json::Object(fields) = self else {
            return false;
        };
        fields
            .iter()
            .any(|(k, v)| k == "command" && matches!(v, Json::String(c) if c.contains(HOOK)))
    }
}

impl From<&Value> for Json {
    fn from(value: &Value) -> Json {
        match value {
            Value::Null => Json::Null,
            Value::Bool(b) => Json::Bool(*b),
            Value::Number(n) => Json::Number(n.clone()),
            Value::String(s) => Json::String(s.clone()),
            Value::Array(items) => Json::Array(items.iter().map(Json::from).collect()),
            // `serde_json`'s objects are in key order; the order of the
            // entries Ascribe writes doesn't matter.
            Value::Object(map) => Json::Object(
                map.iter()
                    .map(|(k, v)| (k.clone(), Json::from(v)))
                    .collect(),
            ),
        }
    }
}

impl Serialize for Json {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Json::Null => serializer.serialize_unit(),
            Json::Bool(b) => serializer.serialize_bool(*b),
            Json::Number(n) => n.serialize(serializer),
            Json::String(s) => serializer.serialize_str(s),
            Json::Array(items) => {
                let mut seq = serializer.serialize_seq(Some(items.len()))?;
                for item in items {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Json::Object(entries) => {
                let mut map = serializer.serialize_map(Some(entries.len()))?;
                for (k, v) in entries {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Json {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Json, D::Error> {
        deserializer.deserialize_any(JsonVisitor)
    }
}

struct JsonVisitor;

impl<'de> Visitor<'de> for JsonVisitor {
    type Value = Json;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JSON")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Json, E> {
        Ok(Json::Null)
    }

    fn visit_bool<E: de::Error>(self, b: bool) -> Result<Json, E> {
        Ok(Json::Bool(b))
    }

    fn visit_i64<E: de::Error>(self, n: i64) -> Result<Json, E> {
        Ok(Json::Number(n.into()))
    }

    fn visit_u64<E: de::Error>(self, n: u64) -> Result<Json, E> {
        Ok(Json::Number(n.into()))
    }

    fn visit_f64<E: de::Error>(self, n: f64) -> Result<Json, E> {
        serde_json::Number::from_f64(n)
            .map(Json::Number)
            .ok_or_else(|| E::custom("a number JSON can't hold"))
    }

    fn visit_str<E: de::Error>(self, s: &str) -> Result<Json, E> {
        Ok(Json::String(s.to_owned()))
    }

    fn visit_string<E: de::Error>(self, s: String) -> Result<Json, E> {
        Ok(Json::String(s))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Json, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element()? {
            items.push(item);
        }
        Ok(Json::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Json, A::Error> {
        let mut entries: Vec<(String, Json)> = Vec::new();
        while let Some((key, value)) = map.next_entry::<String, Json>()? {
            // A repeated key: the last one counts, as for `serde_json`.
            entries.retain(|(k, _)| *k != key);
            entries.push((key, value));
        }
        Ok(Json::Object(entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_merge_keeps_other_entries_and_their_order() {
        let old = r#"{
  "permissions": { "allow": ["Bash(npm test)"] },
  "hooks": {
    "PostToolUse": [
      { "matcher": "Write", "hooks": [
        { "type": "command", "command": "prettier --write" },
        { "type": "command", "command": "ascribe agents hook claude-code --event edit" }
      ] }
    ],
    "Stop": [ { "hooks": [ { "type": "command", "command": "old/ascribe agents hook claude-code --event stop" } ] } ]
  },
  "model": "opus"
}"#;
        let new = claude_settings(Some(old), &Ascribe::on_path()).unwrap();
        let keys: Vec<&str> = new
            .lines()
            .filter(|l| l.starts_with("  \""))
            .map(|l| l.trim().split('"').nth(1).unwrap())
            .collect();
        assert_eq!(keys, ["permissions", "hooks", "model"]);
        let value: Value = serde_json::from_str(&new).unwrap();
        // Ascribe's hook is updated where it was, beside the user's, whose
        // group keeps its matcher.
        let post = value["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(post.len(), 1);
        assert_eq!(post[0]["matcher"], "Write");
        assert_eq!(post[0]["hooks"][0]["command"], "prettier --write");
        assert_eq!(
            post[0]["hooks"][1]["command"],
            "ascribe agents hook claude-code --event edit"
        );
        let stop = value["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 1);
        assert_eq!(
            stop[0]["hooks"][0]["command"],
            "ascribe agents hook claude-code --event stop"
        );
        // Merging again changes nothing.
        assert_eq!(
            claude_settings(Some(&new), &Ascribe::on_path()).unwrap(),
            new
        );
    }

    #[test]
    fn a_new_hook_runs_after_each_edit_tool() {
        let new = claude_settings(None, &Ascribe::on_path()).unwrap();
        let value: Value = serde_json::from_str(&new).unwrap();
        assert_eq!(
            value["hooks"]["PostToolUse"][0]["matcher"],
            "Write|Edit|MultiEdit"
        );
        assert_eq!(value["hooks"]["PostToolUse"][0]["hooks"][0]["timeout"], 10);
    }

    #[test]
    fn a_timeout_the_user_set_is_kept_and_a_second_hook_dropped() {
        let old = r#"{"hooks": {"Stop": [
  {"hooks": [{"type": "command", "command": "ascribe agents hook claude-code --event stop", "timeout": 120}]},
  {"hooks": [{"type": "command", "command": "ascribe agents hook claude-code --event stop"}]}
]}}"#;
        let new = claude_settings(Some(old), &Ascribe::on_path()).unwrap();
        let value: Value = serde_json::from_str(&new).unwrap();
        let stop = value["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 1);
        assert_eq!(stop[0]["hooks"][0]["timeout"], 120);
    }

    #[test]
    fn layout_alone_isnt_a_change() {
        let new = claude_settings(None, &Ascribe::on_path()).unwrap();
        let value: Value = serde_json::from_str(&new).unwrap();
        let four = serde_json::to_string(&value).unwrap();
        assert_ne!(four, new);
        assert!(same_meaning(&four, &new));
        assert!(!same_meaning(&four.replace("10", "11"), &new));
    }

    #[test]
    fn mcp_json_adds_its_server_only_when_asked() {
        let old = r#"{"mcpServers":{"other":{"command":"x"}}}"#;
        assert_eq!(mcp_json(Some(old), &Ascribe::on_path(), false), Ok(None));
        let kept = r#"{"mcpServers":{"ascribe":{"command":"old","env":{"A":"1"}}}}"#;
        let new = mcp_json(Some(kept), &Ascribe::on_path(), false)
            .unwrap()
            .unwrap();
        let value: Value = serde_json::from_str(&new).unwrap();
        assert_eq!(value["mcpServers"]["ascribe"]["command"], "ascribe");
        assert_eq!(value["mcpServers"]["ascribe"]["args"][0], "mcp");
        assert_eq!(value["mcpServers"]["ascribe"]["env"]["A"], "1");
    }

    #[test]
    fn a_file_that_isnt_an_object_is_refused() {
        assert!(claude_settings(Some("[1]"), &Ascribe::on_path()).is_err());
        assert!(claude_settings(Some("{ nope"), &Ascribe::on_path()).is_err());
        assert!(claude_settings(Some(r#"{"hooks": []}"#), &Ascribe::on_path()).is_err());
    }

    #[test]
    fn mcp_json_keeps_other_servers() {
        let old = r#"{"mcpServers":{"other":{"command":"x"}}}"#;
        let pinned = Ascribe::pinned("node_modules/.bin/ascribe");
        let new = mcp_json(Some(old), &pinned, true).unwrap().unwrap();
        let value: Value = serde_json::from_str(&new).unwrap();
        assert_eq!(value["mcpServers"]["other"]["command"], "x");
        // `.mcp.json` can't name the repository's root on every system.
        assert_eq!(value["mcpServers"]["ascribe"]["command"], "ascribe");
    }

    #[test]
    fn a_pinned_binary_is_named_for_each_shell() {
        let pinned = Ascribe::pinned("docs/node_modules/.bin/ascribe");
        let claude = claude_settings(None, &pinned).unwrap();
        assert!(claude.contains(
            r#""command": "\"$CLAUDE_PROJECT_DIR\"/docs/node_modules/.bin/ascribe agents hook claude-code --event edit""#
        ), "{claude}");
        let copilot = copilot_hooks(&pinned);
        assert!(copilot.contains(r#""bash": "\"$(git rev-parse --show-toplevel)/docs/node_modules/.bin/ascribe\" agents hook copilot --event stop""#), "{copilot}");
        assert!(copilot.contains(r#""powershell": "& \"$(git rev-parse --show-toplevel)/docs/node_modules/.bin/ascribe.cmd\" agents hook copilot --event stop""#), "{copilot}");
        let codex = codex_hooks(None, &pinned).unwrap();
        assert!(codex.contains(r#""command": "\"$(git rev-parse --show-toplevel)/docs/node_modules/.bin/ascribe\" agents hook codex --event edit""#), "{codex}");
        assert!(
            codex.contains(r#""commandWindows": "ascribe agents hook codex --event edit""#),
            "{codex}"
        );
    }
}
