import { useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTheme } from "../theme";
import { Icon } from "./ui";
import type { AppTheme } from "../api";
const choices: {
  value: AppTheme;
  label: string;
  detail: string;
  icon: string;
}[] = [
  {
    value: "light",
    label: "Light",
    detail: "Always use light appearance",
    icon: "sun",
  },
  {
    value: "dark",
    label: "Dark",
    detail: "Always use dark appearance",
    icon: "moon",
  },
  {
    value: "reconx",
    label: "ReconX",
    detail: "Original burgundy workspace",
    icon: "palette",
  },
];
export default function ThemeMenu({
  location,
  rail = false,
}: {
  location: string;
  rail?: boolean;
}) {
  const theme = useTheme();
  const open = theme.activeMenu === location;
  const trigger = useRef<HTMLButtonElement>(null);
  const menu = useRef<HTMLDivElement>(null);
  const items = useRef<(HTMLButtonElement | null)[]>([]);
  const [position, setPosition] = useState({ left: 8, top: 8 });
  const close = (restoreFocus = true) => {
    theme.setActiveMenu(null);
    if (restoreFocus) trigger.current?.focus();
  };
  useLayoutEffect(() => {
    if (!open) return;
    const rect = trigger.current!.getBoundingClientRect();
    setPosition({
      left: Math.max(8, Math.min(rect.left, window.innerWidth - 288)),
      top: rail
        ? Math.max(
            8,
            rect.top -
              (menu.current?.getBoundingClientRect().height ?? 245) -
              8,
          )
        : rect.bottom + 8,
    });
    items.current[
      choices.findIndex((c) => c.value === theme.preference)
    ]?.focus();
    const dismiss = (e: PointerEvent) => {
      if (
        !menu.current?.contains(e.target as Node) &&
        !trigger.current?.contains(e.target as Node)
      )
        theme.setActiveMenu(null);
    };
    const resize = () => theme.setActiveMenu(null);
    document.addEventListener("pointerdown", dismiss);
    window.addEventListener("resize", resize);
    return () => {
      document.removeEventListener("pointerdown", dismiss);
      window.removeEventListener("resize", resize);
    };
  }, [open, theme.preference, theme.setActiveMenu, rail]);
  return (
    <>
      <button
        ref={trigger}
        type="button"
        className={
          rail ? "rail-item theme-trigger" : "icon-button theme-trigger"
        }
        aria-label={`${location} appearance`}
        title={`Appearance: ${theme.preference}`}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-controls={open ? "appearance-menu" : undefined}
        onClick={() => theme.setActiveMenu(open ? null : location)}
        onKeyDown={(e) => {
          if (e.key === "ArrowDown" || e.key === "ArrowUp") {
            e.preventDefault();
            theme.setActiveMenu(location);
          }
        }}
      >
        <Icon
          name={
            theme.preference === "reconx"
              ? "palette"
              : theme.preference === "dark"
                ? "moon"
                : "sun"
          }
          size={rail ? 21 : 19}
        />
      </button>
      {open &&
        createPortal(
          <div
            ref={menu}
            id="appearance-menu"
            role="menu"
            aria-label="Appearance menu"
            className="theme-menu"
            style={position}
            onKeyDown={(e) => {
              const index = items.current.findIndex(
                (item) => item === document.activeElement,
              );
              if (e.key === "Escape") {
                e.preventDefault();
                close();
              }
              if (e.key === "Tab") close();
              if (["ArrowDown", "ArrowUp", "Home", "End"].includes(e.key)) {
                e.preventDefault();
                const next =
                  e.key === "Home"
                    ? 0
                    : e.key === "End"
                      ? 2
                      : (index + (e.key === "ArrowDown" ? 1 : 2)) % 3;
                items.current[next]?.focus();
              }
            }}
          >
            <div className="theme-menu-heading">Appearance</div>
            {choices.map((choice, index) => (
              <button
                ref={(el) => {
                  items.current[index] = el;
                }}
                key={choice.value}
                type="button"
                role="menuitemradio"
                aria-checked={theme.preference === choice.value}
                onClick={() => {
                  void theme.setTheme(choice.value).catch(() => {});
                  close();
                }}
              >
                <Icon name={choice.icon} size={20} />
                <span>
                  <b>{choice.label}</b>
                  <small>{choice.detail}</small>
                </span>
                {theme.preference === choice.value && (
                  <Icon name="check" size={18} />
                )}
              </button>
            ))}
          </div>,
          document.body,
        )}
    </>
  );
}
