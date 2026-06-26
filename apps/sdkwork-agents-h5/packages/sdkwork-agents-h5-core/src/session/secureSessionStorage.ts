export function isDesktopSecureSessionStorageEnabled(): boolean {
  return false;
}

export async function readDesktopSecureSessionRawValue(): Promise<string | null> {
  return null;
}

export async function writeDesktopSecureSessionRawValue(_value: string): Promise<void> {
  return;
}

export async function clearDesktopSecureSessionRawValue(): Promise<void> {
  return;
}
