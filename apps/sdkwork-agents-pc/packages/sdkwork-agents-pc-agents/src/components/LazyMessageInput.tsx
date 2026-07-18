import { lazy, Suspense, type FC } from 'react';

import type { MessageInputProps } from './MessageInput';

const MessageInput = lazy(() =>
  import('./MessageInput').then((module) => ({ default: module.MessageInput })),
);

export const LazyMessageInput: FC<MessageInputProps> = (props) => (
  <Suspense fallback={<div className="h-20 animate-pulse rounded-xl bg-white/5" />}>
    <MessageInput {...props} />
  </Suspense>
);
