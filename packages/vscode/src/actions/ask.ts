// Shows a wizard's steps in VS Code's quick input: a quick pick to pick one
// or several, an input box to enter text, with the step counter ("2 of 3")
// and a Back button. The steps themselves are built in `steps.ts`.

import * as vscode from "vscode";
import type { Choice, Position, Prompter, Shown, Step } from "./steps.js";

interface ChoiceItem extends vscode.QuickPickItem {
  value?: string;
}

/** The quick input. */
export class QuickInputPrompter implements Prompter {
  show(step: Step, position: Position): Promise<Shown> {
    return step.kind === "text" ? this.text(step, position) : this.pick(step, position);
  }

  private pick(step: Extract<Step, { kind: "pick" | "pickMany" }>, at: Position): Promise<Shown> {
    const input = vscode.window.createQuickPick<ChoiceItem>();
    frame(input, at);
    input.placeholder = step.prompt;
    input.canSelectMany = step.kind === "pickMany";
    input.matchOnDescription = true;
    input.items = items(step.choices);
    const previous = new Set(at.previous === undefined ? [] : [at.previous].flat());
    const earlier = input.items.filter(
      (item) => item.value !== undefined && previous.has(item.value),
    );
    if (step.kind === "pickMany") input.selectedItems = earlier;
    else if (earlier.length > 0) input.activeItems = earlier;
    return new Promise((resolve) => {
      const finish = settle(input, resolve);
      input.onDidTriggerButton((button) => {
        if (button === vscode.QuickInputButtons.Back) finish("back");
      });
      input.onDidAccept(() => {
        if (step.kind === "pickMany") {
          const values = input.selectedItems.flatMap((item) => item.value ?? []);
          if (values.length === 0) {
            input.placeholder = `${step.prompt} Choose at least one.`;
            return;
          }
          finish({ answer: values });
          return;
        }
        const value = (input.selectedItems[0] ?? input.activeItems[0])?.value;
        if (value !== undefined) finish({ answer: value });
      });
      input.show();
    });
  }

  private text(step: Extract<Step, { kind: "text" }>, at: Position): Promise<Shown> {
    const input = vscode.window.createInputBox();
    frame(input, at);
    input.prompt = step.prompt;
    if (step.placeholder !== undefined) input.placeholder = step.placeholder;
    input.value = typeof at.previous === "string" ? at.previous : (step.value ?? "");
    return new Promise((resolve) => {
      const finish = settle(input, resolve);
      input.onDidTriggerButton((button) => {
        if (button === vscode.QuickInputButtons.Back) finish("back");
      });
      input.onDidChangeValue((value) => {
        // Not while the box is still empty: no complaint before any typing.
        input.validationMessage = value === "" ? undefined : step.validate(value);
      });
      input.onDidAccept(() => {
        const problem = step.validate(input.value);
        if (problem) input.validationMessage = problem;
        else finish({ answer: input.value });
      });
      input.show();
    });
  }
}

/** The title, the step counter, and Back after the first step. */
function frame(
  input: vscode.QuickInput & { buttons: readonly vscode.QuickInputButton[] },
  at: Position,
): void {
  input.title = at.title;
  if (at.total > 1) {
    input.step = at.step;
    input.totalSteps = at.total;
  }
  input.buttons = at.step > 1 ? [vscode.QuickInputButtons.Back] : [];
}

/** Resolves once, with what ended the step, and disposes the input; hiding it cancels. */
function settle(input: vscode.QuickInput, resolve: (shown: Shown) => void): (shown: Shown) => void {
  let done = false;
  const finish = (shown: Shown): void => {
    if (done) return;
    done = true;
    resolve(shown);
    input.dispose();
  };
  input.onDidHide(() => finish(undefined));
  return finish;
}

/** The choices, with a separator above each run of a section. */
function items(choices: Choice[]): ChoiceItem[] {
  const out: ChoiceItem[] = [];
  let section: string | undefined;
  for (const choice of choices) {
    if (choice.section !== undefined && choice.section !== section) {
      out.push({ label: choice.section, kind: vscode.QuickPickItemKind.Separator });
    }
    section = choice.section;
    out.push({
      label: choice.label,
      ...(choice.description === undefined ? {} : { description: choice.description }),
      ...(choice.detail === undefined ? {} : { detail: choice.detail }),
      value: choice.value,
    });
  }
  return out;
}
