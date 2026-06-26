import { CREATE_AGENT_ROUTE } from "@sdkwork/agents-pc-shell";

export function createRoutes() {
  return [
    { path: "/", label: "agents-home" },
    { path: `/${CREATE_AGENT_ROUTE}`, label: "create-agent" },
  ];
}
