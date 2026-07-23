import 'package:sdkwork_agents_flutter_mobile_core/sdkwork_agents_flutter_mobile_core.dart';

typedef SdkClients = AgentsAppSdkClients;

const String _configuredAppApiBaseUrl = String.fromEnvironment(
  'SDKWORK_AGENTS_APP_API_BASE_URL',
  defaultValue: 'http://127.0.0.1:8095/app/v3/api',
);

SdkClients createSdkClients({
  String? appApiBaseUrl,
  String? authToken,
  String? accessToken,
}) {
  return createAgentsAppSdkClients(
    appApiBaseUrl: appApiBaseUrl ?? _configuredAppApiBaseUrl,
    authToken: authToken,
    accessToken: accessToken,
  );
}
