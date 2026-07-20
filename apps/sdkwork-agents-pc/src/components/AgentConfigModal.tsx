import React from 'react';
import { X, Save } from 'lucide-react';
import { useAgentState } from '../contexts/AgentStateContext';

interface AgentConfigModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const AgentConfigModal: React.FC<AgentConfigModalProps> = ({ isOpen, onClose }) => {
  const { 
    agentName, setAgentName, 
    systemPrompt, setSystemPrompt,
    activeModel, setActiveModel 
  } = useAgentState();

  const [localName, setLocalName] = React.useState(agentName);
  const [localPrompt, setLocalPrompt] = React.useState(systemPrompt);
  const [localModel, setLocalModel] = React.useState(activeModel);

  React.useEffect(() => {
    if (isOpen) {
      setLocalName(agentName);
      setLocalPrompt(systemPrompt);
      setLocalModel(activeModel);
    }
  }, [isOpen, agentName, systemPrompt, activeModel]);

  if (!isOpen) return null;

  const handleSave = () => {
    setAgentName(localName);
    setSystemPrompt(localPrompt);
    setActiveModel(localModel);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-md bg-white dark:bg-[#1C1C1E] border border-gray-200 dark:border-white/10 rounded-2xl shadow-xl overflow-hidden animate-in fade-in zoom-in-95 duration-200">
        <div className="flex justify-between items-center px-6 py-4 border-b border-gray-200 dark:border-white/10">
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">Agent Configuration</h2>
          <button 
            onClick={onClose}
            className="p-1.5 rounded-md hover:bg-gray-100 dark:hover:bg-white/10 text-gray-500 transition-colors"
          >
            <X size={18} />
          </button>
        </div>

        <div className="p-6 space-y-5">
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Display Name
            </label>
            <input
              type="text"
              value={localName}
              onChange={(e) => setLocalName(e.target.value)}
              className="w-full px-3 py-2 bg-gray-50 dark:bg-[#2C2C2E] border border-gray-200 dark:border-white/10 rounded-lg text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="e.g. GPT-4o Assistant"
            />
          </div>

          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Active Model
            </label>
            <select
              value={localModel}
              onChange={(e) => setLocalModel(e.target.value)}
              className="w-full px-3 py-2 bg-gray-50 dark:bg-[#2C2C2E] border border-gray-200 dark:border-white/10 rounded-lg text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="gpt-4o">GPT-4o</option>
              <option value="gpt-4-turbo">GPT-4 Turbo</option>
              <option value="claude-3-opus">Claude 3 Opus</option>
              <option value="claude-3-sonnet">Claude 3 Sonnet</option>
              <option value="gemini-1.5-pro">Gemini 1.5 Pro</option>
            </select>
          </div>

          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              System Prompt
            </label>
            <textarea
              value={localPrompt}
              onChange={(e) => setLocalPrompt(e.target.value)}
              rows={4}
              className="w-full px-3 py-2 bg-gray-50 dark:bg-[#2C2C2E] border border-gray-200 dark:border-white/10 rounded-lg text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
              placeholder="Enter system instructions..."
            />
          </div>
        </div>

        <div className="flex justify-end gap-3 px-6 py-4 bg-gray-50 dark:bg-[#18181A] border-t border-gray-200 dark:border-white/10">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-white/10 rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors shadow-sm"
          >
            <Save size={16} />
            Save Changes
          </button>
        </div>
      </div>
    </div>
  );
};
