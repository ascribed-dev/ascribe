//! Containers, groups, and arms (SPEC §3.5, §3.6, §3.9, §3.10).
//!
//! A scope is walked with a stack of open containers. An opener pushes a
//! frame, an end line pops one, and a groupable opener joins the nearest open
//! group of the same directive instead of pushing (SPEC §3.6).

use tessera_core::{Builtin, Issue, Origin, Span, diagnostics};

use super::{Pass, Scope};
use crate::tree::*;

/// One open container, or one open group.
pub(super) struct Frame {
    /// The container's opener, or for a group the opener of the arm that's
    /// open now.
    opener: DirectiveLine,
    /// Where the opener's block starts: its title line, or the `@`.
    start: usize,
    /// The blocks collected so far (the current arm's, for a group).
    children: Vec<Block>,
    /// The arms already finished. `Some` exactly when the frame is a group.
    arms: Option<Vec<Arm>>,
}

/// Adds a block to the innermost open container, or to the scope.
pub(super) fn push(frames: &mut [Frame], root: &mut Vec<Block>, block: Block) {
    match frames.last_mut() {
        Some(frame) => frame.children.push(block),
        None => root.push(block),
    }
}

impl Pass<'_> {
    /// Handles a directive line: line form stays a sibling; an opener starts a
    /// container, a group, or a new arm.
    pub(super) fn directive(
        &mut self,
        span: Span,
        line: DirectiveLine,
        frames: &mut Vec<Frame>,
        root: &mut Vec<Block>,
    ) {
        let options = self.options;
        let Some(schema) = options.schema(&line.name) else {
            push(
                frames,
                root,
                Block {
                    span,
                    kind: BlockKind::Directive(line),
                },
            );
            return;
        };
        if line.title.is_none() && schema.title == tessera_core::TitleRule::Required {
            self.report_title_missing(&line, schema.origin);
        }

        // SPEC §3.5 (resolved Q16): the form is decided by the line's colon, and an
        // error in it doesn't change what the line opens. A colon line opens
        // a container even where that's an error, and a container-only
        // directive without its colon opens one as if it had it, so each
        // mistake is reported once and the `@end` still matches.
        let container_only = schema.forms.container && !schema.forms.line;
        if line.form == Form::Container && !schema.forms.container {
            let at = line.colon.unwrap_or(line.name_span);
            let issue = Issue::new(diagnostics::CONTAINER_COLON_UNEXPECTED, self.location(at))
                .with_arg("name", line.name.clone());
            self.report(issue);
        } else if container_only && line.colon.is_none() && line.attributes_closed {
            // SPEC §3.3 (resolved Q36): with an unclosed attribute block the colon
            // can't be known, so only the block is reported.
            let issue = Issue::new(
                diagnostics::CONTAINER_COLON_MISSING,
                self.location(line.name_span),
            )
            .with_arg("name", line.name.clone());
            self.report(issue);
        }
        if line.form != Form::Container && !container_only {
            push(
                frames,
                root,
                Block {
                    span,
                    kind: BlockKind::Directive(line),
                },
            );
            return;
        }

        let start = span.start();
        if schema.groupable {
            // SPEC §3.6: an opener joins the nearest open group of the same
            // directive in this scope, even with containers open inside the
            // current arm.
            let group = frames
                .iter()
                .rposition(|f| f.arms.is_some() && f.opener.name == line.name);
            if let Some(k) = group {
                self.next_arm(frames, k, line, start);
                return;
            }
        }
        self.open.push(line.name.clone());
        // SPEC §3.10 (resolved Q17): a group is one container level, however
        // many arms it has. SPEC §3.10 (resolved Q33): levels count containers open in
        // enclosing list items and block quotes too.
        if self.open.len() > 2 {
            let issue = Issue::new(
                diagnostics::CONTAINER_NESTING_DEEP,
                self.location(line.name_span),
            )
            .with_arg("depth", self.open.len().to_string());
            self.report(issue);
        }
        frames.push(Frame {
            opener: line,
            start,
            children: Vec::new(),
            arms: schema.groupable.then(Vec::new),
        });
    }

    /// A groupable opener begins the next arm of the open group at `frames[k]`,
    /// ending the previous arm. Containers still open inside that arm are
    /// reported at this opener, and closed there (SPEC §3.6).
    fn next_arm(&mut self, frames: &mut Vec<Frame>, k: usize, line: DirectiveLine, start: usize) {
        while frames.len() > k + 1 {
            let Some(inner) = frames.pop() else { break };
            self.open.pop();
            let opened_on = self.line(inner.opener.span.start()) + 1;
            let issue = Issue::new(
                diagnostics::CONTAINER_OPEN_AT_ARM,
                self.location(line.name_span),
            )
            .with_arg("name", inner.opener.name.clone())
            .with_arg("line", opened_on.to_string())
            .with_arg("group", line.name.clone());
            self.report(issue);
            let node = self.finish_frame(inner, None);
            if let Some(outer) = frames.last_mut() {
                outer.children.push(node);
            }
        }
        let Some(frame) = frames.get_mut(k) else {
            return;
        };
        let opener = std::mem::replace(&mut frame.opener, line);
        let opener_start = std::mem::replace(&mut frame.start, start);
        let children = std::mem::take(&mut frame.children);
        let arm = self.make_arm(opener, opener_start, children);
        if let Some(arms) = &mut frame.arms {
            arms.push(arm);
        }
    }

    /// Handles an end line: it closes the innermost open container of this
    /// scope. With none open here, it's an error.
    pub(super) fn end_line(
        &mut self,
        span: Span,
        end: EndLine,
        frames: &mut Vec<Frame>,
        root: &mut Vec<Block>,
    ) {
        if let Some(frame) = frames.pop() {
            self.open.pop();
            let node = self.finish_frame(frame, Some(end));
            push(frames, root, node);
            return;
        }
        // SPEC §3.9 rule 3 (resolved Q19): an end line in another CommonMark container
        // than an opener it would otherwise close doesn't close it. It's
        // `end-indent-mismatch` when a container is open around it, or one was
        // left unclosed earlier (which is reported there too); only an end
        // line with nothing anywhere to close is `end-unmatched`.
        let would_close = self
            .open
            .last()
            .cloned()
            .or_else(|| self.orphans.pop().map(|(name, _)| name));
        let issue = match would_close {
            Some(name) => Issue::new(diagnostics::END_INDENT_MISMATCH, self.location(end.span))
                .with_arg("name", name),
            None => Issue::new(diagnostics::END_UNMATCHED, self.location(end.span)),
        };
        self.report(issue);
        push(
            frames,
            root,
            Block {
                span,
                kind: BlockKind::End(end),
            },
        );
    }

    /// Reports and closes every container still open when a scope ends,
    /// innermost first, at the opener (SPEC §3.5). The note about arms is in
    /// [`Pass::next_arm`]: an unclosed container in a group is reported where
    /// the next arm begins when there is one.
    pub(super) fn close_unclosed(
        &mut self,
        mut frames: Vec<Frame>,
        scope: Scope,
        root: &mut Vec<Block>,
    ) {
        for frame in &frames {
            self.orphans
                .push((frame.opener.name.clone(), frame.opener.name_span));
        }
        while let Some(frame) = frames.pop() {
            self.open.pop();
            // A group is reported at its first opener (SPEC §3.6, resolved Q17).
            let first = frame
                .arms
                .as_ref()
                .and_then(|arms| arms.first())
                .map_or(frame.opener.name_span, |arm| arm.opener.name_span);
            let issue = Issue::new(diagnostics::CONTAINER_UNCLOSED, self.location(first))
                .with_arg("name", frame.opener.name.clone())
                .with_arg("end", scope.end());
            self.report(issue);
            let node = self.finish_frame(frame, None);
            push(&mut frames, root, node);
        }
    }

    /// Finishes a container or a group as a block.
    fn finish_frame(&mut self, frame: Frame, end: Option<EndLine>) -> Block {
        let Frame {
            opener,
            start,
            children,
            arms,
        } = frame;
        let end_span = end.as_ref().map(|e| e.span.end());
        let Some(mut arms) = arms else {
            let mut children = children;
            self.finish_children(&mut children, Scope::Container);
            let last = children.last().map_or(0, |b| b.span.end());
            let to = end_span.unwrap_or_else(|| last.max(opener.span.end()));
            return Block {
                span: Span::new(start, to),
                kind: BlockKind::Container(Container {
                    opener,
                    children,
                    end,
                }),
            };
        };
        arms.push(self.make_arm(opener, start, children));
        let name = arms
            .first()
            .map_or_else(String::new, |arm| arm.opener.name.clone());
        self.check_group(&name, &arms);
        let first = arms.first().map_or(start, |arm| arm.span.start());
        let last = arms.last().map_or(first, |arm| arm.span.end());
        Block {
            span: Span::new(first, end_span.unwrap_or(last)),
            kind: BlockKind::Group(Group { name, arms, end }),
        }
    }

    /// Finishes one arm: checks the arm's own rules and makes its node.
    fn make_arm(&mut self, opener: DirectiveLine, start: usize, mut children: Vec<Block>) -> Arm {
        self.finish_children(&mut children, Scope::Arm);
        self.check_arm(&opener);
        let last = children.last().map_or(0, |b| b.span.end());
        Arm {
            span: Span::new(start, last.max(opener.span.end())),
            title: opener.title.clone(),
            opener,
            children,
        }
    }

    fn is_variant(&self, name: &str) -> bool {
        self.options
            .schema(name)
            .is_some_and(|s| s.origin == Origin::Builtin(Builtin::Variant))
    }

    /// An arm has attributes or a title, never both and never neither
    /// (SPEC §4.3).
    fn check_arm(&mut self, opener: &DirectiveLine) {
        if !self.is_variant(&opener.name) {
            return;
        }
        let issue = match (has_attributes(opener), opener.title.is_some()) {
            (true, true) => Issue::new(
                diagnostics::VARIANT_ARM_KIND,
                self.location(opener.name_span),
            ),
            (false, false) => Issue::new(
                diagnostics::VARIANT_ARM_KIND,
                self.location(opener.name_span),
            )
            .with_variant("neither"),
            _ => return,
        };
        self.report(issue);
    }

    /// The rules about a whole `@variant` group: all arms labeled or all
    /// dimensional, and dimensional arms share a key (SPEC §4.3).
    ///
    /// SPEC §3.6 (resolved Q17): these are reported at the group's first opener.
    fn check_group(&mut self, name: &str, arms: &[Arm]) {
        if !self.is_variant(name) {
            return;
        }
        // Arms that are neither or both were reported on their own.
        let labeled = arms
            .iter()
            .filter(|a| a.title.is_some() && !has_attributes(&a.opener))
            .count();
        let dimensional: Vec<&Arm> = arms
            .iter()
            .filter(|a| a.title.is_none() && has_attributes(&a.opener))
            .collect();
        let Some(first) = arms.first() else { return };
        let at = self.location(first.opener.name_span);
        if labeled > 0 && !dimensional.is_empty() {
            self.report(Issue::new(diagnostics::VARIANT_MIXED_ARMS, at));
            return;
        }
        if dimensional.len() < 2 {
            return;
        }
        let keys = |arm: &Arm| -> Vec<String> {
            arm.opener
                .attributes
                .iter()
                .flat_map(|block| block.attributes.iter().map(|a| a.key.clone()))
                .collect()
        };
        let mut common = keys(dimensional[0]);
        for arm in &dimensional[1..] {
            let these = keys(arm);
            common.retain(|k| these.contains(k));
        }
        if common.is_empty() {
            self.report(Issue::new(diagnostics::VARIANT_NO_SHARED_DIMENSION, at));
        }
    }

    /// A directive whose schema requires a title has none.
    fn report_title_missing(&mut self, line: &DirectiveLine, origin: Origin) {
        let issue = match origin {
            Origin::Builtin(_) => Issue::new(
                diagnostics::DETAILS_TITLE_MISSING,
                self.location(line.name_span),
            ),
            Origin::Widget => Issue::new(diagnostics::WIDGET_SCHEMA, self.location(line.name_span))
                .with_arg("name", line.name.clone())
                .with_arg("detail", "it requires a title line directly above it"),
        };
        self.report(issue);
    }
}

fn has_attributes(line: &DirectiveLine) -> bool {
    line.attributes
        .as_ref()
        .is_some_and(|block| !block.attributes.is_empty())
}
