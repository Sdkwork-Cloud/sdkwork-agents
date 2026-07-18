#!/bin/sh
set -eu

fail() {
  echo "$1" >&2
  exit "${2:-64}"
}

require_profile_value() {
  if [ -z "$2" ]; then
    fail "selected profile must declare $1" 65
  fi
}

verify_existing_security_value() {
  key=$1
  expected=$2
  if printenv "$key" >/dev/null 2>&1; then
    actual=$(printenv "$key")
    if [ "$actual" != "$expected" ]; then
      fail "operator environment $key conflicts with the selected SDKWork Agents profile" 65
    fi
  fi
}

profile_id=${SDKWORK_AGENTS_PROFILE_ID-}
if [ -z "$profile_id" ]; then
  fail "SDKWORK_AGENTS_PROFILE_ID must select an etc/topology profile" 64
fi

case "$profile_id" in
  standalone.development | standalone.test | standalone.staging | standalone.production | \
  cloud.development | cloud.test | cloud.staging | cloud.production)
    ;;
  *)
    fail "invalid SDKWORK_AGENTS_PROFILE_ID: $profile_id" 64
    ;;
esac

if printenv SDKWORK_AGENTS_PROFILE_FILE >/dev/null 2>&1; then
  fail "SDKWORK_AGENTS_PROFILE_FILE is not supported; materialize the selected profile at /app/etc/topology" 64
fi

if printenv SDKWORK_AGENTS_CORS_ALLOWED_ORIGINS >/dev/null 2>&1; then
  fail "SDKWORK_AGENTS_CORS_ALLOWED_ORIGINS is not supported; use the selected profile's SDKWORK_CORS_ALLOWED_ORIGINS" 65
fi

profile_file="/app/etc/topology/${profile_id}.env"
if [ ! -r "$profile_file" ]; then
  fail "SDKWork Agents profile file is not readable: $profile_file" 66
fi

expected_deployment_profile=${profile_id%%.*}
expected_environment=${profile_id#*.}
carriage_return=$(printf '\r')
profile_file_id=
profile_deployment_profile=
profile_agents_environment=
profile_environment=
profile_cors_allowed_origins=
profile_cors_declared=false

while IFS= read -r line || [ -n "$line" ]; do
  line=${line%"$carriage_return"}
  case "$line" in
    '' | \#*) continue ;;
  esac

  case "$line" in
    *=*) ;;
    *) fail "invalid SDKWork Agents profile line: $line" 65 ;;
  esac

  key=${line%%=*}
  value=${line#*=}
  case "$key" in
    '' | *[!A-Za-z0-9_]*)
      fail "invalid environment key in SDKWork Agents profile: $key" 65
      ;;
  esac

  case "$key" in
    SDKWORK_AGENTS_PROFILE_ID)
      [ -z "$profile_file_id" ] || fail "selected profile declares SDKWORK_AGENTS_PROFILE_ID more than once" 65
      profile_file_id=$value
      ;;
    SDKWORK_AGENTS_DEPLOYMENT_PROFILE)
      [ -z "$profile_deployment_profile" ] || fail "selected profile declares SDKWORK_AGENTS_DEPLOYMENT_PROFILE more than once" 65
      profile_deployment_profile=$value
      ;;
    SDKWORK_AGENTS_ENVIRONMENT)
      [ -z "$profile_agents_environment" ] || fail "selected profile declares SDKWORK_AGENTS_ENVIRONMENT more than once" 65
      profile_agents_environment=$value
      ;;
    SDKWORK_ENVIRONMENT)
      [ -z "$profile_environment" ] || fail "selected profile declares SDKWORK_ENVIRONMENT more than once" 65
      profile_environment=$value
      ;;
    SDKWORK_CORS_ALLOWED_ORIGINS)
      [ "$profile_cors_declared" = false ] || fail "selected profile declares SDKWORK_CORS_ALLOWED_ORIGINS more than once" 65
      profile_cors_declared=true
      profile_cors_allowed_origins=$value
      ;;
    SDKWORK_AGENTS_CORS_ALLOWED_ORIGINS)
      fail "selected profile must use SDKWORK_CORS_ALLOWED_ORIGINS, not SDKWORK_AGENTS_CORS_ALLOWED_ORIGINS" 65
      ;;
    *)
      if ! printenv "$key" >/dev/null 2>&1; then
        export "$key=$value"
      fi
      ;;
  esac
done < "$profile_file"

require_profile_value SDKWORK_AGENTS_PROFILE_ID "$profile_file_id"
require_profile_value SDKWORK_AGENTS_DEPLOYMENT_PROFILE "$profile_deployment_profile"
require_profile_value SDKWORK_AGENTS_ENVIRONMENT "$profile_agents_environment"
require_profile_value SDKWORK_ENVIRONMENT "$profile_environment"

if [ "$profile_file_id" != "$profile_id" ]; then
  fail "selected profile SDKWORK_AGENTS_PROFILE_ID does not match $profile_id" 65
fi
if [ "$profile_deployment_profile" != "$expected_deployment_profile" ]; then
  fail "selected profile SDKWORK_AGENTS_DEPLOYMENT_PROFILE does not match $profile_id" 65
fi
if [ "$profile_agents_environment" != "$expected_environment" ] || [ "$profile_environment" != "$expected_environment" ]; then
  fail "selected profile lifecycle environment does not match $profile_id" 65
fi

verify_existing_security_value SDKWORK_AGENTS_PROFILE_ID "$profile_file_id"
verify_existing_security_value SDKWORK_AGENTS_DEPLOYMENT_PROFILE "$profile_deployment_profile"
verify_existing_security_value SDKWORK_AGENTS_ENVIRONMENT "$profile_agents_environment"
verify_existing_security_value SDKWORK_ENVIRONMENT "$profile_environment"

export SDKWORK_AGENTS_PROFILE_ID="$profile_file_id"
export SDKWORK_AGENTS_DEPLOYMENT_PROFILE="$profile_deployment_profile"
export SDKWORK_AGENTS_ENVIRONMENT="$profile_agents_environment"
export SDKWORK_ENVIRONMENT="$profile_environment"

case "$expected_environment" in
  development)
    if [ "$profile_cors_declared" = true ]; then
      fail "development profile must not set SDKWORK_CORS_ALLOWED_ORIGINS" 65
    fi
    ;;
  test | staging | production)
    if [ "$profile_cors_declared" != true ] || [ -z "$profile_cors_allowed_origins" ]; then
      fail "selected production-like profile must declare SDKWORK_CORS_ALLOWED_ORIGINS" 65
    fi
    ;;
  *) fail "selected profile has an unsupported lifecycle environment: $expected_environment" 65 ;;
esac

if printenv SDKWORK_CORS_ALLOWED_ORIGINS >/dev/null 2>&1; then
  effective_cors_allowed_origins=$(printenv SDKWORK_CORS_ALLOWED_ORIGINS)
else
  effective_cors_allowed_origins=$profile_cors_allowed_origins
fi
if [ "$expected_environment" != development ] && [ -z "$effective_cors_allowed_origins" ]; then
  fail "selected production-like profile must provide SDKWORK_CORS_ALLOWED_ORIGINS" 65
fi
if [ -n "$effective_cors_allowed_origins" ]; then
  export SDKWORK_CORS_ALLOWED_ORIGINS="$effective_cors_allowed_origins"
fi

exec "$@"
