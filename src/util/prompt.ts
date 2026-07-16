import readline from "node:readline";
import pc from "picocolors";

export interface SelectOption {
  value: string;
  label: string;
  disabled: boolean;
  hint?: string;
  selected: boolean;
}

/** Check if we're in an interactive terminal. */
export function isInteractive(): boolean {
  return process.stdout.isTTY === true;
}

export async function confirm(message: string, defaultValue = false): Promise<boolean> {
  if (!isInteractive()) return defaultValue;
  return new Promise((resolve) => {
    const { stdin, stdout } = process;
    const hint = defaultValue ? "(Y/n)" : "(y/N)";
    stdout.write(`${pc.cyan("?")} ${message} ${pc.dim(hint)} `);
    readline.emitKeypressEvents(stdin);
    if (stdin.isTTY) stdin.setRawMode(true);
    stdin.resume();
    const onKey = (str: string) => {
      const lower = str.toLowerCase();
      if (lower === "y") {
        cleanup();
        resolve(true);
      } else if (lower === "n") {
        cleanup();
        resolve(false);
      } else if (str === "\r" || str === "\n") {
        cleanup();
        resolve(defaultValue);
      } else if (str === "\u0003") {
        cleanup();
        process.exit(1);
      }
    };
    function cleanup() {
      stdout.write("\n");
      if (stdin.isTTY) stdin.setRawMode(false);
      stdin.removeListener("data", onKey);
      stdin.pause();
    }
    stdin.on("data", onKey);
  });
}

export async function multiSelect(title: string, options: SelectOption[]): Promise<string[]> {
  if (!isInteractive()) {
    return options.filter((o) => o.selected && !o.disabled).map((o) => o.value);
  }

  const items = options.map((o) => ({ ...o }));
  let cursor = items.findIndex((o) => !o.disabled);
  if (cursor === -1) cursor = 0;

  return new Promise((resolve) => {
    const { stdin, stdout } = process;
    stdout.write("\x1B[?25l"); // Hide cursor

    let firstRender = true;
    const render = () => {
      if (!firstRender) {
        stdout.write(`\x1B[${items.length + 1}A`);
      }
      firstRender = false;

      // Title
      stdout.write(`\x1B[2K\r${pc.cyan("?")} ${pc.bold(title)}\n`);
      // Instructions
      const upDown = pc.yellow("↑/↓");
      const space = pc.yellow("<space>");
      const aKey = pc.yellow("<a>");
      const iKey = pc.yellow("<i>");
      const enterKey = pc.yellow("<enter>");
      stdout.write(
        `\x1B[2K\r  ${upDown} move ${pc.dim("·")} ${space} select ${pc.dim("·")} ${aKey} all ${pc.dim("·")} ${iKey} invert ${pc.dim("·")} ${enterKey} confirm\n`,
      );

      // Calculate max label length for perfect column alignment
      const maxLen = Math.max(...items.map((i) => i.label.length));

      for (let i = 0; i < items.length; i++) {
        const item = items[i];
        if (!item) continue;
        const isHovered = i === cursor;

        let prefix = "  ";
        if (isHovered) prefix = pc.cyan("❯ ");

        let box = item.selected ? pc.green("◉") : "◯";
        if (item.disabled) box = pc.dim("·");

        const paddedLabel = item.label.padEnd(maxLen);
        const tag = item.disabled ? pc.yellow("[MISSING]") : pc.green("[READY]");
        const hint = item.hint ? pc.dim(item.hint) : "";

        const line = `\x1b[2K\r${prefix}${box}  ${paddedLabel}  ${tag}  ${hint}`;
        if (i === items.length - 1) {
          stdout.write(line); // No newline on last item to prevent scrolling
        } else {
          stdout.write(`${line}\n`);
        }
      }
    };

    const cleanup = () => {
      stdout.write(`\x1B[${items.length + 1}A\x1B[J`); // Moved up one less since we didn't print final \n
      const selectedLabels = items.filter((i) => i.selected && !i.disabled).map((i) => i.label);
      const answer = selectedLabels.length > 0 ? selectedLabels.join(", ") : "None";
      stdout.write(
        `\x1B[2K\r${pc.cyan("?")} ${pc.bold(title)} ${pc.dim("·")} ${pc.cyan(answer)}\n`,
      );
      stdout.write("\x1B[?25h"); // Show cursor
      if (stdin.isTTY) stdin.setRawMode(false);
      stdin.removeListener("keypress", onKeyPress);
      stdin.pause();
    };

    render();

    readline.emitKeypressEvents(stdin);
    if (stdin.isTTY) stdin.setRawMode(true);
    stdin.resume();

    const onKeyPress = (_str: string, key: readline.Key) => {
      if (key.ctrl && key.name === "c") {
        cleanup();
        process.exit(1);
      }
      if (key.name === "up" || key.name === "k") {
        let next = cursor - 1;
        while (next >= 0 && items[next]?.disabled) next--;
        if (next >= 0) {
          cursor = next;
          render();
        }
      } else if (key.name === "down" || key.name === "j") {
        let next = cursor + 1;
        while (next < items.length && items[next]?.disabled) next++;
        if (next < items.length) {
          cursor = next;
          render();
        }
      } else if (key.name === "space") {
        const current = items[cursor];
        if (current && !current.disabled) {
          current.selected = !current.selected;
          render();
        }
      } else if (key.name === "a") {
        const allSelected = items.every((i) => i.disabled || i.selected);
        for (const item of items) {
          if (!item.disabled) item.selected = !allSelected;
        }
        render();
      } else if (key.name === "i") {
        for (const item of items) {
          if (!item.disabled) item.selected = !item.selected;
        }
        render();
      } else if (key.name === "return") {
        cleanup();
        resolve(items.filter((i) => i.selected && !i.disabled).map((i) => i.value));
      }
    };

    stdin.on("keypress", onKeyPress);
  });
}
