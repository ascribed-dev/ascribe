//! What Ascribe writes for agents, and where each agent looks for it: the
//! project's rules in `AGENTS.md` and the other instruction files, and the
//! skill. This is the binary's one place that knows about particular
//! agents; the rules themselves come from `ascribe_query::rules`, which
//! knows none.

pub mod checker;
pub mod command;
pub mod copilot;
pub mod hook;
pub mod index;
pub mod markers;
#[cfg(test)]
pub(crate) mod plugin;
pub mod prompts;
pub mod settings;
pub mod skill;
pub mod sync;
