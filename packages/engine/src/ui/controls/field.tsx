import { Input, Show, Text, View, formatNumber, packClass, type UiEvent } from "@origingame/engine/ui";

export interface TextFieldProps {
  /** The field's visible label, also its name for screen readers. */
  label: string;
  /** The value; the game owns it and writes back each `onChange`. */
  value?: string | undefined;
  placeholder?: string | undefined;
  /** A line under the field explaining it; replaced by `error` while there is one. */
  description?: string | undefined;
  /** What is wrong with the value; non-empty marks the field invalid. */
  error?: string | undefined;
  /** Marks the field invalid without words (the error is shown elsewhere). */
  invalid?: boolean | undefined;
  /** A mark after the label; the game still checks the value. */
  required?: boolean | undefined;
  /** The most characters it takes; shows a count of them under the field. */
  maxLength?: number | undefined;
  type?: "text" | "password" | "number" | "email" | "url" | "tel" | "search" | undefined;
  disabled?: boolean | undefined;
  readOnly?: boolean | undefined;
  autofocus?: boolean | undefined;
  /** Classes after the field group's own. */
  class?: string | undefined;
  onChange?: ((e: UiEvent, value: string) => void) | undefined;
  /** Enter pressed in a single-line field. */
  onSubmit?: ((e: UiEvent) => void) | undefined;
}

export interface TextAreaProps extends Omit<TextFieldProps, "type" | "onSubmit"> {
  /** Lines tall at least (default 3). */
  rows?: number | undefined;
}

function FieldShell(props: TextFieldProps & { children: any }) {
  const invalid = () => props.invalid === true || (props.error ?? "") !== "";
  const length = () => (props.value ?? "").length;
  return (
    <View class={`flex-col gap-1.5 w-full ${props.class ?? ""}`} data-invalid={invalid()} data-disabled={props.disabled === true}>
      <View class="flex-row items-baseline gap-1">
        <Text class={packClass("Field", "label")}>{props.label}</Text>
        <Show when={props.required}>
          <Text class="text-danger font-bold" hidden>*</Text>
        </Show>
      </View>
      {props.children}
      <Show when={invalid() || props.description || props.maxLength}>
        <View class="flex-row items-start gap-2">
          <Show
            when={(props.error ?? "") !== ""}
            fallback={<Text class={`grow shrink basis-0 ${packClass("Field", "hint")}`}>{props.description ?? ""}</Text>}
          >
            <Text class={`grow shrink basis-0 ${packClass("Field", "error")}`} role="alert">{props.error}</Text>
          </Show>
          <Show when={props.maxLength}>
            <Text class={`shrink-0 tabular-nums ${packClass("Field", "hint")} data-over:text-danger`} data-over={length() >= (props.maxLength ?? 0)}>
              {`${formatNumber(length())} / ${formatNumber(props.maxLength ?? 0)}`}
            </Text>
          </Show>
        </View>
      </Show>
    </View>
  );
}

/**
 * A labelled single-line text field: the label (with a required mark), the
 * pack's field box (`Field.frame`, every state), and under it the
 * description, or the error while there is one, and a character count when
 * it has a `maxLength`. The game owns the value.
 */
export function TextField(props: TextFieldProps) {
  const invalid = () => props.invalid === true || (props.error ?? "") !== "";
  return (
    <FieldShell {...props}>
      <Input
        class={`w-full ${packClass("Field", "frame")}`}
        label={props.label}
        description={props.error || props.description}
        value={props.value ?? ""}
        placeholder={props.placeholder}
        type={props.type}
        maxLength={props.maxLength}
        readOnly={props.readOnly}
        disabled={props.disabled}
        invalid={invalid()}
        autofocus={props.autofocus}
        onChange={(e) => props.onChange?.(e, props.maxLength ? e.value.slice(0, props.maxLength) : e.value)}
        onKey={(e) => {
          if (e.key === "Enter") props.onSubmit?.(e);
        }}
      />
    </FieldShell>
  );
}

/** A labelled multi-line text field, as TextField, `rows` lines tall at least (`Field.area`). */
export function TextArea(props: TextAreaProps) {
  const invalid = () => props.invalid === true || (props.error ?? "") !== "";
  return (
    <FieldShell {...props}>
      <Input
        class={`w-full ${packClass("Field", "area")}`}
        label={props.label}
        description={props.error || props.description}
        value={props.value ?? ""}
        placeholder={props.placeholder}
        multiline
        rows={props.rows ?? 3}
        maxLength={props.maxLength}
        readOnly={props.readOnly}
        disabled={props.disabled}
        invalid={invalid()}
        autofocus={props.autofocus}
        onChange={(e) => props.onChange?.(e, props.maxLength ? e.value.slice(0, props.maxLength) : e.value)}
      />
    </FieldShell>
  );
}

const change = () => {};

export const ground = "panel";

export const fixtures: Record<string, Partial<TextFieldProps>> = {
  default: { label: "Display name", placeholder: "Your name", value: "", onChange: change },
  described: { label: "Clan tag", description: "2 to 5 letters.", value: "NYX", maxLength: 5, required: true, onChange: change },
  error: { label: "Email", value: "nyx@", error: "Enter a full address.", type: "email", onChange: change },
  disabled: { label: "Server", value: "eu-west", disabled: true },
};
