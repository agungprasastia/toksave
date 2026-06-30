import pc from "picocolors";

const BAR_WIDTH = 20;
const LABEL_WIDTH = 26;

export class Progress {
  private timer: ReturnType<typeof setInterval> | null = null;
  private progress = 0;
  private label = "";
  private tty = process.stdout.isTTY;

  start(label: string) {
    this.label = label;
    this.progress = 0;

    if (this.tty) {
      this.render();
      this.timer = setInterval(() => {
        // Fake progress that approaches 90%
        this.progress += (95 - this.progress) * 0.05;
        this.render();
      }, 50);
    } else {
      console.log(label);
    }
  }

  stop(message: string, isError = false) {
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }

    if (this.tty) {
      this.progress = isError ? this.progress : 100;

      const p = Math.floor(this.progress);
      const blocks = Math.floor((p / 100) * BAR_WIDTH);
      const color = isError ? pc.red : pc.green;
      const bar = color("█".repeat(blocks)) + pc.dim("█".repeat(BAR_WIDTH - blocks));
      const percent = isError ? pc.red(`${p}%`.padStart(4)) : pc.green(`${p}%`.padStart(4));

      // Remove ansi codes to measure actual length for perfect padding
      // biome-ignore lint/suspicious/noControlCharactersInRegex: Ansi color codes require control characters
      const plainMessage = message.replace(/\x1B\[\d+m/g, "");
      const padding = Math.max(0, LABEL_WIDTH - plainMessage.length);
      const spaces = " ".repeat(padding);

      process.stdout.write(`\x1b[2K\r  ${message}${spaces}[${bar}] ${percent}\n`);
    } else {
      console.log(`  ${message}`);
    }
  }

  update(text: string) {
    this.label = text;
    if (this.tty) this.render();
  }

  private render() {
    const p = Math.floor(this.progress);
    const blocks = Math.floor((p / 100) * BAR_WIDTH);
    const bar = pc.cyan("█".repeat(blocks)) + pc.dim("█".repeat(BAR_WIDTH - blocks));
    const percent = pc.cyan(`${p}%`.padStart(4));

    // biome-ignore lint/suspicious/noControlCharactersInRegex: Ansi color codes require control characters
    const plainLabel = this.label.replace(/\x1B\[\d+m/g, "");
    const padding = Math.max(0, LABEL_WIDTH - plainLabel.length);
    const spaces = " ".repeat(padding);

    process.stdout.write(`\x1b[2K\r  ${this.label}${spaces}[${bar}] ${percent}`);
  }
}
