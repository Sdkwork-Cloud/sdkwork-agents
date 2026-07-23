export const AGENTS_SDK_OWNER = 'sdkwork-agents';
export const AGENTS_SDK_OWNERSHIP_STANDARD_VERSION = '2026-06-26';

const APPBASE_APP_DEPENDENCY = {
  workspace: 'sdkwork-iam-app-sdk',
  role: 'appbase-identity-and-session-capability',
  required: true,
  dependencyMode: 'consumer-sdk',
  apiPrefix: '/app/v3/api',
  apiAuthority: 'sdkwork-iam-app-api',
  generatedTransportImportPolicy: 'forbidden',
  packageByLanguage: {
    typescript: '@sdkwork/iam-app-sdk',
  },
};

const APPBASE_BACKEND_DEPENDENCY = {
  workspace: 'sdkwork-iam-backend-sdk',
  role: 'appbase-backend-management-capability',
  required: true,
  dependencyMode: 'consumer-sdk',
  apiPrefix: '/backend/v3/api',
  apiAuthority: 'sdkwork-iam-backend-api',
  generatedTransportImportPolicy: 'forbidden',
  packageByLanguage: {
    typescript: '@sdkwork/iam-backend-sdk',
  },
};

const DRIVE_APP_DEPENDENCY = {
  workspace: 'sdkwork-drive-app-sdk',
  role: 'drive-uploader-file-storage-capability',
  required: true,
  dependencyMode: 'consumer-sdk',
  apiPrefix: '/app/v3/api',
  apiAuthority: 'sdkwork-drive.app',
  generatedTransportImportPolicy: 'forbidden',
  packageByLanguage: {
    typescript: '@sdkwork/drive-app-sdk',
  },
};

const PROMPTS_APP_DEPENDENCY = {
  workspace: 'sdkwork-prompts-app-sdk',
  role: 'project-instructions-capability',
  required: true,
  dependencyMode: 'consumer-sdk',
  apiPrefix: '/app/v3/api',
  apiAuthority: 'sdkwork-prompts-app-api',
  generatedTransportImportPolicy: 'forbidden',
  packageByLanguage: {
    typescript: '@sdkwork/prompts-app-sdk',
  },
};

const MEMORY_APP_DEPENDENCY = {
  workspace: 'sdkwork-memory-app-sdk',
  role: 'project-memory-selection-capability',
  required: true,
  dependencyMode: 'consumer-sdk',
  apiPrefix: '/app/v3/api',
  apiAuthority: 'sdkwork-memory.app',
  generatedTransportImportPolicy: 'forbidden',
  packageByLanguage: {
    typescript: '@sdkwork/memory-app-sdk',
  },
};

export const AGENTS_SDK_FAMILIES = [
  {
    key: 'open',
    familyDir: 'sdkwork-agents-sdk',
    authority: 'sdkwork-agents-open-api',
    title: 'SDKWork Agent Open API',
    description: 'Developer-facing agents Open API for SDKWork integrations.',
    sourceOpenApi: 'crates/sdkwork-intelligence-agents-service/specs/openapi/agents-open-api.openapi.yaml',
    apiPrefix: '/agent/v3/api',
    sdkName: 'sdkwork-agents-sdk',
    sdkType: 'custom',
    sdkSurface: 'open',
    externalSdkgenProfileSupported: true,
    packageName: '@sdkwork/agents-sdk',
    npmPackageName: '@sdkwork/agents-sdk',
    languagePackageDir: 'sdkwork-agents-sdk-typescript',
    audience: 'developer and integration authors',
    capability: 'agents-open-sdk',
    sdkOwner: AGENTS_SDK_OWNER,
    sdkDependencies: [],
  },
  {
    key: 'app',
    familyDir: 'sdkwork-agents-app-sdk',
    authority: 'sdkwork-agents-app-api',
    title: 'SDKWork Agent Business App API',
    description: 'App-facing managed agent APIs for SDKWork user-facing clients.',
    sourceOpenApi: 'crates/sdkwork-intelligence-agents-service/specs/openapi/agents-app-api.openapi.yaml',
    sourcePrefix: '/app/v3/api',
    apiPrefix: '/app/v3/api',
    sdkName: 'sdkwork-agents-app-sdk',
    sdkType: 'app',
    sdkSurface: 'app',
    externalSdkgenProfileSupported: true,
    packageName: '@sdkwork/agents-app-sdk',
    npmPackageName: '@sdkwork/agents-app-sdk',
    languagePackageDir: 'sdkwork-agents-app-sdk-typescript',
    additionalLanguageTargets: [
      {
        language: 'flutter',
        workspace: 'sdkwork-agents-app-sdk-flutter',
        packageName: 'sdkwork_agents_app_sdk',
        manifestFile: 'pubspec.yaml',
        entrypoint: 'lib/sdkwork_agents_app_sdk.dart',
      },
    ],
    audience: 'app, desktop, mobile, H5, and user-facing clients',
    capability: 'agents-app-sdk',
    sdkOwner: AGENTS_SDK_OWNER,
    sdkDependencies: [
      APPBASE_APP_DEPENDENCY,
      DRIVE_APP_DEPENDENCY,
      PROMPTS_APP_DEPENDENCY,
      MEMORY_APP_DEPENDENCY,
    ],
  },
  {
    key: 'backend',
    familyDir: 'sdkwork-agents-backend-sdk',
    authority: 'sdkwork-agents-backend-api',
    title: 'SDKWork Agent Business Backend API',
    description: 'Backend-facing managed agent APIs for SDKWork operator and control-plane clients.',
    sourceOpenApi:
      'crates/sdkwork-intelligence-agents-service/specs/openapi/agents-backend-api.openapi.yaml',
    sourcePrefix: '/backend/v3/api',
    apiPrefix: '/backend/v3/api',
    sdkName: 'sdkwork-agents-backend-sdk',
    sdkType: 'backend',
    sdkSurface: 'backend',
    externalSdkgenProfileSupported: true,
    packageName: '@sdkwork/agents-backend-sdk',
    npmPackageName: '@sdkwork/agents-backend-sdk',
    languagePackageDir: 'sdkwork-agents-backend-sdk-typescript',
    audience: 'backend console, operators, automation, and control-plane integrations',
    capability: 'agents-backend-sdk',
    sdkOwner: AGENTS_SDK_OWNER,
    sdkDependencies: [APPBASE_BACKEND_DEPENDENCY],
  },
];

export function resolveAgentsSdkFamily(keyOrFamilyDir) {
  const family = AGENTS_SDK_FAMILIES.find(
    (candidate) => candidate.key === keyOrFamilyDir || candidate.familyDir === keyOrFamilyDir,
  );
  if (!family) {
    throw new Error(`Unknown agents SDK family: ${keyOrFamilyDir}`);
  }
  return family;
}

export function resolveAgentsSdkLanguageTargets(family) {
  return [
    {
      language: 'typescript',
      workspace: family.languagePackageDir,
      packageName: family.packageName,
      npmPackageName: family.npmPackageName,
      manifestFile: 'package.json',
      entrypoint: 'src/index.ts',
    },
    ...(family.additionalLanguageTargets ?? []),
  ];
}

export function forbiddenAgentsApiPrefixesFor(family) {
  return AGENTS_SDK_FAMILIES.map((candidate) => candidate.apiPrefix).filter(
    (prefix) => prefix !== family.apiPrefix,
  );
}
