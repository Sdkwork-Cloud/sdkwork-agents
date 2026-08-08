export {
  createClient,
  // The generated transport exports `SdkworkCustomClient`/`SdkworkCustomConfig`;
  // the facade keeps the stable `SdkworkClient`/`SdkworkConfig` consumer names
  // as aliases so `@sdkwork/agents-sdk` imports compile against the generated
  // surface.
  SdkworkCustomClient as SdkworkClient,
} from '../generated/server-openapi/src/index';
export type {
  SdkworkCustomConfig as SdkworkConfig,
} from '../generated/server-openapi/src/types/common';
export * from '../generated/server-openapi/src/types';
export * from '../generated/server-openapi/src/api';
export * from '../generated/server-openapi/src/http';
export * from '../generated/server-openapi/src/auth';
