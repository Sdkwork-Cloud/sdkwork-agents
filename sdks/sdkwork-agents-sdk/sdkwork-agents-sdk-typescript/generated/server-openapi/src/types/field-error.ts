export interface FieldError {
  /** JSON Pointer or dot-path to the invalid field, for example `/agentId` or `data.item.code`. */
  field: string;
  /** Safe validation message for logs and optional UI display. MUST NOT reveal secrets or enumeration hints. */
  message: string;
  /** Optional field-level validation subcode when ProblemDetail.code is 40001 VALIDATION_ERROR. */
  code?: number;
}
