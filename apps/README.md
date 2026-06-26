# apps/

Application: sdkwork-agents
Status: active
Owner: SDKWork maintainers
Specs: APPLICATION_SPEC.md, SDKWORK_WORKSPACE_SPEC.md

## Primary App Surface

The repository root is the primary runnable app surface.
The repository root `sdkwork.app.config.json` governs the primary application manifest.

## Directory Index

| Directory | Surface role | Runnable | Purpose | Entry |
| --- | --- | --- | --- | --- |
| sdkwork-agents-flutter-mobile | flutter-mobile | yes | SDKWork Agents Mobile flutter-mobile application root. | `sdkwork-agents-flutter-mobile/` |
| sdkwork-agents-h5 | h5 | yes | SDKWork Agents H5 h5 application root. | `sdkwork-agents-h5/` |
| sdkwork-agents-mini-program | mini-program | yes | SDKWork Agents Mini Program mini-program application root. | `sdkwork-agents-mini-program/` |
| sdkwork-agents-pc | pc | yes | SDKWork Agents PC pc application root. | `sdkwork-agents-pc/` |

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
