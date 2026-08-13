import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { getAgentsAppSdkClientWithSession } from '@sdkwork/agents-pc-core/sdk/agentsAppSdkClient';

export interface AppliedCustomProvider {
  vendorCode: string;
  vendorName: string;
  modelId: string;
}

interface CustomProviderDialogProps {
  open: boolean;
  onClose: () => void;
  onApplied: (provider: AppliedCustomProvider) => void;
}

/**
 * Playground custom LLM provider configuration.
 *
 * Applies a client-provided OpenAI-compatible provider (base URL + API key +
 * model) to the playground chat agent through
 * `POST /app/v3/api/ai/model_configurations/apply` (engineId=rig,
 * agentId=agent.chat.default). The applied model then joins the chat model
 * picker and every turn is executed by the RIG agent engine through the
 * custom provider backend.
 */
export const CustomProviderDialog: React.FC<CustomProviderDialogProps> = ({
  open,
  onClose,
  onApplied,
}) => {
  const { t } = useTranslation('chat');
  const [vendorCode, setVendorCode] = useState('openai');
  const [vendorName, setVendorName] = useState('OpenAI');
  const [baseUrl, setBaseUrl] = useState('');
  const [apiKey, setApiKey] = useState('');
  const [modelId, setModelId] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!open) {
    return null;
  }

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    const trimmedVendor = vendorCode.trim();
    const trimmedUrl = baseUrl.trim();
    const trimmedKey = apiKey.trim();
    const trimmedModel = modelId.trim();
    if (!trimmedVendor || !trimmedUrl || !trimmedKey || !trimmedModel) {
      setError(t('customProvider.requiredFields'));
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const client = getAgentsAppSdkClientWithSession();
      // engineId=rig selects the RIG agent engine provider; agentId binds the
      // configuration to the playground chat agent scope (per-agent host).
      await client.ai.agents.modelConfigurations.apply({
        configurationId: 'playground.custom-provider',
        engineId: 'rig',
        agentId: 'agent.chat.default',
        vendorCode: trimmedVendor,
        baseUrl: trimmedUrl,
        apiKey: trimmedKey,
        defaultModelId: trimmedModel,
        supportedModelIds: [trimmedModel],
      });
      onApplied({
        vendorCode: trimmedVendor,
        vendorName: vendorName.trim() || trimmedVendor,
        modelId: trimmedModel,
      });
      // Keep the dialog open so the user can adjust credentials; the model
      // picker already switched to the custom model.
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      setError(`${t('customProvider.applyFailed')} ${message}`);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
    >
      <div
        className="w-[460px] max-w-[92vw] rounded-xl bg-white dark:bg-[#1e1e1e] p-5 shadow-xl"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-base font-semibold text-gray-800 dark:text-gray-100">
            {t('customProvider.title')}
          </h2>
          <button
            type="button"
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
            aria-label={t('customProvider.close')}
          >
            ✕
          </button>
        </div>
        <form onSubmit={handleSubmit} className="space-y-3">
          <div className="grid grid-cols-2 gap-3">
            <label className="block">
              <span className="text-xs text-gray-500 dark:text-gray-400">
                {t('customProvider.vendorCode')}
              </span>
              <input
                className="mt-1 w-full rounded-md border border-gray-300 dark:border-[#333] bg-white dark:bg-[#262626] px-2 py-1.5 text-sm text-gray-800 dark:text-gray-100"
                value={vendorCode}
                onChange={(event) => setVendorCode(event.target.value)}
                placeholder="openai"
                maxLength={128}
              />
            </label>
            <label className="block">
              <span className="text-xs text-gray-500 dark:text-gray-400">
                {t('customProvider.vendorName')}
              </span>
              <input
                className="mt-1 w-full rounded-md border border-gray-300 dark:border-[#333] bg-white dark:bg-[#262626] px-2 py-1.5 text-sm text-gray-800 dark:text-gray-100"
                value={vendorName}
                onChange={(event) => setVendorName(event.target.value)}
                placeholder="OpenAI"
                maxLength={128}
              />
            </label>
          </div>
          <label className="block">
            <span className="text-xs text-gray-500 dark:text-gray-400">
              {t('customProvider.baseUrl')}
            </span>
            <input
              className="mt-1 w-full rounded-md border border-gray-300 dark:border-[#333] bg-white dark:bg-[#262626] px-2 py-1.5 text-sm text-gray-800 dark:text-gray-100"
              value={baseUrl}
              onChange={(event) => setBaseUrl(event.target.value)}
              placeholder="https://api.openai.com/v1"
              maxLength={2048}
            />
          </label>
          <label className="block">
            <span className="text-xs text-gray-500 dark:text-gray-400">
              {t('customProvider.apiKey')}
            </span>
            <input
              type="password"
              className="mt-1 w-full rounded-md border border-gray-300 dark:border-[#333] bg-white dark:bg-[#262626] px-2 py-1.5 text-sm text-gray-800 dark:text-gray-100"
              value={apiKey}
              onChange={(event) => setApiKey(event.target.value)}
              maxLength={16384}
            />
          </label>
          <label className="block">
            <span className="text-xs text-gray-500 dark:text-gray-400">
              {t('customProvider.model')}
            </span>
            <input
              className="mt-1 w-full rounded-md border border-gray-300 dark:border-[#333] bg-white dark:bg-[#262626] px-2 py-1.5 text-sm text-gray-800 dark:text-gray-100"
              value={modelId}
              onChange={(event) => setModelId(event.target.value)}
              placeholder="gpt-4o-mini"
              maxLength={256}
            />
          </label>
          {error && (
            <p className="text-xs text-red-500 break-words" role="alert">
              {error}
            </p>
          )}
          <div className="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="rounded-md px-3 py-1.5 text-sm text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-[#2f2f2f]"
            >
              {t('customProvider.cancel')}
            </button>
            <button
              type="submit"
              disabled={submitting}
              className="rounded-md bg-blue-600 px-3 py-1.5 text-sm text-white hover:bg-blue-700 disabled:opacity-50"
            >
              {submitting ? t('customProvider.applying') : t('customProvider.apply')}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
