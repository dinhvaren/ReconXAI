import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import userEvent from "@testing-library/user-event";
import { Button, Dropdown, PasswordInput, Tabs } from "./components/controls";

afterEach(cleanup);
it("explains a disabled operation to keyboard users without allowing execution", async () => {
  const user = userEvent.setup();
  const execute = vi.fn();
  render(
    <Button
      disabled
      disabledReason="Approve the current plan first"
      onClick={execute}
    >
      Execute
    </Button>,
  );
  const action = screen.getByRole("button", { name: "Execute" });
  expect(action).toBeDisabled();
  expect(action).toHaveAccessibleDescription("Approve the current plan first");
  await user.tab();
  expect(document.activeElement).toHaveAttribute(
    "aria-label",
    "Approve the current plan first",
  );
  await user.keyboard("{Enter}");
  await user.click(action);
  expect(execute).not.toHaveBeenCalled();
});
it("keeps a credential masked until explicitly revealed without changing its value", async () => {
  const user = userEvent.setup();
  render(
    <PasswordInput aria-label="Credential" defaultValue="synthetic-only" />,
  );
  const input = screen.getByLabelText("Credential");
  expect(input).toHaveAttribute("type", "password");
  await user.click(screen.getByRole("button", { name: "Show credential" }));
  expect(input).toHaveAttribute("type", "text");
  expect(input).toHaveValue("synthetic-only");
  await user.click(screen.getByRole("button", { name: "Hide credential" }));
  expect(input).toHaveAttribute("type", "password");
});
it("retains the actual button and ref across asynchronous prerequisite changes", async () => {
  const execute = vi.fn();
  const { rerender } = render(
    <Button
      disabled
      disabledReason="Loading provider catalog"
      onClick={execute}
    >
      Configure
    </Button>,
  );
  const button = screen.getByRole("button", { name: "Configure" });
  rerender(<Button onClick={execute}>Configure</Button>);
  expect(screen.getByRole("button", { name: "Configure" })).toBe(button);
  expect(button).toBeEnabled();
  await userEvent.click(button);
  expect(execute).toHaveBeenCalledOnce();
});
it("menu keyboard navigation skips unavailable actions and restores trigger focus", async () => {
  const user = userEvent.setup();
  const run = vi.fn();
  render(
    <Dropdown
      label="Project actions"
      actions={[
        { label: "Open", run },
        { label: "Delete", run, disabledReason: "An analysis job is active" },
        { label: "Export", run },
      ]}
    />,
  );
  const trigger = screen.getByRole("button", { name: "Project actions" });
  await user.click(trigger);
  expect(screen.getByRole("menuitem", { name: "Open" })).toHaveFocus();
  await user.keyboard("{ArrowDown}");
  expect(screen.getByRole("menuitem", { name: "Export" })).toHaveFocus();
  await user.keyboard("{Escape}");
  expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  expect(trigger).toHaveFocus();
  expect(run).not.toHaveBeenCalled();
});
it("tabs expose selected state and support directional and boundary keys", async () => {
  const user = userEvent.setup();
  const change = vi.fn();
  render(
    <Tabs
      label="Extensions"
      items={[
        { id: "discover", label: "Discover" },
        { id: "installed", label: "Installed" },
      ]}
      value="discover"
      onChange={change}
    />,
  );
  await user.tab();
  await user.keyboard("{End}");
  expect(change).toHaveBeenCalledWith("installed");
  expect(screen.getByRole("tab", { name: "Installed" })).toHaveFocus();
});
