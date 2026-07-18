export const CoreHostAdapter = {
  // Mock host adapters that would interface with Tauri
  openExternal: (url: string) => {
    console.log('Opening external URL:', url);
  },
  windowControl: (action: 'minimize' | 'maximize' | 'close') => {
    console.log('Window control action:', action);
  }
};
