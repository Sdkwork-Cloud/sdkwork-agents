# sdkwork-agents-tool-video

Video media tool sub-crate for SDKWork Agents.

Owns the `video` category of the extensible media tool family:

| tool_id | capability |
| --- | --- |
| `video.create` | Video generation task submission (`/v1/videos`) |
| `video.retrieve` | Task status and asset retrieval (`/v1/videos/{id}`) |
| `video.list` | Video listing (`/v1/videos`) |
| `video.edits.create` | Video edit task (`/v1/videos/edits`) |
| `video.extensions.create` | Video extension task (`/v1/videos/extensions`) |
| `video.remix.create` | Video remix task (`/v1/videos/{id}/remix`) |
| `video.characters.create` | Character creation (`/v1/videos/characters`) |
| `video.characters.list` | Character retrieval (`/v1/videos/characters/{id}`) |

Video generation is asynchronous: `video.create` submits a task and returns
`{ "taskId": "..." }`; `video.retrieve` polls the task until the asset URL is
available. All calls use the caller's auth token for cloudrouter account-pool
routing.
