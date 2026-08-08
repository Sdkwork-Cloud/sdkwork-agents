-- Minimal bootstrap seed for agents.
--
-- Agents owns no default business content: workspaces, projects, sessions,
-- agents and tasks are all user-created. This script exists as the
-- deterministic baseline marker for the `standard` seed profile; it must
-- remain a no-op so seed runs never fabricate tenant rows.
SELECT 1;
