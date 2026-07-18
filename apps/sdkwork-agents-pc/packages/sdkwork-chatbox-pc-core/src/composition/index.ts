import { CoreHostAdapter } from '../host';
import { CHATBOX_PC_MODULES } from '../modules';

export interface ChatboxPcComposition {
  host: typeof CoreHostAdapter;
  modules: typeof CHATBOX_PC_MODULES;
}

export function createChatboxPcComposition(
  host: typeof CoreHostAdapter = CoreHostAdapter,
): ChatboxPcComposition {
  return {
    host,
    modules: CHATBOX_PC_MODULES,
  };
}
