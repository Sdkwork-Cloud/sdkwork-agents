import 'package:flutter_test/flutter_test.dart';
import 'package:sdkwork_agents_app_sdk/sdkwork_agents_app_sdk.dart';
import 'package:sdkwork_agents_flutter_mobile_core/sdkwork_agents_flutter_mobile_core.dart';

void main() {
  test('constructs the generated Agents App SDK from one surface URL', () {
    final clients = createAgentsAppSdkClients(
      appApiBaseUrl: 'https://agents.example.com/app/v3/api/',
    );

    expect(clients.appApiBaseUrl, 'https://agents.example.com/app/v3/api');
    expect(clients.agents, isA<SdkworkAppClient>());
    expect(
      resolveAgentsTransportBaseUrl(clients.appApiBaseUrl),
      'https://agents.example.com',
    );
  });

  test('rejects a non-surface URL and a duplicate surface prefix', () {
    expect(
      () => createAgentsAppSdkClients(
        appApiBaseUrl: 'https://agents.example.com',
      ),
      throwsArgumentError,
    );
    expect(
      () => createAgentsAppSdkClients(
        appApiBaseUrl: 'https://agents.example.com/app/v3/api/app/v3/api',
      ),
      throwsArgumentError,
    );
  });

  test('exposes typed session, turn, and session-item methods', () {
    final client = createAgentsAppSdkClients(
      appApiBaseUrl: 'https://agents.example.com/app/v3/api',
    ).agents;

    final Future<AgentSessionResponse?> Function(
      String,
      CreateAgentSessionRequest,
    )
    createSession = client.ai.agentsSessionsCreate;
    final Stream<AgentTurnStreamEvent> Function(
      String,
      String,
      CreateAgentTurnRequest, [
      bool?,
    ])
    streamTurn = client.ai.agentsTurnsStream;
    final Future<AgentSessionItemListResponse?> Function(
      String,
      String, [
      String?,
      int?,
      String?,
      String?,
      String?,
    ])
    listItems = client.ai.agentsSessionItemsList;

    expect(createSession, isNotNull);
    expect(streamTurn, isNotNull);
    expect(listItems, isNotNull);
  });
}
