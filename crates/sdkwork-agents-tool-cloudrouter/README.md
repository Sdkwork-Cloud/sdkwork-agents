# sdkwork-agents-tool-cloudrouter

Shared sdkwork-cloudrouter open-api client adapter for the SDKWork Agents media
tool family.

Every media tool category crate (audio/video/music/sound-effect/image) calls
the cloudrouter gateway through this adapter instead of constructing raw HTTP
calls. The adapter owns:

- gateway base URL resolution (`SDKWORK_AGENTS_CLOUDROUTER_BASE_URL`, default
  `http://127.0.0.1:3900`), shared with the chat turn executor;
- auth-token injection (`Authorization: Bearer <auth token>`) for cloudrouter
  account-pool routing — the caller's login token selects the tenant account
  group upstream, no API key required;
- a dedicated blocking Tokio runtime so synchronous kernel
  `ToolProvider::invoke_tool` calls can drive async SDK calls;
- error mapping from cloudrouter SDK errors to the media tool error taxonomy
  with actionable hints for common gateway failures.
