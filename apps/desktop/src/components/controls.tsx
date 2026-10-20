import {
  useEffect,
  useId,
  useRef,
  useState,
  type ComponentPropsWithRef,
  type InputHTMLAttributes,
  type ReactNode,
  type SelectHTMLAttributes,
} from "react";
import { Icon } from "./icons";

export type ButtonProps = ComponentPropsWithRef<"button"> & {
  variant?: "primary" | "secondary" | "ghost" | "destructive";
  size?: "compact" | "standard" | "large";
  disabledReason?: string;
};
export function Tooltip({
  label,
  children,
  disabled = false,
}: {
  label: string;
  children: ReactNode;
  disabled?: boolean;
}) {
  const id = useId();
  return (
    <span
      className="control-hint"
      tabIndex={disabled ? 0 : undefined}
      aria-label={disabled ? label : undefined}
    >
      {children}
      <span id={id} role="tooltip" className="control-tooltip">
        {label}
      </span>
    </span>
  );
}
export function Button({
  variant,
  size = "standard",
  disabledReason,
  className = "",
  children,
  ...props
}: ButtonProps) {
  const id = useId();
  const reason = props.disabled ? disabledReason || props.title : undefined;
  const classes = className || `button ${variant ?? "secondary"}`;
  const button = (
    <button
      {...props}
      className={`${classes} control-${size}`}
      title={reason || props.title}
      aria-describedby={
        [props["aria-describedby"], reason ? id : undefined]
          .filter(Boolean)
          .join(" ") || undefined
      }
    >
      {children}
    </button>
  );
  // Keep the native button mounted when an async prerequisite changes. Replacing
  // it on disabled->enabled loses focus, refs and an in-flight pointer gesture.
  return (
    <span
      className={reason ? "control-hint" : "control-wrap"}
      tabIndex={reason ? 0 : undefined}
      aria-label={reason}
    >
      {button}
      {reason && (
        <span id={id} role="tooltip" className="control-tooltip">
          {reason}
        </span>
      )}
    </span>
  );
}
export function IconButton({
  icon,
  label,
  ...props
}: Omit<ButtonProps, "children"> & { icon: string; label: string }) {
  return (
    <Button
      {...props}
      className={props.className ?? "icon-button"}
      aria-label={label}
      title={props.title ?? label}
    >
      <Icon name={icon} />
    </Button>
  );
}
export function Toolbar({
  children,
  label,
}: {
  children: ReactNode;
  label: string;
}) {
  return (
    <div className="rx-toolbar" role="group" aria-label={label}>
      {children}
    </div>
  );
}
export function PanelHeader({
  title,
  children,
}: {
  title: string;
  children?: ReactNode;
}) {
  return (
    <div className="panel-heading">
      <h2>{title}</h2>
      {children}
    </div>
  );
}
export function EmptyState({
  title,
  description,
  icon = "folder",
  action,
  secondary,
}: {
  title: string;
  description: string;
  icon?: string;
  action?: ReactNode;
  secondary?: ReactNode;
}) {
  return (
    <div className="rx-empty">
      <Icon name={icon} size={28} />
      <div>
        <h2>{title}</h2>
        <p>{description}</p>
      </div>
      {(action || secondary) && (
        <div className="inline-actions">
          {action}
          {secondary}
        </div>
      )}
    </div>
  );
}
export function LoadingState({ label = "Loading" }: { label?: string }) {
  return (
    <div role="status" className="rx-loading">
      <Icon name="loading" />
      {label}
    </div>
  );
}
export function TextInput(props: InputHTMLAttributes<HTMLInputElement>) {
  return <input {...props} className={`rx-input ${props.className ?? ""}`} />;
}
export function SearchField(
  props: Omit<InputHTMLAttributes<HTMLInputElement>, "type">,
) {
  return (
    <span className="rx-search">
      <Icon name="search" size={16} />
      <TextInput {...props} type="search" />
    </span>
  );
}
export function PasswordInput(
  props: Omit<InputHTMLAttributes<HTMLInputElement>, "type">,
) {
  const [visible, setVisible] = useState(false);
  return (
    <span className="rx-password">
      <TextInput
        {...props}
        type={visible ? "text" : "password"}
        autoComplete={props.autoComplete ?? "off"}
      />
      <IconButton
        icon={visible ? "eye-off" : "eye"}
        label={visible ? "Hide credential" : "Show credential"}
        type="button"
        onClick={() => setVisible(!visible)}
      />
    </span>
  );
}
export function Select(props: SelectHTMLAttributes<HTMLSelectElement>) {
  return <select {...props} className={`rx-select ${props.className ?? ""}`} />;
}
export function Checkbox({
  label,
  ...props
}: Omit<InputHTMLAttributes<HTMLInputElement>, "type"> & { label: string }) {
  return (
    <label className="rx-checkbox">
      <input {...props} type="checkbox" />
      {label}
    </label>
  );
}
export function Switch({
  label,
  ...props
}: Omit<InputHTMLAttributes<HTMLInputElement>, "type"> & { label: string }) {
  return (
    <label className="rx-checkbox">
      <input {...props} type="checkbox" role="switch" />
      {label}
    </label>
  );
}
export function ProgressBar({
  value,
  max = 100,
  label,
}: {
  value?: number;
  max?: number;
  label: string;
}) {
  return <progress aria-label={label} value={value} max={max} />;
}
export function DataTable({
  headings,
  children,
  label,
}: {
  headings: string[];
  children: ReactNode;
  label: string;
}) {
  return (
    <div className="table-scroll">
      <table aria-label={label}>
        <thead>
          <tr>
            {headings.map((h) => (
              <th key={h} scope="col">
                {h}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>{children}</tbody>
      </table>
    </div>
  );
}
export function Tabs({
  items,
  value,
  onChange,
  label,
}: {
  items: { id: string; label: string }[];
  value: string;
  onChange: (value: string) => void;
  label: string;
}) {
  const ref = useRef<HTMLDivElement>(null);
  return (
    <div
      ref={ref}
      role="tablist"
      className="rx-tabs"
      aria-label={label}
      onKeyDown={(event) => {
        const index = items.findIndex((item) => item.id === value);
        const next =
          event.key === "Home"
            ? 0
            : event.key === "End"
              ? items.length - 1
              : event.key === "ArrowRight"
                ? (index + 1) % items.length
                : event.key === "ArrowLeft"
                  ? (index + items.length - 1) % items.length
                  : null;
        if (next !== null) {
          event.preventDefault();
          onChange(items[next].id);
          ref.current
            ?.querySelectorAll<HTMLButtonElement>("button")
            [next]?.focus();
        }
      }}
    >
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          role="tab"
          aria-selected={value === item.id}
          tabIndex={value === item.id ? 0 : -1}
          onClick={() => onChange(item.id)}
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}
export type MenuAction = {
  label: string;
  run: () => void;
  disabledReason?: string;
  destructive?: boolean;
};
export function Dropdown({
  label,
  icon = "more",
  actions,
}: {
  label: string;
  icon?: string;
  actions: MenuAction[];
}) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const id = useId();
  useEffect(() => {
    if (!open) return;
    ref.current
      ?.querySelector<HTMLButtonElement>('[role="menuitem"]:not(:disabled)')
      ?.focus();
    const close = (e: PointerEvent) => {
      if (!ref.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("pointerdown", close);
    return () => document.removeEventListener("pointerdown", close);
  }, [open]);
  return (
    <div className="rx-dropdown" ref={ref}>
      <button
        ref={trigger}
        className="icon-button"
        type="button"
        aria-label={label}
        aria-haspopup="menu"
        aria-controls={open ? id : undefined}
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        onKeyDown={(e) => {
          if (e.key === "ArrowDown") {
            e.preventDefault();
            setOpen(true);
          }
        }}
      >
        <Icon name={icon} />
      </button>
      {open && (
        <div
          id={id}
          role="menu"
          aria-label={label}
          className="rx-menu"
          onKeyDown={(e) => {
            const buttons = [
              ...(ref.current?.querySelectorAll<HTMLButtonElement>(
                '[role="menuitem"]:not(:disabled)',
              ) ?? []),
            ];
            const index = buttons.indexOf(
              document.activeElement as HTMLButtonElement,
            );
            if (e.key === "Escape") {
              e.preventDefault();
              setOpen(false);
              trigger.current?.focus();
            }
            if (e.key === "Tab") setOpen(false);
            const next =
              e.key === "Home"
                ? 0
                : e.key === "End"
                  ? buttons.length - 1
                  : e.key === "ArrowDown"
                    ? (index + 1) % buttons.length
                    : e.key === "ArrowUp"
                      ? (index + buttons.length - 1) % buttons.length
                      : null;
            if (next !== null) {
              e.preventDefault();
              buttons[next]?.focus();
            }
          }}
        >
          {actions.map((action) => (
            <Button
              key={action.label}
              className={`rx-menu-item ${action.destructive ? "danger" : ""}`}
              role="menuitem"
              type="button"
              disabled={!!action.disabledReason}
              disabledReason={action.disabledReason}
              onClick={() => {
                setOpen(false);
                trigger.current?.focus();
                action.run();
              }}
            >
              {action.label}
            </Button>
          ))}
        </div>
      )}
    </div>
  );
}
export function ContextMenu(
  props: Omit<Parameters<typeof Dropdown>[0], "icon">,
) {
  return <Dropdown {...props} icon="more" />;
}
export function SplitButton({
  label,
  onClick,
  actions,
  ...props
}: ButtonProps & { label: string; actions: MenuAction[] }) {
  return (
    <div className="rx-split">
      <Button {...props} onClick={onClick}>
        {label}
      </Button>
      <Dropdown label={`${label} options`} icon="down" actions={actions} />
    </div>
  );
}
export function Drawer({
  title,
  onClose,
  children,
  className = "",
  closeLabel,
}: {
  title: string;
  onClose: () => void;
  children: ReactNode;
  className?: string;
  closeLabel?: string;
}) {
  return (
    <aside className={`rx-drawer ${className}`} aria-label={title}>
      <PanelHeader title={title}>
        <IconButton
          label={closeLabel ?? `Close ${title}`}
          icon="close"
          onClick={onClose}
        />
      </PanelHeader>
      {children}
    </aside>
  );
}
export function Toast({
  children,
  error = false,
  onDismiss,
}: {
  children: ReactNode;
  error?: boolean;
  onDismiss: () => void;
}) {
  return (
    <div
      role={error ? "alert" : "status"}
      className={`banner ${error ? "error" : "success"}`}
    >
      <Icon name={error ? "flag" : "check"} />
      <span>{children}</span>
      <IconButton
        label="Dismiss notification"
        icon="close"
        onClick={onDismiss}
      />
    </div>
  );
}
