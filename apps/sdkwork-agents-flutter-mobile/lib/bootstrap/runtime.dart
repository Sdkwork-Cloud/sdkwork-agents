import 'host_adapters.dart';
import 'iam_runtime.dart';
import 'routes.dart';
import 'sdk_clients.dart';

class AgentsMobileRuntime {
  const AgentsMobileRuntime({required this.sdkClients});

  final SdkClients sdkClients;
}

Future<AgentsMobileRuntime> bootstrap() async {
  createIamRuntime();
  registerHostAdapters();
  final sdkClients = createSdkClients();
  createRoutes();
  return AgentsMobileRuntime(sdkClients: sdkClients);
}
