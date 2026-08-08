# apps/

Application: sdkwork-agents
Status: active
Owner: SDKWork maintainers
Specs: APPLICATION_SPEC.md, SDKWORK_WORKSPACE_SPEC.md

## Primary App Surface

The repository root is the primary runnable app surface.
The repository root `sdkwork.app.config.json` governs the primary application manifest.

## Directory Index

| Directory | Surface role | Runnable | Commercial MVP | Purpose | Entry |
| --- | --- | --- | --- | --- | --- |
| sdkwork-agents-pc | pc | yes | yes | SDKWork Agents PC production surface for IAM-backed Agent lifecycle, chat, and Drive media. | `sdkwork-agents-pc/` |
| sdkwork-agents-h5 | h5 | yes | yes | SDKWork Agents H5 application root. | `sdkwork-agents-h5/` |
| sdkwork-agents-mini-program | mini-program | yes | native agents list + H5 WebView bridge | Native agents list plus an H5 WebView bridge; no login flow or chat surface yet (pre-GA). | `sdkwork-agents-mini-program/` |
| sdkwork-agents-flutter-mobile | flutter-mobile | no | out of GA scope | Flutter scaffold kept for architecture tracking until an owned Dart app SDK exists. | `sdkwork-agents-flutter-mobile/` |

## Allowed Content

- Selected language/architecture application roots with `README.md`, `AGENTS.md`, `.sdkwork/`, and `specs/` when authored packages exist.
- Architecture-local `packages/`, `config/`, `src/`, `lib/`, `App/`, or `entry/` directories required by the owning architecture standard.

## Forbidden Content

- Repository-root API contracts, generated SDK workspaces, Rust crates, or deployment descriptors moved under `apps/`.
- Runtime secrets, user-private state, generated SDK transport output, or cross-application copied business logic.

## Related Specs

- `../sdkwork-specs/APPLICATION_SPEC.md`
- `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`
- `../sdkwork-specs/APP_CLIENT_ARCHITECTURE_ALIGNMENT_SPEC.md`

## Verification

```bash
node ../sdkwork-specs/tools/check-apps-directory-index.mjs --root .
```
