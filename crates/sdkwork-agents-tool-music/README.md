# sdkwork-agents-tool-music

Music media tool sub-crate for SDKWork Agents.

Owns the `music` category of the extensible media tool family, backed by the
cloudrouter Suno-compatible music generation surface:

| tool_id | capability |
| --- | --- |
| `music.generations.create` | Music generation task submission (`/suno/v1/music/generations`) |
| `music.generations.list` | Music task status and asset retrieval (`/suno/v1/music/generations/{task_id}`) |

Music generation is asynchronous: `music.generations.create` returns
`{ "taskId": "..." }`; `music.generations.list` polls the task until audio
assets are available. All calls use the caller's auth token for cloudrouter
account-pool routing.
