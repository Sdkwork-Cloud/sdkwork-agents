import 'package:flutter/material.dart';

import 'auth_gate.dart';

class AgentsApp extends StatelessWidget {
  const AgentsApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'SDKWork Agents',
      theme: ThemeData(colorSchemeSeed: const Color(0xFF0F766E)),
      home: const AuthGate(),
    );
  }
}
