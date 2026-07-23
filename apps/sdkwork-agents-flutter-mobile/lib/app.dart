import 'package:flutter/material.dart';

import 'auth_gate.dart';
import 'bootstrap/runtime.dart';

class AgentsApp extends StatelessWidget {
  const AgentsApp({required this.runtime, super.key});

  final AgentsMobileRuntime runtime;

  @override
  Widget build(BuildContext context) {
    return AgentsRuntimeScope(
      runtime: runtime,
      child: MaterialApp(
        title: 'SDKWork Agents',
        theme: ThemeData(colorSchemeSeed: const Color(0xFF0F766E)),
        home: const AuthGate(),
      ),
    );
  }
}

class AgentsRuntimeScope extends InheritedWidget {
  const AgentsRuntimeScope({
    required this.runtime,
    required super.child,
    super.key,
  });

  final AgentsMobileRuntime runtime;

  static AgentsMobileRuntime of(BuildContext context) {
    final scope = context
        .dependOnInheritedWidgetOfExactType<AgentsRuntimeScope>();
    if (scope == null) {
      throw StateError('AgentsRuntimeScope is not available.');
    }
    return scope.runtime;
  }

  @override
  bool updateShouldNotify(AgentsRuntimeScope oldWidget) {
    return !identical(runtime, oldWidget.runtime);
  }
}
