# sdkwork-agents-tool-sound-effect

Sound-effect media tool sub-crate for SDKWork Agents.

Owns the `sound-effect` category of the extensible media tool family:

| tool_id | capability |
| --- | --- |
| `sound-effect.generate` | Sound effect generation (capability pending) |

The tool definition and category taxonomy are in place; invocation returns a
`capability_missing` error because the cloudrouter open-api surface does not
yet expose a sound-effect endpoint. When the upstream surface opens, the
implementation slots into this crate without touching the taxonomy or any
other category crate.
