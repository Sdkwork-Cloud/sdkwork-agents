Map<String, dynamic>? _sdkworkAsMap(dynamic value) {
  if (value is Map<String, dynamic>) {
    return value;
  }
  if (value is Map) {
    return value.map((key, item) => MapEntry(key.toString(), item));
  }
  return null;
}

List<dynamic>? _sdkworkAsList(dynamic value) {
  return value is List ? value : null;
}

class SdkWorkApiResponse {
  final int code;
  final dynamic data;
  final String traceId;

  SdkWorkApiResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory SdkWorkApiResponse.fromJson(Map<String, dynamic> json) {
    return SdkWorkApiResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('SdkWorkApiResponse.code is required');
        }
        return value;
      })(),
      data: json['data'],
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('SdkWorkApiResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class SdkWorkResourceData {
  final dynamic item;

  SdkWorkResourceData({
    required this.item
  });

  factory SdkWorkResourceData.fromJson(Map<String, dynamic> json) {
    return SdkWorkResourceData(
      item: json['item']
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'item': item,
    };
  }
}

class SdkWorkPageData {
  final List<dynamic> items;
  final PageInfo pageInfo;

  SdkWorkPageData({
    required this.items,
    required this.pageInfo
  });

  factory SdkWorkPageData.fromJson(Map<String, dynamic> json) {
    return SdkWorkPageData(
      items: (() {
        final list = _sdkworkAsList(json['items']);
        if (list == null) {
          throw FormatException('SdkWorkPageData.items is required');
        }
        return list
            .map((item) => item)
            .whereType<dynamic>()
            .toList();
      })(),
      pageInfo: (() {
        final map = _sdkworkAsMap(json['pageInfo']);
        if (map == null) {
          throw FormatException('SdkWorkPageData.pageInfo is required');
        }
        return PageInfo.fromJson(map);
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'items': items.map((item) => item).toList(),
      'pageInfo': pageInfo.toJson(),
    };
  }
}

class PageInfo {
  final String mode;
  final int? page;
  final int? pageSize;
  final String? totalItems;
  final int? totalPages;
  final String? nextCursor;
  final bool? hasMore;

  PageInfo({
    required this.mode,
    this.page,
    this.pageSize,
    this.totalItems,
    this.totalPages,
    this.nextCursor,
    this.hasMore
  });

  factory PageInfo.fromJson(Map<String, dynamic> json) {
    return PageInfo(
      mode: (() {
        final value = json['mode']?.toString();
        if (value == null) {
          throw FormatException('PageInfo.mode is required');
        }
        return value;
      })(),
      page: json['page'] is int ? json['page'] : null,
      pageSize: json['pageSize'] is int ? json['pageSize'] : null,
      totalItems: json['totalItems']?.toString(),
      totalPages: json['totalPages'] is int ? json['totalPages'] : null,
      nextCursor: json['nextCursor']?.toString(),
      hasMore: json['hasMore'] is bool ? json['hasMore'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'mode': mode,
      'page': page,
      'pageSize': pageSize,
      'totalItems': totalItems,
      'totalPages': totalPages,
      'nextCursor': nextCursor,
      'hasMore': hasMore,
    };
  }
}

class AgentRecord {
  final String id;
  final String agentId;
  final String tenantId;
  final String organizationId;
  final String ownerUserId;
  final String code;
  final String displayName;
  final String? description;
  final Map<String, dynamic> manifest;
  final Map<String, dynamic>? defaultCodeTaskIntent;
  final AgentManagementProfile? managementProfile;
  final String? implementationProviderId;
  final String? implementationKind;
  final String implementationType;
  final String status;
  final String visibility;
  final List<String> tags;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? deletedAt;

  AgentRecord({
    required this.id,
    required this.agentId,
    required this.tenantId,
    required this.organizationId,
    required this.ownerUserId,
    required this.code,
    required this.displayName,
    this.description,
    required this.manifest,
    this.defaultCodeTaskIntent,
    this.managementProfile,
    this.implementationProviderId,
    this.implementationKind,
    required this.implementationType,
    required this.status,
    required this.visibility,
    required this.tags,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.deletedAt
  });

  factory AgentRecord.fromJson(Map<String, dynamic> json) {
    return AgentRecord(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.id is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.agentId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.organizationId is required');
        }
        return value;
      })(),
      ownerUserId: (() {
        final value = json['ownerUserId']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.ownerUserId is required');
        }
        return value;
      })(),
      code: (() {
        final value = json['code']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.code is required');
        }
        return value;
      })(),
      displayName: (() {
        final value = json['displayName']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.displayName is required');
        }
        return value;
      })(),
      description: json['description']?.toString(),
      manifest: (() {
        final map = _sdkworkAsMap(json['manifest']);
        if (map == null) {
          throw FormatException('AgentRecord.manifest is required');
        }
        return map;
      })(),
      defaultCodeTaskIntent: _sdkworkAsMap(json['defaultCodeTaskIntent']),
      managementProfile: (() {
        final map = _sdkworkAsMap(json['managementProfile']);
        return map == null ? null : AgentManagementProfile.fromJson(map);
      })(),
      implementationProviderId: json['implementationProviderId']?.toString(),
      implementationKind: json['implementationKind']?.toString(),
      implementationType: (() {
        final value = json['implementationType']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.implementationType is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.status is required');
        }
        return value;
      })(),
      visibility: (() {
        final value = json['visibility']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.visibility is required');
        }
        return value;
      })(),
      tags: (() {
        final list = _sdkworkAsList(json['tags']);
        if (list == null) {
          throw FormatException('AgentRecord.tags is required');
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentRecord.updatedAt is required');
        }
        return value;
      })(),
      deletedAt: json['deletedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'agentId': agentId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'ownerUserId': ownerUserId,
      'code': code,
      'displayName': displayName,
      'description': description,
      'manifest': manifest,
      'defaultCodeTaskIntent': defaultCodeTaskIntent,
      'managementProfile': managementProfile?.toJson(),
      'implementationProviderId': implementationProviderId,
      'implementationKind': implementationKind,
      'implementationType': implementationType,
      'status': status,
      'visibility': visibility,
      'tags': tags.map((item) => item).toList(),
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'deletedAt': deletedAt,
    };
  }
}

class AgentManagementProfile {
  final String? author;
  final String? avatar;
  final String? categoryId;
  final String? color;
  final bool? debugMode;
  final String? iconName;
  final bool? jsonMode;
  final List<String>? knowledgeBaseIds;
  final bool? memoryEnabled;
  final String? model;
  final List<String>? skillIds;
  final List<String>? suggestedPrompts;
  final String? systemPrompt;
  final double? temperature;
  final List<String>? toolIds;
  final String? type;
  final String? users;
  final List<String>? voiceIds;
  final String? welcomeMessage;

  AgentManagementProfile({
    this.author,
    this.avatar,
    this.categoryId,
    this.color,
    this.debugMode,
    this.iconName,
    this.jsonMode,
    this.knowledgeBaseIds,
    this.memoryEnabled,
    this.model,
    this.skillIds,
    this.suggestedPrompts,
    this.systemPrompt,
    this.temperature,
    this.toolIds,
    this.type,
    this.users,
    this.voiceIds,
    this.welcomeMessage
  });

  factory AgentManagementProfile.fromJson(Map<String, dynamic> json) {
    return AgentManagementProfile(
      author: json['author']?.toString(),
      avatar: json['avatar']?.toString(),
      categoryId: json['categoryId']?.toString(),
      color: json['color']?.toString(),
      debugMode: json['debugMode'] is bool ? json['debugMode'] : null,
      iconName: json['iconName']?.toString(),
      jsonMode: json['jsonMode'] is bool ? json['jsonMode'] : null,
      knowledgeBaseIds: (() {
        final list = _sdkworkAsList(json['knowledgeBaseIds']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      memoryEnabled: json['memoryEnabled'] is bool ? json['memoryEnabled'] : null,
      model: json['model']?.toString(),
      skillIds: (() {
        final list = _sdkworkAsList(json['skillIds']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      suggestedPrompts: (() {
        final list = _sdkworkAsList(json['suggestedPrompts']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      systemPrompt: json['systemPrompt']?.toString(),
      temperature: json['temperature'] is num ? json['temperature'].toDouble() : null,
      toolIds: (() {
        final list = _sdkworkAsList(json['toolIds']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      type: json['type']?.toString(),
      users: json['users']?.toString(),
      voiceIds: (() {
        final list = _sdkworkAsList(json['voiceIds']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      welcomeMessage: json['welcomeMessage']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'author': author,
      'avatar': avatar,
      'categoryId': categoryId,
      'color': color,
      'debugMode': debugMode,
      'iconName': iconName,
      'jsonMode': jsonMode,
      'knowledgeBaseIds': knowledgeBaseIds?.map((item) => item).toList(),
      'memoryEnabled': memoryEnabled,
      'model': model,
      'skillIds': skillIds?.map((item) => item).toList(),
      'suggestedPrompts': suggestedPrompts?.map((item) => item).toList(),
      'systemPrompt': systemPrompt,
      'temperature': temperature,
      'toolIds': toolIds?.map((item) => item).toList(),
      'type': type,
      'users': users,
      'voiceIds': voiceIds?.map((item) => item).toList(),
      'welcomeMessage': welcomeMessage,
    };
  }
}

class AgentResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentResponse.fromJson(Map<String, dynamic> json) {
    return AgentResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentListResponse.fromJson(Map<String, dynamic> json) {
    return AgentListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentProviderBindingRecord {
  final String tenantId;
  final String agentId;
  final String bindingId;
  final String providerId;
  final String implementationKind;
  final String configurationProfileId;
  final List<String> capabilities;
  final bool active;
  final String version;
  final String createdAt;
  final String updatedAt;

  AgentProviderBindingRecord({
    required this.tenantId,
    required this.agentId,
    required this.bindingId,
    required this.providerId,
    required this.implementationKind,
    required this.configurationProfileId,
    required this.capabilities,
    required this.active,
    required this.version,
    required this.createdAt,
    required this.updatedAt
  });

  factory AgentProviderBindingRecord.fromJson(Map<String, dynamic> json) {
    return AgentProviderBindingRecord(
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.tenantId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.agentId is required');
        }
        return value;
      })(),
      bindingId: (() {
        final value = json['bindingId']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.bindingId is required');
        }
        return value;
      })(),
      providerId: (() {
        final value = json['providerId']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.providerId is required');
        }
        return value;
      })(),
      implementationKind: (() {
        final value = json['implementationKind']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.implementationKind is required');
        }
        return value;
      })(),
      configurationProfileId: (() {
        final value = json['configurationProfileId']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.configurationProfileId is required');
        }
        return value;
      })(),
      capabilities: (() {
        final list = _sdkworkAsList(json['capabilities']);
        if (list == null) {
          throw FormatException('AgentProviderBindingRecord.capabilities is required');
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      active: (() {
        final value = json['active'];
        if (value is! bool) {
          throw FormatException('AgentProviderBindingRecord.active is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingRecord.updatedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'tenantId': tenantId,
      'agentId': agentId,
      'bindingId': bindingId,
      'providerId': providerId,
      'implementationKind': implementationKind,
      'configurationProfileId': configurationProfileId,
      'capabilities': capabilities.map((item) => item).toList(),
      'active': active,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
    };
  }
}

class AgentProviderBindingResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentProviderBindingResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentProviderBindingResponse.fromJson(Map<String, dynamic> json) {
    return AgentProviderBindingResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentProviderBindingResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentProviderBindingResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentProviderBindingListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentProviderBindingListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentProviderBindingListResponse.fromJson(Map<String, dynamic> json) {
    return AgentProviderBindingListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentProviderBindingListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentProviderBindingListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentProviderBindingListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentRuntimeExecutionRecord {
  final String tenantId;
  final String agentId;
  final String executionId;
  final String operation;
  final String status;
  final Map<String, dynamic> inputPayload;
  final Map<String, dynamic> outputPayload;
  final String requestedAt;
  final String completedAt;

  AgentRuntimeExecutionRecord({
    required this.tenantId,
    required this.agentId,
    required this.executionId,
    required this.operation,
    required this.status,
    required this.inputPayload,
    required this.outputPayload,
    required this.requestedAt,
    required this.completedAt
  });

  factory AgentRuntimeExecutionRecord.fromJson(Map<String, dynamic> json) {
    return AgentRuntimeExecutionRecord(
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionRecord.tenantId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionRecord.agentId is required');
        }
        return value;
      })(),
      executionId: (() {
        final value = json['executionId']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionRecord.executionId is required');
        }
        return value;
      })(),
      operation: (() {
        final value = json['operation']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionRecord.operation is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionRecord.status is required');
        }
        return value;
      })(),
      inputPayload: (() {
        final map = _sdkworkAsMap(json['inputPayload']);
        if (map == null) {
          throw FormatException('AgentRuntimeExecutionRecord.inputPayload is required');
        }
        return map;
      })(),
      outputPayload: (() {
        final map = _sdkworkAsMap(json['outputPayload']);
        if (map == null) {
          throw FormatException('AgentRuntimeExecutionRecord.outputPayload is required');
        }
        return map;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionRecord.requestedAt is required');
        }
        return value;
      })(),
      completedAt: (() {
        final value = json['completedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionRecord.completedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'tenantId': tenantId,
      'agentId': agentId,
      'executionId': executionId,
      'operation': operation,
      'status': status,
      'inputPayload': inputPayload,
      'outputPayload': outputPayload,
      'requestedAt': requestedAt,
      'completedAt': completedAt,
    };
  }
}

class AgentRuntimeExecutionResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentRuntimeExecutionResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentRuntimeExecutionResponse.fromJson(Map<String, dynamic> json) {
    return AgentRuntimeExecutionResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentRuntimeExecutionResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentRuntimeExecutionResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentRuntimeExecutionResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentCompositionSlotRecord {
  final String id;
  final String tenantId;
  final String organizationId;
  final String agentId;
  final String slotId;
  final String slotKind;
  final String targetModule;
  final String targetRef;
  final String? targetVersionRef;
  final int priority;
  final bool enabled;
  final String policyJson;
  final String status;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? deletedAt;

  AgentCompositionSlotRecord({
    required this.id,
    required this.tenantId,
    required this.organizationId,
    required this.agentId,
    required this.slotId,
    required this.slotKind,
    required this.targetModule,
    required this.targetRef,
    this.targetVersionRef,
    required this.priority,
    required this.enabled,
    required this.policyJson,
    required this.status,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.deletedAt
  });

  factory AgentCompositionSlotRecord.fromJson(Map<String, dynamic> json) {
    return AgentCompositionSlotRecord(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.id is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.organizationId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.agentId is required');
        }
        return value;
      })(),
      slotId: (() {
        final value = json['slotId']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.slotId is required');
        }
        return value;
      })(),
      slotKind: (() {
        final value = json['slotKind']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.slotKind is required');
        }
        return value;
      })(),
      targetModule: (() {
        final value = json['targetModule']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.targetModule is required');
        }
        return value;
      })(),
      targetRef: (() {
        final value = json['targetRef']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.targetRef is required');
        }
        return value;
      })(),
      targetVersionRef: json['targetVersionRef']?.toString(),
      priority: (() {
        final value = json['priority'];
        if (value is! int) {
          throw FormatException('AgentCompositionSlotRecord.priority is required');
        }
        return value;
      })(),
      enabled: (() {
        final value = json['enabled'];
        if (value is! bool) {
          throw FormatException('AgentCompositionSlotRecord.enabled is required');
        }
        return value;
      })(),
      policyJson: (() {
        final value = json['policyJson']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.policyJson is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.status is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotRecord.updatedAt is required');
        }
        return value;
      })(),
      deletedAt: json['deletedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'agentId': agentId,
      'slotId': slotId,
      'slotKind': slotKind,
      'targetModule': targetModule,
      'targetRef': targetRef,
      'targetVersionRef': targetVersionRef,
      'priority': priority,
      'enabled': enabled,
      'policyJson': policyJson,
      'status': status,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'deletedAt': deletedAt,
    };
  }
}

class AgentCompositionSlotResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentCompositionSlotResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentCompositionSlotResponse.fromJson(Map<String, dynamic> json) {
    return AgentCompositionSlotResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentCompositionSlotResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentCompositionSlotResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentCompositionSlotListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentCompositionSlotListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentCompositionSlotListResponse.fromJson(Map<String, dynamic> json) {
    return AgentCompositionSlotListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentCompositionSlotListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentCompositionSlotListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentCompositionSlotListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentCompositionSlotRequest {
  final String slotId;
  final String slotKind;
  final String targetModule;
  final String targetRef;
  final String? targetVersionRef;
  final int? priority;
  final bool? enabled;
  final String? policyJson;
  final String requestedAt;

  CreateAgentCompositionSlotRequest({
    required this.slotId,
    required this.slotKind,
    required this.targetModule,
    required this.targetRef,
    this.targetVersionRef,
    this.priority,
    this.enabled,
    this.policyJson,
    required this.requestedAt
  });

  factory CreateAgentCompositionSlotRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentCompositionSlotRequest(
      slotId: (() {
        final value = json['slotId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentCompositionSlotRequest.slotId is required');
        }
        return value;
      })(),
      slotKind: (() {
        final value = json['slotKind']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentCompositionSlotRequest.slotKind is required');
        }
        return value;
      })(),
      targetModule: (() {
        final value = json['targetModule']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentCompositionSlotRequest.targetModule is required');
        }
        return value;
      })(),
      targetRef: (() {
        final value = json['targetRef']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentCompositionSlotRequest.targetRef is required');
        }
        return value;
      })(),
      targetVersionRef: json['targetVersionRef']?.toString(),
      priority: json['priority'] is int ? json['priority'] : null,
      enabled: json['enabled'] is bool ? json['enabled'] : null,
      policyJson: json['policyJson']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentCompositionSlotRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'slotId': slotId,
      'slotKind': slotKind,
      'targetModule': targetModule,
      'targetRef': targetRef,
      'targetVersionRef': targetVersionRef,
      'priority': priority,
      'enabled': enabled,
      'policyJson': policyJson,
      'requestedAt': requestedAt,
    };
  }
}

class UpdateAgentCompositionSlotRequest {
  final String? expectedVersion;
  final String? slotKind;
  final String? targetModule;
  final String? targetRef;
  final String? targetVersionRef;
  final int? priority;
  final bool? enabled;
  final String? policyJson;
  final String requestedAt;

  UpdateAgentCompositionSlotRequest({
    this.expectedVersion,
    this.slotKind,
    this.targetModule,
    this.targetRef,
    this.targetVersionRef,
    this.priority,
    this.enabled,
    this.policyJson,
    required this.requestedAt
  });

  factory UpdateAgentCompositionSlotRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentCompositionSlotRequest(
      expectedVersion: json['expectedVersion']?.toString(),
      slotKind: json['slotKind']?.toString(),
      targetModule: json['targetModule']?.toString(),
      targetRef: json['targetRef']?.toString(),
      targetVersionRef: json['targetVersionRef']?.toString(),
      priority: json['priority'] is int ? json['priority'] : null,
      enabled: json['enabled'] is bool ? json['enabled'] : null,
      policyJson: json['policyJson']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentCompositionSlotRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'slotKind': slotKind,
      'targetModule': targetModule,
      'targetRef': targetRef,
      'targetVersionRef': targetVersionRef,
      'priority': priority,
      'enabled': enabled,
      'policyJson': policyJson,
      'requestedAt': requestedAt,
    };
  }
}

class CreateAgentRequest {
  final String agentId;
  final String code;
  final String displayName;
  final String? description;
  final Map<String, dynamic> manifest;
  final Map<String, dynamic>? defaultCodeTaskIntent;
  final AgentManagementProfile? managementProfile;
  final String? implementationProviderId;
  final String? implementationKind;
  final String? implementationType;
  final String visibility;
  final List<String>? tags;
  final String requestedAt;

  CreateAgentRequest({
    required this.agentId,
    required this.code,
    required this.displayName,
    this.description,
    required this.manifest,
    this.defaultCodeTaskIntent,
    this.managementProfile,
    this.implementationProviderId,
    this.implementationKind,
    this.implementationType,
    required this.visibility,
    this.tags,
    required this.requestedAt
  });

  factory CreateAgentRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentRequest(
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentRequest.agentId is required');
        }
        return value;
      })(),
      code: (() {
        final value = json['code']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentRequest.code is required');
        }
        return value;
      })(),
      displayName: (() {
        final value = json['displayName']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentRequest.displayName is required');
        }
        return value;
      })(),
      description: json['description']?.toString(),
      manifest: (() {
        final map = _sdkworkAsMap(json['manifest']);
        if (map == null) {
          throw FormatException('CreateAgentRequest.manifest is required');
        }
        return map;
      })(),
      defaultCodeTaskIntent: _sdkworkAsMap(json['defaultCodeTaskIntent']),
      managementProfile: (() {
        final map = _sdkworkAsMap(json['managementProfile']);
        return map == null ? null : AgentManagementProfile.fromJson(map);
      })(),
      implementationProviderId: json['implementationProviderId']?.toString(),
      implementationKind: json['implementationKind']?.toString(),
      implementationType: json['implementationType']?.toString(),
      visibility: (() {
        final value = json['visibility']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentRequest.visibility is required');
        }
        return value;
      })(),
      tags: (() {
        final list = _sdkworkAsList(json['tags']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'agentId': agentId,
      'code': code,
      'displayName': displayName,
      'description': description,
      'manifest': manifest,
      'defaultCodeTaskIntent': defaultCodeTaskIntent,
      'managementProfile': managementProfile?.toJson(),
      'implementationProviderId': implementationProviderId,
      'implementationKind': implementationKind,
      'implementationType': implementationType,
      'visibility': visibility,
      'tags': tags?.map((item) => item).toList(),
      'requestedAt': requestedAt,
    };
  }
}

class CreateAgentProviderBindingRequest {
  final String bindingId;
  final String providerId;
  final String implementationKind;
  final String configurationProfileId;
  final List<String>? capabilities;
  final bool? makeDefault;
  final String requestedAt;

  CreateAgentProviderBindingRequest({
    required this.bindingId,
    required this.providerId,
    required this.implementationKind,
    required this.configurationProfileId,
    this.capabilities,
    this.makeDefault,
    required this.requestedAt
  });

  factory CreateAgentProviderBindingRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentProviderBindingRequest(
      bindingId: (() {
        final value = json['bindingId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProviderBindingRequest.bindingId is required');
        }
        return value;
      })(),
      providerId: (() {
        final value = json['providerId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProviderBindingRequest.providerId is required');
        }
        return value;
      })(),
      implementationKind: (() {
        final value = json['implementationKind']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProviderBindingRequest.implementationKind is required');
        }
        return value;
      })(),
      configurationProfileId: (() {
        final value = json['configurationProfileId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProviderBindingRequest.configurationProfileId is required');
        }
        return value;
      })(),
      capabilities: (() {
        final list = _sdkworkAsList(json['capabilities']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      makeDefault: json['makeDefault'] is bool ? json['makeDefault'] : null,
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProviderBindingRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'bindingId': bindingId,
      'providerId': providerId,
      'implementationKind': implementationKind,
      'configurationProfileId': configurationProfileId,
      'capabilities': capabilities?.map((item) => item).toList(),
      'makeDefault': makeDefault,
      'requestedAt': requestedAt,
    };
  }
}

class ActivateAgentProviderBindingRequest {
  final String requestedAt;

  ActivateAgentProviderBindingRequest({
    required this.requestedAt
  });

  factory ActivateAgentProviderBindingRequest.fromJson(Map<String, dynamic> json) {
    return ActivateAgentProviderBindingRequest(
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('ActivateAgentProviderBindingRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'requestedAt': requestedAt,
    };
  }
}

class CreateAgentPreviewResponseRequest {
  final String executionId;
  final String content;
  final bool? debugMode;
  final bool? memoryEnabled;
  final String? model;
  final double? temperature;
  final Map<String, dynamic>? inputPayload;
  final String requestedAt;

  CreateAgentPreviewResponseRequest({
    required this.executionId,
    required this.content,
    this.debugMode,
    this.memoryEnabled,
    this.model,
    this.temperature,
    this.inputPayload,
    required this.requestedAt
  });

  factory CreateAgentPreviewResponseRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentPreviewResponseRequest(
      executionId: (() {
        final value = json['executionId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentPreviewResponseRequest.executionId is required');
        }
        return value;
      })(),
      content: (() {
        final value = json['content']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentPreviewResponseRequest.content is required');
        }
        return value;
      })(),
      debugMode: json['debugMode'] is bool ? json['debugMode'] : null,
      memoryEnabled: json['memoryEnabled'] is bool ? json['memoryEnabled'] : null,
      model: json['model']?.toString(),
      temperature: json['temperature'] is num ? json['temperature'].toDouble() : null,
      inputPayload: _sdkworkAsMap(json['inputPayload']),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentPreviewResponseRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'executionId': executionId,
      'content': content,
      'debugMode': debugMode,
      'memoryEnabled': memoryEnabled,
      'model': model,
      'temperature': temperature,
      'inputPayload': inputPayload,
      'requestedAt': requestedAt,
    };
  }
}

class CreateAgentPromptOptimizationRequest {
  final String executionId;
  final String prompt;
  final Map<String, dynamic>? inputPayload;
  final String requestedAt;

  CreateAgentPromptOptimizationRequest({
    required this.executionId,
    required this.prompt,
    this.inputPayload,
    required this.requestedAt
  });

  factory CreateAgentPromptOptimizationRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentPromptOptimizationRequest(
      executionId: (() {
        final value = json['executionId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentPromptOptimizationRequest.executionId is required');
        }
        return value;
      })(),
      prompt: (() {
        final value = json['prompt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentPromptOptimizationRequest.prompt is required');
        }
        return value;
      })(),
      inputPayload: _sdkworkAsMap(json['inputPayload']),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentPromptOptimizationRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'executionId': executionId,
      'prompt': prompt,
      'inputPayload': inputPayload,
      'requestedAt': requestedAt,
    };
  }
}

class UpdateAgentRequest {
  final String? displayName;
  final String? description;
  final Map<String, dynamic>? manifest;
  final String? visibility;
  final List<String>? tags;
  final Map<String, dynamic>? defaultCodeTaskIntent;
  final AgentManagementProfile? managementProfile;
  final String? implementationProviderId;
  final String? implementationKind;
  final String? implementationType;
  final String? expectedVersion;
  final String requestedAt;

  UpdateAgentRequest({
    this.displayName,
    this.description,
    this.manifest,
    this.visibility,
    this.tags,
    this.defaultCodeTaskIntent,
    this.managementProfile,
    this.implementationProviderId,
    this.implementationKind,
    this.implementationType,
    this.expectedVersion,
    required this.requestedAt
  });

  factory UpdateAgentRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentRequest(
      displayName: json['displayName']?.toString(),
      description: json['description']?.toString(),
      manifest: _sdkworkAsMap(json['manifest']),
      visibility: json['visibility']?.toString(),
      tags: (() {
        final list = _sdkworkAsList(json['tags']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => item?.toString())
            .whereType<String>()
            .toList();
      })(),
      defaultCodeTaskIntent: _sdkworkAsMap(json['defaultCodeTaskIntent']),
      managementProfile: (() {
        final map = _sdkworkAsMap(json['managementProfile']);
        return map == null ? null : AgentManagementProfile.fromJson(map);
      })(),
      implementationProviderId: json['implementationProviderId']?.toString(),
      implementationKind: json['implementationKind']?.toString(),
      implementationType: json['implementationType']?.toString(),
      expectedVersion: json['expectedVersion']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'displayName': displayName,
      'description': description,
      'manifest': manifest,
      'visibility': visibility,
      'tags': tags?.map((item) => item).toList(),
      'defaultCodeTaskIntent': defaultCodeTaskIntent,
      'managementProfile': managementProfile?.toJson(),
      'implementationProviderId': implementationProviderId,
      'implementationKind': implementationKind,
      'implementationType': implementationType,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class RestoreAgentRequest {
  final String? expectedVersion;
  final String requestedAt;

  RestoreAgentRequest({
    this.expectedVersion,
    required this.requestedAt
  });

  factory RestoreAgentRequest.fromJson(Map<String, dynamic> json) {
    return RestoreAgentRequest(
      expectedVersion: json['expectedVersion']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('RestoreAgentRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentWorkspaceRecord {
  final String id;
  final String workspaceId;
  final String tenantId;
  final String organizationId;
  final String ownerUserId;
  final String name;
  final String? description;
  final bool isDefault;
  final String status;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? archivedAt;

  AgentWorkspaceRecord({
    required this.id,
    required this.workspaceId,
    required this.tenantId,
    required this.organizationId,
    required this.ownerUserId,
    required this.name,
    this.description,
    required this.isDefault,
    required this.status,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.archivedAt
  });

  factory AgentWorkspaceRecord.fromJson(Map<String, dynamic> json) {
    return AgentWorkspaceRecord(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.id is required');
        }
        return value;
      })(),
      workspaceId: (() {
        final value = json['workspaceId']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.workspaceId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.organizationId is required');
        }
        return value;
      })(),
      ownerUserId: (() {
        final value = json['ownerUserId']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.ownerUserId is required');
        }
        return value;
      })(),
      name: (() {
        final value = json['name']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.name is required');
        }
        return value;
      })(),
      description: json['description']?.toString(),
      isDefault: (() {
        final value = json['isDefault'];
        if (value is! bool) {
          throw FormatException('AgentWorkspaceRecord.isDefault is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.status is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceRecord.updatedAt is required');
        }
        return value;
      })(),
      archivedAt: json['archivedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'workspaceId': workspaceId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'ownerUserId': ownerUserId,
      'name': name,
      'description': description,
      'isDefault': isDefault,
      'status': status,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'archivedAt': archivedAt,
    };
  }
}

class AgentWorkspaceResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentWorkspaceResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentWorkspaceResponse.fromJson(Map<String, dynamic> json) {
    return AgentWorkspaceResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentWorkspaceResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentWorkspaceResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentWorkspaceListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentWorkspaceListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentWorkspaceListResponse.fromJson(Map<String, dynamic> json) {
    return AgentWorkspaceListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentWorkspaceListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentWorkspaceListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class EnsureDefaultAgentWorkspaceRequest {
  final String? name;

  EnsureDefaultAgentWorkspaceRequest({
    this.name
  });

  factory EnsureDefaultAgentWorkspaceRequest.fromJson(Map<String, dynamic> json) {
    return EnsureDefaultAgentWorkspaceRequest(
      name: json['name']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'name': name,
    };
  }
}

class CreateAgentWorkspaceRequest {
  final String name;
  final String? description;

  CreateAgentWorkspaceRequest({
    required this.name,
    this.description
  });

  factory CreateAgentWorkspaceRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentWorkspaceRequest(
      name: (() {
        final value = json['name']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentWorkspaceRequest.name is required');
        }
        return value;
      })(),
      description: json['description']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'name': name,
      'description': description,
    };
  }
}

class UpdateAgentWorkspaceRequest {
  final String expectedVersion;
  final String? name;
  final String? description;

  UpdateAgentWorkspaceRequest({
    required this.expectedVersion,
    this.name,
    this.description
  });

  factory UpdateAgentWorkspaceRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentWorkspaceRequest(
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentWorkspaceRequest.expectedVersion is required');
        }
        return value;
      })(),
      name: json['name']?.toString(),
      description: json['description']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'name': name,
      'description': description,
    };
  }
}

class AgentWorkspaceMutationRequest {
  final String expectedVersion;

  AgentWorkspaceMutationRequest({
    required this.expectedVersion
  });

  factory AgentWorkspaceMutationRequest.fromJson(Map<String, dynamic> json) {
    return AgentWorkspaceMutationRequest(
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('AgentWorkspaceMutationRequest.expectedVersion is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
    };
  }
}

class AgentProjectRecord {
  final String id;
  final String projectId;
  final String workspaceId;
  final String tenantId;
  final String organizationId;
  final String ownerUserId;
  final String name;
  final String? description;
  final String visibility;
  final String status;
  final String driveAccessMode;
  final String? defaultAgentId;
  final String? defaultModelId;
  final String? importSourceKind;
  final String? importSourceRef;
  final String? driveSpaceId;
  final String? driveRootEntryId;
  final String? driveLogicalPath;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? archivedAt;

  AgentProjectRecord({
    required this.id,
    required this.projectId,
    required this.workspaceId,
    required this.tenantId,
    required this.organizationId,
    required this.ownerUserId,
    required this.name,
    this.description,
    required this.visibility,
    required this.status,
    required this.driveAccessMode,
    this.defaultAgentId,
    this.defaultModelId,
    this.importSourceKind,
    this.importSourceRef,
    this.driveSpaceId,
    this.driveRootEntryId,
    this.driveLogicalPath,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.archivedAt
  });

  factory AgentProjectRecord.fromJson(Map<String, dynamic> json) {
    return AgentProjectRecord(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.id is required');
        }
        return value;
      })(),
      projectId: (() {
        final value = json['projectId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.projectId is required');
        }
        return value;
      })(),
      workspaceId: (() {
        final value = json['workspaceId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.workspaceId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.organizationId is required');
        }
        return value;
      })(),
      ownerUserId: (() {
        final value = json['ownerUserId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.ownerUserId is required');
        }
        return value;
      })(),
      name: (() {
        final value = json['name']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.name is required');
        }
        return value;
      })(),
      description: json['description']?.toString(),
      visibility: (() {
        final value = json['visibility']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.visibility is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.status is required');
        }
        return value;
      })(),
      driveAccessMode: (() {
        final value = json['driveAccessMode']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.driveAccessMode is required');
        }
        return value;
      })(),
      defaultAgentId: json['defaultAgentId']?.toString(),
      defaultModelId: json['defaultModelId']?.toString(),
      importSourceKind: json['importSourceKind']?.toString(),
      importSourceRef: json['importSourceRef']?.toString(),
      driveSpaceId: json['driveSpaceId']?.toString(),
      driveRootEntryId: json['driveRootEntryId']?.toString(),
      driveLogicalPath: json['driveLogicalPath']?.toString(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectRecord.updatedAt is required');
        }
        return value;
      })(),
      archivedAt: json['archivedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'projectId': projectId,
      'workspaceId': workspaceId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'ownerUserId': ownerUserId,
      'name': name,
      'description': description,
      'visibility': visibility,
      'status': status,
      'driveAccessMode': driveAccessMode,
      'defaultAgentId': defaultAgentId,
      'defaultModelId': defaultModelId,
      'importSourceKind': importSourceKind,
      'importSourceRef': importSourceRef,
      'driveSpaceId': driveSpaceId,
      'driveRootEntryId': driveRootEntryId,
      'driveLogicalPath': driveLogicalPath,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'archivedAt': archivedAt,
    };
  }
}

class AgentProjectResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentProjectResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentProjectResponse.fromJson(Map<String, dynamic> json) {
    return AgentProjectResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentProjectResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentProjectResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentProjectListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentProjectListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentProjectListResponse.fromJson(Map<String, dynamic> json) {
    return AgentProjectListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentProjectListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentProjectListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentProjectRequest {
  final String? projectId;
  final String? workspaceId;
  final String name;
  final String? description;
  final String? visibility;
  final String? driveAccessMode;
  final String? defaultAgentId;
  final String? defaultModelId;

  CreateAgentProjectRequest({
    this.projectId,
    this.workspaceId,
    required this.name,
    this.description,
    this.visibility,
    this.driveAccessMode,
    this.defaultAgentId,
    this.defaultModelId
  });

  factory CreateAgentProjectRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentProjectRequest(
      projectId: json['projectId']?.toString(),
      workspaceId: json['workspaceId']?.toString(),
      name: (() {
        final value = json['name']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProjectRequest.name is required');
        }
        return value;
      })(),
      description: json['description']?.toString(),
      visibility: json['visibility']?.toString(),
      driveAccessMode: json['driveAccessMode']?.toString(),
      defaultAgentId: json['defaultAgentId']?.toString(),
      defaultModelId: json['defaultModelId']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'projectId': projectId,
      'workspaceId': workspaceId,
      'name': name,
      'description': description,
      'visibility': visibility,
      'driveAccessMode': driveAccessMode,
      'defaultAgentId': defaultAgentId,
      'defaultModelId': defaultModelId,
    };
  }
}

class ImportAgentProjectRequest {
  final String workspaceId;
  final String? projectId;
  final String name;
  final String? description;
  final String sourceKind;
  final String sourceRef;
  final String driveSpaceId;
  final String driveRootEntryId;
  final String? driveLogicalPath;

  ImportAgentProjectRequest({
    required this.workspaceId,
    this.projectId,
    required this.name,
    this.description,
    required this.sourceKind,
    required this.sourceRef,
    required this.driveSpaceId,
    required this.driveRootEntryId,
    this.driveLogicalPath
  });

  factory ImportAgentProjectRequest.fromJson(Map<String, dynamic> json) {
    return ImportAgentProjectRequest(
      workspaceId: (() {
        final value = json['workspaceId']?.toString();
        if (value == null) {
          throw FormatException('ImportAgentProjectRequest.workspaceId is required');
        }
        return value;
      })(),
      projectId: json['projectId']?.toString(),
      name: (() {
        final value = json['name']?.toString();
        if (value == null) {
          throw FormatException('ImportAgentProjectRequest.name is required');
        }
        return value;
      })(),
      description: json['description']?.toString(),
      sourceKind: (() {
        final value = json['sourceKind']?.toString();
        if (value == null) {
          throw FormatException('ImportAgentProjectRequest.sourceKind is required');
        }
        return value;
      })(),
      sourceRef: (() {
        final value = json['sourceRef']?.toString();
        if (value == null) {
          throw FormatException('ImportAgentProjectRequest.sourceRef is required');
        }
        return value;
      })(),
      driveSpaceId: (() {
        final value = json['driveSpaceId']?.toString();
        if (value == null) {
          throw FormatException('ImportAgentProjectRequest.driveSpaceId is required');
        }
        return value;
      })(),
      driveRootEntryId: (() {
        final value = json['driveRootEntryId']?.toString();
        if (value == null) {
          throw FormatException('ImportAgentProjectRequest.driveRootEntryId is required');
        }
        return value;
      })(),
      driveLogicalPath: json['driveLogicalPath']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'workspaceId': workspaceId,
      'projectId': projectId,
      'name': name,
      'description': description,
      'sourceKind': sourceKind,
      'sourceRef': sourceRef,
      'driveSpaceId': driveSpaceId,
      'driveRootEntryId': driveRootEntryId,
      'driveLogicalPath': driveLogicalPath,
    };
  }
}

class UpdateAgentProjectRequest {
  final String? expectedVersion;
  final String? name;
  final String? description;
  final String? visibility;
  final String? driveAccessMode;
  final String? defaultAgentId;
  final String? defaultModelId;

  UpdateAgentProjectRequest({
    this.expectedVersion,
    this.name,
    this.description,
    this.visibility,
    this.driveAccessMode,
    this.defaultAgentId,
    this.defaultModelId
  });

  factory UpdateAgentProjectRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentProjectRequest(
      expectedVersion: json['expectedVersion']?.toString(),
      name: json['name']?.toString(),
      description: json['description']?.toString(),
      visibility: json['visibility']?.toString(),
      driveAccessMode: json['driveAccessMode']?.toString(),
      defaultAgentId: json['defaultAgentId']?.toString(),
      defaultModelId: json['defaultModelId']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'name': name,
      'description': description,
      'visibility': visibility,
      'driveAccessMode': driveAccessMode,
      'defaultAgentId': defaultAgentId,
      'defaultModelId': defaultModelId,
    };
  }
}

class AgentProjectMutationRequest {
  final String? expectedVersion;

  AgentProjectMutationRequest({
    this.expectedVersion
  });

  factory AgentProjectMutationRequest.fromJson(Map<String, dynamic> json) {
    return AgentProjectMutationRequest(
      expectedVersion: json['expectedVersion']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
    };
  }
}

class AgentProjectCompositionSlotRecord {
  final String id;
  final String tenantId;
  final String organizationId;
  final String projectId;
  final String slotId;
  final String slotKind;
  final String targetModule;
  final String targetRef;
  final String? targetVersionRef;
  final int priority;
  final bool enabled;
  final String policyJson;
  final String createdBy;
  final String updatedBy;
  final String version;
  final String createdAt;
  final String updatedAt;

  AgentProjectCompositionSlotRecord({
    required this.id,
    required this.tenantId,
    required this.organizationId,
    required this.projectId,
    required this.slotId,
    required this.slotKind,
    required this.targetModule,
    required this.targetRef,
    this.targetVersionRef,
    required this.priority,
    required this.enabled,
    required this.policyJson,
    required this.createdBy,
    required this.updatedBy,
    required this.version,
    required this.createdAt,
    required this.updatedAt
  });

  factory AgentProjectCompositionSlotRecord.fromJson(Map<String, dynamic> json) {
    return AgentProjectCompositionSlotRecord(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.id is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.organizationId is required');
        }
        return value;
      })(),
      projectId: (() {
        final value = json['projectId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.projectId is required');
        }
        return value;
      })(),
      slotId: (() {
        final value = json['slotId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.slotId is required');
        }
        return value;
      })(),
      slotKind: (() {
        final value = json['slotKind']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.slotKind is required');
        }
        return value;
      })(),
      targetModule: (() {
        final value = json['targetModule']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.targetModule is required');
        }
        return value;
      })(),
      targetRef: (() {
        final value = json['targetRef']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.targetRef is required');
        }
        return value;
      })(),
      targetVersionRef: json['targetVersionRef']?.toString(),
      priority: (() {
        final value = json['priority'];
        if (value is! int) {
          throw FormatException('AgentProjectCompositionSlotRecord.priority is required');
        }
        return value;
      })(),
      enabled: (() {
        final value = json['enabled'];
        if (value is! bool) {
          throw FormatException('AgentProjectCompositionSlotRecord.enabled is required');
        }
        return value;
      })(),
      policyJson: (() {
        final value = json['policyJson']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.policyJson is required');
        }
        return value;
      })(),
      createdBy: (() {
        final value = json['createdBy']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.createdBy is required');
        }
        return value;
      })(),
      updatedBy: (() {
        final value = json['updatedBy']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.updatedBy is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotRecord.updatedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'projectId': projectId,
      'slotId': slotId,
      'slotKind': slotKind,
      'targetModule': targetModule,
      'targetRef': targetRef,
      'targetVersionRef': targetVersionRef,
      'priority': priority,
      'enabled': enabled,
      'policyJson': policyJson,
      'createdBy': createdBy,
      'updatedBy': updatedBy,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
    };
  }
}

class AgentProjectCompositionSlotResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentProjectCompositionSlotResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentProjectCompositionSlotResponse.fromJson(Map<String, dynamic> json) {
    return AgentProjectCompositionSlotResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentProjectCompositionSlotResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentProjectCompositionSlotResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentProjectCompositionSlotListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentProjectCompositionSlotListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentProjectCompositionSlotListResponse.fromJson(Map<String, dynamic> json) {
    return AgentProjectCompositionSlotListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentProjectCompositionSlotListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentProjectCompositionSlotListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentProjectCompositionSlotListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentProjectCompositionSlotRequest {
  final String slotId;
  final String slotKind;
  final String targetModule;
  final String targetRef;
  final String? targetVersionRef;
  final int? priority;
  final bool? enabled;
  final String? policyJson;

  CreateAgentProjectCompositionSlotRequest({
    required this.slotId,
    required this.slotKind,
    required this.targetModule,
    required this.targetRef,
    this.targetVersionRef,
    this.priority,
    this.enabled,
    this.policyJson
  });

  factory CreateAgentProjectCompositionSlotRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentProjectCompositionSlotRequest(
      slotId: (() {
        final value = json['slotId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProjectCompositionSlotRequest.slotId is required');
        }
        return value;
      })(),
      slotKind: (() {
        final value = json['slotKind']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProjectCompositionSlotRequest.slotKind is required');
        }
        return value;
      })(),
      targetModule: (() {
        final value = json['targetModule']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProjectCompositionSlotRequest.targetModule is required');
        }
        return value;
      })(),
      targetRef: (() {
        final value = json['targetRef']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentProjectCompositionSlotRequest.targetRef is required');
        }
        return value;
      })(),
      targetVersionRef: json['targetVersionRef']?.toString(),
      priority: json['priority'] is int ? json['priority'] : null,
      enabled: json['enabled'] is bool ? json['enabled'] : null,
      policyJson: json['policyJson']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'slotId': slotId,
      'slotKind': slotKind,
      'targetModule': targetModule,
      'targetRef': targetRef,
      'targetVersionRef': targetVersionRef,
      'priority': priority,
      'enabled': enabled,
      'policyJson': policyJson,
    };
  }
}

class UpdateAgentProjectCompositionSlotRequest {
  final String expectedVersion;
  final String? slotKind;
  final String? targetModule;
  final String? targetRef;
  final String? targetVersionRef;
  final bool? clearTargetVersionRef;
  final int? priority;
  final bool? enabled;
  final String? policyJson;

  UpdateAgentProjectCompositionSlotRequest({
    required this.expectedVersion,
    this.slotKind,
    this.targetModule,
    this.targetRef,
    this.targetVersionRef,
    this.clearTargetVersionRef,
    this.priority,
    this.enabled,
    this.policyJson
  });

  factory UpdateAgentProjectCompositionSlotRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentProjectCompositionSlotRequest(
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentProjectCompositionSlotRequest.expectedVersion is required');
        }
        return value;
      })(),
      slotKind: json['slotKind']?.toString(),
      targetModule: json['targetModule']?.toString(),
      targetRef: json['targetRef']?.toString(),
      targetVersionRef: json['targetVersionRef']?.toString(),
      clearTargetVersionRef: json['clearTargetVersionRef'] is bool ? json['clearTargetVersionRef'] : null,
      priority: json['priority'] is int ? json['priority'] : null,
      enabled: json['enabled'] is bool ? json['enabled'] : null,
      policyJson: json['policyJson']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'slotKind': slotKind,
      'targetModule': targetModule,
      'targetRef': targetRef,
      'targetVersionRef': targetVersionRef,
      'clearTargetVersionRef': clearTargetVersionRef,
      'priority': priority,
      'enabled': enabled,
      'policyJson': policyJson,
    };
  }
}

class AgentResourceUserStateRecord {
  final String id;
  final String tenantId;
  final String organizationId;
  final String userId;
  final String resourceType;
  final String resourceId;
  final String? pinnedAt;
  final String? hiddenAt;
  final String? lastOpenedAt;
  final String? lastReadItemSequence;
  final String? customTitle;
  final String version;
  final String createdAt;
  final String updatedAt;

  AgentResourceUserStateRecord({
    required this.id,
    required this.tenantId,
    required this.organizationId,
    required this.userId,
    required this.resourceType,
    required this.resourceId,
    this.pinnedAt,
    this.hiddenAt,
    this.lastOpenedAt,
    this.lastReadItemSequence,
    this.customTitle,
    required this.version,
    required this.createdAt,
    required this.updatedAt
  });

  factory AgentResourceUserStateRecord.fromJson(Map<String, dynamic> json) {
    return AgentResourceUserStateRecord(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.id is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.organizationId is required');
        }
        return value;
      })(),
      userId: (() {
        final value = json['userId']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.userId is required');
        }
        return value;
      })(),
      resourceType: (() {
        final value = json['resourceType']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.resourceType is required');
        }
        return value;
      })(),
      resourceId: (() {
        final value = json['resourceId']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.resourceId is required');
        }
        return value;
      })(),
      pinnedAt: json['pinnedAt']?.toString(),
      hiddenAt: json['hiddenAt']?.toString(),
      lastOpenedAt: json['lastOpenedAt']?.toString(),
      lastReadItemSequence: json['lastReadItemSequence']?.toString(),
      customTitle: json['customTitle']?.toString(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateRecord.updatedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'userId': userId,
      'resourceType': resourceType,
      'resourceId': resourceId,
      'pinnedAt': pinnedAt,
      'hiddenAt': hiddenAt,
      'lastOpenedAt': lastOpenedAt,
      'lastReadItemSequence': lastReadItemSequence,
      'customTitle': customTitle,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
    };
  }
}

class AgentResourceUserStateResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentResourceUserStateResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentResourceUserStateResponse.fromJson(Map<String, dynamic> json) {
    return AgentResourceUserStateResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentResourceUserStateResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentResourceUserStateResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentResourceUserStateListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentResourceUserStateListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentResourceUserStateListResponse.fromJson(Map<String, dynamic> json) {
    return AgentResourceUserStateListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentResourceUserStateListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentResourceUserStateListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentResourceUserStateListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class UpdateAgentSessionUserStateRequest {
  final String? expectedVersion;
  final bool? pinned;
  final bool? hidden;
  final bool? markOpened;
  final String? lastReadItemSequence;
  final String? customTitle;
  final bool? clearCustomTitle;

  UpdateAgentSessionUserStateRequest({
    this.expectedVersion,
    this.pinned,
    this.hidden,
    this.markOpened,
    this.lastReadItemSequence,
    this.customTitle,
    this.clearCustomTitle
  });

  factory UpdateAgentSessionUserStateRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentSessionUserStateRequest(
      expectedVersion: json['expectedVersion']?.toString(),
      pinned: json['pinned'] is bool ? json['pinned'] : null,
      hidden: json['hidden'] is bool ? json['hidden'] : null,
      markOpened: json['markOpened'] is bool ? json['markOpened'] : null,
      lastReadItemSequence: json['lastReadItemSequence']?.toString(),
      customTitle: json['customTitle']?.toString(),
      clearCustomTitle: json['clearCustomTitle'] is bool ? json['clearCustomTitle'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'pinned': pinned,
      'hidden': hidden,
      'markOpened': markOpened,
      'lastReadItemSequence': lastReadItemSequence,
      'customTitle': customTitle,
      'clearCustomTitle': clearCustomTitle,
    };
  }
}

class AgentItemFeedbackRecord {
  final String id;
  final String tenantId;
  final String organizationId;
  final String itemId;
  final String userId;
  final String rating;
  final String? reasonCode;
  final String? comment;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? deletedAt;

  AgentItemFeedbackRecord({
    required this.id,
    required this.tenantId,
    required this.organizationId,
    required this.itemId,
    required this.userId,
    required this.rating,
    this.reasonCode,
    this.comment,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.deletedAt
  });

  factory AgentItemFeedbackRecord.fromJson(Map<String, dynamic> json) {
    return AgentItemFeedbackRecord(
      id: (() {
        final value = json['id']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.id is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.organizationId is required');
        }
        return value;
      })(),
      itemId: (() {
        final value = json['itemId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.itemId is required');
        }
        return value;
      })(),
      userId: (() {
        final value = json['userId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.userId is required');
        }
        return value;
      })(),
      rating: (() {
        final value = json['rating']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.rating is required');
        }
        return value;
      })(),
      reasonCode: json['reasonCode']?.toString(),
      comment: json['comment']?.toString(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackRecord.updatedAt is required');
        }
        return value;
      })(),
      deletedAt: json['deletedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'id': id,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'itemId': itemId,
      'userId': userId,
      'rating': rating,
      'reasonCode': reasonCode,
      'comment': comment,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'deletedAt': deletedAt,
    };
  }
}

class AgentItemFeedbackResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentItemFeedbackResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentItemFeedbackResponse.fromJson(Map<String, dynamic> json) {
    return AgentItemFeedbackResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentItemFeedbackResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentItemFeedbackResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentItemFeedbackListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentItemFeedbackListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentItemFeedbackListResponse.fromJson(Map<String, dynamic> json) {
    return AgentItemFeedbackListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentItemFeedbackListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentItemFeedbackListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemFeedbackListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class UpdateAgentItemFeedbackRequest {
  final String? expectedVersion;
  final String? rating;
  final bool? clearFeedback;
  final String? reasonCode;
  final String? comment;

  UpdateAgentItemFeedbackRequest({
    this.expectedVersion,
    this.rating,
    this.clearFeedback,
    this.reasonCode,
    this.comment
  });

  factory UpdateAgentItemFeedbackRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentItemFeedbackRequest(
      expectedVersion: json['expectedVersion']?.toString(),
      rating: json['rating']?.toString(),
      clearFeedback: json['clearFeedback'] is bool ? json['clearFeedback'] : null,
      reasonCode: json['reasonCode']?.toString(),
      comment: json['comment']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'rating': rating,
      'clearFeedback': clearFeedback,
      'reasonCode': reasonCode,
      'comment': comment,
    };
  }
}

class AgentTaskRecord {
  final String taskId;
  final String agentId;
  final String title;
  final String prompt;
  final String status;
  final String? externalRef;
  final String? metadataJson;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? startedAt;
  final String? completedAt;
  final String? cancelledAt;

  AgentTaskRecord({
    required this.taskId,
    required this.agentId,
    required this.title,
    required this.prompt,
    required this.status,
    this.externalRef,
    this.metadataJson,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.startedAt,
    this.completedAt,
    this.cancelledAt
  });

  factory AgentTaskRecord.fromJson(Map<String, dynamic> json) {
    return AgentTaskRecord(
      taskId: (() {
        final value = json['taskId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.taskId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.agentId is required');
        }
        return value;
      })(),
      title: (() {
        final value = json['title']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.title is required');
        }
        return value;
      })(),
      prompt: (() {
        final value = json['prompt']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.prompt is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.status is required');
        }
        return value;
      })(),
      externalRef: json['externalRef']?.toString(),
      metadataJson: json['metadataJson']?.toString(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskRecord.updatedAt is required');
        }
        return value;
      })(),
      startedAt: json['startedAt']?.toString(),
      completedAt: json['completedAt']?.toString(),
      cancelledAt: json['cancelledAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'taskId': taskId,
      'agentId': agentId,
      'title': title,
      'prompt': prompt,
      'status': status,
      'externalRef': externalRef,
      'metadataJson': metadataJson,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'startedAt': startedAt,
      'completedAt': completedAt,
      'cancelledAt': cancelledAt,
    };
  }
}

class AgentTaskResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentTaskResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentTaskResponse.fromJson(Map<String, dynamic> json) {
    return AgentTaskResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentTaskResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentTaskResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentTaskListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentTaskListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentTaskListResponse.fromJson(Map<String, dynamic> json) {
    return AgentTaskListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentTaskListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentTaskListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTaskListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentTaskRequest {
  final String title;
  final String prompt;
  final String? externalRef;
  final String? metadataJson;
  final String requestedAt;

  CreateAgentTaskRequest({
    required this.title,
    required this.prompt,
    this.externalRef,
    this.metadataJson,
    required this.requestedAt
  });

  factory CreateAgentTaskRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentTaskRequest(
      title: (() {
        final value = json['title']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTaskRequest.title is required');
        }
        return value;
      })(),
      prompt: (() {
        final value = json['prompt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTaskRequest.prompt is required');
        }
        return value;
      })(),
      externalRef: json['externalRef']?.toString(),
      metadataJson: json['metadataJson']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTaskRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'title': title,
      'prompt': prompt,
      'externalRef': externalRef,
      'metadataJson': metadataJson,
      'requestedAt': requestedAt,
    };
  }
}

class CancelAgentTaskRequest {
  final String? expectedVersion;
  final String requestedAt;

  CancelAgentTaskRequest({
    this.expectedVersion,
    required this.requestedAt
  });

  factory CancelAgentTaskRequest.fromJson(Map<String, dynamic> json) {
    return CancelAgentTaskRequest(
      expectedVersion: json['expectedVersion']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CancelAgentTaskRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AppUpdateAgentSessionRequest {
  final String? expectedVersion;
  final String? title;
  final String? projectId;
  final bool? clearProject;

  AppUpdateAgentSessionRequest({
    this.expectedVersion,
    this.title,
    this.projectId,
    this.clearProject
  });

  factory AppUpdateAgentSessionRequest.fromJson(Map<String, dynamic> json) {
    return AppUpdateAgentSessionRequest(
      expectedVersion: json['expectedVersion']?.toString(),
      title: json['title']?.toString(),
      projectId: json['projectId']?.toString(),
      clearProject: json['clearProject'] is bool ? json['clearProject'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'title': title,
      'projectId': projectId,
      'clearProject': clearProject,
    };
  }
}

class CodeEngineCatalogListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  CodeEngineCatalogListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory CodeEngineCatalogListResponse.fromJson(Map<String, dynamic> json) {
    return CodeEngineCatalogListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('CodeEngineCatalogListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('CodeEngineCatalogListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('CodeEngineCatalogListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CodeEngineCatalog {
  final List<CodeEngineCatalogEngine> engines;

  CodeEngineCatalog({
    required this.engines
  });

  factory CodeEngineCatalog.fromJson(Map<String, dynamic> json) {
    return CodeEngineCatalog(
      engines: (() {
        final list = _sdkworkAsList(json['engines']);
        if (list == null) {
          throw FormatException('CodeEngineCatalog.engines is required');
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : CodeEngineCatalogEngine.fromJson(map);
      })())
            .whereType<CodeEngineCatalogEngine>()
            .toList();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'engines': engines.map((item) => item.toJson()).toList(),
    };
  }
}

class CodeEngineCatalogEngine {
  final String engineKey;
  final String agentId;
  final String bindingId;
  final List<CodeEngineModelCatalogEntry> models;

  CodeEngineCatalogEngine({
    required this.engineKey,
    required this.agentId,
    required this.bindingId,
    required this.models
  });

  factory CodeEngineCatalogEngine.fromJson(Map<String, dynamic> json) {
    return CodeEngineCatalogEngine(
      engineKey: (() {
        final value = json['engineKey']?.toString();
        if (value == null) {
          throw FormatException('CodeEngineCatalogEngine.engineKey is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('CodeEngineCatalogEngine.agentId is required');
        }
        return value;
      })(),
      bindingId: (() {
        final value = json['bindingId']?.toString();
        if (value == null) {
          throw FormatException('CodeEngineCatalogEngine.bindingId is required');
        }
        return value;
      })(),
      models: (() {
        final list = _sdkworkAsList(json['models']);
        if (list == null) {
          throw FormatException('CodeEngineCatalogEngine.models is required');
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : CodeEngineModelCatalogEntry.fromJson(map);
      })())
            .whereType<CodeEngineModelCatalogEntry>()
            .toList();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'engineKey': engineKey,
      'agentId': agentId,
      'bindingId': bindingId,
      'models': models.map((item) => item.toJson()).toList(),
    };
  }
}

class CodeEngineModelCatalogEntry {
  final String engineKey;
  final String modelId;
  final String label;
  final String description;
  final String providerId;
  final String bindingId;
  final bool defaultForEngine;

  CodeEngineModelCatalogEntry({
    required this.engineKey,
    required this.modelId,
    required this.label,
    required this.description,
    required this.providerId,
    required this.bindingId,
    required this.defaultForEngine
  });

  factory CodeEngineModelCatalogEntry.fromJson(Map<String, dynamic> json) {
    return CodeEngineModelCatalogEntry(
      engineKey: (() {
        final value = json['engineKey']?.toString();
        if (value == null) {
          throw FormatException('CodeEngineModelCatalogEntry.engineKey is required');
        }
        return value;
      })(),
      modelId: (() {
        final value = json['modelId']?.toString();
        if (value == null) {
          throw FormatException('CodeEngineModelCatalogEntry.modelId is required');
        }
        return value;
      })(),
      label: (() {
        final value = json['label']?.toString();
        if (value == null) {
          throw FormatException('CodeEngineModelCatalogEntry.label is required');
        }
        return value;
      })(),
      description: (() {
        final value = json['description']?.toString();
        if (value == null) {
          throw FormatException('CodeEngineModelCatalogEntry.description is required');
        }
        return value;
      })(),
      providerId: (() {
        final value = json['providerId']?.toString();
        if (value == null) {
          throw FormatException('CodeEngineModelCatalogEntry.providerId is required');
        }
        return value;
      })(),
      bindingId: (() {
        final value = json['bindingId']?.toString();
        if (value == null) {
          throw FormatException('CodeEngineModelCatalogEntry.bindingId is required');
        }
        return value;
      })(),
      defaultForEngine: (() {
        final value = json['defaultForEngine'];
        if (value is! bool) {
          throw FormatException('CodeEngineModelCatalogEntry.defaultForEngine is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'engineKey': engineKey,
      'modelId': modelId,
      'label': label,
      'description': description,
      'providerId': providerId,
      'bindingId': bindingId,
      'defaultForEngine': defaultForEngine,
    };
  }
}

class McpServerMarketplaceListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  McpServerMarketplaceListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory McpServerMarketplaceListResponse.fromJson(Map<String, dynamic> json) {
    return McpServerMarketplaceListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('McpServerMarketplaceListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('McpServerMarketplaceListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('McpServerMarketplaceListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class McpServerMarketplaceRecord {
  final String agentId;
  final String slotId;
  final String serverId;
  final String targetModule;
  final String targetRef;
  final String? targetVersionRef;
  final bool enabled;
  final int priority;

  McpServerMarketplaceRecord({
    required this.agentId,
    required this.slotId,
    required this.serverId,
    required this.targetModule,
    required this.targetRef,
    this.targetVersionRef,
    required this.enabled,
    required this.priority
  });

  factory McpServerMarketplaceRecord.fromJson(Map<String, dynamic> json) {
    return McpServerMarketplaceRecord(
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('McpServerMarketplaceRecord.agentId is required');
        }
        return value;
      })(),
      slotId: (() {
        final value = json['slotId']?.toString();
        if (value == null) {
          throw FormatException('McpServerMarketplaceRecord.slotId is required');
        }
        return value;
      })(),
      serverId: (() {
        final value = json['serverId']?.toString();
        if (value == null) {
          throw FormatException('McpServerMarketplaceRecord.serverId is required');
        }
        return value;
      })(),
      targetModule: (() {
        final value = json['targetModule']?.toString();
        if (value == null) {
          throw FormatException('McpServerMarketplaceRecord.targetModule is required');
        }
        return value;
      })(),
      targetRef: (() {
        final value = json['targetRef']?.toString();
        if (value == null) {
          throw FormatException('McpServerMarketplaceRecord.targetRef is required');
        }
        return value;
      })(),
      targetVersionRef: json['targetVersionRef']?.toString(),
      enabled: (() {
        final value = json['enabled'];
        if (value is! bool) {
          throw FormatException('McpServerMarketplaceRecord.enabled is required');
        }
        return value;
      })(),
      priority: (() {
        final value = json['priority'];
        if (value is! int) {
          throw FormatException('McpServerMarketplaceRecord.priority is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'agentId': agentId,
      'slotId': slotId,
      'serverId': serverId,
      'targetModule': targetModule,
      'targetRef': targetRef,
      'targetVersionRef': targetVersionRef,
      'enabled': enabled,
      'priority': priority,
    };
  }
}

class AgentSessionRecord {
  final String sessionId;
  final String tenantId;
  final String organizationId;
  final String agentId;
  final String ownerUserId;
  final String? projectId;
  final String sessionKind;
  final String entrySurface;
  final String? sourceModule;
  final String? sourceContextKind;
  final String? sourceContextId;
  final String? parentSessionId;
  final String? forkedFromTurnId;
  final String? title;
  final String status;
  final String itemCount;
  final String lastItemSequence;
  final String totalInputTokens;
  final String totalOutputTokens;
  final String? idempotencyKey;
  final String? payloadHash;
  final String createdBy;
  final String updatedBy;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? lastItemAt;
  final String? closedAt;
  final String? archivedAt;
  final String? archivedBy;
  final String? deletedAt;
  final String? deletedBy;
  final String? retentionUntil;

  AgentSessionRecord({
    required this.sessionId,
    required this.tenantId,
    required this.organizationId,
    required this.agentId,
    required this.ownerUserId,
    this.projectId,
    required this.sessionKind,
    required this.entrySurface,
    this.sourceModule,
    this.sourceContextKind,
    this.sourceContextId,
    this.parentSessionId,
    this.forkedFromTurnId,
    this.title,
    required this.status,
    required this.itemCount,
    required this.lastItemSequence,
    required this.totalInputTokens,
    required this.totalOutputTokens,
    this.idempotencyKey,
    this.payloadHash,
    required this.createdBy,
    required this.updatedBy,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.lastItemAt,
    this.closedAt,
    this.archivedAt,
    this.archivedBy,
    this.deletedAt,
    this.deletedBy,
    this.retentionUntil
  });

  factory AgentSessionRecord.fromJson(Map<String, dynamic> json) {
    return AgentSessionRecord(
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.sessionId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.organizationId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.agentId is required');
        }
        return value;
      })(),
      ownerUserId: (() {
        final value = json['ownerUserId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.ownerUserId is required');
        }
        return value;
      })(),
      projectId: json['projectId']?.toString(),
      sessionKind: (() {
        final value = json['sessionKind']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.sessionKind is required');
        }
        return value;
      })(),
      entrySurface: (() {
        final value = json['entrySurface']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.entrySurface is required');
        }
        return value;
      })(),
      sourceModule: json['sourceModule']?.toString(),
      sourceContextKind: json['sourceContextKind']?.toString(),
      sourceContextId: json['sourceContextId']?.toString(),
      parentSessionId: json['parentSessionId']?.toString(),
      forkedFromTurnId: json['forkedFromTurnId']?.toString(),
      title: json['title']?.toString(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.status is required');
        }
        return value;
      })(),
      itemCount: (() {
        final value = json['itemCount']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.itemCount is required');
        }
        return value;
      })(),
      lastItemSequence: (() {
        final value = json['lastItemSequence']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.lastItemSequence is required');
        }
        return value;
      })(),
      totalInputTokens: (() {
        final value = json['totalInputTokens']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.totalInputTokens is required');
        }
        return value;
      })(),
      totalOutputTokens: (() {
        final value = json['totalOutputTokens']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.totalOutputTokens is required');
        }
        return value;
      })(),
      idempotencyKey: json['idempotencyKey']?.toString(),
      payloadHash: json['payloadHash']?.toString(),
      createdBy: (() {
        final value = json['createdBy']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.createdBy is required');
        }
        return value;
      })(),
      updatedBy: (() {
        final value = json['updatedBy']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.updatedBy is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRecord.updatedAt is required');
        }
        return value;
      })(),
      lastItemAt: json['lastItemAt']?.toString(),
      closedAt: json['closedAt']?.toString(),
      archivedAt: json['archivedAt']?.toString(),
      archivedBy: json['archivedBy']?.toString(),
      deletedAt: json['deletedAt']?.toString(),
      deletedBy: json['deletedBy']?.toString(),
      retentionUntil: json['retentionUntil']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'sessionId': sessionId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'agentId': agentId,
      'ownerUserId': ownerUserId,
      'projectId': projectId,
      'sessionKind': sessionKind,
      'entrySurface': entrySurface,
      'sourceModule': sourceModule,
      'sourceContextKind': sourceContextKind,
      'sourceContextId': sourceContextId,
      'parentSessionId': parentSessionId,
      'forkedFromTurnId': forkedFromTurnId,
      'title': title,
      'status': status,
      'itemCount': itemCount,
      'lastItemSequence': lastItemSequence,
      'totalInputTokens': totalInputTokens,
      'totalOutputTokens': totalOutputTokens,
      'idempotencyKey': idempotencyKey,
      'payloadHash': payloadHash,
      'createdBy': createdBy,
      'updatedBy': updatedBy,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'lastItemAt': lastItemAt,
      'closedAt': closedAt,
      'archivedAt': archivedAt,
      'archivedBy': archivedBy,
      'deletedAt': deletedAt,
      'deletedBy': deletedBy,
      'retentionUntil': retentionUntil,
    };
  }
}

class AgentSessionResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentSessionListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionListResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentSessionRequest {
  final String? sessionId;
  final String? projectId;
  final String sessionKind;
  final String entrySurface;
  final String? sourceModule;
  final String? sourceContextKind;
  final String? sourceContextId;
  final String? parentSessionId;
  final String? forkedFromTurnId;
  final String? title;
  final String idempotencyKey;
  final String payloadHash;
  final String requestedAt;

  CreateAgentSessionRequest({
    this.sessionId,
    this.projectId,
    required this.sessionKind,
    required this.entrySurface,
    this.sourceModule,
    this.sourceContextKind,
    this.sourceContextId,
    this.parentSessionId,
    this.forkedFromTurnId,
    this.title,
    required this.idempotencyKey,
    required this.payloadHash,
    required this.requestedAt
  });

  factory CreateAgentSessionRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentSessionRequest(
      sessionId: json['sessionId']?.toString(),
      projectId: json['projectId']?.toString(),
      sessionKind: (() {
        final value = json['sessionKind']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRequest.sessionKind is required');
        }
        return value;
      })(),
      entrySurface: (() {
        final value = json['entrySurface']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRequest.entrySurface is required');
        }
        return value;
      })(),
      sourceModule: json['sourceModule']?.toString(),
      sourceContextKind: json['sourceContextKind']?.toString(),
      sourceContextId: json['sourceContextId']?.toString(),
      parentSessionId: json['parentSessionId']?.toString(),
      forkedFromTurnId: json['forkedFromTurnId']?.toString(),
      title: json['title']?.toString(),
      idempotencyKey: (() {
        final value = json['idempotencyKey']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRequest.idempotencyKey is required');
        }
        return value;
      })(),
      payloadHash: (() {
        final value = json['payloadHash']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRequest.payloadHash is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'sessionId': sessionId,
      'projectId': projectId,
      'sessionKind': sessionKind,
      'entrySurface': entrySurface,
      'sourceModule': sourceModule,
      'sourceContextKind': sourceContextKind,
      'sourceContextId': sourceContextId,
      'parentSessionId': parentSessionId,
      'forkedFromTurnId': forkedFromTurnId,
      'title': title,
      'idempotencyKey': idempotencyKey,
      'payloadHash': payloadHash,
      'requestedAt': requestedAt,
    };
  }
}

class CloseAgentSessionRequest {
  final String expectedVersion;
  final String requestedAt;

  CloseAgentSessionRequest({
    required this.expectedVersion,
    required this.requestedAt
  });

  factory CloseAgentSessionRequest.fromJson(Map<String, dynamic> json) {
    return CloseAgentSessionRequest(
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('CloseAgentSessionRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CloseAgentSessionRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentItemDriveRefRecord {
  final String resourceRole;
  final String driveSpaceId;
  final String driveNodeId;
  final String? mediaResourceId;
  final String? objectBlobId;
  final String? resourceHash;
  final String? altText;
  final int sortOrder;
  final String status;
  final String createdBy;
  final String createdAt;
  final String updatedAt;
  final String? deletedAt;
  final String? retentionUntil;

  AgentItemDriveRefRecord({
    required this.resourceRole,
    required this.driveSpaceId,
    required this.driveNodeId,
    this.mediaResourceId,
    this.objectBlobId,
    this.resourceHash,
    this.altText,
    required this.sortOrder,
    required this.status,
    required this.createdBy,
    required this.createdAt,
    required this.updatedAt,
    this.deletedAt,
    this.retentionUntil
  });

  factory AgentItemDriveRefRecord.fromJson(Map<String, dynamic> json) {
    return AgentItemDriveRefRecord(
      resourceRole: (() {
        final value = json['resourceRole']?.toString();
        if (value == null) {
          throw FormatException('AgentItemDriveRefRecord.resourceRole is required');
        }
        return value;
      })(),
      driveSpaceId: (() {
        final value = json['driveSpaceId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemDriveRefRecord.driveSpaceId is required');
        }
        return value;
      })(),
      driveNodeId: (() {
        final value = json['driveNodeId']?.toString();
        if (value == null) {
          throw FormatException('AgentItemDriveRefRecord.driveNodeId is required');
        }
        return value;
      })(),
      mediaResourceId: json['mediaResourceId']?.toString(),
      objectBlobId: json['objectBlobId']?.toString(),
      resourceHash: json['resourceHash']?.toString(),
      altText: json['altText']?.toString(),
      sortOrder: (() {
        final value = json['sortOrder'];
        if (value is! int) {
          throw FormatException('AgentItemDriveRefRecord.sortOrder is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentItemDriveRefRecord.status is required');
        }
        return value;
      })(),
      createdBy: (() {
        final value = json['createdBy']?.toString();
        if (value == null) {
          throw FormatException('AgentItemDriveRefRecord.createdBy is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentItemDriveRefRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentItemDriveRefRecord.updatedAt is required');
        }
        return value;
      })(),
      deletedAt: json['deletedAt']?.toString(),
      retentionUntil: json['retentionUntil']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'resourceRole': resourceRole,
      'driveSpaceId': driveSpaceId,
      'driveNodeId': driveNodeId,
      'mediaResourceId': mediaResourceId,
      'objectBlobId': objectBlobId,
      'resourceHash': resourceHash,
      'altText': altText,
      'sortOrder': sortOrder,
      'status': status,
      'createdBy': createdBy,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'deletedAt': deletedAt,
      'retentionUntil': retentionUntil,
    };
  }
}

class AgentSessionItemRecord {
  final String tenantId;
  final String organizationId;
  final String sessionId;
  final String itemId;
  final String kind;
  final String? content;
  final String? contentType;
  final String status;
  final String sequence;
  final String inputTokens;
  final String outputTokens;
  final String? modelId;
  final String? providerId;
  final String? toolName;
  final String? toolCallId;
  final Map<String, dynamic>? toolArguments;
  final Map<String, dynamic>? toolResult;
  final String? parentItemId;
  final String? turnId;
  final List<AgentItemDriveRefRecord> driveRefs;
  final String createdBy;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? completedAt;
  final String? redactedAt;
  final String? redactedBy;
  final String? retentionUntil;

  AgentSessionItemRecord({
    required this.tenantId,
    required this.organizationId,
    required this.sessionId,
    required this.itemId,
    required this.kind,
    this.content,
    this.contentType,
    required this.status,
    required this.sequence,
    required this.inputTokens,
    required this.outputTokens,
    this.modelId,
    this.providerId,
    this.toolName,
    this.toolCallId,
    this.toolArguments,
    this.toolResult,
    this.parentItemId,
    this.turnId,
    required this.driveRefs,
    required this.createdBy,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.completedAt,
    this.redactedAt,
    this.redactedBy,
    this.retentionUntil
  });

  factory AgentSessionItemRecord.fromJson(Map<String, dynamic> json) {
    return AgentSessionItemRecord(
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.organizationId is required');
        }
        return value;
      })(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.sessionId is required');
        }
        return value;
      })(),
      itemId: (() {
        final value = json['itemId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.itemId is required');
        }
        return value;
      })(),
      kind: (() {
        final value = json['kind']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.kind is required');
        }
        return value;
      })(),
      content: json['content']?.toString(),
      contentType: json['contentType']?.toString(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.status is required');
        }
        return value;
      })(),
      sequence: (() {
        final value = json['sequence']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.sequence is required');
        }
        return value;
      })(),
      inputTokens: (() {
        final value = json['inputTokens']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.inputTokens is required');
        }
        return value;
      })(),
      outputTokens: (() {
        final value = json['outputTokens']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.outputTokens is required');
        }
        return value;
      })(),
      modelId: json['modelId']?.toString(),
      providerId: json['providerId']?.toString(),
      toolName: json['toolName']?.toString(),
      toolCallId: json['toolCallId']?.toString(),
      toolArguments: _sdkworkAsMap(json['toolArguments']),
      toolResult: _sdkworkAsMap(json['toolResult']),
      parentItemId: json['parentItemId']?.toString(),
      turnId: json['turnId']?.toString(),
      driveRefs: (() {
        final list = _sdkworkAsList(json['driveRefs']);
        if (list == null) {
          throw FormatException('AgentSessionItemRecord.driveRefs is required');
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentItemDriveRefRecord.fromJson(map);
      })())
            .whereType<AgentItemDriveRefRecord>()
            .toList();
      })(),
      createdBy: (() {
        final value = json['createdBy']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.createdBy is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemRecord.updatedAt is required');
        }
        return value;
      })(),
      completedAt: json['completedAt']?.toString(),
      redactedAt: json['redactedAt']?.toString(),
      redactedBy: json['redactedBy']?.toString(),
      retentionUntil: json['retentionUntil']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'tenantId': tenantId,
      'organizationId': organizationId,
      'sessionId': sessionId,
      'itemId': itemId,
      'kind': kind,
      'content': content,
      'contentType': contentType,
      'status': status,
      'sequence': sequence,
      'inputTokens': inputTokens,
      'outputTokens': outputTokens,
      'modelId': modelId,
      'providerId': providerId,
      'toolName': toolName,
      'toolCallId': toolCallId,
      'toolArguments': toolArguments,
      'toolResult': toolResult,
      'parentItemId': parentItemId,
      'turnId': turnId,
      'driveRefs': driveRefs.map((item) => item.toJson()).toList(),
      'createdBy': createdBy,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'completedAt': completedAt,
      'redactedAt': redactedAt,
      'redactedBy': redactedBy,
      'retentionUntil': retentionUntil,
    };
  }
}

class AgentSessionItemResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionItemResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionItemResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionItemResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionItemResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionItemResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentSessionItemListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionItemListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionItemListResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionItemListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionItemListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionItemListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionItemListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentTurnRecord {
  final String turnId;
  final String tenantId;
  final String organizationId;
  final String sessionId;
  final String agentId;
  final String ownerUserId;
  final String? runtimeBindingId;
  final String? clientRequestId;
  final String idempotencyKey;
  final String payloadHash;
  final String requestItemId;
  final String? responseItemId;
  final String turnMode;
  final String status;
  final String? requestedModelId;
  final String? providerBindingId;
  final String? modelId;
  final String? providerId;
  final String inputTokens;
  final String outputTokens;
  final String cachedTokens;
  final String? finishReason;
  final String? errorCode;
  final String? errorDetail;
  final String? traceId;
  final int attemptCount;
  final int maxAttempts;
  final String? nextRetryAt;
  final String availableAt;
  final String? leaseOwner;
  final String? leaseExpiresAt;
  final String fencingToken;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? startedAt;
  final String? completedAt;
  final String? cancelRequestedAt;
  final String? cancelledAt;
  final String? retentionUntil;

  AgentTurnRecord({
    required this.turnId,
    required this.tenantId,
    required this.organizationId,
    required this.sessionId,
    required this.agentId,
    required this.ownerUserId,
    this.runtimeBindingId,
    this.clientRequestId,
    required this.idempotencyKey,
    required this.payloadHash,
    required this.requestItemId,
    this.responseItemId,
    required this.turnMode,
    required this.status,
    this.requestedModelId,
    this.providerBindingId,
    this.modelId,
    this.providerId,
    required this.inputTokens,
    required this.outputTokens,
    required this.cachedTokens,
    this.finishReason,
    this.errorCode,
    this.errorDetail,
    this.traceId,
    required this.attemptCount,
    required this.maxAttempts,
    this.nextRetryAt,
    required this.availableAt,
    this.leaseOwner,
    this.leaseExpiresAt,
    required this.fencingToken,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.startedAt,
    this.completedAt,
    this.cancelRequestedAt,
    this.cancelledAt,
    this.retentionUntil
  });

  factory AgentTurnRecord.fromJson(Map<String, dynamic> json) {
    return AgentTurnRecord(
      turnId: (() {
        final value = json['turnId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.turnId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.organizationId is required');
        }
        return value;
      })(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.sessionId is required');
        }
        return value;
      })(),
      agentId: (() {
        final value = json['agentId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.agentId is required');
        }
        return value;
      })(),
      ownerUserId: (() {
        final value = json['ownerUserId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.ownerUserId is required');
        }
        return value;
      })(),
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      clientRequestId: json['clientRequestId']?.toString(),
      idempotencyKey: (() {
        final value = json['idempotencyKey']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.idempotencyKey is required');
        }
        return value;
      })(),
      payloadHash: (() {
        final value = json['payloadHash']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.payloadHash is required');
        }
        return value;
      })(),
      requestItemId: (() {
        final value = json['requestItemId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.requestItemId is required');
        }
        return value;
      })(),
      responseItemId: json['responseItemId']?.toString(),
      turnMode: (() {
        final value = json['turnMode']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.turnMode is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.status is required');
        }
        return value;
      })(),
      requestedModelId: json['requestedModelId']?.toString(),
      providerBindingId: json['providerBindingId']?.toString(),
      modelId: json['modelId']?.toString(),
      providerId: json['providerId']?.toString(),
      inputTokens: (() {
        final value = json['inputTokens']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.inputTokens is required');
        }
        return value;
      })(),
      outputTokens: (() {
        final value = json['outputTokens']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.outputTokens is required');
        }
        return value;
      })(),
      cachedTokens: (() {
        final value = json['cachedTokens']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.cachedTokens is required');
        }
        return value;
      })(),
      finishReason: json['finishReason']?.toString(),
      errorCode: json['errorCode']?.toString(),
      errorDetail: json['errorDetail']?.toString(),
      traceId: json['traceId']?.toString(),
      attemptCount: (() {
        final value = json['attemptCount'];
        if (value is! int) {
          throw FormatException('AgentTurnRecord.attemptCount is required');
        }
        return value;
      })(),
      maxAttempts: (() {
        final value = json['maxAttempts'];
        if (value is! int) {
          throw FormatException('AgentTurnRecord.maxAttempts is required');
        }
        return value;
      })(),
      nextRetryAt: json['nextRetryAt']?.toString(),
      availableAt: (() {
        final value = json['availableAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.availableAt is required');
        }
        return value;
      })(),
      leaseOwner: json['leaseOwner']?.toString(),
      leaseExpiresAt: json['leaseExpiresAt']?.toString(),
      fencingToken: (() {
        final value = json['fencingToken']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.fencingToken is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnRecord.updatedAt is required');
        }
        return value;
      })(),
      startedAt: json['startedAt']?.toString(),
      completedAt: json['completedAt']?.toString(),
      cancelRequestedAt: json['cancelRequestedAt']?.toString(),
      cancelledAt: json['cancelledAt']?.toString(),
      retentionUntil: json['retentionUntil']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'turnId': turnId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'sessionId': sessionId,
      'agentId': agentId,
      'ownerUserId': ownerUserId,
      'runtimeBindingId': runtimeBindingId,
      'clientRequestId': clientRequestId,
      'idempotencyKey': idempotencyKey,
      'payloadHash': payloadHash,
      'requestItemId': requestItemId,
      'responseItemId': responseItemId,
      'turnMode': turnMode,
      'status': status,
      'requestedModelId': requestedModelId,
      'providerBindingId': providerBindingId,
      'modelId': modelId,
      'providerId': providerId,
      'inputTokens': inputTokens,
      'outputTokens': outputTokens,
      'cachedTokens': cachedTokens,
      'finishReason': finishReason,
      'errorCode': errorCode,
      'errorDetail': errorDetail,
      'traceId': traceId,
      'attemptCount': attemptCount,
      'maxAttempts': maxAttempts,
      'nextRetryAt': nextRetryAt,
      'availableAt': availableAt,
      'leaseOwner': leaseOwner,
      'leaseExpiresAt': leaseExpiresAt,
      'fencingToken': fencingToken,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'startedAt': startedAt,
      'completedAt': completedAt,
      'cancelRequestedAt': cancelRequestedAt,
      'cancelledAt': cancelledAt,
      'retentionUntil': retentionUntil,
    };
  }
}

class AgentTurnResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentTurnResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentTurnResponse.fromJson(Map<String, dynamic> json) {
    return AgentTurnResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentTurnResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentTurnResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentTurnListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentTurnListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentTurnListResponse.fromJson(Map<String, dynamic> json) {
    return AgentTurnListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentTurnListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentTurnListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentTurnRequest {
  final String? turnId;
  final String content;
  final String? contentType;
  final String turnMode;
  final String? runtimeBindingId;
  final String? requestedModelId;
  final String idempotencyKey;
  final String payloadHash;
  final String? clientRequestId;
  final List<Map<String, dynamic>>? driveRefs;
  final String requestedAt;

  CreateAgentTurnRequest({
    this.turnId,
    required this.content,
    this.contentType,
    required this.turnMode,
    this.runtimeBindingId,
    this.requestedModelId,
    required this.idempotencyKey,
    required this.payloadHash,
    this.clientRequestId,
    this.driveRefs,
    required this.requestedAt
  });

  factory CreateAgentTurnRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentTurnRequest(
      turnId: json['turnId']?.toString(),
      content: (() {
        final value = json['content']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTurnRequest.content is required');
        }
        return value;
      })(),
      contentType: json['contentType']?.toString(),
      turnMode: (() {
        final value = json['turnMode']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTurnRequest.turnMode is required');
        }
        return value;
      })(),
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      requestedModelId: json['requestedModelId']?.toString(),
      idempotencyKey: (() {
        final value = json['idempotencyKey']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTurnRequest.idempotencyKey is required');
        }
        return value;
      })(),
      payloadHash: (() {
        final value = json['payloadHash']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTurnRequest.payloadHash is required');
        }
        return value;
      })(),
      clientRequestId: json['clientRequestId']?.toString(),
      driveRefs: (() {
        final list = _sdkworkAsList(json['driveRefs']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => _sdkworkAsMap(item))
            .whereType<Map<String, dynamic>>()
            .toList();
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentTurnRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'turnId': turnId,
      'content': content,
      'contentType': contentType,
      'turnMode': turnMode,
      'runtimeBindingId': runtimeBindingId,
      'requestedModelId': requestedModelId,
      'idempotencyKey': idempotencyKey,
      'payloadHash': payloadHash,
      'clientRequestId': clientRequestId,
      'driveRefs': driveRefs?.map((item) => item).toList(),
      'requestedAt': requestedAt,
    };
  }
}

class AgentTurnExecutionResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentTurnExecutionResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentTurnExecutionResponse.fromJson(Map<String, dynamic> json) {
    return AgentTurnExecutionResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentTurnExecutionResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentTurnExecutionResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnExecutionResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentTurnStreamEvent {
  final String eventType;
  final int? index;
  final String? delta;
  final AgentTurnExecutionResponse? response;

  AgentTurnStreamEvent({
    required this.eventType,
    this.index,
    this.delta,
    this.response
  });

  factory AgentTurnStreamEvent.fromJson(Map<String, dynamic> json) {
    return AgentTurnStreamEvent(
      eventType: (() {
        final value = json['eventType']?.toString();
        if (value == null) {
          throw FormatException('AgentTurnStreamEvent.eventType is required');
        }
        return value;
      })(),
      index: json['index'] is int ? json['index'] : null,
      delta: json['delta']?.toString(),
      response: (() {
        final map = _sdkworkAsMap(json['response']);
        return map == null ? null : AgentTurnExecutionResponse.fromJson(map);
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'eventType': eventType,
      'index': index,
      'delta': delta,
      'response': response?.toJson(),
    };
  }
}

class CancelAgentTurnRequest {
  final String expectedVersion;
  final String requestedAt;

  CancelAgentTurnRequest({
    required this.expectedVersion,
    required this.requestedAt
  });

  factory CancelAgentTurnRequest.fromJson(Map<String, dynamic> json) {
    return CancelAgentTurnRequest(
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('CancelAgentTurnRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CancelAgentTurnRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentInteractionOption {
  final String value;
  final String label;

  AgentInteractionOption({
    required this.value,
    required this.label
  });

  factory AgentInteractionOption.fromJson(Map<String, dynamic> json) {
    return AgentInteractionOption(
      value: (() {
        final value = json['value']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionOption.value is required');
        }
        return value;
      })(),
      label: (() {
        final value = json['label']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionOption.label is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'value': value,
      'label': label,
    };
  }
}

class AgentInteractionResolution {
  final String outcome;
  final String? answer;
  final String? selectedOptionValue;
  final String? reason;

  AgentInteractionResolution({
    required this.outcome,
    this.answer,
    this.selectedOptionValue,
    this.reason
  });

  factory AgentInteractionResolution.fromJson(Map<String, dynamic> json) {
    return AgentInteractionResolution(
      outcome: (() {
        final value = json['outcome']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionResolution.outcome is required');
        }
        return value;
      })(),
      answer: json['answer']?.toString(),
      selectedOptionValue: json['selectedOptionValue']?.toString(),
      reason: json['reason']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'outcome': outcome,
      'answer': answer,
      'selectedOptionValue': selectedOptionValue,
      'reason': reason,
    };
  }
}

class AgentInteractionRecord {
  final String interactionId;
  final String tenantId;
  final String organizationId;
  final String sessionId;
  final String? turnId;
  final String? runtimeBindingId;
  final String? providerInteractionId;
  final String kind;
  final String status;
  final String prompt;
  final List<AgentInteractionOption> options;
  final AgentInteractionResolution? resolution;
  final String? claimOwner;
  final String? claimExpiresAt;
  final String fencingToken;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? resolvedAt;
  final String? retentionUntil;

  AgentInteractionRecord({
    required this.interactionId,
    required this.tenantId,
    required this.organizationId,
    required this.sessionId,
    this.turnId,
    this.runtimeBindingId,
    this.providerInteractionId,
    required this.kind,
    required this.status,
    required this.prompt,
    required this.options,
    this.resolution,
    this.claimOwner,
    this.claimExpiresAt,
    required this.fencingToken,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.resolvedAt,
    this.retentionUntil
  });

  factory AgentInteractionRecord.fromJson(Map<String, dynamic> json) {
    return AgentInteractionRecord(
      interactionId: (() {
        final value = json['interactionId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.interactionId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.organizationId is required');
        }
        return value;
      })(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.sessionId is required');
        }
        return value;
      })(),
      turnId: json['turnId']?.toString(),
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      providerInteractionId: json['providerInteractionId']?.toString(),
      kind: (() {
        final value = json['kind']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.kind is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.status is required');
        }
        return value;
      })(),
      prompt: (() {
        final value = json['prompt']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.prompt is required');
        }
        return value;
      })(),
      options: (() {
        final list = _sdkworkAsList(json['options']);
        if (list == null) {
          throw FormatException('AgentInteractionRecord.options is required');
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentInteractionOption.fromJson(map);
      })())
            .whereType<AgentInteractionOption>()
            .toList();
      })(),
      resolution: (() {
        final map = _sdkworkAsMap(json['resolution']);
        return map == null ? null : AgentInteractionResolution.fromJson(map);
      })(),
      claimOwner: json['claimOwner']?.toString(),
      claimExpiresAt: json['claimExpiresAt']?.toString(),
      fencingToken: (() {
        final value = json['fencingToken']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.fencingToken is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionRecord.updatedAt is required');
        }
        return value;
      })(),
      resolvedAt: json['resolvedAt']?.toString(),
      retentionUntil: json['retentionUntil']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'interactionId': interactionId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'sessionId': sessionId,
      'turnId': turnId,
      'runtimeBindingId': runtimeBindingId,
      'providerInteractionId': providerInteractionId,
      'kind': kind,
      'status': status,
      'prompt': prompt,
      'options': options.map((item) => item.toJson()).toList(),
      'resolution': resolution?.toJson(),
      'claimOwner': claimOwner,
      'claimExpiresAt': claimExpiresAt,
      'fencingToken': fencingToken,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'resolvedAt': resolvedAt,
      'retentionUntil': retentionUntil,
    };
  }
}

class AgentInteractionResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentInteractionResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentInteractionResponse.fromJson(Map<String, dynamic> json) {
    return AgentInteractionResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentInteractionResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentInteractionResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentInteractionListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentInteractionListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentInteractionListResponse.fromJson(Map<String, dynamic> json) {
    return AgentInteractionListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentInteractionListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentInteractionListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentInteractionRequest {
  final String? interactionId;
  final String? turnId;
  final String? runtimeBindingId;
  final String? providerInteractionId;
  final String kind;
  final String prompt;
  final List<AgentInteractionOption>? options;
  final String? retentionUntil;
  final String requestedAt;

  CreateAgentInteractionRequest({
    this.interactionId,
    this.turnId,
    this.runtimeBindingId,
    this.providerInteractionId,
    required this.kind,
    required this.prompt,
    this.options,
    this.retentionUntil,
    required this.requestedAt
  });

  factory CreateAgentInteractionRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentInteractionRequest(
      interactionId: json['interactionId']?.toString(),
      turnId: json['turnId']?.toString(),
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      providerInteractionId: json['providerInteractionId']?.toString(),
      kind: (() {
        final value = json['kind']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentInteractionRequest.kind is required');
        }
        return value;
      })(),
      prompt: (() {
        final value = json['prompt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentInteractionRequest.prompt is required');
        }
        return value;
      })(),
      options: (() {
        final list = _sdkworkAsList(json['options']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : AgentInteractionOption.fromJson(map);
      })())
            .whereType<AgentInteractionOption>()
            .toList();
      })(),
      retentionUntil: json['retentionUntil']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentInteractionRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'interactionId': interactionId,
      'turnId': turnId,
      'runtimeBindingId': runtimeBindingId,
      'providerInteractionId': providerInteractionId,
      'kind': kind,
      'prompt': prompt,
      'options': options?.map((item) => item.toJson()).toList(),
      'retentionUntil': retentionUntil,
      'requestedAt': requestedAt,
    };
  }
}

class ClaimAgentInteractionRequest {
  final String claimOwner;
  final int? leaseSeconds;
  final String expectedVersion;
  final String requestedAt;

  ClaimAgentInteractionRequest({
    required this.claimOwner,
    this.leaseSeconds,
    required this.expectedVersion,
    required this.requestedAt
  });

  factory ClaimAgentInteractionRequest.fromJson(Map<String, dynamic> json) {
    return ClaimAgentInteractionRequest(
      claimOwner: (() {
        final value = json['claimOwner']?.toString();
        if (value == null) {
          throw FormatException('ClaimAgentInteractionRequest.claimOwner is required');
        }
        return value;
      })(),
      leaseSeconds: json['leaseSeconds'] is int ? json['leaseSeconds'] : null,
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('ClaimAgentInteractionRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('ClaimAgentInteractionRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'claimOwner': claimOwner,
      'leaseSeconds': leaseSeconds,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentInteractionClaimResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentInteractionClaimResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentInteractionClaimResponse.fromJson(Map<String, dynamic> json) {
    return AgentInteractionClaimResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentInteractionClaimResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentInteractionClaimResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentInteractionClaimResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class ApproveAgentInteractionRequest {
  final bool approved;
  final String? reason;
  final String claimToken;
  final String fencingToken;
  final String expectedVersion;
  final String requestedAt;

  ApproveAgentInteractionRequest({
    required this.approved,
    this.reason,
    required this.claimToken,
    required this.fencingToken,
    required this.expectedVersion,
    required this.requestedAt
  });

  factory ApproveAgentInteractionRequest.fromJson(Map<String, dynamic> json) {
    return ApproveAgentInteractionRequest(
      approved: (() {
        final value = json['approved'];
        if (value is! bool) {
          throw FormatException('ApproveAgentInteractionRequest.approved is required');
        }
        return value;
      })(),
      reason: json['reason']?.toString(),
      claimToken: (() {
        final value = json['claimToken']?.toString();
        if (value == null) {
          throw FormatException('ApproveAgentInteractionRequest.claimToken is required');
        }
        return value;
      })(),
      fencingToken: (() {
        final value = json['fencingToken']?.toString();
        if (value == null) {
          throw FormatException('ApproveAgentInteractionRequest.fencingToken is required');
        }
        return value;
      })(),
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('ApproveAgentInteractionRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('ApproveAgentInteractionRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'approved': approved,
      'reason': reason,
      'claimToken': claimToken,
      'fencingToken': fencingToken,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AnswerAgentInteractionRequest {
  final String answer;
  final String? selectedOptionValue;
  final bool rejected;
  final String claimToken;
  final String fencingToken;
  final String expectedVersion;
  final String requestedAt;

  AnswerAgentInteractionRequest({
    required this.answer,
    this.selectedOptionValue,
    required this.rejected,
    required this.claimToken,
    required this.fencingToken,
    required this.expectedVersion,
    required this.requestedAt
  });

  factory AnswerAgentInteractionRequest.fromJson(Map<String, dynamic> json) {
    return AnswerAgentInteractionRequest(
      answer: (() {
        final value = json['answer']?.toString();
        if (value == null) {
          throw FormatException('AnswerAgentInteractionRequest.answer is required');
        }
        return value;
      })(),
      selectedOptionValue: json['selectedOptionValue']?.toString(),
      rejected: (() {
        final value = json['rejected'];
        if (value is! bool) {
          throw FormatException('AnswerAgentInteractionRequest.rejected is required');
        }
        return value;
      })(),
      claimToken: (() {
        final value = json['claimToken']?.toString();
        if (value == null) {
          throw FormatException('AnswerAgentInteractionRequest.claimToken is required');
        }
        return value;
      })(),
      fencingToken: (() {
        final value = json['fencingToken']?.toString();
        if (value == null) {
          throw FormatException('AnswerAgentInteractionRequest.fencingToken is required');
        }
        return value;
      })(),
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('AnswerAgentInteractionRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('AnswerAgentInteractionRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'answer': answer,
      'selectedOptionValue': selectedOptionValue,
      'rejected': rejected,
      'claimToken': claimToken,
      'fencingToken': fencingToken,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentSessionCheckpointRecord {
  final String checkpointId;
  final String tenantId;
  final String organizationId;
  final String sessionId;
  final String? turnId;
  final String? runtimeBindingId;
  final String checkpointKind;
  final String? providerCheckpointRef;
  final String? driveSpaceId;
  final String? driveNodeId;
  final bool resumable;
  final String status;
  final String createdBy;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? restoredAt;
  final String? invalidatedAt;
  final String? retentionUntil;

  AgentSessionCheckpointRecord({
    required this.checkpointId,
    required this.tenantId,
    required this.organizationId,
    required this.sessionId,
    this.turnId,
    this.runtimeBindingId,
    required this.checkpointKind,
    this.providerCheckpointRef,
    this.driveSpaceId,
    this.driveNodeId,
    required this.resumable,
    required this.status,
    required this.createdBy,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.restoredAt,
    this.invalidatedAt,
    this.retentionUntil
  });

  factory AgentSessionCheckpointRecord.fromJson(Map<String, dynamic> json) {
    return AgentSessionCheckpointRecord(
      checkpointId: (() {
        final value = json['checkpointId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.checkpointId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.organizationId is required');
        }
        return value;
      })(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.sessionId is required');
        }
        return value;
      })(),
      turnId: json['turnId']?.toString(),
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      checkpointKind: (() {
        final value = json['checkpointKind']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.checkpointKind is required');
        }
        return value;
      })(),
      providerCheckpointRef: json['providerCheckpointRef']?.toString(),
      driveSpaceId: json['driveSpaceId']?.toString(),
      driveNodeId: json['driveNodeId']?.toString(),
      resumable: (() {
        final value = json['resumable'];
        if (value is! bool) {
          throw FormatException('AgentSessionCheckpointRecord.resumable is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.status is required');
        }
        return value;
      })(),
      createdBy: (() {
        final value = json['createdBy']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.createdBy is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointRecord.updatedAt is required');
        }
        return value;
      })(),
      restoredAt: json['restoredAt']?.toString(),
      invalidatedAt: json['invalidatedAt']?.toString(),
      retentionUntil: json['retentionUntil']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'checkpointId': checkpointId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'sessionId': sessionId,
      'turnId': turnId,
      'runtimeBindingId': runtimeBindingId,
      'checkpointKind': checkpointKind,
      'providerCheckpointRef': providerCheckpointRef,
      'driveSpaceId': driveSpaceId,
      'driveNodeId': driveNodeId,
      'resumable': resumable,
      'status': status,
      'createdBy': createdBy,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'restoredAt': restoredAt,
      'invalidatedAt': invalidatedAt,
      'retentionUntil': retentionUntil,
    };
  }
}

class AgentSessionCheckpointResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionCheckpointResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionCheckpointResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionCheckpointResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionCheckpointResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionCheckpointResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentSessionCheckpointListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionCheckpointListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionCheckpointListResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionCheckpointListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionCheckpointListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionCheckpointListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionCheckpointListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentSessionCheckpointRequest {
  final String? checkpointId;
  final String? turnId;
  final String? runtimeBindingId;
  final String checkpointKind;
  final String? providerCheckpointRef;
  final String? driveSpaceId;
  final String? driveNodeId;
  final bool resumable;
  final String? retentionUntil;
  final String requestedAt;

  CreateAgentSessionCheckpointRequest({
    this.checkpointId,
    this.turnId,
    this.runtimeBindingId,
    required this.checkpointKind,
    this.providerCheckpointRef,
    this.driveSpaceId,
    this.driveNodeId,
    required this.resumable,
    this.retentionUntil,
    required this.requestedAt
  });

  factory CreateAgentSessionCheckpointRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentSessionCheckpointRequest(
      checkpointId: json['checkpointId']?.toString(),
      turnId: json['turnId']?.toString(),
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      checkpointKind: (() {
        final value = json['checkpointKind']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionCheckpointRequest.checkpointKind is required');
        }
        return value;
      })(),
      providerCheckpointRef: json['providerCheckpointRef']?.toString(),
      driveSpaceId: json['driveSpaceId']?.toString(),
      driveNodeId: json['driveNodeId']?.toString(),
      resumable: (() {
        final value = json['resumable'];
        if (value is! bool) {
          throw FormatException('CreateAgentSessionCheckpointRequest.resumable is required');
        }
        return value;
      })(),
      retentionUntil: json['retentionUntil']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionCheckpointRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'checkpointId': checkpointId,
      'turnId': turnId,
      'runtimeBindingId': runtimeBindingId,
      'checkpointKind': checkpointKind,
      'providerCheckpointRef': providerCheckpointRef,
      'driveSpaceId': driveSpaceId,
      'driveNodeId': driveNodeId,
      'resumable': resumable,
      'retentionUntil': retentionUntil,
      'requestedAt': requestedAt,
    };
  }
}

class RestoreAgentSessionCheckpointRequest {
  final String expectedVersion;
  final String requestedAt;

  RestoreAgentSessionCheckpointRequest({
    required this.expectedVersion,
    required this.requestedAt
  });

  factory RestoreAgentSessionCheckpointRequest.fromJson(Map<String, dynamic> json) {
    return RestoreAgentSessionCheckpointRequest(
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('RestoreAgentSessionCheckpointRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('RestoreAgentSessionCheckpointRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class InvalidateAgentSessionCheckpointRequest {
  final String? reason;
  final String expectedVersion;
  final String requestedAt;

  InvalidateAgentSessionCheckpointRequest({
    this.reason,
    required this.expectedVersion,
    required this.requestedAt
  });

  factory InvalidateAgentSessionCheckpointRequest.fromJson(Map<String, dynamic> json) {
    return InvalidateAgentSessionCheckpointRequest(
      reason: json['reason']?.toString(),
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('InvalidateAgentSessionCheckpointRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('InvalidateAgentSessionCheckpointRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'reason': reason,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class AgentSessionRuntimeBindingRecord {
  final String runtimeBindingId;
  final String tenantId;
  final String organizationId;
  final String sessionId;
  final String? runtimeLocationId;
  final String hostMode;
  final String transportKind;
  final String providerBindingId;
  final String modelId;
  final String providerId;
  final String? nativeSessionId;
  final String? nativeSessionTreeId;
  final String? nativeParentSessionId;
  final String? nativeForkedFromSessionId;
  final String status;
  final bool isCurrent;
  final String version;
  final String createdAt;
  final String updatedAt;
  final String? activatedAt;
  final String? deactivatedAt;

  AgentSessionRuntimeBindingRecord({
    required this.runtimeBindingId,
    required this.tenantId,
    required this.organizationId,
    required this.sessionId,
    this.runtimeLocationId,
    required this.hostMode,
    required this.transportKind,
    required this.providerBindingId,
    required this.modelId,
    required this.providerId,
    this.nativeSessionId,
    this.nativeSessionTreeId,
    this.nativeParentSessionId,
    this.nativeForkedFromSessionId,
    required this.status,
    required this.isCurrent,
    required this.version,
    required this.createdAt,
    required this.updatedAt,
    this.activatedAt,
    this.deactivatedAt
  });

  factory AgentSessionRuntimeBindingRecord.fromJson(Map<String, dynamic> json) {
    return AgentSessionRuntimeBindingRecord(
      runtimeBindingId: (() {
        final value = json['runtimeBindingId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.runtimeBindingId is required');
        }
        return value;
      })(),
      tenantId: (() {
        final value = json['tenantId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.tenantId is required');
        }
        return value;
      })(),
      organizationId: (() {
        final value = json['organizationId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.organizationId is required');
        }
        return value;
      })(),
      sessionId: (() {
        final value = json['sessionId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.sessionId is required');
        }
        return value;
      })(),
      runtimeLocationId: json['runtimeLocationId']?.toString(),
      hostMode: (() {
        final value = json['hostMode']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.hostMode is required');
        }
        return value;
      })(),
      transportKind: (() {
        final value = json['transportKind']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.transportKind is required');
        }
        return value;
      })(),
      providerBindingId: (() {
        final value = json['providerBindingId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.providerBindingId is required');
        }
        return value;
      })(),
      modelId: (() {
        final value = json['modelId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.modelId is required');
        }
        return value;
      })(),
      providerId: (() {
        final value = json['providerId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.providerId is required');
        }
        return value;
      })(),
      nativeSessionId: json['nativeSessionId']?.toString(),
      nativeSessionTreeId: json['nativeSessionTreeId']?.toString(),
      nativeParentSessionId: json['nativeParentSessionId']?.toString(),
      nativeForkedFromSessionId: json['nativeForkedFromSessionId']?.toString(),
      status: (() {
        final value = json['status']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.status is required');
        }
        return value;
      })(),
      isCurrent: (() {
        final value = json['isCurrent'];
        if (value is! bool) {
          throw FormatException('AgentSessionRuntimeBindingRecord.isCurrent is required');
        }
        return value;
      })(),
      version: (() {
        final value = json['version']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.version is required');
        }
        return value;
      })(),
      createdAt: (() {
        final value = json['createdAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.createdAt is required');
        }
        return value;
      })(),
      updatedAt: (() {
        final value = json['updatedAt']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingRecord.updatedAt is required');
        }
        return value;
      })(),
      activatedAt: json['activatedAt']?.toString(),
      deactivatedAt: json['deactivatedAt']?.toString()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'runtimeBindingId': runtimeBindingId,
      'tenantId': tenantId,
      'organizationId': organizationId,
      'sessionId': sessionId,
      'runtimeLocationId': runtimeLocationId,
      'hostMode': hostMode,
      'transportKind': transportKind,
      'providerBindingId': providerBindingId,
      'modelId': modelId,
      'providerId': providerId,
      'nativeSessionId': nativeSessionId,
      'nativeSessionTreeId': nativeSessionTreeId,
      'nativeParentSessionId': nativeParentSessionId,
      'nativeForkedFromSessionId': nativeForkedFromSessionId,
      'status': status,
      'isCurrent': isCurrent,
      'version': version,
      'createdAt': createdAt,
      'updatedAt': updatedAt,
      'activatedAt': activatedAt,
      'deactivatedAt': deactivatedAt,
    };
  }
}

class AgentSessionRuntimeBindingResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionRuntimeBindingResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionRuntimeBindingResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionRuntimeBindingResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionRuntimeBindingResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionRuntimeBindingResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class AgentSessionRuntimeBindingListResponse {
  final int code;
  final dynamic data;
  final String traceId;

  AgentSessionRuntimeBindingListResponse({
    required this.code,
    required this.data,
    required this.traceId
  });

  factory AgentSessionRuntimeBindingListResponse.fromJson(Map<String, dynamic> json) {
    return AgentSessionRuntimeBindingListResponse(
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('AgentSessionRuntimeBindingListResponse.code is required');
        }
        return value;
      })(),
      data: (() {
        final map = _sdkworkAsMap(json['data']);
        if (map == null) {
          throw FormatException('AgentSessionRuntimeBindingListResponse.data is required');
        }
        return map;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('AgentSessionRuntimeBindingListResponse.traceId is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'code': code,
      'data': data,
      'traceId': traceId,
    };
  }
}

class CreateAgentSessionRuntimeBindingRequest {
  final String? runtimeBindingId;
  final String? runtimeLocationId;
  final String hostMode;
  final String transportKind;
  final String providerBindingId;
  final String modelId;
  final String providerId;
  final String? nativeSessionId;
  final String? nativeSessionTreeId;
  final String? nativeParentSessionId;
  final String? nativeForkedFromSessionId;
  final String requestedAt;

  CreateAgentSessionRuntimeBindingRequest({
    this.runtimeBindingId,
    this.runtimeLocationId,
    required this.hostMode,
    required this.transportKind,
    required this.providerBindingId,
    required this.modelId,
    required this.providerId,
    this.nativeSessionId,
    this.nativeSessionTreeId,
    this.nativeParentSessionId,
    this.nativeForkedFromSessionId,
    required this.requestedAt
  });

  factory CreateAgentSessionRuntimeBindingRequest.fromJson(Map<String, dynamic> json) {
    return CreateAgentSessionRuntimeBindingRequest(
      runtimeBindingId: json['runtimeBindingId']?.toString(),
      runtimeLocationId: json['runtimeLocationId']?.toString(),
      hostMode: (() {
        final value = json['hostMode']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRuntimeBindingRequest.hostMode is required');
        }
        return value;
      })(),
      transportKind: (() {
        final value = json['transportKind']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRuntimeBindingRequest.transportKind is required');
        }
        return value;
      })(),
      providerBindingId: (() {
        final value = json['providerBindingId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRuntimeBindingRequest.providerBindingId is required');
        }
        return value;
      })(),
      modelId: (() {
        final value = json['modelId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRuntimeBindingRequest.modelId is required');
        }
        return value;
      })(),
      providerId: (() {
        final value = json['providerId']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRuntimeBindingRequest.providerId is required');
        }
        return value;
      })(),
      nativeSessionId: json['nativeSessionId']?.toString(),
      nativeSessionTreeId: json['nativeSessionTreeId']?.toString(),
      nativeParentSessionId: json['nativeParentSessionId']?.toString(),
      nativeForkedFromSessionId: json['nativeForkedFromSessionId']?.toString(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('CreateAgentSessionRuntimeBindingRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'runtimeBindingId': runtimeBindingId,
      'runtimeLocationId': runtimeLocationId,
      'hostMode': hostMode,
      'transportKind': transportKind,
      'providerBindingId': providerBindingId,
      'modelId': modelId,
      'providerId': providerId,
      'nativeSessionId': nativeSessionId,
      'nativeSessionTreeId': nativeSessionTreeId,
      'nativeParentSessionId': nativeParentSessionId,
      'nativeForkedFromSessionId': nativeForkedFromSessionId,
      'requestedAt': requestedAt,
    };
  }
}

class UpdateAgentSessionRuntimeBindingRequest {
  final String? runtimeLocationId;
  final bool? clearRuntimeLocation;
  final String? hostMode;
  final String? transportKind;
  final String? providerBindingId;
  final String? modelId;
  final String? providerId;
  final String? nativeSessionId;
  final String? nativeSessionTreeId;
  final String? nativeParentSessionId;
  final String? nativeForkedFromSessionId;
  final String expectedVersion;
  final String requestedAt;

  UpdateAgentSessionRuntimeBindingRequest({
    this.runtimeLocationId,
    this.clearRuntimeLocation,
    this.hostMode,
    this.transportKind,
    this.providerBindingId,
    this.modelId,
    this.providerId,
    this.nativeSessionId,
    this.nativeSessionTreeId,
    this.nativeParentSessionId,
    this.nativeForkedFromSessionId,
    required this.expectedVersion,
    required this.requestedAt
  });

  factory UpdateAgentSessionRuntimeBindingRequest.fromJson(Map<String, dynamic> json) {
    return UpdateAgentSessionRuntimeBindingRequest(
      runtimeLocationId: json['runtimeLocationId']?.toString(),
      clearRuntimeLocation: json['clearRuntimeLocation'] is bool ? json['clearRuntimeLocation'] : null,
      hostMode: json['hostMode']?.toString(),
      transportKind: json['transportKind']?.toString(),
      providerBindingId: json['providerBindingId']?.toString(),
      modelId: json['modelId']?.toString(),
      providerId: json['providerId']?.toString(),
      nativeSessionId: json['nativeSessionId']?.toString(),
      nativeSessionTreeId: json['nativeSessionTreeId']?.toString(),
      nativeParentSessionId: json['nativeParentSessionId']?.toString(),
      nativeForkedFromSessionId: json['nativeForkedFromSessionId']?.toString(),
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentSessionRuntimeBindingRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('UpdateAgentSessionRuntimeBindingRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'runtimeLocationId': runtimeLocationId,
      'clearRuntimeLocation': clearRuntimeLocation,
      'hostMode': hostMode,
      'transportKind': transportKind,
      'providerBindingId': providerBindingId,
      'modelId': modelId,
      'providerId': providerId,
      'nativeSessionId': nativeSessionId,
      'nativeSessionTreeId': nativeSessionTreeId,
      'nativeParentSessionId': nativeParentSessionId,
      'nativeForkedFromSessionId': nativeForkedFromSessionId,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class ChangeAgentSessionRuntimeBindingStatusRequest {
  final String? reason;
  final String expectedVersion;
  final String requestedAt;

  ChangeAgentSessionRuntimeBindingStatusRequest({
    this.reason,
    required this.expectedVersion,
    required this.requestedAt
  });

  factory ChangeAgentSessionRuntimeBindingStatusRequest.fromJson(Map<String, dynamic> json) {
    return ChangeAgentSessionRuntimeBindingStatusRequest(
      reason: json['reason']?.toString(),
      expectedVersion: (() {
        final value = json['expectedVersion']?.toString();
        if (value == null) {
          throw FormatException('ChangeAgentSessionRuntimeBindingStatusRequest.expectedVersion is required');
        }
        return value;
      })(),
      requestedAt: (() {
        final value = json['requestedAt']?.toString();
        if (value == null) {
          throw FormatException('ChangeAgentSessionRuntimeBindingStatusRequest.requestedAt is required');
        }
        return value;
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'reason': reason,
      'expectedVersion': expectedVersion,
      'requestedAt': requestedAt,
    };
  }
}

class FieldError {
  final String field;
  final String message;
  final int? code;

  FieldError({
    required this.field,
    required this.message,
    this.code
  });

  factory FieldError.fromJson(Map<String, dynamic> json) {
    return FieldError(
      field: (() {
        final value = json['field']?.toString();
        if (value == null) {
          throw FormatException('FieldError.field is required');
        }
        return value;
      })(),
      message: (() {
        final value = json['message']?.toString();
        if (value == null) {
          throw FormatException('FieldError.message is required');
        }
        return value;
      })(),
      code: json['code'] is int ? json['code'] : null
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'field': field,
      'message': message,
      'code': code,
    };
  }
}

class ProblemDetail {
  final String type;
  final String title;
  final int status;
  final String? detail;
  final String? instance;
  final int code;
  final String traceId;
  final List<FieldError>? errors;

  ProblemDetail({
    required this.type,
    required this.title,
    required this.status,
    this.detail,
    this.instance,
    required this.code,
    required this.traceId,
    this.errors
  });

  factory ProblemDetail.fromJson(Map<String, dynamic> json) {
    return ProblemDetail(
      type: (() {
        final value = json['type']?.toString();
        if (value == null) {
          throw FormatException('ProblemDetail.type is required');
        }
        return value;
      })(),
      title: (() {
        final value = json['title']?.toString();
        if (value == null) {
          throw FormatException('ProblemDetail.title is required');
        }
        return value;
      })(),
      status: (() {
        final value = json['status'];
        if (value is! int) {
          throw FormatException('ProblemDetail.status is required');
        }
        return value;
      })(),
      detail: json['detail']?.toString(),
      instance: json['instance']?.toString(),
      code: (() {
        final value = json['code'];
        if (value is! int) {
          throw FormatException('ProblemDetail.code is required');
        }
        return value;
      })(),
      traceId: (() {
        final value = json['traceId']?.toString();
        if (value == null) {
          throw FormatException('ProblemDetail.traceId is required');
        }
        return value;
      })(),
      errors: (() {
        final list = _sdkworkAsList(json['errors']);
        if (list == null) {
          return null;
        }
        return list
            .map((item) => (() {
        final map = _sdkworkAsMap(item);
        return map == null ? null : FieldError.fromJson(map);
      })())
            .whereType<FieldError>()
            .toList();
      })()
    );
  }

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'type': type,
      'title': title,
      'status': status,
      'detail': detail,
      'instance': instance,
      'code': code,
      'traceId': traceId,
      'errors': errors?.map((item) => item.toJson()).toList(),
    };
  }
}
