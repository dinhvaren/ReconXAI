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
      { name: "Targets & Scope", icon: "shield", implemented: true },
    ],
  },
  {
    group: "Research",
    items: [
      { name: "Reconnaissance", icon: "globe", implemented: true },
      { name: "Web Security", icon: "code", implemented: false },
      { name: "API Security", icon: "api", implemented: false },
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
      { name: "AI Workspace", icon: "ai", implemented: true },
      { name: "Agent Workflows", icon: "workflow", implemented: true },
      { name: "MCP Hub", icon: "plug", implemented: true },
      { name: "Tool Manager", icon: "terminal", implemented: true },
    ],
  },
  {
    group: "Records",
    items: [
      { name: "Findings", icon: "flag", implemented: true },
      { name: "Evidence", icon: "layers", implemented: true },
      { name: "Reports", icon: "file", implemented: true },
      { name: "Settings", icon: "settings", implemented: true },
    ],
  },
];
export const routes = navigation.flatMap((g) => g.items);
