/** Something that can be let go of, as `vscode.Disposable` is. */
export interface Disposable {
  dispose(): unknown;
}

/** An event to listen to, shaped like `vscode.Event` so it can be given where one is expected. */
export type Event<T> = (
  listener: (value: T) => unknown,
  thisArgs?: unknown,
  disposables?: Disposable[],
) => Disposable;

/**
 * `vscode.EventEmitter` for modules that don't load VS Code, so the unit
 * tests can run them. Listeners are called in the order they were added.
 */
export class Emitter<T> {
  private listeners: ((value: T) => unknown)[] = [];

  readonly event: Event<T> = (listener, thisArgs, disposables) => {
    const bound = (value: T): unknown => listener.call(thisArgs, value);
    this.listeners.push(bound);
    const subscription = {
      dispose: () => {
        this.listeners = this.listeners.filter((other) => other !== bound);
      },
    };
    disposables?.push(subscription);
    return subscription;
  };

  fire(value: T): void {
    for (const listener of this.listeners) listener(value);
  }

  dispose(): void {
    this.listeners = [];
  }
}
