import 'package:sdkwork_agents_app_sdk/sdkwork_agents_app_sdk.dart';

const String agentsAppApiPrefix = '/app/v3/api';

class AgentsAppSdkClients {
  const AgentsAppSdkClients({
    required this.appApiBaseUrl,
    required this.agents,
  });

  final String appApiBaseUrl;
  final SdkworkAppClient agents;
}

AgentsAppSdkClients createAgentsAppSdkClients({
  required String appApiBaseUrl,
  String? authToken,
  String? accessToken,
}) {
  final normalizedSurfaceUrl = normalizeAgentsAppApiBaseUrl(appApiBaseUrl);
  return AgentsAppSdkClients(
    appApiBaseUrl: normalizedSurfaceUrl,
    agents: SdkworkAppClient.withBaseUrl(
      baseUrl: resolveAgentsTransportBaseUrl(normalizedSurfaceUrl),
      authToken: authToken,
      accessToken: accessToken,
    ),
  );
}

String normalizeAgentsAppApiBaseUrl(String value) {
  final normalized = value.trim().replaceFirst(RegExp(r'/+$'), '');
  final uri = Uri.tryParse(normalized);
  if (uri == null ||
      !uri.hasScheme ||
      (uri.scheme != 'http' && uri.scheme != 'https') ||
      uri.host.isEmpty ||
      uri.hasQuery ||
      uri.hasFragment ||
      !uri.path.endsWith(agentsAppApiPrefix)) {
    throw ArgumentError.value(
      value,
      'appApiBaseUrl',
      'must be an absolute HTTP(S) URL ending with $agentsAppApiPrefix',
    );
  }
  if (uri.path
      .substring(0, uri.path.length - agentsAppApiPrefix.length)
      .endsWith(agentsAppApiPrefix)) {
    throw ArgumentError.value(
      value,
      'appApiBaseUrl',
      'must contain $agentsAppApiPrefix exactly once',
    );
  }
  return normalized;
}

String resolveAgentsTransportBaseUrl(String appApiBaseUrl) {
  final normalized = normalizeAgentsAppApiBaseUrl(appApiBaseUrl);
  final uri = Uri.parse(normalized);
  final transportPath = uri.path.substring(
    0,
    uri.path.length - agentsAppApiPrefix.length,
  );
  return uri
      .replace(path: transportPath)
      .toString()
      .replaceFirst(RegExp(r'/+$'), '');
}
