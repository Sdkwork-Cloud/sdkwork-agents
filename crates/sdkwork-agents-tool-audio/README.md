# sdkwork-agents-tool-audio

Audio media tool sub-crate for SDKWork Agents.

Owns the `audio` category of the extensible media tool family:

| tool_id | capability |
| --- | --- |
| `audio.speech.create` | Text-to-speech synthesis (`/v1/audio/speech`) |
| `audio.transcriptions.create` | Speech-to-text transcription (`/v1/audio/transcriptions`) |
| `audio.translations.create` | Audio translation (`/v1/audio/translations`) |
| `audio.voices.list` | Available voices (`/v1/audio/voices`) |

Every tool calls the cloudrouter gateway through
`sdkwork-agents-tool-cloudrouter` with the caller's auth token (account-pool
routing). The provider implements both the media tool contract
(`MediaToolProvider`) and the kernel `ToolProvider` SPI so it can later be
registered into an `AgentRuntime` unchanged.
