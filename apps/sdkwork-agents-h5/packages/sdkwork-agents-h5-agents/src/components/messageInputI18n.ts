const MESSAGE_INPUT_COPY: Record<string, string> = {
  "chat.messageInput.defaultPlaceholder": "发送消息...",
  "chat.messageInput.agentTypingPlaceholder": "智能体正在回复...",
  "chat.messageInput.replyPrefix": "回复 {{name}}:",
  "chat.messageInput.dropToSend": "松开鼠标发送文件",
  "chat.messageInput.actions.sendFile": "发送文件",
  "chat.messageInput.actions.screenshot": "截图 (Alt+A)",
  "chat.messageInput.actions.emoji": "表情",
  "chat.messageInput.actions.history": "聊天记录",
  "chat.messageInput.actions.stopRecording": "停止录音并发送",
  "chat.messageInput.actions.recordVoice": "录制语音消息",
  "chat.messageInput.actions.stopGenerating": "停止生成",
  "chat.messageInput.actions.send": "发送 (Enter)",
  "chat.messageInput.toast.stickerNeedsFile": "表情图片需要本地文件或 Drive 资源后才能发送",
  "chat.messageInput.toast.voiceTooShort": "说话时间太短",
  "chat.messageInput.toast.voiceGenerationFailed": "语音文件生成失败，请重试",
  "chat.messageInput.toast.microphoneDenied": "无法访问麦克风，请检查权限后重试",
  "chat.messageInput.toast.screenshotUnsupported": "当前浏览器不支持网页截图",
  "chat.messageInput.toast.screenshotDenied": "无权限进行截图（或在新标签页中打开应用重试）",
  "chat.messageInput.toast.screenshotCancelled": "取消截图",
};

export function translateMessageInput(
  key: string,
  params?: Record<string, string>,
): string {
  const template = MESSAGE_INPUT_COPY[key] ?? key;
  if (!params) {
    return template;
  }
  return Object.entries(params).reduce(
    (value, [name, replacement]) => value.replaceAll(`{{${name}}}`, replacement),
    template,
  );
}
