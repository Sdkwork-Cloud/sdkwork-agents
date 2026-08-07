# sdkwork-agents-tool-image

Image media tool sub-crate for SDKWork Agents.

Owns the `image` category of the extensible media tool family:

| tool_id | capability |
| --- | --- |
| `image.generations.create` | Text-to-image generation (`/v1/images/generations`) |
| `image.edits.create` | Image edit (`/v1/images/edits`) |
| `image.variations.create` | Image variation (`/v1/images/variations`) |

Vendor wire fields (e.g. `image_url`) are normalized into `MediaResource`
shape (`kind`/`source`/`url`) before leaving the provider, per
`MEDIA_RESOURCE_SPEC`. All calls use the caller's auth token for cloudrouter
account-pool routing.
