export interface NavItem {
  name: string;
  icon: string;
  implemented: boolean;
}
export const navigation: { group: string; items: NavItem[] }[] = [
  {
    group: "Workspace",
    items: [
      { name: "Dashboard", icon: "grid", implemented: true },
      { name: "Projects", icon: "folder", implemented: true },
      { name: "Targets & Scope", icon: "crosshair", implemented: true },
    ],
  },
  {
    group: "Research",
    items: [
      { name: "Reconnaissance", icon: "radar", implemented: true },
      { name: "Web Security", icon: "globe", implemented: true },
      { name: "API Security", icon: "api", implemented: true },
      { name: "Network Security", icon: "network", implemented: false },
      { name: "Reverse Engineering", icon: "binary", implemented: true },
      { name: "Mobile Security", icon: "phone", implemented: false },
      { name: "Cloud & Containers", icon: "cloud", implemented: false },
      { name: "CI/CD Security", icon: "pipeline", implemented: false },
    ],
  },
  {
    group: "Intelligence",
    items: [
      { name: "AI Providers", icon: "ai", implemented: true },
      { name: "AI Workspace", icon: "ai", implemented: true },
      { name: "Agent Workflows", icon: "workflow", implemented: true },
      { name: "MCP Hub", icon: "plug", implemented: true },
      { name: "Tool Manager", icon: "wrench", implemented: true },
      { name: "Extensions", icon: "blocks", implemented: true },
    ],
  },
  {
    group: "Records",
    items: [
      { name: "Findings", icon: "flag", implemented: true },
      { name: "Evidence", icon: "archive", implemented: true },
      { name: "Reports", icon: "file", implemented: true },
    ],
  },
];
export const routes = [
  ...navigation.flatMap((g) => g.items),
  { name: "Settings", icon: "settings", implemented: true },
];
