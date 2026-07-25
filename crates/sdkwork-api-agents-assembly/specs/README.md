# sdkwork-api-agents-assembly Specs

Component root: `crates/sdkwork-api-agents-assembly`

API assembly manifest, business-router composition, and verification contract.

`assemble_app_api_contribution` is the dependency-integration entrypoint. It returns one
host-neutral contribution containing the unwrapped executable App API router, route manifest,
OpenAPI document, permission catalog, Agents domain-context injector, and readiness check. The
route manifest, OpenAPI auth inventory, and permission catalog are all derived from the same
generated App API route inventory. Agents background reconciliation remains owner-internal and is
not part of the gateway composition contract.

The contribution does not install CORS, Web Framework middleware, infrastructure probe paths, or
listener policy. A composing gateway merges all selected owner contributions, installs every
domain-context injector, and then applies one combined Web Framework pipeline. In production the
Agents state uses the canonical PostgreSQL store; explicit development auth bypass uses the
in-memory repository.
