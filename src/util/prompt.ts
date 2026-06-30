import * as clack from "@clack/prompts";

export interface SelectOption {
  value: string;
  label: string;
  disabled: boolean;
  hint?: string;
  selected: boolean;
}

/** Interactive multi-select. Returns chosen values. */
export async function multiSelect(title: string, options: SelectOption[]): Promise<string[]> {
  const result = await clack.multiselect({
    message: title,
    options: options.map((o) => ({
      value: o.value,
      label: o.label,
      hint: o.disabled ? (o.hint ?? "not available") : (o.hint ?? undefined),
    })),
    initialValues: options.filter((o) => o.selected && !o.disabled).map((o) => o.value),
    required: false,
  });

  if (clack.isCancel(result)) {
    return [];
  }

  return (result as string[]).filter((v) => {
    const opt = options.find((o) => o.value === v);
    return opt && !opt.disabled;
  });
}

/** Check if we're in an interactive terminal. */
export function isInteractive(): boolean {
  return process.stdout.isTTY === true;
}
