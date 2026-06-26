import 'host_adapters.dart';
import 'iam_runtime.dart';
import 'routes.dart';
import 'sdk_clients.dart';

Future<void> bootstrap() async {
  createIamRuntime();
  registerHostAdapters();
  createSdkClients();
  createRoutes();
}
