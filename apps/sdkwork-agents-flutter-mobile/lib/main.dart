import 'package:flutter/material.dart';

import 'app.dart';
import 'bootstrap/runtime.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await bootstrap();
  runApp(const AgentsApp());
}
