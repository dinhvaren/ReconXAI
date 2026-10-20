import { Dropdown } from "./controls";
export default function QuickActions({
  run,
}: {
  run: (action: string) => void;
}) {
  return (
    <Dropdown
      label="Quick actions"
      icon="plus"
      actions={[
        "New Project",
        "Add Target",
        "Start AI Research",
        "Import File",
        "Import Repository",
        "Start Recon",
        "Start API Testing",
        "Plan local analysis",
        "Add MCP Server",
        "Install Tools",
        "Create Finding",
        "Export Report",
        "View Jobs",
      ].map((label) => ({ label, run: () => run(label) }))}
    />
  );
}
