import { useState, useEffect } from 'react';

export interface ModelSettings {
  apiKey: string;
  baseUrl: string;
  temperature: number;
}

const DEFAULT_SETTINGS: ModelSettings = {
  apiKey: '',
  baseUrl: '',
  temperature: 0.7,
};

export const useModelSettings = (vendor: string) => {
  const settingsKey = `sdkwork-agents-${vendor.toLowerCase()}-settings`;
  
  const [settings, setSettings] = useState<ModelSettings>(() => {
    const saved = localStorage.getItem(settingsKey);
    return saved ? JSON.parse(saved) : DEFAULT_SETTINGS;
  });

  useEffect(() => {
    localStorage.setItem(settingsKey, JSON.stringify(settings));
  }, [settings, settingsKey]);

  return { settings, setSettings };
};

export const getModelSettings = (vendor: string): ModelSettings => {
  const settingsKey = `sdkwork-agents-${vendor.toLowerCase()}-settings`;
  const saved = localStorage.getItem(settingsKey);
  return saved ? JSON.parse(saved) : DEFAULT_SETTINGS;
};
