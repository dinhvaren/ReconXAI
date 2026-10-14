import { useEffect, useRef, useState } from "react";
import { Icon } from "./ui";
export default function QuickActions({
  run,
}: {
  run: (action: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (!open) return;
    const close = (e: PointerEvent) => {
      if (!ref.current?.contains(e.target as Node)) setOpen(false);
    };
    document.addEventListener("pointerdown", close);
    return () => document.removeEventListener("pointerdown", close);
  }, [open]);
  return (
    <div className="quick-actions" ref={ref}>
      <button
        ref={trigger}
        className="icon-button"
        aria-label="Quick actions"
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        onKeyDown={(e) => {
          if (e.key === "ArrowDown") {
            e.preventDefault();
            setOpen(true);
            setTimeout(
              () =>
                ref.current
                  ?.querySelector<HTMLButtonElement>('[role="menuitem"]')
                  ?.focus(),
              0,
            );
          }
        }}
      >
        <Icon name="plus" />
      </button>
      {open && (
        <div
          className="quick-actions-menu"
          role="menu"
          aria-label="Quick actions menu"
          onKeyDown={(e) => {
            const buttons = Array.from(
              ref.current?.querySelectorAll<HTMLButtonElement>(
                '[role="menuitem"]:not(:disabled)',
              ) ?? [],
            );
            const index = buttons.indexOf(
              document.activeElement as HTMLButtonElement,
            );
            if (e.key === "Escape") {
              setOpen(false);
              trigger.current?.focus();
            }
            if (e.key === "ArrowDown" || e.key === "ArrowUp") {
              e.preventDefault();
              buttons[
                (index + (e.key === "ArrowDown" ? 1 : buttons.length - 1)) %
                  buttons.length
              ]?.focus();
            }
          }}
        >
          <span>Workspace actions</span>
          {[
            "New Project",
            "Add Target",
            "Import File",
            "Import Repository",
            "View Jobs",
            "Open AI Workspace",
            "Install Tools",
          ].map((action) => (
            <button
              role="menuitem"
              key={action}
              onClick={() => {
                setOpen(false);
                run(action);
              }}
            >
              {action}
            </button>
          ))}
          <button
            role="menuitem"
            disabled
            title="MCP server management is not implemented yet"
          >
            Add MCP Server — unavailable
          </button>
          <button
            role="menuitem"
            disabled
            title="Target workflow orchestration is not implemented yet"
          >
            Auto Research — unavailable
          </button>
        </div>
      )}
    </div>
  );
}
