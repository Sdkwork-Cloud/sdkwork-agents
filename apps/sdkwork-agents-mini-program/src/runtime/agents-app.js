"use strict";
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);

// src/bootstrap/runtimeBundle.ts
var runtimeBundle_exports = {};
__export(runtimeBundle_exports, {
  bootstrapAgentsMiniProgram: () => bootstrapAgentsMiniProgram
});
module.exports = __toCommonJS(runtimeBundle_exports);

// ../../../sdkwork-im/node_modules/.pnpm-codex-new/@sdkwork+sdk-common@1.0.3/node_modules/@sdkwork/sdk-common/dist/core/types.js
var DEFAULT_RETRY_CONFIG = {
  maxRetries: 3,
  retryDelay: 1e3,
  retryBackoff: "exponential",
  maxRetryDelay: 3e4
};
var DEFAULT_CACHE_CONFIG = {
  enabled: false,
  ttl: 300 * 1e3,
  maxSize: 100
};
var SUCCESS_CODES = [
  0,
  200,
  2e3,
  "0",
  "200",
  "2000"
];
var HTTP_STATUS = {
  OK: 200,
  CREATED: 201,
  NO_CONTENT: 204,
  BAD_REQUEST: 400,
  UNAUTHORIZED: 401,
  FORBIDDEN: 403,
  NOT_FOUND: 404,
  METHOD_NOT_ALLOWED: 405,
  CONFLICT: 409,
  UNPROCESSABLE_ENTITY: 422,
  TOO_MANY_REQUESTS: 429,
  INTERNAL_SERVER_ERROR: 500,
  BAD_GATEWAY: 502,
  SERVICE_UNAVAILABLE: 503,
  GATEWAY_TIMEOUT: 504
};
var MIME_TYPES = {
  JSON: "application/json",
  FORM_DATA: "multipart/form-data",
  URL_ENCODED: "application/x-www-form-urlencoded",
  OCTET_STREAM: "application/octet-stream",
  TEXT_PLAIN: "text/plain",
  TEXT_HTML: "text/html"
};

// ../../../sdkwork-im/node_modules/.pnpm-codex-new/@sdkwork+sdk-common@1.0.3/node_modules/@sdkwork/sdk-common/dist/auth/token-manager.js
var DefaultAuthTokenManager = class {
  constructor(initialTokens, events) {
    __publicField(this, "tokens", {});
    __publicField(this, "events");
    if (initialTokens) {
      this.tokens = { ...initialTokens };
      if (initialTokens.expiresIn && !initialTokens.expiresAt) this.tokens.expiresAt = Date.now() + initialTokens.expiresIn * 1e3;
    }
    this.events = events;
  }
  getAccessToken() {
    return this.tokens.accessToken;
  }
  getAuthToken() {
    return this.tokens.authToken;
  }
  getRefreshToken() {
    return this.tokens.refreshToken;
  }
  getTokens() {
    return { ...this.tokens };
  }
  setTokens(tokens) {
    var _a, _b;
    this.tokens = { ...tokens };
    if (tokens.expiresIn && !tokens.expiresAt) this.tokens.expiresAt = Date.now() + tokens.expiresIn * 1e3;
    (_b = (_a = this.events) == null ? void 0 : _a.onTokenSet) == null ? void 0 : _b.call(_a, this.tokens);
  }
  setAccessToken(token) {
    var _a, _b;
    this.tokens.accessToken = token;
    (_b = (_a = this.events) == null ? void 0 : _a.onTokenSet) == null ? void 0 : _b.call(_a, this.tokens);
  }
  setAuthToken(token) {
    var _a, _b;
    this.tokens.authToken = token;
    (_b = (_a = this.events) == null ? void 0 : _a.onTokenSet) == null ? void 0 : _b.call(_a, this.tokens);
  }
  setRefreshToken(token) {
    this.tokens.refreshToken = token;
  }
  clearTokens() {
    var _a, _b;
    this.tokens = {};
    (_b = (_a = this.events) == null ? void 0 : _a.onTokenCleared) == null ? void 0 : _b.call(_a);
  }
  clearAuthToken() {
    this.tokens.authToken = void 0;
  }
  clearAccessToken() {
    this.tokens.accessToken = void 0;
  }
  isExpired() {
    var _a, _b;
    if (!this.tokens.expiresAt) return false;
    const expired = Date.now() >= this.tokens.expiresAt;
    if (expired) (_b = (_a = this.events) == null ? void 0 : _a.onTokenExpired) == null ? void 0 : _b.call(_a);
    return expired;
  }
  isValid() {
    return this.hasToken() && !this.isExpired();
  }
  hasToken() {
    return !!(this.tokens.accessToken || this.tokens.authToken);
  }
  hasAuthToken() {
    return !!this.tokens.authToken;
  }
  hasAccessToken() {
    return !!this.tokens.accessToken;
  }
  willExpireIn(seconds) {
    if (!this.tokens.expiresAt) return false;
    return Date.now() + seconds * 1e3 >= this.tokens.expiresAt;
  }
};
function buildAuthHeaders(authMode, apiKey, tokenManager) {
  const headers = {};
  if (authMode === "apikey") {
    if (apiKey) headers["Authorization"] = `Bearer ${apiKey}`;
  } else if (authMode === "dual-token") {
    if (tokenManager) {
      const accessToken = tokenManager.getAccessToken();
      const authToken = tokenManager.getAuthToken();
      if (accessToken) headers["Access-Token"] = accessToken;
      if (authToken) headers["Authorization"] = `Bearer ${authToken}`;
    }
  }
  return headers;
}

// ../../../sdkwork-im/node_modules/.pnpm-codex-new/@sdkwork+sdk-common@1.0.3/node_modules/@sdkwork/sdk-common/dist/utils/logger.js
var LOG_LEVELS = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
  silent: 4
};
var ConsoleLogger = class {
  constructor(config = {}) {
    __publicField(this, "level");
    __publicField(this, "prefix");
    __publicField(this, "timestamp");
    __publicField(this, "colors");
    var _a, _b, _c, _d;
    this.level = (_a = config.level) != null ? _a : "info";
    this.prefix = (_b = config.prefix) != null ? _b : "[SDK]";
    this.timestamp = (_c = config.timestamp) != null ? _c : true;
    this.colors = (_d = config.colors) != null ? _d : true;
  }
  formatMessage(level, message) {
    const parts = [];
    if (this.timestamp) parts.push((/* @__PURE__ */ new Date()).toISOString());
    parts.push(this.prefix);
    parts.push(`[${level.toUpperCase()}]`);
    parts.push(message);
    return parts.join(" ");
  }
  getColorCode(level) {
    if (!this.colors) return "";
    return {
      debug: "\x1B[36m",
      info: "\x1B[32m",
      warn: "\x1B[33m",
      error: "\x1B[31m",
      silent: ""
    }[level];
  }
  getResetCode() {
    return this.colors ? "\x1B[0m" : "";
  }
  log(level, message, ...args) {
    if (LOG_LEVELS[level] < LOG_LEVELS[this.level]) return;
    const formattedMessage = this.formatMessage(level, message);
    const output = `${this.getColorCode(level)}${formattedMessage}${this.getResetCode()}`;
    switch (level) {
      case "debug":
        console.debug(output, ...args);
        break;
      case "info":
        console.info(output, ...args);
        break;
      case "warn":
        console.warn(output, ...args);
        break;
      case "error":
        console.error(output, ...args);
        break;
    }
  }
  debug(message, ...args) {
    this.log("debug", message, ...args);
  }
  info(message, ...args) {
    this.log("info", message, ...args);
  }
  warn(message, ...args) {
    this.log("warn", message, ...args);
  }
  error(message, ...args) {
    this.log("error", message, ...args);
  }
  setLevel(level) {
    this.level = level;
  }
};
var noopLogger = {
  debug: () => {
  },
  info: () => {
  },
  warn: () => {
  },
  error: () => {
  },
  log: () => {
  },
  setLevel: () => {
  }
};
function createLogger(config) {
  if ((config == null ? void 0 : config.level) === "silent") return noopLogger;
  return new ConsoleLogger(config);
}

// ../../../sdkwork-im/node_modules/.pnpm-codex-new/@sdkwork+sdk-common@1.0.3/node_modules/@sdkwork/sdk-common/dist/utils/cache.js
var MemoryCacheStore = class {
  constructor(config = {}) {
    __publicField(this, "cache", /* @__PURE__ */ new Map());
    __publicField(this, "maxSize");
    __publicField(this, "defaultTtl");
    var _a, _b;
    this.maxSize = (_a = config.maxSize) != null ? _a : DEFAULT_CACHE_CONFIG.maxSize;
    this.defaultTtl = (_b = config.ttl) != null ? _b : DEFAULT_CACHE_CONFIG.ttl;
  }
  get(key) {
    const entry = this.cache.get(key);
    if (!entry) return null;
    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      return null;
    }
    return entry.value;
  }
  set(key, value, ttl) {
    if (this.cache.size >= this.maxSize) this.evictOldest();
    const expiresAt = Date.now() + (ttl != null ? ttl : this.defaultTtl);
    this.cache.set(key, {
      value,
      expiresAt
    });
  }
  has(key) {
    const entry = this.cache.get(key);
    if (!entry) return false;
    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      return false;
    }
    return true;
  }
  delete(key) {
    return this.cache.delete(key);
  }
  clear() {
    this.cache.clear();
  }
  size() {
    return this.cache.size;
  }
  evictOldest() {
    let oldestKey = null;
    let oldestTime = Infinity;
    for (const [key, entry] of this.cache) if (entry.expiresAt < oldestTime) {
      oldestTime = entry.expiresAt;
      oldestKey = key;
    }
    if (oldestKey) this.cache.delete(oldestKey);
  }
};
function createCacheStore(config) {
  return new MemoryCacheStore(config);
}

// ../../../sdkwork-im/node_modules/.pnpm-codex-new/@sdkwork+sdk-common@1.0.3/node_modules/@sdkwork/sdk-common/dist/errors.js
var SdkError = class extends Error {
  constructor(message, code = "UNKNOWN", httpStatus, options) {
    super(message, { cause: options == null ? void 0 : options.cause });
    __publicField(this, "code");
    __publicField(this, "httpStatus");
    __publicField(this, "details");
    __publicField(this, "timestamp");
    __publicField(this, "traceId");
    __publicField(this, "metadata");
    this.name = this.constructor.name;
    this.code = code;
    this.httpStatus = httpStatus;
    this.details = options == null ? void 0 : options.details;
    this.timestamp = Date.now();
    this.traceId = options == null ? void 0 : options.traceId;
    this.metadata = options == null ? void 0 : options.metadata;
    Object.setPrototypeOf(this, new.target.prototype);
  }
  static fromApiResult(result, httpStatus) {
    const code = String(result.code);
    const message = result.msg || result.message || "Unknown error";
    switch (code) {
      case "400":
      case "4000":
        return new ValidationError(message);
      case "401":
      case "4010":
        return new AuthenticationError(message);
      case "403":
      case "4030":
        return new ForbiddenError(message);
      case "404":
      case "4040":
        return new NotFoundError(message);
      case "409":
      case "4090":
        return new ConflictError(message);
      case "429":
      case "4290":
        return new RateLimitError(message);
      default:
        if (code.startsWith("5")) return new ServerError(message, httpStatus != null ? httpStatus : HTTP_STATUS.INTERNAL_SERVER_ERROR);
        return new BusinessError(message, result.code, result.data);
    }
  }
  static fromHttpStatus(status, message) {
    const defaultMessage = message != null ? message : `HTTP Error ${status}`;
    switch (status) {
      case HTTP_STATUS.BAD_REQUEST:
      case HTTP_STATUS.UNPROCESSABLE_ENTITY:
        return new ValidationError(defaultMessage);
      case HTTP_STATUS.UNAUTHORIZED:
        return new AuthenticationError(defaultMessage);
      case HTTP_STATUS.FORBIDDEN:
        return new ForbiddenError(defaultMessage);
      case HTTP_STATUS.NOT_FOUND:
        return new NotFoundError(defaultMessage);
      case HTTP_STATUS.METHOD_NOT_ALLOWED:
        return new ValidationError(defaultMessage);
      case HTTP_STATUS.CONFLICT:
        return new ConflictError(defaultMessage);
      case HTTP_STATUS.TOO_MANY_REQUESTS:
        return new RateLimitError(defaultMessage);
      case HTTP_STATUS.INTERNAL_SERVER_ERROR:
        return new ServerError(defaultMessage, status);
      case HTTP_STATUS.BAD_GATEWAY:
        return new BadGatewayError(defaultMessage);
      case HTTP_STATUS.SERVICE_UNAVAILABLE:
        return new ServiceUnavailableError(defaultMessage);
      case HTTP_STATUS.GATEWAY_TIMEOUT:
        return new GatewayTimeoutError(defaultMessage);
      default:
        if (status >= 500) return new ServerError(defaultMessage, status);
        return new NetworkError(defaultMessage);
    }
  }
  toJSON() {
    return {
      name: this.name,
      message: this.message,
      code: this.code,
      httpStatus: this.httpStatus,
      details: this.details,
      timestamp: this.timestamp,
      traceId: this.traceId,
      metadata: this.metadata
    };
  }
  toString() {
    return `${this.name}: ${this.message} (code: ${this.code})`;
  }
  isRetryable() {
    return isRetryableError(this);
  }
  isAuthError() {
    return this.code === "UNAUTHORIZED" || this.code === "TOKEN_EXPIRED" || this.code === "TOKEN_INVALID";
  }
  isNetworkError() {
    return this.code === "NETWORK_ERROR" || this.code === "TIMEOUT";
  }
  isClientError() {
    return this.httpStatus !== void 0 && this.httpStatus >= 400 && this.httpStatus < 500;
  }
  isServerError() {
    return this.httpStatus !== void 0 && this.httpStatus >= 500;
  }
};
var NetworkError = class extends SdkError {
  constructor(message = "Network error", options) {
    super(message, "NETWORK_ERROR", void 0, options);
  }
};
var TimeoutError = class extends SdkError {
  constructor(message = "Request timeout", timeout, options) {
    super(message, "TIMEOUT", void 0, options);
    __publicField(this, "timeout");
    this.timeout = timeout;
  }
  toJSON() {
    return {
      ...super.toJSON(),
      timeout: this.timeout
    };
  }
};
var CancelledError = class extends SdkError {
  constructor(message = "Request cancelled", options) {
    super(message, "CANCELLED", void 0, options);
  }
};
var AuthenticationError = class extends SdkError {
  constructor(message = "Authentication failed", options) {
    super(message, "UNAUTHORIZED", HTTP_STATUS.UNAUTHORIZED, options);
  }
};
var ForbiddenError = class extends SdkError {
  constructor(message = "Access forbidden", options) {
    super(message, "FORBIDDEN", HTTP_STATUS.FORBIDDEN, options);
  }
};
var NotFoundError = class extends SdkError {
  constructor(message = "Resource not found", options) {
    super(message, "NOT_FOUND", HTTP_STATUS.NOT_FOUND, options);
  }
};
var ValidationError = class extends SdkError {
  constructor(message = "Validation error", details, options) {
    super(message, "VALIDATION_ERROR", HTTP_STATUS.BAD_REQUEST, {
      ...options,
      details
    });
  }
};
var ConflictError = class extends SdkError {
  constructor(message = "Resource conflict", options) {
    super(message, "CONFLICT", HTTP_STATUS.CONFLICT, options);
  }
};
var RateLimitError = class extends SdkError {
  constructor(message = "Rate limit exceeded", retryAfter, options) {
    super(message, "RATE_LIMIT", HTTP_STATUS.TOO_MANY_REQUESTS, options);
    __publicField(this, "retryAfter");
    this.retryAfter = retryAfter;
  }
  toJSON() {
    return {
      ...super.toJSON(),
      retryAfter: this.retryAfter
    };
  }
};
var ServerError = class extends SdkError {
  constructor(message = "Server error", httpStatus = HTTP_STATUS.INTERNAL_SERVER_ERROR, options) {
    super(message, "SERVER_ERROR", httpStatus, options);
  }
};
var BadGatewayError = class extends ServerError {
  constructor(message = "Bad gateway", options) {
    super(message, HTTP_STATUS.BAD_GATEWAY, options);
    this.code = "BAD_GATEWAY";
  }
};
var ServiceUnavailableError = class extends ServerError {
  constructor(message = "Service unavailable", options) {
    super(message, HTTP_STATUS.SERVICE_UNAVAILABLE, options);
    this.code = "SERVICE_UNAVAILABLE";
  }
};
var GatewayTimeoutError = class extends ServerError {
  constructor(message = "Gateway timeout", options) {
    super(message, HTTP_STATUS.GATEWAY_TIMEOUT, options);
    this.code = "GATEWAY_TIMEOUT";
  }
};
var BusinessError = class extends SdkError {
  constructor(message, code, data, options) {
    super(message, "BUSINESS_ERROR", void 0, options);
    __publicField(this, "businessCode");
    __publicField(this, "data");
    this.businessCode = code;
    this.data = data;
  }
  toJSON() {
    return {
      ...super.toJSON(),
      businessCode: this.businessCode,
      data: this.data
    };
  }
};
function isRetryableError(error) {
  if (!(error instanceof SdkError)) return false;
  return error instanceof NetworkError || error instanceof TimeoutError || error instanceof ServerError || error instanceof RateLimitError || error instanceof BadGatewayError || error instanceof ServiceUnavailableError || error instanceof GatewayTimeoutError;
}

// ../../../sdkwork-im/node_modules/.pnpm-codex-new/@sdkwork+sdk-common@1.0.3/node_modules/@sdkwork/sdk-common/dist/utils/retry.js
function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
function calculateDelay(attempt, baseDelay, backoff, maxDelay) {
  let delay;
  switch (backoff) {
    case "fixed":
      delay = baseDelay;
      break;
    case "linear":
      delay = baseDelay * attempt;
      break;
    case "exponential":
      delay = baseDelay * Math.pow(2, attempt - 1);
      break;
    default:
      delay = baseDelay;
  }
  return Math.min(delay, maxDelay);
}
function shouldRetry(error, attempt, config) {
  if (attempt >= config.maxRetries) return false;
  if (config.retryCondition) return config.retryCondition(error, attempt);
  return isRetryableError(error);
}
async function withRetry(fn, config = {}) {
  const fullConfig = {
    ...DEFAULT_RETRY_CONFIG,
    ...config
  };
  let lastError;
  let attempt = 0;
  while (attempt <= fullConfig.maxRetries) try {
    return await fn();
  } catch (error) {
    lastError = error;
    attempt++;
    if (!shouldRetry(lastError, attempt, fullConfig)) throw lastError;
    await sleep(calculateDelay(attempt, fullConfig.retryDelay, fullConfig.retryBackoff, fullConfig.maxRetryDelay));
  }
  throw lastError;
}

// ../../../sdkwork-im/node_modules/.pnpm-codex-new/@sdkwork+sdk-common@1.0.3/node_modules/@sdkwork/sdk-common/dist/utils/string.js
var StringUtils;
(function(_StringUtils) {
  function isEmpty(value) {
    return value === null || value === void 0 || value === "";
  }
  _StringUtils.isEmpty = isEmpty;
  function isNotEmpty(value) {
    return !isEmpty(value);
  }
  _StringUtils.isNotEmpty = isNotEmpty;
  function isBlank(value) {
    if (isEmpty(value)) return true;
    if (typeof value !== "string") return false;
    return value.trim().length === 0;
  }
  _StringUtils.isBlank = isBlank;
  function isNotBlank(value) {
    return !isBlank(value);
  }
  _StringUtils.isNotBlank = isNotBlank;
  function trim(value) {
    var _a;
    return (_a = value == null ? void 0 : value.trim()) != null ? _a : "";
  }
  _StringUtils.trim = trim;
  function trimStart(value) {
    var _a;
    return (_a = value == null ? void 0 : value.trimStart()) != null ? _a : "";
  }
  _StringUtils.trimStart = trimStart;
  function trimEnd(value) {
    var _a;
    return (_a = value == null ? void 0 : value.trimEnd()) != null ? _a : "";
  }
  _StringUtils.trimEnd = trimEnd;
  function toLowerCase(value) {
    var _a;
    return (_a = value == null ? void 0 : value.toLowerCase()) != null ? _a : "";
  }
  _StringUtils.toLowerCase = toLowerCase;
  function toUpperCase(value) {
    var _a;
    return (_a = value == null ? void 0 : value.toUpperCase()) != null ? _a : "";
  }
  _StringUtils.toUpperCase = toUpperCase;
  function capitalize(value) {
    if (isEmpty(value)) return "";
    return value.charAt(0).toUpperCase() + value.slice(1).toLowerCase();
  }
  _StringUtils.capitalize = capitalize;
  function capitalizeWords(value) {
    if (isEmpty(value)) return "";
    return value.split(/\s+/).map(capitalize).join(" ");
  }
  _StringUtils.capitalizeWords = capitalizeWords;
  function camelCase(value) {
    if (isEmpty(value)) return "";
    return value.replace(/[-_\s]+(.)?/g, (_, char) => char ? char.toUpperCase() : "").replace(/^(.)/, (char) => char.toLowerCase());
  }
  _StringUtils.camelCase = camelCase;
  function pascalCase(value) {
    if (isEmpty(value)) return "";
    const camel = camelCase(value);
    return camel.charAt(0).toUpperCase() + camel.slice(1);
  }
  _StringUtils.pascalCase = pascalCase;
  function kebabCase(value) {
    if (isEmpty(value)) return "";
    return value.replace(/([a-z])([A-Z])/g, "$1-$2").replace(/[\s_]+/g, "-").toLowerCase();
  }
  _StringUtils.kebabCase = kebabCase;
  function snakeCase(value) {
    if (isEmpty(value)) return "";
    return value.replace(/([a-z])([A-Z])/g, "$1_$2").replace(/[\s-]+/g, "_").toLowerCase();
  }
  _StringUtils.snakeCase = snakeCase;
  function constantCase(value) {
    return snakeCase(value).toUpperCase();
  }
  _StringUtils.constantCase = constantCase;
  function truncate(value, length, suffix = "...") {
    if (isEmpty(value) || value.length <= length) return value != null ? value : "";
    return value.slice(0, length - suffix.length) + suffix;
  }
  _StringUtils.truncate = truncate;
  function truncateWords(value, wordCount2, suffix = "...") {
    if (isEmpty(value)) return "";
    const words2 = value.split(/\s+/);
    if (words2.length <= wordCount2) return value;
    return words2.slice(0, wordCount2).join(" ") + suffix;
  }
  _StringUtils.truncateWords = truncateWords;
  function padStart(value, length, padChar = " ") {
    var _a;
    return (_a = value == null ? void 0 : value.padStart(length, padChar)) != null ? _a : "";
  }
  _StringUtils.padStart = padStart;
  function padEnd(value, length, padChar = " ") {
    var _a;
    return (_a = value == null ? void 0 : value.padEnd(length, padChar)) != null ? _a : "";
  }
  _StringUtils.padEnd = padEnd;
  function repeat(value, count) {
    if (isEmpty(value) || count <= 0) return "";
    return value.repeat(count);
  }
  _StringUtils.repeat = repeat;
  function reverse(value) {
    if (isEmpty(value)) return "";
    return value.split("").reverse().join("");
  }
  _StringUtils.reverse = reverse;
  function startsWith(value, prefix) {
    var _a;
    return (_a = value == null ? void 0 : value.startsWith(prefix)) != null ? _a : false;
  }
  _StringUtils.startsWith = startsWith;
  function endsWith(value, suffix) {
    var _a;
    return (_a = value == null ? void 0 : value.endsWith(suffix)) != null ? _a : false;
  }
  _StringUtils.endsWith = endsWith;
  function contains(value, search) {
    var _a;
    return (_a = value == null ? void 0 : value.includes(search)) != null ? _a : false;
  }
  _StringUtils.contains = contains;
  function containsIgnoreCase(value, search) {
    var _a;
    return (_a = value == null ? void 0 : value.toLowerCase().includes(search.toLowerCase())) != null ? _a : false;
  }
  _StringUtils.containsIgnoreCase = containsIgnoreCase;
  function indexOf(value, search) {
    var _a;
    return (_a = value == null ? void 0 : value.indexOf(search)) != null ? _a : -1;
  }
  _StringUtils.indexOf = indexOf;
  function lastIndexOf(value, search) {
    var _a;
    return (_a = value == null ? void 0 : value.lastIndexOf(search)) != null ? _a : -1;
  }
  _StringUtils.lastIndexOf = lastIndexOf;
  function substring(value, start, end) {
    if (isEmpty(value)) return "";
    return end !== void 0 ? value.slice(start, end) : value.slice(start);
  }
  _StringUtils.substring = substring;
  function slice(value, start, end) {
    return substring(value, start, end);
  }
  _StringUtils.slice = slice;
  function split(value, separator, limit) {
    if (isEmpty(value)) return [];
    return value.split(separator, limit);
  }
  _StringUtils.split = split;
  function join(values, separator = "") {
    var _a;
    return (_a = values == null ? void 0 : values.join(separator)) != null ? _a : "";
  }
  _StringUtils.join = join;
  function replace(value, search, replacement) {
    var _a;
    return (_a = value == null ? void 0 : value.replace(search, replacement)) != null ? _a : "";
  }
  _StringUtils.replace = replace;
  function replaceAll(value, search, replacement) {
    var _a;
    return (_a = value == null ? void 0 : value.replaceAll(search, replacement)) != null ? _a : "";
  }
  _StringUtils.replaceAll = replaceAll;
  function remove(value, search) {
    var _a;
    return (_a = value == null ? void 0 : value.replace(search, "")) != null ? _a : "";
  }
  _StringUtils.remove = remove;
  function removeAll(value, search) {
    var _a;
    const regex = typeof search === "string" ? new RegExp(search, "g") : new RegExp(search.source, `${search.flags}g`);
    return (_a = value == null ? void 0 : value.replace(regex, "")) != null ? _a : "";
  }
  _StringUtils.removeAll = removeAll;
  function countOccurrences(value, search) {
    if (isEmpty(value) || isEmpty(search)) return 0;
    return (value.match(new RegExp(escapeRegex(search), "g")) || []).length;
  }
  _StringUtils.countOccurrences = countOccurrences;
  function escapeHtml(value) {
    var _a;
    const htmlEntities = {
      "&": "&amp;",
      "<": "&lt;",
      ">": "&gt;",
      '"': "&quot;",
      "'": "&#39;"
    };
    return (_a = value == null ? void 0 : value.replace(/[&<>"']/g, (char) => htmlEntities[char] || char)) != null ? _a : "";
  }
  _StringUtils.escapeHtml = escapeHtml;
  function unescapeHtml(value) {
    var _a;
    const htmlEntities = {
      "&amp;": "&",
      "&lt;": "<",
      "&gt;": ">",
      "&quot;": '"',
      "&#39;": "'",
      "&#x27;": "'",
      "&apos;": "'"
    };
    return (_a = value == null ? void 0 : value.replace(/&(?:amp|lt|gt|quot|#39|#x27|apos);/g, (entity) => htmlEntities[entity] || entity)) != null ? _a : "";
  }
  _StringUtils.unescapeHtml = unescapeHtml;
  function escapeRegex(value) {
    var _a;
    return (_a = value == null ? void 0 : value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")) != null ? _a : "";
  }
  _StringUtils.escapeRegex = escapeRegex;
  function isNumeric(value) {
    if (isEmpty(value)) return false;
    return !isNaN(Number(value)) && !isNaN(parseFloat(value));
  }
  _StringUtils.isNumeric = isNumeric;
  function isAlpha(value) {
    if (isEmpty(value)) return false;
    return /^[a-zA-Z]+$/.test(value);
  }
  _StringUtils.isAlpha = isAlpha;
  function isAlphanumeric(value) {
    if (isEmpty(value)) return false;
    return /^[a-zA-Z0-9]+$/.test(value);
  }
  _StringUtils.isAlphanumeric = isAlphanumeric;
  function isHex(value) {
    if (isEmpty(value)) return false;
    return /^[0-9a-fA-F]+$/.test(value);
  }
  _StringUtils.isHex = isHex;
  function isUuid(value) {
    if (isEmpty(value)) return false;
    return /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(value);
  }
  _StringUtils.isUuid = isUuid;
  function isEmail(value) {
    if (isEmpty(value)) return false;
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value);
  }
  _StringUtils.isEmail = isEmail;
  function isUrl(value) {
    if (isEmpty(value)) return false;
    try {
      new URL(value);
      return true;
    } catch {
      return false;
    }
  }
  _StringUtils.isUrl = isUrl;
  function isPhoneNumber(value) {
    if (isEmpty(value)) return false;
    return /^\+?[\d\s-()]{10,}$/.test(value);
  }
  _StringUtils.isPhoneNumber = isPhoneNumber;
  function mask(value, start, end, maskChar = "*") {
    if (isEmpty(value)) return "";
    const actualStart = Math.max(0, start);
    const actualEnd = Math.min(value.length, end);
    if (actualStart >= actualEnd) return value;
    const masked = maskChar.repeat(actualEnd - actualStart);
    return value.slice(0, actualStart) + masked + value.slice(actualEnd);
  }
  _StringUtils.mask = mask;
  function maskEmail(value) {
    if (!isEmail(value)) return value;
    const parts = value.split("@");
    const localPart = parts[0];
    const domain = parts[1];
    if (!localPart || !domain) return value;
    return `${mask(localPart, 2, localPart.length - 2)}@${domain}`;
  }
  _StringUtils.maskEmail = maskEmail;
  function maskPhone(value) {
    if (isEmpty(value)) return value;
    const digits = value.replace(/\D/g, "");
    if (digits.length < 7) return value;
    return mask(digits, 3, digits.length - 4);
  }
  _StringUtils.maskPhone = maskPhone;
  function maskCreditCard(value) {
    if (isEmpty(value)) return value;
    const digits = value.replace(/\D/g, "");
    if (digits.length < 8) return value;
    return mask(digits, 4, digits.length - 4);
  }
  _StringUtils.maskCreditCard = maskCreditCard;
  function formatNumber(value, options) {
    const num = typeof value === "string" ? parseFloat(value) : value;
    if (isNaN(num)) return "";
    return num.toLocaleString(void 0, options);
  }
  _StringUtils.formatNumber = formatNumber;
  function formatCurrency(value, currency = "USD", locale) {
    const num = typeof value === "string" ? parseFloat(value) : value;
    if (isNaN(num)) return "";
    return num.toLocaleString(locale, {
      style: "currency",
      currency
    });
  }
  _StringUtils.formatCurrency = formatCurrency;
  function formatPercentage(value, decimals = 0) {
    const num = typeof value === "string" ? parseFloat(value) : value;
    if (isNaN(num)) return "";
    return `${(num * 100).toFixed(decimals)}%`;
  }
  _StringUtils.formatPercentage = formatPercentage;
  function formatBytes(bytes, decimals = 2) {
    if (bytes === 0) return "0 Bytes";
    const k = 1024;
    const sizes = [
      "Bytes",
      "KB",
      "MB",
      "GB",
      "TB",
      "PB"
    ];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(decimals))} ${sizes[i]}`;
  }
  _StringUtils.formatBytes = formatBytes;
  function random(length = 16, charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789") {
    let result = "";
    for (let i = 0; i < length; i++) result += charset.charAt(Math.floor(Math.random() * charset.length));
    return result;
  }
  _StringUtils.random = random;
  function uuid() {
    return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (c) => {
      const r = Math.random() * 16 | 0;
      return (c === "x" ? r : r & 3 | 8).toString(16);
    });
  }
  _StringUtils.uuid = uuid;
  function slugify(value) {
    var _a;
    return (_a = value == null ? void 0 : value.toLowerCase().trim().replace(/[^\w\s-]/g, "").replace(/[\s_-]+/g, "-").replace(/^-+|-+$/g, "")) != null ? _a : "";
  }
  _StringUtils.slugify = slugify;
  function unslugify(value) {
    var _a;
    return (_a = value == null ? void 0 : value.replace(/-/g, " ").replace(/\b\w/g, (char) => char.toUpperCase())) != null ? _a : "";
  }
  _StringUtils.unslugify = unslugify;
  function levenshteinDistance(a, b) {
    const matrix = [];
    for (let i = 0; i <= b.length; i++) matrix[i] = [i];
    for (let j = 0; j <= a.length; j++) if (matrix[0]) matrix[0][j] = j;
    for (let i = 1; i <= b.length; i++) for (let j = 1; j <= a.length; j++) if (b.charAt(i - 1) === a.charAt(j - 1)) matrix[i][j] = matrix[i - 1][j - 1];
    else matrix[i][j] = Math.min(matrix[i - 1][j - 1] + 1, matrix[i][j - 1] + 1, matrix[i - 1][j] + 1);
    return matrix[b.length][a.length];
  }
  _StringUtils.levenshteinDistance = levenshteinDistance;
  function similarity(a, b) {
    if (isEmpty(a) && isEmpty(b)) return 1;
    if (isEmpty(a) || isEmpty(b)) return 0;
    return 1 - levenshteinDistance(a, b) / Math.max(a.length, b.length);
  }
  _StringUtils.similarity = similarity;
  function fuzzyMatch(text, pattern, threshold = 0.6) {
    return similarity(text, pattern) >= threshold;
  }
  _StringUtils.fuzzyMatch = fuzzyMatch;
  function equals(a, b, ignoreCase = false) {
    if (ignoreCase) return (a == null ? void 0 : a.toLowerCase()) === (b == null ? void 0 : b.toLowerCase());
    return a === b;
  }
  _StringUtils.equals = equals;
  function equalsIgnoreCase(a, b) {
    return equals(a, b, true);
  }
  _StringUtils.equalsIgnoreCase = equalsIgnoreCase;
  function wordCount(value) {
    if (isEmpty(value)) return 0;
    return value.trim().split(/\s+/).filter(Boolean).length;
  }
  _StringUtils.wordCount = wordCount;
  function characterCount(value, includeSpaces = true) {
    if (isEmpty(value)) return 0;
    return includeSpaces ? value.length : value.replace(/\s/g, "").length;
  }
  _StringUtils.characterCount = characterCount;
  function lineCount(value) {
    if (isEmpty(value)) return 0;
    return value.split(/\r?\n/).length;
  }
  _StringUtils.lineCount = lineCount;
  function splitLines(value) {
    if (isEmpty(value)) return [];
    return value.split(/\r?\n/);
  }
  _StringUtils.splitLines = splitLines;
  function words(value) {
    if (isEmpty(value)) return [];
    return value.trim().split(/\s+/).filter(Boolean);
  }
  _StringUtils.words = words;
  function charAt(value, index) {
    var _a;
    return (_a = value == null ? void 0 : value.charAt(index)) != null ? _a : "";
  }
  _StringUtils.charAt = charAt;
  function charCodeAt(value, index) {
    var _a;
    return (_a = value == null ? void 0 : value.charCodeAt(index)) != null ? _a : NaN;
  }
  _StringUtils.charCodeAt = charCodeAt;
  function fromCharCode(...codes) {
    return String.fromCharCode(...codes);
  }
  _StringUtils.fromCharCode = fromCharCode;
  function insert(value, index, insertValue) {
    if (isEmpty(value)) return insertValue;
    return value.slice(0, index) + insertValue + value.slice(index);
  }
  _StringUtils.insert = insert;
  function swapCase(value) {
    var _a;
    return (_a = value == null ? void 0 : value.replace(/[a-zA-Z]/g, (char) => {
      return char === char.toUpperCase() ? char.toLowerCase() : char.toUpperCase();
    })) != null ? _a : "";
  }
  _StringUtils.swapCase = swapCase;
  function surround(value, wrapper) {
    return `${wrapper}${value}${wrapper}`;
  }
  _StringUtils.surround = surround;
  function quote(value, quoteChar = '"') {
    return `${quoteChar}${value}${quoteChar}`;
  }
  _StringUtils.quote = quote;
  function unquote(value) {
    if (isEmpty(value)) return "";
    if (value.startsWith('"') && value.endsWith('"') || value.startsWith("'") && value.endsWith("'") || value.startsWith("`") && value.endsWith("`")) return value.slice(1, -1);
    return value;
  }
  _StringUtils.unquote = unquote;
  function wrap(value, prefix, suffix = prefix) {
    return `${prefix}${value}${suffix}`;
  }
  _StringUtils.wrap = wrap;
  function unwrap(value, prefix, suffix = prefix) {
    if (isEmpty(value)) return "";
    if (value.startsWith(prefix) && value.endsWith(suffix)) return value.slice(prefix.length, -suffix.length);
    return value;
  }
  _StringUtils.unwrap = unwrap;
  function template(templateStr, values) {
    var _a;
    return (_a = templateStr == null ? void 0 : templateStr.replace(/\{\{(\w+)\}\}/g, (_, key) => {
      var _a2;
      return String((_a2 = values[key]) != null ? _a2 : "");
    })) != null ? _a : "";
  }
  _StringUtils.template = template;
  function interpolate(templateStr, values) {
    return template(templateStr, values);
  }
  _StringUtils.interpolate = interpolate;
  function dedent(value) {
    const lines = value.split("\n");
    const minIndent = Math.min(...lines.filter((line) => line.trim().length > 0).map((line) => {
      var _a, _b;
      return (_b = (_a = line.match(/^\s*/)) == null ? void 0 : _a[0].length) != null ? _b : 0;
    }));
    return lines.map((line) => line.slice(minIndent)).join("\n");
  }
  _StringUtils.dedent = dedent;
  function indent(value, spaces = 2) {
    const indentation = " ".repeat(spaces);
    return value.split("\n").map((line) => indentation + line).join("\n");
  }
  _StringUtils.indent = indent;
  function center(value, width, padChar = " ") {
    if (isEmpty(value) || value.length >= width) return value != null ? value : "";
    const padding = width - value.length;
    const leftPad = Math.floor(padding / 2);
    const rightPad = padding - leftPad;
    return padChar.repeat(leftPad) + value + padChar.repeat(rightPad);
  }
  _StringUtils.center = center;
  function alignLeft(value, width, padChar = " ") {
    return padEnd(value, width, padChar);
  }
  _StringUtils.alignLeft = alignLeft;
  function alignRight(value, width, padChar = " ") {
    return padStart(value, width, padChar);
  }
  _StringUtils.alignRight = alignRight;
  function alignCenter(value, width, padChar = " ") {
    return center(value, width, padChar);
  }
  _StringUtils.alignCenter = alignCenter;
  function toBoolean(value) {
    return [
      "true",
      "1",
      "yes",
      "on",
      "y"
    ].includes(value == null ? void 0 : value.toLowerCase().trim());
  }
  _StringUtils.toBoolean = toBoolean;
  function toNumber(value, defaultValue = 0) {
    const num = parseFloat(value);
    return isNaN(num) ? defaultValue : num;
  }
  _StringUtils.toNumber = toNumber;
  function toArray(value, separator = ",") {
    return split(value, separator);
  }
  _StringUtils.toArray = toArray;
  function hashCode(value) {
    let hash = 0;
    for (let i = 0; i < value.length; i++) {
      const char = value.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash;
    }
    return hash;
  }
  _StringUtils.hashCode = hashCode;
  function isPalindrome(value) {
    const cleaned = value.toLowerCase().replace(/[^a-z0-9]/g, "");
    return cleaned === cleaned.split("").reverse().join("");
  }
  _StringUtils.isPalindrome = isPalindrome;
  function isAnagram(a, b) {
    const normalize = (s) => s.toLowerCase().replace(/[^a-z0-9]/g, "").split("").sort().join("");
    return normalize(a) === normalize(b);
  }
  _StringUtils.isAnagram = isAnagram;
  function reverseWords(value) {
    var _a;
    return (_a = value == null ? void 0 : value.split(/\s+/).reverse().join(" ")) != null ? _a : "";
  }
  _StringUtils.reverseWords = reverseWords;
  function sortCharacters(value) {
    var _a;
    return (_a = value == null ? void 0 : value.split("").sort().join("")) != null ? _a : "";
  }
  _StringUtils.sortCharacters = sortCharacters;
  function uniqueCharacters(value) {
    return [...new Set(value)].join("");
  }
  _StringUtils.uniqueCharacters = uniqueCharacters;
  function removeDuplicates(value) {
    var _a;
    return (_a = value == null ? void 0 : value.split("").filter((char, index, arr) => arr.indexOf(char) === index).join("")) != null ? _a : "";
  }
  _StringUtils.removeDuplicates = removeDuplicates;
  function longestCommonSubstring(a, b) {
    if (isEmpty(a) || isEmpty(b)) return "";
    const matrix = Array(a.length + 1).fill(null).map(() => Array(b.length + 1).fill(0));
    let maxLength = 0;
    let endIndex = 0;
    for (let i = 1; i <= a.length; i++) for (let j = 1; j <= b.length; j++) if (a[i - 1] === b[j - 1]) {
      matrix[i][j] = matrix[i - 1][j - 1] + 1;
      if (matrix[i][j] > maxLength) {
        maxLength = matrix[i][j];
        endIndex = i;
      }
    }
    return a.slice(endIndex - maxLength, endIndex);
  }
  _StringUtils.longestCommonSubstring = longestCommonSubstring;
  function longestCommonPrefix(strings) {
    var _a, _b, _c;
    if (strings.length === 0) return "";
    if (strings.length === 1) return (_a = strings[0]) != null ? _a : "";
    const sorted = [...strings].sort();
    const first = (_b = sorted[0]) != null ? _b : "";
    const last = (_c = sorted[sorted.length - 1]) != null ? _c : "";
    let i = 0;
    while (i < first.length && first[i] === last[i]) i++;
    return first.slice(0, i);
  }
  _StringUtils.longestCommonPrefix = longestCommonPrefix;
  function longestCommonSuffix(strings) {
    return longestCommonPrefix(strings.map((s) => {
      var _a;
      return (_a = s == null ? void 0 : s.split("").reverse().join("")) != null ? _a : "";
    })).split("").reverse().join("");
  }
  _StringUtils.longestCommonSuffix = longestCommonSuffix;
  function truncateMiddle(value, maxLength, separator = "...") {
    if (isEmpty(value) || value.length <= maxLength) return value != null ? value : "";
    const charsToShow = maxLength - separator.length;
    const frontChars = Math.ceil(charsToShow / 2);
    const backChars = Math.floor(charsToShow / 2);
    return value.slice(0, frontChars) + separator + value.slice(-backChars);
  }
  _StringUtils.truncateMiddle = truncateMiddle;
  function ellipsis(value, maxLength) {
    return truncate(value, maxLength, "...");
  }
  _StringUtils.ellipsis = ellipsis;
  function ellipsisMiddle(value, maxLength) {
    return truncateMiddle(value, maxLength, "...");
  }
  _StringUtils.ellipsisMiddle = ellipsisMiddle;
  function pad(value, length, padChar = " ") {
    return center(value, length, padChar);
  }
  _StringUtils.pad = pad;
  function padCenter(value, length, padChar = " ") {
    return center(value, length, padChar);
  }
  _StringUtils.padCenter = padCenter;
  function isAscii(value) {
    return /^[\x00-\x7F]*$/.test(value);
  }
  _StringUtils.isAscii = isAscii;
  function isLowerCase(value) {
    return value === value.toLowerCase();
  }
  _StringUtils.isLowerCase = isLowerCase;
  function isUpperCase(value) {
    return value === value.toUpperCase();
  }
  _StringUtils.isUpperCase = isUpperCase;
  function isCapitalized(value) {
    return value.charAt(0) === value.charAt(0).toUpperCase();
  }
  _StringUtils.isCapitalized = isCapitalized;
  function swapPrefix(value, oldPrefix, newPrefix) {
    if (value.startsWith(oldPrefix)) return newPrefix + value.slice(oldPrefix.length);
    return value;
  }
  _StringUtils.swapPrefix = swapPrefix;
  function swapSuffix(value, oldSuffix, newSuffix) {
    if (value.endsWith(oldSuffix)) return value.slice(0, -oldSuffix.length) + newSuffix;
    return value;
  }
  _StringUtils.swapSuffix = swapSuffix;
  function ensurePrefix(value, prefix) {
    return value.startsWith(prefix) ? value : prefix + value;
  }
  _StringUtils.ensurePrefix = ensurePrefix;
  function ensureSuffix(value, suffix) {
    return value.endsWith(suffix) ? value : value + suffix;
  }
  _StringUtils.ensureSuffix = ensureSuffix;
  function removePrefix(value, prefix) {
    return value.startsWith(prefix) ? value.slice(prefix.length) : value;
  }
  _StringUtils.removePrefix = removePrefix;
  function removeSuffix(value, suffix) {
    return value.endsWith(suffix) ? value.slice(0, -suffix.length) : value;
  }
  _StringUtils.removeSuffix = removeSuffix;
  function take(value, n) {
    var _a;
    return (_a = value == null ? void 0 : value.slice(0, n)) != null ? _a : "";
  }
  _StringUtils.take = take;
  function takeRight(value, n) {
    var _a;
    return (_a = value == null ? void 0 : value.slice(-n)) != null ? _a : "";
  }
  _StringUtils.takeRight = takeRight;
  function takeWhile(value, predicate) {
    let result = "";
    for (const char of value != null ? value : "") {
      if (!predicate(char)) break;
      result += char;
    }
    return result;
  }
  _StringUtils.takeWhile = takeWhile;
  function takeRightWhile(value, predicate) {
    var _a, _b;
    let result = "";
    for (let i = ((_a = value == null ? void 0 : value.length) != null ? _a : 0) - 1; i >= 0; i--) {
      const char = (_b = value == null ? void 0 : value.charAt(i)) != null ? _b : "";
      if (!predicate(char)) break;
      result = char + result;
    }
    return result;
  }
  _StringUtils.takeRightWhile = takeRightWhile;
  function drop(value, n) {
    var _a;
    return (_a = value == null ? void 0 : value.slice(n)) != null ? _a : "";
  }
  _StringUtils.drop = drop;
  function dropRight(value, n) {
    var _a;
    return (_a = value == null ? void 0 : value.slice(0, -n)) != null ? _a : "";
  }
  _StringUtils.dropRight = dropRight;
  function dropWhile(value, predicate) {
    var _a;
    let i = 0;
    for (const char of value != null ? value : "") {
      if (!predicate(char)) break;
      i++;
    }
    return (_a = value == null ? void 0 : value.slice(i)) != null ? _a : "";
  }
  _StringUtils.dropWhile = dropWhile;
  function dropRightWhile(value, predicate) {
    var _a, _b, _c;
    let i = ((_a = value == null ? void 0 : value.length) != null ? _a : 0) - 1;
    while (i >= 0 && predicate((_b = value == null ? void 0 : value.charAt(i)) != null ? _b : "")) i--;
    return (_c = value == null ? void 0 : value.slice(0, i + 1)) != null ? _c : "";
  }
  _StringUtils.dropRightWhile = dropRightWhile;
  function countLines(value) {
    return lineCount(value);
  }
  _StringUtils.countLines = countLines;
  function getLine(value, lineNumber) {
    var _a;
    return (_a = splitLines(value)[lineNumber]) != null ? _a : "";
  }
  _StringUtils.getLine = getLine;
  function getLines(value) {
    return splitLines(value);
  }
  _StringUtils.getLines = getLines;
  function isSingleLine(value) {
    return !(value == null ? void 0 : value.includes("\n"));
  }
  _StringUtils.isSingleLine = isSingleLine;
  function isMultiLine(value) {
    var _a;
    return (_a = value == null ? void 0 : value.includes("\n")) != null ? _a : false;
  }
  _StringUtils.isMultiLine = isMultiLine;
  function normalizeLineEndings(value, lineEnding = "\n") {
    var _a;
    return (_a = value == null ? void 0 : value.replace(/\r\n|\r|\n/g, lineEnding)) != null ? _a : "";
  }
  _StringUtils.normalizeLineEndings = normalizeLineEndings;
  function toCamelCase(value) {
    return camelCase(value);
  }
  _StringUtils.toCamelCase = toCamelCase;
  function toKebabCase(value) {
    return kebabCase(value);
  }
  _StringUtils.toKebabCase = toKebabCase;
  function toSnakeCase(value) {
    return snakeCase(value);
  }
  _StringUtils.toSnakeCase = toSnakeCase;
  function toPascalCase(value) {
    return pascalCase(value);
  }
  _StringUtils.toPascalCase = toPascalCase;
  function toConstantCase(value) {
    return constantCase(value);
  }
  _StringUtils.toConstantCase = toConstantCase;
  function toSentenceCase(value) {
    if (isEmpty(value)) return "";
    return value.charAt(0).toUpperCase() + value.slice(1).toLowerCase();
  }
  _StringUtils.toSentenceCase = toSentenceCase;
  function toTitleCase(value) {
    return capitalizeWords(value);
  }
  _StringUtils.toTitleCase = toTitleCase;
  function toCapitalCase(value) {
    return capitalizeWords(value);
  }
  _StringUtils.toCapitalCase = toCapitalCase;
  function toDotCase(value) {
    var _a;
    return (_a = value == null ? void 0 : value.replace(/([a-z])([A-Z])/g, "$1.$2").replace(/[-_\s]+/g, ".").toLowerCase()) != null ? _a : "";
  }
  _StringUtils.toDotCase = toDotCase;
  function toPathCase(value) {
    var _a;
    return (_a = value == null ? void 0 : value.replace(/([a-z])([A-Z])/g, "$1/$2").replace(/[-_\s]+/g, "/").toLowerCase()) != null ? _a : "";
  }
  _StringUtils.toPathCase = toPathCase;
  function stripTags(value) {
    var _a;
    return (_a = value == null ? void 0 : value.replace(/<[^>]*>/g, "")) != null ? _a : "";
  }
  _StringUtils.stripTags = stripTags;
  function stripNumbers(value) {
    var _a;
    return (_a = value == null ? void 0 : value.replace(/\d+/g, "")) != null ? _a : "";
  }
  _StringUtils.stripNumbers = stripNumbers;
  function stripWhitespace(value) {
    var _a;
    return (_a = value == null ? void 0 : value.replace(/\s+/g, "")) != null ? _a : "";
  }
  _StringUtils.stripWhitespace = stripWhitespace;
  function stripPunctuation(value) {
    var _a;
    return (_a = value == null ? void 0 : value.replace(/[^\w\s]/g, "")) != null ? _a : "";
  }
  _StringUtils.stripPunctuation = stripPunctuation;
  function normalizeWhitespace(value) {
    var _a;
    return (_a = value == null ? void 0 : value.replace(/\s+/g, " ").trim()) != null ? _a : "";
  }
  _StringUtils.normalizeWhitespace = normalizeWhitespace;
  function includesAll(value, searches) {
    return searches.every((search) => {
      var _a;
      return (_a = value == null ? void 0 : value.includes(search)) != null ? _a : false;
    });
  }
  _StringUtils.includesAll = includesAll;
  function includesAny(value, searches) {
    return searches.some((search) => {
      var _a;
      return (_a = value == null ? void 0 : value.includes(search)) != null ? _a : false;
    });
  }
  _StringUtils.includesAny = includesAny;
})(StringUtils || (StringUtils = {}));

// ../../../sdkwork-im/node_modules/.pnpm-codex-new/@sdkwork+sdk-common@1.0.3/node_modules/@sdkwork/sdk-common/dist/utils/encoding.js
var Encoding;
(function(_Encoding) {
  function base64Encode(input) {
    var _a, _b, _c;
    let bytes;
    if (typeof input === "string") bytes = new TextEncoder().encode(input);
    else bytes = input;
    const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let result = "";
    let i = 0;
    while (i < bytes.length) {
      const a = (_a = bytes[i++]) != null ? _a : 0;
      const b = i < bytes.length ? (_b = bytes[i++]) != null ? _b : 0 : 0;
      const c = i < bytes.length ? (_c = bytes[i++]) != null ? _c : 0 : 0;
      const bitmap = a << 16 | b << 8 | c;
      result += chars[bitmap >> 18 & 63];
      result += chars[bitmap >> 12 & 63];
      result += i > bytes.length + 1 ? "=" : chars[bitmap >> 6 & 63];
      result += i > bytes.length ? "=" : chars[bitmap & 63];
    }
    return result;
  }
  _Encoding.base64Encode = base64Encode;
  function base64Decode(input) {
    var _a, _b, _c, _d;
    const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    input = input.replace(/[^A-Za-z0-9+/]/g, "");
    const len = input.length;
    let result = "";
    let i = 0;
    while (i < len) {
      const a = chars.indexOf((_a = input[i++]) != null ? _a : "");
      const b = chars.indexOf((_b = input[i++]) != null ? _b : "");
      const c = chars.indexOf((_c = input[i++]) != null ? _c : "");
      const d = chars.indexOf((_d = input[i++]) != null ? _d : "");
      const bitmap = a << 18 | b << 12 | c << 6 | d;
      result += String.fromCharCode(bitmap >> 16 & 255);
      if (c !== 64 && input[i - 2] !== "=") result += String.fromCharCode(bitmap >> 8 & 255);
      if (d !== 64 && input[i - 1] !== "=") result += String.fromCharCode(bitmap & 255);
    }
    return result;
  }
  _Encoding.base64Decode = base64Decode;
  function base64UrlEncode(input) {
    return base64Encode(input).replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
  }
  _Encoding.base64UrlEncode = base64UrlEncode;
  function base64UrlDecode(input) {
    input = input.replace(/-/g, "+").replace(/_/g, "/");
    const pad = input.length % 4;
    if (pad) input += "=".repeat(4 - pad);
    return base64Decode(input);
  }
  _Encoding.base64UrlDecode = base64UrlDecode;
  function base64ToBytes(base64) {
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
    return bytes;
  }
  _Encoding.base64ToBytes = base64ToBytes;
  function bytesToBase64(bytes) {
    var _a;
    let binary = "";
    for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode((_a = bytes[i]) != null ? _a : 0);
    return btoa(binary);
  }
  _Encoding.bytesToBase64 = bytesToBase64;
  function utf8Encode(input) {
    return new TextEncoder().encode(input);
  }
  _Encoding.utf8Encode = utf8Encode;
  function utf8Decode(input) {
    return new TextDecoder().decode(input);
  }
  _Encoding.utf8Decode = utf8Decode;
  function hexEncode(input) {
    const bytes = typeof input === "string" ? utf8Encode(input) : input;
    return Array.from(bytes).map((byte) => byte.toString(16).padStart(2, "0")).join("");
  }
  _Encoding.hexEncode = hexEncode;
  function hexDecode(input) {
    const bytes = new Uint8Array(input.length / 2);
    for (let i = 0; i < input.length; i += 2) bytes[i / 2] = parseInt(input.substr(i, 2), 16);
    return utf8Decode(bytes);
  }
  _Encoding.hexDecode = hexDecode;
  function hexToBytes(hex) {
    const bytes = new Uint8Array(hex.length / 2);
    for (let i = 0; i < hex.length; i += 2) bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
    return bytes;
  }
  _Encoding.hexToBytes = hexToBytes;
  function bytesToHex(bytes) {
    return Array.from(bytes).map((byte) => byte.toString(16).padStart(2, "0")).join("");
  }
  _Encoding.bytesToHex = bytesToHex;
  function urlEncode(input) {
    return encodeURIComponent(input);
  }
  _Encoding.urlEncode = urlEncode;
  function urlDecode(input) {
    return decodeURIComponent(input);
  }
  _Encoding.urlDecode = urlDecode;
  function urlEncodeComponent(input) {
    return encodeURIComponent(input);
  }
  _Encoding.urlEncodeComponent = urlEncodeComponent;
  function urlDecodeComponent(input) {
    return decodeURIComponent(input);
  }
  _Encoding.urlDecodeComponent = urlDecodeComponent;
  function htmlEncode(input) {
    const htmlEntities = {
      "&": "&amp;",
      "<": "&lt;",
      ">": "&gt;",
      '"': "&quot;",
      "'": "&#39;",
      "/": "&#x2F;",
      "`": "&#x60;",
      "=": "&#x3D;"
    };
    return input.replace(/[&<>"'`=/]/g, (char) => htmlEntities[char] || char);
  }
  _Encoding.htmlEncode = htmlEncode;
  function htmlDecode(input) {
    const htmlEntities = {
      "&amp;": "&",
      "&lt;": "<",
      "&gt;": ">",
      "&quot;": '"',
      "&#39;": "'",
      "&#x27;": "'",
      "&#x2F;": "/",
      "&#x60;": "`",
      "&#x3D;": "=",
      "&nbsp;": " "
    };
    return input.replace(/&[^;]+;/g, (entity) => htmlEntities[entity] || entity);
  }
  _Encoding.htmlDecode = htmlDecode;
  function jsonEncode(value, replacer, space) {
    return JSON.stringify(value, replacer, space);
  }
  _Encoding.jsonEncode = jsonEncode;
  function jsonDecode(input) {
    return JSON.parse(input);
  }
  _Encoding.jsonDecode = jsonDecode;
  function jsonEncodePretty(value, indent = 2) {
    return JSON.stringify(value, null, indent);
  }
  _Encoding.jsonEncodePretty = jsonEncodePretty;
  function tryJsonDecode(input, defaultValue) {
    try {
      return JSON.parse(input);
    } catch {
      return defaultValue;
    }
  }
  _Encoding.tryJsonDecode = tryJsonDecode;
  function isJson(input) {
    try {
      JSON.parse(input);
      return true;
    } catch {
      return false;
    }
  }
  _Encoding.isJson = isJson;
  function xmlEncode(input) {
    const xmlEntities = {
      "&": "&amp;",
      "<": "&lt;",
      ">": "&gt;",
      '"': "&quot;",
      "'": "&apos;"
    };
    return input.replace(/[&<>"']/g, (char) => xmlEntities[char] || char);
  }
  _Encoding.xmlEncode = xmlEncode;
  function xmlDecode(input) {
    const xmlEntities = {
      "&amp;": "&",
      "&lt;": "<",
      "&gt;": ">",
      "&quot;": '"',
      "&apos;": "'"
    };
    return input.replace(/&[^;]+;/g, (entity) => xmlEntities[entity] || entity);
  }
  _Encoding.xmlDecode = xmlDecode;
  function escapeRegex(input) {
    return input.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  }
  _Encoding.escapeRegex = escapeRegex;
  function escapeSql(input) {
    return input.replace(/[\0\x08\x09\x1a\n\r"'\\\%]/g, (char) => {
      return {
        "\0": "\\0",
        "\b": "\\b",
        "	": "\\t",
        "": "\\z",
        "\n": "\\n",
        "\r": "\\r",
        '"': '\\"',
        "'": "\\'",
        "\\": "\\\\",
        "%": "\\%"
      }[char] || char;
    });
  }
  _Encoding.escapeSql = escapeSql;
  function escapeShell(input) {
    return input.replace(/[^A-Za-z0-9_\-.,:\/@\n]/g, (char) => {
      if (char === "\n") return "'\\n'";
      return `\\${char}`;
    });
  }
  _Encoding.escapeShell = escapeShell;
  function escapeCString(input) {
    return input.replace(/[\\"'\n\r\t\b\f\v\0]/g, (char) => {
      return {
        "\\": "\\\\",
        '"': '\\"',
        "'": "\\'",
        "\n": "\\n",
        "\r": "\\r",
        "	": "\\t",
        "\b": "\\b",
        "\f": "\\f",
        "\v": "\\v",
        "\0": "\\0"
      }[char] || char;
    });
  }
  _Encoding.escapeCString = escapeCString;
  function unescapeCString(input) {
    return input.replace(/\\([\\\"'nrtbfv0])/g, (_, char) => {
      return {
        "\\": "\\",
        '"': '"',
        "'": "'",
        "n": "\n",
        "r": "\r",
        "t": "	",
        "b": "\b",
        "f": "\f",
        "v": "\v",
        "0": "\0"
      }[char] || char;
    });
  }
  _Encoding.unescapeCString = unescapeCString;
  function camelToSnake(input) {
    return input.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
  }
  _Encoding.camelToSnake = camelToSnake;
  function snakeToCamel(input) {
    return input.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
  }
  _Encoding.snakeToCamel = snakeToCamel;
  function camelToKebab(input) {
    return input.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);
  }
  _Encoding.camelToKebab = camelToKebab;
  function kebabToCamel(input) {
    return input.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
  }
  _Encoding.kebabToCamel = kebabToCamel;
  function camelToPascal(input) {
    return input.charAt(0).toUpperCase() + input.slice(1);
  }
  _Encoding.camelToPascal = camelToPascal;
  function pascalToCamel(input) {
    return input.charAt(0).toLowerCase() + input.slice(1);
  }
  _Encoding.pascalToCamel = pascalToCamel;
  function pascalToSnake(input) {
    return camelToSnake(input);
  }
  _Encoding.pascalToSnake = pascalToSnake;
  function snakeToPascal(input) {
    return camelToPascal(snakeToCamel(input));
  }
  _Encoding.snakeToPascal = snakeToPascal;
  function pascalToKebab(input) {
    return camelToKebab(input);
  }
  _Encoding.pascalToKebab = pascalToKebab;
  function kebabToPascal(input) {
    return camelToPascal(kebabToCamel(input));
  }
  _Encoding.kebabToPascal = kebabToPascal;
  function toSnakeCase(input) {
    return input.replace(/([a-z])([A-Z])/g, "$1_$2").replace(/[-\s]+/g, "_").toLowerCase();
  }
  _Encoding.toSnakeCase = toSnakeCase;
  function toKebabCase(input) {
    return input.replace(/([a-z])([A-Z])/g, "$1-$2").replace(/[_\s]+/g, "-").toLowerCase();
  }
  _Encoding.toKebabCase = toKebabCase;
  function toCamelCase(input) {
    return input.replace(/[-_\s]+(.)?/g, (_, char) => char ? char.toUpperCase() : "").replace(/^(.)/, (char) => char.toLowerCase());
  }
  _Encoding.toCamelCase = toCamelCase;
  function toPascalCase(input) {
    const camel = toCamelCase(input);
    return camel.charAt(0).toUpperCase() + camel.slice(1);
  }
  _Encoding.toPascalCase = toPascalCase;
  function toConstantCase(input) {
    return toSnakeCase(input).toUpperCase();
  }
  _Encoding.toConstantCase = toConstantCase;
  function toSentenceCase(input) {
    return input.charAt(0).toUpperCase() + input.slice(1).toLowerCase();
  }
  _Encoding.toSentenceCase = toSentenceCase;
  function toTitleCase(input) {
    return input.replace(/\b\w/g, (char) => char.toUpperCase());
  }
  _Encoding.toTitleCase = toTitleCase;
  function toCapitalCase(input) {
    return input.replace(/[-_\s]+(.)?/g, (_, char) => char ? ` ${char.toUpperCase()}` : "").trim();
  }
  _Encoding.toCapitalCase = toCapitalCase;
  function toDotCase(input) {
    return input.replace(/([a-z])([A-Z])/g, "$1.$2").replace(/[-_\s]+/g, ".").toLowerCase();
  }
  _Encoding.toDotCase = toDotCase;
  function toPathCase(input) {
    return input.replace(/([a-z])([A-Z])/g, "$1/$2").replace(/[-_\s]+/g, "/").toLowerCase();
  }
  _Encoding.toPathCase = toPathCase;
  function rot13(input) {
    return input.replace(/[a-zA-Z]/g, (char) => {
      const start = char <= "Z" ? 65 : 97;
      return String.fromCharCode((char.charCodeAt(0) - start + 13) % 26 + start);
    });
  }
  _Encoding.rot13 = rot13;
  function caesarCipher(input, shift) {
    return input.replace(/[a-zA-Z]/g, (char) => {
      const start = char <= "Z" ? 65 : 97;
      const shifted = ((char.charCodeAt(0) - start + shift) % 26 + 26) % 26;
      return String.fromCharCode(shifted + start);
    });
  }
  _Encoding.caesarCipher = caesarCipher;
  function caesarDecipher(input, shift) {
    return caesarCipher(input, -shift);
  }
  _Encoding.caesarDecipher = caesarDecipher;
  function xorEncode(input, key) {
    var _a, _b;
    const inputBytes = utf8Encode(input);
    const keyBytes = utf8Encode(key);
    const result = new Uint8Array(inputBytes.length);
    for (let i = 0; i < inputBytes.length; i++) result[i] = ((_a = inputBytes[i]) != null ? _a : 0) ^ ((_b = keyBytes[i % keyBytes.length]) != null ? _b : 0);
    return bytesToHex(result);
  }
  _Encoding.xorEncode = xorEncode;
  function xorDecode(input, key) {
    var _a, _b;
    const inputBytes = hexToBytes(input);
    const keyBytes = utf8Encode(key);
    const result = new Uint8Array(inputBytes.length);
    for (let i = 0; i < inputBytes.length; i++) result[i] = ((_a = inputBytes[i]) != null ? _a : 0) ^ ((_b = keyBytes[i % keyBytes.length]) != null ? _b : 0);
    return utf8Decode(result);
  }
  _Encoding.xorDecode = xorDecode;
  function charCodeEncode(input) {
    return Array.from(input).map((char) => char.charCodeAt(0));
  }
  _Encoding.charCodeEncode = charCodeEncode;
  function charCodeDecode(codes) {
    return String.fromCharCode(...codes);
  }
  _Encoding.charCodeDecode = charCodeDecode;
  function binaryEncode(input) {
    return Array.from(input).map((char) => char.charCodeAt(0).toString(2).padStart(8, "0")).join(" ");
  }
  _Encoding.binaryEncode = binaryEncode;
  function binaryDecode(input) {
    return input.split(/\s+/).map((byte) => String.fromCharCode(parseInt(byte, 2))).join("");
  }
  _Encoding.binaryDecode = binaryDecode;
  function octalEncode(input) {
    return Array.from(input).map((char) => char.charCodeAt(0).toString(8).padStart(3, "0")).join(" ");
  }
  _Encoding.octalEncode = octalEncode;
  function octalDecode(input) {
    return input.split(/\s+/).map((byte) => String.fromCharCode(parseInt(byte, 8))).join("");
  }
  _Encoding.octalDecode = octalDecode;
  function decimalEncode(input) {
    return Array.from(input).map((char) => char.charCodeAt(0).toString(10)).join(" ");
  }
  _Encoding.decimalEncode = decimalEncode;
  function decimalDecode(input) {
    return input.split(/\s+/).map((code) => String.fromCharCode(parseInt(code, 10))).join("");
  }
  _Encoding.decimalDecode = decimalDecode;
  function punycodeEncode(input) {
    const prefix = "xn--";
    if (input.startsWith(prefix)) return input;
    const asciiPart = input.replace(/[^\x00-\x7F]/g, "");
    const nonAsciiPart = input.replace(/[\x00-\x7F]/g, "");
    if (!nonAsciiPart) return input;
    return prefix + asciiPart + "-" + nonAsciiPart.split("").map((c) => c.charCodeAt(0).toString(36)).join("");
  }
  _Encoding.punycodeEncode = punycodeEncode;
  function slugify(input) {
    return input.toLowerCase().trim().replace(/[^\w\s-]/g, "").replace(/[\s_-]+/g, "-").replace(/^-+|-+$/g, "");
  }
  _Encoding.slugify = slugify;
  function unslugify(input) {
    return input.replace(/-/g, " ").replace(/\b\w/g, (char) => char.toUpperCase());
  }
  _Encoding.unslugify = unslugify;
  function queryStringEncode(params) {
    return Object.entries(params).filter(([, value]) => value !== void 0 && value !== null).map(([key, value]) => {
      if (Array.isArray(value)) return value.map((v) => `${urlEncode(key)}=${urlEncode(String(v))}`).join("&");
      return `${urlEncode(key)}=${urlEncode(String(value))}`;
    }).join("&");
  }
  _Encoding.queryStringEncode = queryStringEncode;
  function queryStringDecode(query) {
    const result = {};
    if (!query) return result;
    query = query.replace(/^[?#]/, "");
    for (const pair of query.split("&")) {
      const parts = pair.split("=");
      const key = parts[0];
      const value = parts[1];
      if (!key) continue;
      const decodedKey = urlDecode(key);
      const decodedValue = value ? urlDecode(value) : "";
      if (result[decodedKey]) if (Array.isArray(result[decodedKey])) result[decodedKey].push(decodedValue);
      else result[decodedKey] = [result[decodedKey], decodedValue];
      else result[decodedKey] = decodedValue;
    }
    return result;
  }
  _Encoding.queryStringDecode = queryStringDecode;
  function formDataEncode(data) {
    return Object.entries(data).filter(([, value]) => value !== void 0 && value !== null).map(([key, value]) => `${urlEncode(key)}=${urlEncode(String(value))}`).join("&");
  }
  _Encoding.formDataEncode = formDataEncode;
  function mimeTypeToExtension(mimeType) {
    return {
      "application/json": "json",
      "application/xml": "xml",
      "application/pdf": "pdf",
      "application/zip": "zip",
      "application/gzip": "gz",
      "application/x-tar": "tar",
      "application/x-rar-compressed": "rar",
      "application/x-7z-compressed": "7z",
      "application/vnd.ms-excel": "xls",
      "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet": "xlsx",
      "application/vnd.ms-powerpoint": "ppt",
      "application/vnd.openxmlformats-officedocument.presentationml.presentation": "pptx",
      "application/msword": "doc",
      "application/vnd.openxmlformats-officedocument.wordprocessingml.document": "docx",
      "text/plain": "txt",
      "text/html": "html",
      "text/css": "css",
      "text/javascript": "js",
      "text/csv": "csv",
      "text/xml": "xml",
      "image/jpeg": "jpg",
      "image/png": "png",
      "image/gif": "gif",
      "image/svg+xml": "svg",
      "image/webp": "webp",
      "image/bmp": "bmp",
      "image/tiff": "tiff",
      "image/x-icon": "ico",
      "audio/mpeg": "mp3",
      "audio/wav": "wav",
      "audio/ogg": "ogg",
      "audio/aac": "aac",
      "video/mp4": "mp4",
      "video/mpeg": "mpeg",
      "video/webm": "webm",
      "video/ogg": "ogv",
      "video/x-msvideo": "avi",
      "video/quicktime": "mov"
    }[mimeType.toLowerCase()] || "";
  }
  _Encoding.mimeTypeToExtension = mimeTypeToExtension;
  function extensionToMimeType(extension) {
    return {
      "json": "application/json",
      "xml": "application/xml",
      "pdf": "application/pdf",
      "zip": "application/zip",
      "gz": "application/gzip",
      "tar": "application/x-tar",
      "rar": "application/x-rar-compressed",
      "7z": "application/x-7z-compressed",
      "xls": "application/vnd.ms-excel",
      "xlsx": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      "ppt": "application/vnd.ms-powerpoint",
      "pptx": "application/vnd.openxmlformats-officedocument.presentationml.presentation",
      "doc": "application/msword",
      "docx": "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
      "txt": "text/plain",
      "html": "text/html",
      "htm": "text/html",
      "css": "text/css",
      "js": "text/javascript",
      "csv": "text/csv",
      "jpg": "image/jpeg",
      "jpeg": "image/jpeg",
      "png": "image/png",
      "gif": "image/gif",
      "svg": "image/svg+xml",
      "webp": "image/webp",
      "bmp": "image/bmp",
      "tiff": "image/tiff",
      "tif": "image/tiff",
      "ico": "image/x-icon",
      "mp3": "audio/mpeg",
      "wav": "audio/wav",
      "ogg": "audio/ogg",
      "aac": "audio/aac",
      "mp4": "video/mp4",
      "mpeg": "video/mpeg",
      "mpg": "video/mpeg",
      "webm": "video/webm",
      "ogv": "video/ogg",
      "avi": "video/x-msvideo",
      "mov": "video/quicktime"
    }[extension.toLowerCase().replace(/^\./, "")] || "application/octet-stream";
  }
  _Encoding.extensionToMimeType = extensionToMimeType;
  function charsetEncode(input, _charset) {
    return new TextEncoder().encode(input);
  }
  _Encoding.charsetEncode = charsetEncode;
  function charsetDecode(input, charset) {
    return new TextDecoder(charset).decode(input);
  }
  _Encoding.charsetDecode = charsetDecode;
  function stripBom(input) {
    if (input.charCodeAt(0) === 65279) return input.slice(1);
    return input;
  }
  _Encoding.stripBom = stripBom;
  function addBom(input, bom = "utf-8") {
    return {
      "utf-8": "\uFEFF",
      "utf-16le": "\uFFFE",
      "utf-16be": "\uFEFF"
    }[bom] + input;
  }
  _Encoding.addBom = addBom;
  function normalizeEncoding(input, fromEncoding, toEncoding) {
    return charsetDecode(charsetEncode(input, fromEncoding), toEncoding);
  }
  _Encoding.normalizeEncoding = normalizeEncoding;
  function isValidBase64(input) {
    if (!input || input.length % 4 !== 0) return false;
    return /^[A-Za-z0-9+/]*={0,2}$/.test(input);
  }
  _Encoding.isValidBase64 = isValidBase64;
  function isValidHex(input) {
    return /^[0-9a-fA-F]*$/.test(input) && input.length % 2 === 0;
  }
  _Encoding.isValidHex = isValidHex;
  function isValidUrl(input) {
    try {
      new URL(input);
      return true;
    } catch {
      return false;
    }
  }
  _Encoding.isValidUrl = isValidUrl;
  function isValidEmail(input) {
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(input);
  }
  _Encoding.isValidEmail = isValidEmail;
  function detectEncoding(input) {
    if (input.charCodeAt(0) === 65279) return "utf-8-bom";
    if (input.charCodeAt(0) === 65534) return "utf-16le";
    if (input.charCodeAt(0) === 65279 && input.charCodeAt(1) === 0) return "utf-16be";
    if (/[\u4e00-\u9fa5]/.test(input)) return "utf-8";
    return "ascii";
  }
  _Encoding.detectEncoding = detectEncoding;
})(Encoding || (Encoding = {}));
Encoding.base64Encode;
Encoding.base64Decode;
Encoding.base64UrlEncode;
Encoding.base64UrlDecode;
Encoding.utf8Encode;
Encoding.utf8Decode;
Encoding.hexEncode;
Encoding.hexDecode;
Encoding.urlEncode;
Encoding.urlDecode;
Encoding.htmlEncode;
Encoding.htmlDecode;
Encoding.jsonEncode;
Encoding.jsonDecode;
Encoding.xmlEncode;
Encoding.xmlDecode;
Encoding.escapeRegex;
Encoding.escapeSql;
Encoding.escapeShell;
Encoding.queryStringEncode;
Encoding.queryStringDecode;
Encoding.slugify;
Encoding.unslugify;

// ../../../sdkwork-im/node_modules/.pnpm-codex-new/@sdkwork+sdk-common@1.0.3/node_modules/@sdkwork/sdk-common/dist/utils/date.js
var MILLISECONDS_IN_SECOND = 1e3;
var MILLISECONDS_IN_MINUTE = 60 * MILLISECONDS_IN_SECOND;
var MILLISECONDS_IN_HOUR = 60 * MILLISECONDS_IN_MINUTE;
var MILLISECONDS_IN_DAY = 24 * MILLISECONDS_IN_HOUR;
var MILLISECONDS_IN_WEEK = 7 * MILLISECONDS_IN_DAY;
var TIME_UNITS_IN_MS = {
  millisecond: 1,
  second: MILLISECONDS_IN_SECOND,
  minute: MILLISECONDS_IN_MINUTE,
  hour: MILLISECONDS_IN_HOUR,
  day: MILLISECONDS_IN_DAY,
  week: MILLISECONDS_IN_WEEK,
  month: 30 * MILLISECONDS_IN_DAY,
  quarter: 90 * MILLISECONDS_IN_DAY,
  year: 365 * MILLISECONDS_IN_DAY
};

// ../../../sdkwork-im/node_modules/.pnpm-codex-new/@sdkwork+sdk-common@1.0.3/node_modules/@sdkwork/sdk-common/dist/http/base-client.js
var BaseHttpClient = class {
  constructor(config) {
    __publicField(this, "config");
    __publicField(this, "authConfig");
    __publicField(this, "logger");
    __publicField(this, "cache");
    __publicField(this, "interceptors");
    __publicField(this, "tenantId");
    __publicField(this, "organizationId");
    __publicField(this, "platform");
    __publicField(this, "userId");
    var _a, _b, _c, _d;
    this.config = {
      baseUrl: config.baseUrl,
      timeout: (_a = config.timeout) != null ? _a : 3e4,
      headers: (_b = config.headers) != null ? _b : {},
      retry: {
        maxRetries: 3,
        retryDelay: 1e3,
        retryBackoff: "exponential",
        maxRetryDelay: 3e4,
        ...config.retry
      },
      cache: {
        enabled: false,
        ttl: 300 * 1e3,
        maxSize: 100,
        ...config.cache
      },
      logger: {
        level: "info",
        prefix: "[SDK]",
        timestamp: true,
        colors: true,
        ...config.logger
      }
    };
    this.logger = createLogger(this.config.logger);
    this.cache = createCacheStore(this.config.cache);
    this.interceptors = (_c = config.interceptors) != null ? _c : {
      request: [],
      response: [],
      error: []
    };
    const authMode = this.determineAuthMode(config);
    this.authConfig = {
      authMode,
      apiKey: config.apiKey,
      tokenManager: (_d = config.tokenManager) != null ? _d : new DefaultAuthTokenManager({
        accessToken: config.accessToken,
        authToken: config.authToken
      })
    };
  }
  determineAuthMode(config) {
    if (config.apiKey) return "apikey";
    return "dual-token";
  }
  getAuthMode() {
    return this.authConfig.authMode;
  }
  setAuthMode(mode) {
    this.authConfig.authMode = mode;
  }
  getTokenManager() {
    return this.authConfig.tokenManager;
  }
  setTokenManager(manager) {
    this.authConfig.tokenManager = manager;
  }
  setApiKey(apiKey) {
    var _a;
    this.authConfig.apiKey = apiKey;
    this.authConfig.authMode = "apikey";
    (_a = this.authConfig.tokenManager) == null ? void 0 : _a.clearTokens();
  }
  setAuthToken(token) {
    var _a;
    (_a = this.authConfig.tokenManager) == null ? void 0 : _a.setAuthToken(token);
    if (this.authConfig.authMode === "apikey") {
      this.authConfig.authMode = "dual-token";
      this.authConfig.apiKey = void 0;
    }
  }
  setAccessToken(token) {
    var _a;
    (_a = this.authConfig.tokenManager) == null ? void 0 : _a.setAccessToken(token);
    if (this.authConfig.authMode === "apikey") {
      this.authConfig.authMode = "dual-token";
      this.authConfig.apiKey = void 0;
    }
  }
  setTenantId(tenantId) {
    this.tenantId = tenantId;
  }
  setOrganizationId(organizationId) {
    this.organizationId = organizationId;
  }
  setPlatform(platform) {
    this.platform = platform;
  }
  setUserId(userId) {
    this.userId = userId;
  }
  clearAuthToken() {
    var _a;
    (_a = this.authConfig.tokenManager) == null ? void 0 : _a.clearTokens();
  }
  addRequestInterceptor(interceptor) {
    this.interceptors.request.push(interceptor);
    return () => {
      const index = this.interceptors.request.indexOf(interceptor);
      if (index > -1) this.interceptors.request.splice(index, 1);
    };
  }
  addResponseInterceptor(interceptor) {
    this.interceptors.response.push(interceptor);
    return () => {
      const index = this.interceptors.response.indexOf(interceptor);
      if (index > -1) this.interceptors.response.splice(index, 1);
    };
  }
  addErrorInterceptor(interceptor) {
    this.interceptors.error.push(interceptor);
    return () => {
      const index = this.interceptors.error.indexOf(interceptor);
      if (index > -1) this.interceptors.error.splice(index, 1);
    };
  }
  clearCache() {
    this.cache.clear();
  }
  getConfig() {
    var _a, _b;
    return {
      baseUrl: this.config.baseUrl,
      timeout: this.config.timeout,
      authMode: this.authConfig.authMode,
      apiKey: this.authConfig.apiKey,
      accessToken: (_a = this.authConfig.tokenManager) == null ? void 0 : _a.getAccessToken(),
      authToken: (_b = this.authConfig.tokenManager) == null ? void 0 : _b.getAuthToken(),
      tenantId: this.tenantId,
      organizationId: this.organizationId,
      platform: this.platform,
      userId: this.userId
    };
  }
  isAuthenticated() {
    var _a, _b;
    return (_b = (_a = this.authConfig.tokenManager) == null ? void 0 : _a.isValid()) != null ? _b : false;
  }
  buildBaseUrl(path, params) {
    let url = `${this.config.baseUrl.replace(/\/$/, "")}${path.startsWith("/") ? path : `/${path}`}`;
    if (params) {
      const searchParams = new URLSearchParams();
      Object.entries(params).forEach(([key, value]) => {
        if (Array.isArray(value)) {
          value.forEach((item) => {
            if (item !== void 0 && item !== null) searchParams.append(key, String(item));
          });
          return;
        }
        if (value !== void 0 && value !== null) searchParams.append(key, String(value));
      });
      const queryString = searchParams.toString();
      if (queryString) url += `?${queryString}`;
    }
    return url;
  }
  buildHeaders(config, skipAuth = false) {
    const headers = {
      "Content-Type": MIME_TYPES.JSON,
      ...this.config.headers,
      ...config.headers
    };
    if (!skipAuth && !config.skipAuth) {
      const authHeaders = buildAuthHeaders(this.authConfig.authMode, this.authConfig.apiKey, this.authConfig.tokenManager);
      Object.assign(headers, authHeaders);
    }
    if (this.tenantId) headers["X-Tenant-Id"] = this.tenantId;
    if (this.organizationId) headers["X-Organization-Id"] = this.organizationId;
    if (this.platform) headers["X-Platform"] = this.platform;
    if (this.userId !== void 0) headers["X-User-Id"] = String(this.userId);
    return headers;
  }
  serializeRequestBody(body, headers) {
    if (body === void 0 || body === null) return;
    if (typeof FormData !== "undefined" && body instanceof FormData) {
      delete headers["Content-Type"];
      return body;
    }
    if (typeof URLSearchParams !== "undefined" && body instanceof URLSearchParams) {
      headers["Content-Type"] = "application/x-www-form-urlencoded;charset=UTF-8";
      return body.toString();
    }
    if (typeof Blob !== "undefined" && body instanceof Blob) {
      delete headers["Content-Type"];
      return body;
    }
    if (typeof ArrayBuffer !== "undefined") {
      if (body instanceof ArrayBuffer) {
        delete headers["Content-Type"];
        return body;
      }
      if (ArrayBuffer.isView(body)) {
        delete headers["Content-Type"];
        return body;
      }
    }
    if (typeof body === "string") {
      headers["Content-Type"] = headers["Content-Type"] || "text/plain;charset=UTF-8";
      return body;
    }
    return JSON.stringify(body);
  }
  async applyRequestInterceptors(config) {
    let processedConfig = config;
    for (const interceptor of this.interceptors.request) processedConfig = await interceptor(processedConfig);
    return processedConfig;
  }
  async applyResponseInterceptors(response, config) {
    let processedResponse = response;
    for (const interceptor of this.interceptors.response) processedResponse = await interceptor(processedResponse, config);
    return processedResponse;
  }
  async applyErrorInterceptors(error, config) {
    for (const interceptor of this.interceptors.error) await interceptor(error, config);
  }
  async handleErrorResponse(response, config) {
    let errorMessage = `HTTP ${response.status}: ${response.statusText}`;
    try {
      const result = await response.json();
      errorMessage = result.msg || result.message || errorMessage;
    } catch {
    }
    const error = SdkError.fromHttpStatus(response.status, errorMessage);
    await this.applyErrorInterceptors(error, config);
    throw error;
  }
  async processResponse(response, config) {
    if (!response.ok) await this.handleErrorResponse(response, config);
    const contentType = response.headers.get("content-type");
    if (contentType == null ? void 0 : contentType.includes(MIME_TYPES.JSON)) {
      const result = await response.json();
      if (!SUCCESS_CODES.includes(result.code) && !SUCCESS_CODES.includes(String(result.code))) throw SdkError.fromApiResult(result, response.status);
      return result.data;
    }
    if (contentType == null ? void 0 : contentType.includes("text/")) return await response.text();
    return await response.json();
  }
  async executeFetch(url, options) {
    const controller = new AbortController();
    let timedOut = false;
    const timeoutId = setTimeout(() => {
      timedOut = true;
      controller.abort();
    }, options.timeout);
    const abortHandler = () => controller.abort();
    if (options.signal) if (options.signal.aborted) controller.abort();
    else options.signal.addEventListener("abort", abortHandler, { once: true });
    try {
      this.logger.debug(`${options.method} ${url}`);
      return await fetch(url, {
        method: options.method,
        headers: options.headers,
        body: options.body,
        signal: controller.signal
      });
    } catch (error) {
      if (error instanceof Error) {
        if (error.name === "AbortError") {
          if (timedOut) throw new TimeoutError(`Request timeout after ${options.timeout}ms`, options.timeout);
          throw new CancelledError("Request was cancelled");
        }
        throw new NetworkError(error.message);
      }
      throw new NetworkError("Unknown network error");
    } finally {
      clearTimeout(timeoutId);
      if (options.signal) options.signal.removeEventListener("abort", abortHandler);
    }
  }
  async execute(config) {
    var _a;
    const processedConfig = await this.applyRequestInterceptors(config);
    const url = this.buildBaseUrl(processedConfig.url, processedConfig.params);
    const headers = this.buildHeaders(processedConfig);
    const serializedBody = this.serializeRequestBody(processedConfig.body, headers);
    const response = await this.executeFetch(url, {
      method: processedConfig.method,
      headers,
      body: serializedBody,
      timeout: (_a = processedConfig.timeout) != null ? _a : this.config.timeout,
      signal: processedConfig.signal
    });
    return this.processResponse(response, processedConfig);
  }
  async upload(path, options) {
    var _a, _b;
    const formData = new FormData();
    formData.append((_a = options.fieldName) != null ? _a : "file", options.file);
    if (options.additionalData) Object.entries(options.additionalData).forEach(([key, value]) => {
      formData.append(key, value);
    });
    const config = {
      url: path,
      method: "POST",
      body: formData,
      skipAuth: false
    };
    const processedConfig = await this.applyRequestInterceptors(config);
    const url = this.buildBaseUrl(processedConfig.url, processedConfig.params);
    const headers = this.buildHeaders(processedConfig);
    delete headers["Content-Type"];
    const response = await this.executeFetch(url, {
      method: "POST",
      headers,
      body: formData,
      timeout: (_b = processedConfig.timeout) != null ? _b : this.config.timeout,
      signal: processedConfig.signal
    });
    return this.processResponse(response, processedConfig);
  }
  async download(path, _options) {
    var _a;
    const config = {
      url: path,
      method: "GET",
      skipAuth: false
    };
    const processedConfig = await this.applyRequestInterceptors(config);
    const url = this.buildBaseUrl(processedConfig.url, processedConfig.params);
    const headers = this.buildHeaders(processedConfig);
    const response = await this.executeFetch(url, {
      method: "GET",
      headers,
      timeout: (_a = processedConfig.timeout) != null ? _a : this.config.timeout,
      signal: processedConfig.signal
    });
    if (!response.ok) await this.handleErrorResponse(response, processedConfig);
    return response.blob();
  }
  async *stream(path, options) {
    var _a, _b, _c, _d;
    const config = {
      url: path,
      method: (_a = options == null ? void 0 : options.method) != null ? _a : "POST",
      body: options == null ? void 0 : options.body,
      headers: options == null ? void 0 : options.headers,
      skipAuth: options == null ? void 0 : options.skipAuth
    };
    const processedConfig = await this.applyRequestInterceptors(config);
    const url = this.buildBaseUrl(processedConfig.url, processedConfig.params);
    const headers = this.buildHeaders(processedConfig);
    const response = await this.executeFetch(url, {
      method: processedConfig.method,
      headers,
      body: processedConfig.body ? JSON.stringify(processedConfig.body) : void 0,
      timeout: (_b = processedConfig.timeout) != null ? _b : this.config.timeout,
      signal: processedConfig.signal
    });
    if (!response.ok) await this.handleErrorResponse(response, processedConfig);
    const reader = (_c = response.body) == null ? void 0 : _c.getReader();
    if (!reader) throw new NetworkError("No response body");
    const decoder = new TextDecoder();
    let buffer = "";
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split("\n");
        buffer = (_d = lines.pop()) != null ? _d : "";
        for (const line of lines) {
          const trimmedLine = line.trim();
          if (trimmedLine === "" || trimmedLine === "data: [DONE]") continue;
          if (trimmedLine.startsWith("data: ")) yield trimmedLine.slice(6);
          else yield trimmedLine;
        }
      }
    } finally {
      reader.releaseLock();
    }
  }
};

// ../../sdks/sdkwork-agents-app-sdk/sdkwork-agents-app-sdk-typescript/generated/server-openapi/src/http/client.ts
var _HttpClient = class _HttpClient extends BaseHttpClient {
  constructor(config) {
    super(config);
  }
  getInternalAuthConfig() {
    const self = this;
    self.authConfig = self.authConfig || {};
    return self.authConfig;
  }
  getInternalHeaders() {
    const self = this;
    self.config = self.config || {};
    self.config.headers = self.config.headers || {};
    return self.config.headers;
  }
  buildRequestHeaders(headers, contentType) {
    const mergedHeaders = {
      ...headers != null ? headers : {}
    };
    if (contentType && contentType.toLowerCase() !== "multipart/form-data") {
      mergedHeaders["Content-Type"] = contentType;
    }
    return Object.keys(mergedHeaders).length > 0 ? mergedHeaders : void 0;
  }
  buildHeaders(config, skipAuth = false) {
    const headers = super.buildHeaders(config, skipAuth);
    if (!skipAuth && !(config == null ? void 0 : config.skipAuth)) {
      return headers;
    }
    [
      _HttpClient.ACCESS_TOKEN_HEADER,
      "Authorization",
      "Access-Token",
      ["X", "API", "Key"].join("-"),
      "X-Tenant-Id",
      "X-Organization-Id",
      "X-Platform",
      "X-User-Id",
      "X-Sdkwork-Tenant-Id",
      "X-Sdkwork-Organization-Id",
      "X-Sdkwork-User-Id"
    ].forEach((key) => {
      delete headers[key];
    });
    this.applyCredentialEntryBootstrapAccessToken(headers);
    return headers;
  }
  buildRequestBody(body, contentType) {
    if (body == null) {
      return body;
    }
    const normalizedContentType = (contentType != null ? contentType : "").toLowerCase();
    if (normalizedContentType === "application/x-www-form-urlencoded") {
      return this.encodeFormBody(body);
    }
    if (normalizedContentType === "multipart/form-data") {
      return this.encodeMultipartBody(body);
    }
    return body;
  }
  encodeMultipartBody(body) {
    if (body instanceof FormData) {
      return body;
    }
    const formData = new FormData();
    if (body instanceof Map) {
      for (const [key, value] of body.entries()) {
        this.appendMultipartValue(formData, String(key), value);
      }
      return formData;
    }
    if (typeof body === "object") {
      const record = body;
      for (const [key, value] of Object.entries(record)) {
        if (this.isMultipartMetadataField(key)) {
          continue;
        }
        this.appendMultipartValue(formData, key, value, this.resolveMultipartFileName(record, key));
      }
      return formData;
    }
    this.appendMultipartValue(formData, "value", body);
    return formData;
  }
  appendMultipartValue(formData, key, value, fileName) {
    if (value == null) {
      return;
    }
    if (Array.isArray(value)) {
      value.forEach((item) => this.appendMultipartValue(formData, key, item, fileName));
      return;
    }
    if (value instanceof Blob) {
      if (fileName) {
        formData.append(key, value, fileName);
        return;
      }
      formData.append(key, value);
      return;
    }
    if (value instanceof Date) {
      formData.append(key, value.toISOString());
      return;
    }
    if (typeof value === "object") {
      formData.append(key, JSON.stringify(value));
      return;
    }
    formData.append(key, String(value));
  }
  resolveMultipartFileName(record, key) {
    const fieldSpecificName = record[`${key}FileName`];
    if (typeof fieldSpecificName === "string" && fieldSpecificName.trim()) {
      return fieldSpecificName.trim();
    }
    const genericName = record.fileName;
    if (key === "file" && typeof genericName === "string" && genericName.trim()) {
      return genericName.trim();
    }
    return void 0;
  }
  isMultipartMetadataField(key) {
    return key === "fileName" || key.endsWith("FileName");
  }
  encodeFormBody(body) {
    if (body instanceof URLSearchParams) {
      return body.toString();
    }
    if (typeof body === "string") {
      return body;
    }
    const params = new URLSearchParams();
    if (body instanceof Map) {
      for (const [key, value] of body.entries()) {
        this.appendFormValue(params, String(key), value);
      }
      return params.toString();
    }
    if (typeof body === "object") {
      for (const [key, value] of Object.entries(body)) {
        this.appendFormValue(params, key, value);
      }
      return params.toString();
    }
    params.append("value", String(body));
    return params.toString();
  }
  appendFormValue(params, key, value) {
    if (value == null) {
      return;
    }
    if (Array.isArray(value)) {
      value.forEach((item) => this.appendFormValue(params, key, item));
      return;
    }
    if (value instanceof Date) {
      params.append(key, value.toISOString());
      return;
    }
    if (typeof value === "object") {
      params.append(key, JSON.stringify(value));
      return;
    }
    params.append(key, String(value));
  }
  setAuthToken(token) {
    super.setAuthToken(token);
  }
  setAccessToken(token) {
    const headers = this.getInternalHeaders();
    headers[_HttpClient.ACCESS_TOKEN_HEADER] = token;
    super.setAccessToken(token);
  }
  setTokenManager(manager) {
    const baseProto = Object.getPrototypeOf(_HttpClient.prototype);
    if (typeof baseProto.setTokenManager === "function") {
      baseProto.setTokenManager.call(this, manager);
      return;
    }
    this.getInternalAuthConfig().tokenManager = manager;
  }
  applyCredentialEntryBootstrapAccessToken(headers) {
    var _a;
    const authConfig = this.getInternalAuthConfig();
    const tokenManager = authConfig.tokenManager;
    const accessToken = (_a = tokenManager == null ? void 0 : tokenManager.getAccessToken) == null ? void 0 : _a.call(tokenManager);
    if (typeof accessToken === "string" && accessToken.length > 0) {
      headers[_HttpClient.ACCESS_TOKEN_HEADER] = accessToken;
    }
  }
  applySdkworkAuthHeaders(headers) {
    var _a;
    const authConfig = this.getInternalAuthConfig();
    const tokenManager = authConfig.tokenManager;
    const accessToken = (_a = tokenManager == null ? void 0 : tokenManager.getAccessToken) == null ? void 0 : _a.call(tokenManager);
    if (!accessToken) {
      return headers;
    }
    return {
      ...headers != null ? headers : {},
      [_HttpClient.ACCESS_TOKEN_HEADER]: accessToken
    };
  }
  unwrapSdkworkV3Payload(payload) {
    if (!_HttpClient.SDKWORK_V3_UNWRAP || payload == null || typeof payload !== "object") {
      return payload;
    }
    const record = payload;
    if (record.code !== 0 || !("data" in record)) {
      return payload;
    }
    const data = record.data;
    if (!data || typeof data !== "object") {
      return data;
    }
    const envelopeData = data;
    if ("items" in envelopeData && "pageInfo" in envelopeData) {
      return data;
    }
    if ("accepted" in envelopeData) {
      return data;
    }
    if ("item" in envelopeData) {
      return envelopeData.item;
    }
    return data;
  }
  async request(path, options = {}) {
    const execute = this.execute;
    if (typeof execute !== "function") {
      throw new Error("BaseHttpClient execute method is not available");
    }
    const { body, headers, contentType, method = "GET", skipAuth, ...rest } = options;
    const requestHeaders = skipAuth ? headers : this.applySdkworkAuthHeaders(headers);
    const payload = await withRetry(
      () => execute.call(this, {
        url: path,
        method,
        ...rest,
        skipAuth,
        body: this.buildRequestBody(body, contentType),
        headers: this.buildRequestHeaders(requestHeaders, body == null ? void 0 : contentType)
      }),
      { maxRetries: 3 }
    );
    return this.unwrapSdkworkV3Payload(payload);
  }
  async *streamJson(path, options = {}) {
    const stream = BaseHttpClient.prototype.stream;
    if (typeof stream !== "function") {
      throw new Error("BaseHttpClient stream method is not available");
    }
    const { body, headers, contentType, method = "GET", skipAuth, ...rest } = options;
    const authHeaders = skipAuth ? headers : this.applySdkworkAuthHeaders(headers);
    const requestHeaders = this.buildRequestHeaders(
      { Accept: "text/event-stream", ...authHeaders != null ? authHeaders : {} },
      body == null ? void 0 : contentType
    );
    for await (const data of stream.call(this, path, {
      method,
      ...rest,
      skipAuth,
      body: this.buildRequestBody(body, contentType),
      headers: requestHeaders
    })) {
      if (data === "[DONE]") {
        return;
      }
      if (typeof data !== "string" || data.trim().length === 0) {
        continue;
      }
      yield JSON.parse(data);
    }
  }
  async get(path, params, headers) {
    return this.request(path, { method: "GET", params, headers });
  }
  async post(path, body, params, headers, contentType) {
    return this.request(path, { method: "POST", body, params, headers, contentType });
  }
  async put(path, body, params, headers, contentType) {
    return this.request(path, { method: "PUT", body, params, headers, contentType });
  }
  async delete(path, params, headers) {
    return this.request(path, { method: "DELETE", params, headers });
  }
  async patch(path, body, params, headers, contentType) {
    return this.request(path, { method: "PATCH", body, params, headers, contentType });
  }
};
_HttpClient.ACCESS_TOKEN_HEADER = "Access-Token";
_HttpClient.SDKWORK_V3_UNWRAP = true;
var HttpClient = _HttpClient;
function createHttpClient(config) {
  return new HttpClient(config);
}

// ../../sdks/sdkwork-agents-app-sdk/sdkwork-agents-app-sdk-typescript/generated/server-openapi/src/api/paths.ts
var APP_API_PREFIX = "/app/v3/api";
function appApiPath(path) {
  if (!path) {
    return APP_API_PREFIX;
  }
  if (/^https?:\/\//i.test(path)) {
    return path;
  }
  const normalizedPrefixRaw = (APP_API_PREFIX || "").trim();
  const normalizedPrefix = normalizedPrefixRaw ? `/${normalizedPrefixRaw.replace(/^\/+|\/+$/g, "")}` : "";
  const normalizedPath = path.startsWith("/") ? path : `/${path}`;
  if (!normalizedPrefix || normalizedPrefix === "/") {
    return normalizedPath;
  }
  if (normalizedPath === normalizedPrefix || normalizedPath.startsWith(`${normalizedPrefix}/`)) {
    return normalizedPath;
  }
  return `${normalizedPrefix}${normalizedPath}`;
}

// ../../sdks/sdkwork-agents-app-sdk/sdkwork-agents-app-sdk-typescript/generated/server-openapi/src/api/ai.ts
var AiAgentsMcpServersApi = class {
  constructor(client) {
    this.client = client;
  }
  /** List MCP marketplace entries from agent composition slots */
  async list() {
    return this.client.get(appApiPath(`/ai/mcp_servers`));
  }
};
var AiAgentsCodeEnginesApi = class {
  constructor(client) {
    this.client = client;
  }
  /** List canonical code-engine catalog */
  async list() {
    return this.client.get(appApiPath(`/ai/code_engines`));
  }
};
var AiAgentsCompositionSlotsApi = class {
  constructor(client) {
    this.client = client;
  }
  /** List composition slots for one managed agent */
  async list(agentId) {
    return this.client.get(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/composition_slots`));
  }
  /** Create a composition slot for one managed agent */
  async create(agentId, body) {
    return this.client.post(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/composition_slots`), body, void 0, void 0, "application/json");
  }
  /** Retrieve one managed agent composition slot */
  async retrieve(agentId, slotId) {
    return this.client.get(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/composition_slots/${serializePathParameter(slotId, { name: "slotId", style: "simple", explode: false })}`));
  }
  /** Update one managed agent composition slot */
  async update(agentId, slotId, body) {
    return this.client.patch(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/composition_slots/${serializePathParameter(slotId, { name: "slotId", style: "simple", explode: false })}`), body, void 0, void 0, "application/json");
  }
  /** Delete one managed agent composition slot */
  async delete(agentId, slotId, params) {
    const query = buildQueryString([
      { name: "expected_version", value: params.expectedVersion, style: "form", explode: true, allowReserved: false },
      { name: "requested_at", value: params.requestedAt, style: "form", explode: true, allowReserved: false }
    ]);
    return this.client.delete(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/composition_slots/${serializePathParameter(slotId, { name: "slotId", style: "simple", explode: false })}`), query));
  }
};
var AiAgentsMessagesApi = class {
  constructor(client) {
    this.client = client;
  }
  /** List messages in one chat session */
  async list(agentId, sessionId, params) {
    const query = buildQueryString([
      { name: "page", value: params == null ? void 0 : params.page, style: "form", explode: true, allowReserved: false },
      { name: "page_size", value: params == null ? void 0 : params.pageSize, style: "form", explode: true, allowReserved: false }
    ]);
    return this.client.get(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/sessions/${serializePathParameter(sessionId, { name: "sessionId", style: "simple", explode: false })}/messages`), query));
  }
  /** Send a user chat message and receive an assistant reply */
  async create(agentId, sessionId, body, params) {
    const query = buildQueryString([
      { name: "stream", value: params == null ? void 0 : params.stream, style: "form", explode: true, allowReserved: false }
    ]);
    return this.client.streamJson(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/sessions/${serializePathParameter(sessionId, { name: "sessionId", style: "simple", explode: false })}/messages`), query), { method: "POST", body, contentType: "application/json" });
  }
  /** Retrieve one chat message */
  async retrieve(agentId, sessionId, messageId) {
    return this.client.get(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/sessions/${serializePathParameter(sessionId, { name: "sessionId", style: "simple", explode: false })}/messages/${serializePathParameter(messageId, { name: "messageId", style: "simple", explode: false })}`));
  }
};
var AiAgentsSessionsApi = class {
  constructor(client) {
    this.client = client;
  }
  /** List chat sessions for one managed agent */
  async list(agentId, params) {
    const query = buildQueryString([
      { name: "page", value: params == null ? void 0 : params.page, style: "form", explode: true, allowReserved: false },
      { name: "page_size", value: params == null ? void 0 : params.pageSize, style: "form", explode: true, allowReserved: false }
    ]);
    return this.client.get(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/sessions`), query));
  }
  /** Create a chat session for one managed agent */
  async create(agentId, body) {
    return this.client.post(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/sessions`), body, void 0, void 0, "application/json");
  }
  /** Retrieve one chat session */
  async retrieve(agentId, sessionId) {
    return this.client.get(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/sessions/${serializePathParameter(sessionId, { name: "sessionId", style: "simple", explode: false })}`));
  }
  /** Close one chat session */
  async close(agentId, sessionId, body) {
    return this.client.post(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/sessions/${serializePathParameter(sessionId, { name: "sessionId", style: "simple", explode: false })}/close`), body, void 0, void 0, "application/json");
  }
};
var AiAgentsPromptOptimizationsApi = class {
  constructor(client) {
    this.client = client;
  }
  /** Create a prompt optimization for one managed agent */
  async create(agentId, body) {
    return this.client.post(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/prompt_optimizations`), body, void 0, void 0, "application/json");
  }
};
var AiAgentsPreviewResponsesApi = class {
  constructor(client) {
    this.client = client;
  }
  /** Create a preview response for one managed agent */
  async create(agentId, body) {
    return this.client.post(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/preview_responses`), body, void 0, void 0, "application/json");
  }
};
var AiAgentsProviderBindingsApi = class {
  constructor(client) {
    this.client = client;
  }
  /** List provider bindings for one managed agent */
  async list(agentId, params) {
    const query = buildQueryString([
      { name: "page", value: params == null ? void 0 : params.page, style: "form", explode: true, allowReserved: false },
      { name: "page_size", value: params == null ? void 0 : params.pageSize, style: "form", explode: true, allowReserved: false }
    ]);
    return this.client.get(appendQueryString(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/provider_bindings`), query));
  }
  /** Create a provider binding for one managed agent */
  async create(agentId, body) {
    return this.client.post(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/provider_bindings`), body, void 0, void 0, "application/json");
  }
  /** Activate one managed agent provider binding */
  async activate(agentId, bindingId, body) {
    return this.client.post(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/provider_bindings/${serializePathParameter(bindingId, { name: "bindingId", style: "simple", explode: false })}/activate`), body, void 0, void 0, "application/json");
  }
};
var AiAgentsApi = class {
  constructor(client) {
    this.client = client;
    this.providerBindings = new AiAgentsProviderBindingsApi(client);
    this.previewResponses = new AiAgentsPreviewResponsesApi(client);
    this.promptOptimizations = new AiAgentsPromptOptimizationsApi(client);
    this.sessions = new AiAgentsSessionsApi(client);
    this.messages = new AiAgentsMessagesApi(client);
    this.compositionSlots = new AiAgentsCompositionSlotsApi(client);
    this.codeEngines = new AiAgentsCodeEnginesApi(client);
    this.mcpServers = new AiAgentsMcpServersApi(client);
  }
  /** List managed agents */
  async list(params) {
    const query = buildQueryString([
      { name: "include_deleted", value: params == null ? void 0 : params.includeDeleted, style: "form", explode: true, allowReserved: false },
      { name: "page", value: params == null ? void 0 : params.page, style: "form", explode: true, allowReserved: false },
      { name: "page_size", value: params == null ? void 0 : params.pageSize, style: "form", explode: true, allowReserved: false },
      { name: "q", value: params == null ? void 0 : params.q, style: "form", explode: true, allowReserved: false }
    ]);
    return this.client.get(appendQueryString(appApiPath(`/ai/agents`), query));
  }
  /** Create a managed agent */
  async create(body) {
    return this.client.post(appApiPath(`/ai/agents`), body, void 0, void 0, "application/json");
  }
  /** Retrieve one managed agent */
  async retrieve(agentId) {
    return this.client.get(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}`));
  }
  /** Update one managed agent */
  async update(agentId, body) {
    return this.client.patch(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}`), body, void 0, void 0, "application/json");
  }
  /** Soft-delete one managed agent */
  async delete(agentId) {
    return this.client.delete(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}`));
  }
  /** Restore one soft-deleted managed agent */
  async restore(agentId, body) {
    return this.client.post(appApiPath(`/ai/agents/${serializePathParameter(agentId, { name: "agentId", style: "simple", explode: false })}/restore`), body, void 0, void 0, "application/json");
  }
};
var AiApi = class {
  constructor(client) {
    this.client = client;
    this.agents = new AiAgentsApi(client);
  }
};
function createAiApi(client) {
  return new AiApi(client);
}
function appendQueryString(path, rawQueryString) {
  const query = rawQueryString.replace(/^\?+/, "");
  if (!query) {
    return path;
  }
  return path.includes("?") ? `${path}&${query}` : `${path}?${query}`;
}
function serializePathParameter(value, spec) {
  if (value === void 0 || value === null) {
    return "";
  }
  const style = spec.style || "simple";
  if (Array.isArray(value)) {
    return serializePathArray(spec.name, value, style, spec.explode);
  }
  if (typeof value === "object") {
    return serializePathObject(spec.name, value, style, spec.explode);
  }
  return pathPrefix(spec.name, style, false) + encodePathValue(serializePathPrimitive(value));
}
function serializePathArray(name, values, style, explode) {
  const serialized = values.filter((item) => item !== void 0 && item !== null).map((item) => encodePathValue(serializePathPrimitive(item)));
  if (serialized.length === 0) {
    return pathPrefix(name, style, false);
  }
  if (style === "matrix") {
    return explode ? serialized.map((item) => `;${name}=${item}`).join("") : `;${name}=${serialized.join(",")}`;
  }
  return pathPrefix(name, style, false) + serialized.join(explode ? "." : ",");
}
function serializePathObject(name, value, style, explode) {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== void 0 && entryValue !== null);
  if (entries.length === 0) {
    return pathPrefix(name, style, true);
  }
  if (style === "matrix") {
    return explode ? entries.map(([key, entryValue]) => `;${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join("") : `;${name}=${entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(",")}`;
  }
  const serialized = explode ? entries.map(([key, entryValue]) => `${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join(style === "label" ? "." : ",") : entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(",");
  return pathPrefix(name, style, true) + serialized;
}
function pathPrefix(name, style, _objectValue) {
  if (style === "label") return ".";
  if (style === "matrix") return `;${name}`;
  return "";
}
function encodePathValue(value) {
  return encodeURIComponent(value);
}
function serializePathPrimitive(value) {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === "object") {
    return JSON.stringify(value);
  }
  return String(value);
}
function buildQueryString(parameters) {
  const pairs = [];
  for (const parameter of parameters) {
    appendSerializedParameter(pairs, parameter);
  }
  return pairs.join("&");
}
function appendSerializedParameter(pairs, parameter) {
  if (parameter.value === void 0 || parameter.value === null) {
    return;
  }
  if (parameter.contentType) {
    pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(JSON.stringify(parameter.value), parameter.allowReserved)}`);
    return;
  }
  const style = parameter.style || "form";
  if (style === "deepObject") {
    appendDeepObjectParameter(pairs, parameter.name, parameter.value, parameter.allowReserved);
    return;
  }
  if (Array.isArray(parameter.value)) {
    appendArrayParameter(pairs, parameter.name, parameter.value, style, parameter.explode, parameter.allowReserved);
    return;
  }
  if (typeof parameter.value === "object") {
    appendObjectParameter(pairs, parameter.name, parameter.value, style, parameter.explode, parameter.allowReserved);
    return;
  }
  pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(serializePrimitive(parameter.value), parameter.allowReserved)}`);
}
function appendArrayParameter(pairs, name, value, style, explode, allowReserved) {
  const values = value.filter((item) => item !== void 0 && item !== null).map((item) => serializePrimitive(item));
  if (values.length === 0) {
    return;
  }
  if (style === "form" && explode) {
    for (const item of values) {
      pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(item, allowReserved)}`);
    }
    return;
  }
  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(values.join(","), allowReserved)}`);
}
function appendObjectParameter(pairs, name, value, style, explode, allowReserved) {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== void 0 && entryValue !== null);
  if (entries.length === 0) {
    return;
  }
  if (style === "form" && explode) {
    for (const [key, entryValue] of entries) {
      pairs.push(`${encodeQueryComponent(key)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
    }
    return;
  }
  const serialized = entries.flatMap(([key, entryValue]) => [key, serializePrimitive(entryValue)]).join(",");
  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serialized, allowReserved)}`);
}
function appendDeepObjectParameter(pairs, name, value, allowReserved) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serializePrimitive(value), allowReserved)}`);
    return;
  }
  for (const [key, entryValue] of Object.entries(value)) {
    if (entryValue === void 0 || entryValue === null) {
      continue;
    }
    pairs.push(`${encodeQueryComponent(`${name}[${key}]`)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
  }
}
function serializePrimitive(value) {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === "object") {
    return JSON.stringify(value);
  }
  return String(value);
}
function encodeQueryComponent(value) {
  return encodeURIComponent(value);
}
function encodeQueryValue(value, allowReserved) {
  const encoded = encodeURIComponent(value);
  if (!allowReserved) {
    return encoded;
  }
  return encoded.replace(/%3A/gi, ":").replace(/%2F/gi, "/").replace(/%3F/gi, "?").replace(/%23/gi, "#").replace(/%5B/gi, "[").replace(/%5D/gi, "]").replace(/%40/gi, "@").replace(/%21/gi, "!").replace(/%24/gi, "$").replace(/%26/gi, "&").replace(/%27/gi, "'").replace(/%28/gi, "(").replace(/%29/gi, ")").replace(/%2A/gi, "*").replace(/%2B/gi, "+").replace(/%2C/gi, ",").replace(/%3B/gi, ";").replace(/%3D/gi, "=");
}

// ../../sdks/sdkwork-agents-app-sdk/sdkwork-agents-app-sdk-typescript/generated/server-openapi/src/sdk.ts
var SdkworkAppClient = class {
  constructor(config) {
    this.httpClient = createHttpClient(config);
    this.ai = createAiApi(this.httpClient);
  }
  setAuthToken(token) {
    this.httpClient.setAuthToken(token);
    return this;
  }
  setAccessToken(token) {
    this.httpClient.setAccessToken(token);
    return this;
  }
  setTokenManager(manager) {
    this.httpClient.setTokenManager(manager);
    return this;
  }
  get http() {
    return this.httpClient;
  }
};
function createClient(config) {
  return new SdkworkAppClient(config);
}

// packages/sdkwork-agents-mp-core/src/session/session.ts
var SDKWORK_AGENTS_MP_SESSION_KEY = "sdkwork-agents-mp:session:v1";
function readWxStorage(key) {
  var _a;
  try {
    const wxApi = globalThis.wx;
    const value = (_a = wxApi == null ? void 0 : wxApi.getStorageSync) == null ? void 0 : _a.call(wxApi, key);
    if (typeof value === "string" && value.trim().length > 0) {
      return value.trim();
    }
  } catch {
    return null;
  }
  return null;
}
function readAppSdkSessionTokens() {
  const raw = readWxStorage(SDKWORK_AGENTS_MP_SESSION_KEY);
  if (!raw) {
    return null;
  }
  try {
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") {
      return null;
    }
    return parsed;
  } catch {
    return null;
  }
}
function resolveAppSdkAccessToken(session) {
  var _a;
  const token = (_a = session == null ? void 0 : session.accessToken) == null ? void 0 : _a.trim();
  return token && token.length > 0 ? token : void 0;
}
function resolveAppSdkAuthToken(session) {
  var _a;
  const token = (_a = session == null ? void 0 : session.authToken) == null ? void 0 : _a.trim();
  return token && token.length > 0 ? token : void 0;
}

// packages/sdkwork-agents-mp-core/src/sdk/agentsAppSdkClient.ts
var agentsAppSdkClient = null;
var configuredBaseUrl = null;
var DEFAULT_MP_APP_API_BASE_URL = "http://127.0.0.1:8095/app/v3/api";
function configureAgentsAppSdkBaseUrl(baseUrl) {
  const normalized = baseUrl.trim().replace(/\/+$/u, "");
  configuredBaseUrl = normalized.length > 0 ? normalized : DEFAULT_MP_APP_API_BASE_URL;
}
function resolveAgentsAppSdkBaseUrl() {
  if (configuredBaseUrl) {
    return configuredBaseUrl;
  }
  return DEFAULT_MP_APP_API_BASE_URL;
}
function createAgentsAppSdkClientConfig(session) {
  const currentSession = session != null ? session : readAppSdkSessionTokens();
  return {
    baseUrl: resolveAgentsAppSdkBaseUrl(),
    accessToken: resolveAppSdkAccessToken(currentSession),
    authToken: resolveAppSdkAuthToken(currentSession),
    platform: "mini-program"
  };
}
function initAgentsAppSdkClient(config = createAgentsAppSdkClientConfig()) {
  agentsAppSdkClient = createClient(config);
  return agentsAppSdkClient;
}

// src/bootstrap/sdkClients.ts
function bootstrapSdkClients(options = {}) {
  if (options.appApiBaseUrl) {
    configureAgentsAppSdkBaseUrl(options.appApiBaseUrl);
  }
  const client = initAgentsAppSdkClient();
  return { agentsAppSdk: client };
}

// src/bootstrap/runtime.ts
function bootstrap(options = {}) {
  const sdk = bootstrapSdkClients(options);
  return { ready: true, ...sdk };
}

// src/bootstrap/runtimeBundle.ts
function bootstrapAgentsMiniProgram(options = {}) {
  return bootstrap(options);
}
