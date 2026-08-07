# sdkwork-agents-tool-intelligence

Intelligence tool sub-crate for SDKWork Agents.

Owns the `intelligence` category of the extensible media tool family — model
discovery and model-adjacent intelligence surfaces on the cloudrouter gateway:

| tool_id | capability |
| --- | --- |
| `model.list` | Discover models available to the caller's account pool (`/v1/models`) |
| `embedding.create` | Text embedding / vectorization (`/v1/embeddings`) |
| `moderation.create` | Content safety moderation (`/v1/moderations`) |

`model.list` lets agents discover the account-pool-available models instead of
hard-coding `default`, and `moderation.create` provides content-safety gating
for generated media flows. All calls use the caller's auth token for
cloudrouter account-pool routing.
