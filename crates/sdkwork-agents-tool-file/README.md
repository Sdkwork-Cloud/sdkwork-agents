# sdkwork-agents-tool-file

File media tool sub-crate for SDKWork Agents.

Owns the `file` category of the extensible media tool family — the gateway-side
file store that feeds every other media tool:

| tool_id | capability |
| --- | --- |
| `file.upload` | Register a file (URL or reference) on the gateway (`/v1/files`) |
| `file.list` | List gateway files |
| `file.retrieve` | Retrieve one file's metadata |
| `file.delete` | Delete a gateway file |
| `file.content` | Fetch a file's content |

`file.upload` returns a gateway `file_id` that the audio transcription/
translation, image edit, and image/video generation tools accept as their
`file` reference — this closes the input chain that makes those tools usable
end to end. All calls use the caller's auth token for cloudrouter
account-pool routing.
