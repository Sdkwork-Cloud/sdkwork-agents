import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const root = path.resolve(import.meta.dirname, '../..');
const contract = JSON.parse(fs.readFileSync(
  path.join(root, 'specs/agent-execution-placement-orchestration.contract.json'),
  'utf8',
));

test('hybrid execution orchestration remains a draft non-implementation contract', () => {
  assert.equal(contract.status, 'draft');
  assert.equal(contract.implementationAuthorized, false);
  assert.equal(contract['x-sdkwork-no-public-api-change'], true);
  assert.equal(contract['x-sdkwork-no-sdk-generation'], true);
  assert.equal(contract['x-sdkwork-no-database-migration'], true);
  assert.equal(contract['x-sdkwork-no-kernel-port-implementation'], true);
});

test('execution target is explicit and orthogonal to host and topology', () => {
  assert.deepEqual(contract.executionTargetCandidate.values, ['LOCAL', 'CLOUD']);
  assert.equal(contract.executionTargetCandidate.sessionCreateRequired, true);
  assert.equal(contract.executionTargetCandidate.taskOverrideRequired, false);
  assert.equal(contract.executionTargetCandidate.taskWithoutOverrideInheritsSession, true);
  assert.equal(contract.executionTargetCandidate.hostModeInferenceAllowed, false);
  assert.equal(contract.executionTargetCandidate.deploymentProfileInferenceAllowed, false);
  assert.equal(contract.executionTargetCandidate.coordinationModeInferenceAllowed, false);
});

test('Task execution requires one persisted canonical Session', () => {
  assert.equal(contract.canonicalTaskRule.persistedSessionReferenceRequired, true);
  assert.equal(contract.canonicalTaskRule.sessionStubAllowed, false);
  assert.equal(
    contract.canonicalTaskRule.workspaceProjectAndOwnerResolvedThroughSession,
    true,
  );
});

test('client intent cannot select physical placement or lease authority', () => {
  const allowed = new Set(contract.executionPlacementIntentCandidate.allowedFields);
  const forbidden = new Set(contract.executionPlacementIntentCandidate.forbiddenClientFields);
  for (const field of [
    'nodeId',
    'nodePoolId',
    'sandboxId',
    'hostPath',
    'volumeId',
    'deviceId',
    'leaseToken',
    'fencingToken',
  ]) {
    assert.equal(allowed.has(field), false);
    assert.equal(forbidden.has(field), true);
  }
});

test('execution placement and provider continuity are separate bindings', () => {
  const separation = contract.bindingSeparationCandidate;
  assert.equal(
    separation.executionPlacementBinding.typeName,
    'AgentExecutionPlacementBinding',
  );
  assert.equal(
    separation.providerSessionBinding.typeName,
    'AgentProviderSessionBinding',
  );
  assert.equal(separation.executionPlacementBinding.clientWritable, false);
  assert.equal(separation.combinedRuntimeBindingMayGainPhysicalFields, false);
  assert.equal(
    separation.currentClientCreatedRuntimeBinding.commercialPlacementEvidence,
    false,
  );
});

test('one Kernel port covers capability, placement, cancellation, restore and release', () => {
  const port = contract.kernelPlacementPortCandidate;
  const operationNames = port.operations.map((operation) => operation.name);
  assert.equal(new Set(operationNames).size, operationNames.length);
  assert.deepEqual(operationNames, [
    'getExecutionTargetCapabilities',
    'reserveExecutionPlacement',
    'renewExecutionPlacement',
    'cancelExecution',
    'restoreExecutionCheckpoint',
    'releaseExecutionPlacement',
  ]);
  assert.equal(port.internalLeaseCredentialMayEnterProductApi, false);
  assert.equal(port.fencingGenerationClientWritable, false);
});

test('Agents never bypasses Kernel for Sandbox lifecycle', () => {
  assert.equal(contract.ownership.agentsMayCallSandboxDirectly, false);
  assert.equal(contract.ownership.productMayCallKernelOrSandboxDirectly, false);
  assert.equal(
    contract.ownership.executionPlacementLeaseRoutingAndFencing,
    'sdkwork-kernel',
  );
  assert.equal(
    contract.ownership.sandboxAdmissionPoolAttachmentAndCleanup,
    'sdkwork-sandbox',
  );
});

test('local profile is durable and does not upload Workspace bytes implicitly', () => {
  assert.equal(contract.localResidencyCandidate.businessStore, 'local-postgresql');
  assert.equal(contract.localResidencyCandidate.birdcoderSqliteMayStoreBusinessFacts, false);
  assert.equal(
    contract.localResidencyCandidate.processMemoryMayBeProductionBusinessAuthority,
    false,
  );
  assert.equal(contract.localResidencyCandidate.implicitWorkspaceUploadAllowed, false);
});

test('commercial concurrency requires distributed admission and transactional evidence', () => {
  const invariants = contract.transactionAndIsolationCandidate;
  assert.equal(invariants.tenantAndOrganizationScopeRequiredOnEveryRepositoryOperation, true);
  assert.equal(invariants.aggregateAuditAndOutboxAtomicityRequired, true);
  assert.equal(invariants.realCancelRestoreAndReconciliationRequired, true);
  assert.equal(
    invariants.processLocalSemaphoreAcceptedAsDistributedAdmissionEvidence,
    false,
  );
});
