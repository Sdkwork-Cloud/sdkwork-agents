import type { ReactNode } from "react";

export { AGENTS_MODULE_ID, CREATE_AGENT_ROUTE, AGENTS_SHELL_MODULES } from "./moduleRegistry";

export interface AgentsRouteDefinition {
  path: string;
  element: ReactNode;
}

export function createAgentsShellRoutes(routes: AgentsRouteDefinition[]) {
  return routes;
}
