import React, { useState, useRef, useEffect } from 'react';
import { Smile, Paperclip, Scissors, Clock, Mic, ArrowUp, StopCircle, X, Reply } from 'lucide-react';
import { useEditor, EditorContent, type Editor } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { motion, AnimatePresence } from 'motion/react';
import { toast } from './Toast';
import { cn } from '@sdkwork/agents-pc-commons';
import { EmojiPicker } from './EmojiPicker';
import { translateMessageInput as t } from './messageInputI18n';

export interface MessageInputProps {
  onSend?: (content: string, type?: 'text'|'image'|'file'|'voice'|'video', extraInfo?: any) => void;
  placeholder?: string;
  disabled?: boolean;
  isTyping?: boolean;
  onStop?: () => void;
  defaultHeight?: number;
  resizable?: boolean;
  replyingTo?: {
    id: string;
    senderName: string;
    content: string;
  };
  onCancelReply?: () => void;
  onHistoryClick?: () => void;
}

function resolveFileMessageType(file: File): 'image' | 'file' | 'video' {
  if (file.type.startsWith('image/')) {
    return 'image';
  }
  if (file.type.startsWith('video/')) {
    return 'video';
  }
  return 'file';
}

function formatFileSize(size: number): string {
  return size > 1024 * 1024
    ? `${(size / (1024 * 1024)).toFixed(1)} MB`
    : `${(size / 1024).toFixed(1)} KB`;
}

function createLocalPreviewUrl(file: Blob): string {
  return URL.createObjectURL(file);
}

function sendFileMessage(
  file: File,
  onSend: NonNullable<MessageInputProps['onSend']>,
  type: 'file' | 'image' | 'video' | 'voice' = resolveFileMessageType(file),
  extraInfo: Record<string, unknown> = {},
): void {
  onSend(createLocalPreviewUrl(file), type, {
    ...extraInfo,
    file,
    fileName: file.name,
    fileSize: formatFileSize(file.size),
    mimeType: file.type,
  });
}

export const MessageInput: React.FC<MessageInputProps> = ({
  onSend,
  placeholder,
  disabled = false,
  isTyping = false,
  onStop,
  defaultHeight = 200,
  resizable = true,
  replyingTo,
  onCancelReply,
  onHistoryClick,
}) => {
  const resolvedPlaceholder = placeholder ?? '输入消息...';
  const agentTypingPlaceholder = '智能体正在输入...';
  const [height, setHeight] = useState(defaultHeight);
  const [showEmojiPicker, setShowEmojiPicker] = useState(false);
  const [activeEmojiTab, setActiveEmojiTab] = useState('emoji');
  const [isEmpty, setIsEmpty] = useState(true);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const isDragging = useRef(false);
  const startY = useRef(0);
  const startHeight = useRef(0);
  const [isRecording, setIsRecording] = useState(false);
  const [voiceDuration, setVoiceDuration] = useState(0);
  const [isDragOver, setIsDragOver] = useState(false);
  const voiceDurationRef = useRef(0);
  const voiceTimerRef = useRef<number | null>(null);
  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const audioChunksRef = useRef<BlobPart[]>([]);

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.dataTransfer.types.includes('Files')) {
      setIsDragOver(true);
    }
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
    
    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      const file = e.dataTransfer.files[0];
      if (onSend) {
        sendFileMessage(file, onSend);
      }
    }
  };



  const handleMouseDown = (e: React.MouseEvent) => {
    if (!resizable) return;
    e.preventDefault();
    isDragging.current = true;
    startY.current = e.clientY;
    startHeight.current = height;
    document.body.style.cursor = 'ns-resize';
  };

  useEffect(() => {
    if (!resizable) return;
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging.current) return;
      const deltaY = startY.current - e.clientY;
      const newHeight = Math.max(120, Math.min(startHeight.current + deltaY, window.innerHeight * 0.8));
      setHeight(newHeight);
    };

    const handleMouseUp = () => {
      isDragging.current = false;
      document.body.style.cursor = 'default';
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [resizable]);

  // Removed outside click here as handled inside EmojiPicker

  const editor = useEditor({
    extensions: [
      StarterKit,
      Placeholder.configure({
        placeholder: isTyping ? agentTypingPlaceholder : resolvedPlaceholder,
        emptyEditorClass: 'is-editor-empty',
      }),
    ],
    content: '',
    editable: !disabled && !isTyping,
    editorProps: {
      attributes: {
        class: 'w-full h-full bg-transparent outline-none text-[15px] text-gray-200 font-sans leading-relaxed',
      },
    },
    onUpdate: ({ editor }: { editor: Editor }) => {
      setIsEmpty(editor.getText().trim().length === 0);
    },
  }, [placeholder, disabled, isTyping, resolvedPlaceholder, agentTypingPlaceholder]);

  const onEmojiClick = React.useCallback((emoji: string) => {
    if (editor && !disabled && !isTyping) {
      editor.commands.insertContent(emoji);
      editor.commands.focus();
    }
  }, [editor, disabled, isTyping]);

  const onStickerClick = React.useCallback((url: string) => {
    void url;
    if (onSend) {
      toast(t('chat.messageInput.toast.stickerNeedsFile'), 'error');
    }
    setShowEmojiPicker(false);
  }, [onSend, t]);

  const handleSend = () => {
    if (!editor || disabled || isTyping) return;
    
    const content = editor.getText().trim();
    if (!content) return;

    if (!onSend) return;
    onSend(content, 'text');
    
    // Clear editor after sending
    editor.commands.clearContent();
    setIsEmpty(true);
    editor.commands.focus();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      e.stopPropagation();
      handleSend();
    }
  };

  const handlePaste = (e: React.ClipboardEvent) => {
    const items = e.clipboardData?.items;
    if (!items) return;

    for (let i = 0; i < items.length; i++) {
        if (items[i].type.indexOf('image') !== -1) {
            const file = items[i].getAsFile();
            if (file && onSend) {
                sendFileMessage(file, onSend, 'image');
                e.preventDefault();
            }
        }
    }
  };

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      const file = e.target.files[0];
      if (onSend) {
        sendFileMessage(file, onSend);
      }
      
      // Reset input so the same file can be selected again if needed
      e.target.value = '';
    }
  };



  const toggleVoiceRecording = async () => {
    if (disabled || isTyping) return;

    if (isRecording) {
      if (mediaRecorderRef.current && mediaRecorderRef.current.state === 'recording') {
        mediaRecorderRef.current.stop();
      } else {
        if (voiceTimerRef.current) {
          clearInterval(voiceTimerRef.current);
          voiceTimerRef.current = null;
        }
        setIsRecording(false);
        const finalDuration = voiceDurationRef.current;
        if (finalDuration >= 1 && onSend) {
          toast(t('chat.messageInput.toast.voiceGenerationFailed'), 'error');
        } else if (finalDuration < 1) {
          toast(t('chat.messageInput.toast.voiceTooShort'), 'error');
        }
      }
      return;
    }

    const startTimer = () => {
      setIsRecording(true);
      setVoiceDuration(0);
      voiceDurationRef.current = 0;
      voiceTimerRef.current = window.setInterval(() => {
        setVoiceDuration(p => {
          voiceDurationRef.current = p + 1;
          return p + 1;
        });
      }, 1000);
    };

    let stream: MediaStream | undefined;
    try {
      stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const mediaRecorder = new MediaRecorder(stream);
      mediaRecorderRef.current = mediaRecorder;
      audioChunksRef.current = [];

      mediaRecorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          audioChunksRef.current.push(event.data);
        }
      };

      mediaRecorder.onstop = async () => {
        const mimeType = mediaRecorderRef.current?.mimeType || 'audio/webm';
        const audioBlob = new Blob(audioChunksRef.current, { type: mimeType });
        
        // Stop all tracks to release microphone
        stream?.getTracks().forEach(track => track.stop());
        
        if (voiceTimerRef.current) {
          clearInterval(voiceTimerRef.current);
          voiceTimerRef.current = null;
        }
        
        setIsRecording(false);
        const finalDuration = voiceDurationRef.current;
        
        // Send actual voice message
        if (finalDuration >= 1 && onSend) {
          const file = new File([audioBlob], `voice-${Date.now()}.webm`, { type: mimeType });
          sendFileMessage(file, onSend, 'voice', { duration: finalDuration, mimeType });
        } else if (finalDuration < 1) {
          toast(t('chat.messageInput.toast.voiceTooShort'), 'error');
        }
      };

      mediaRecorder.start();
      startTimer();
      
    } catch (err) {
      console.error('Error accessing microphone:', err);
      stream?.getTracks().forEach(track => track.stop());
      if (voiceTimerRef.current) {
        clearInterval(voiceTimerRef.current);
        voiceTimerRef.current = null;
      }
      mediaRecorderRef.current = null;
      audioChunksRef.current = [];
      voiceDurationRef.current = 0;
      setVoiceDuration(0);
      setIsRecording(false);
      toast(t('chat.messageInput.toast.microphoneDenied'), 'error');
    }
  };

  return (
    <div 
      className="shrink-0 px-2.5 pb-2.5 pt-3 flex flex-col bg-[#1e1e1e] relative"
      style={{ 
        height: resizable ? `${height}px` : 'auto', 
        minHeight: resizable ? '120px' : `${defaultHeight}px`,
        maxHeight: '50vh'
      }}
    >
      {/* Drag Handle */}
      {resizable && (
        <div 
          className="absolute top-0 left-0 right-0 h-3 cursor-ns-resize hover:bg-white/5 transition-colors z-10 flex items-center justify-center group"
          onMouseDown={handleMouseDown}
        >
          <div className="w-10 h-1 rounded-full bg-white/20 opacity-0 group-hover:opacity-100 transition-opacity" />
        </div>
      )}

      {/* AI Style Input Container */}
      <div className={`bg-[#2b2b2d] rounded-2xl flex flex-col shadow-sm transition-all focus-within:bg-[#2f2f33] h-full relative ${disabled || isTyping ? 'opacity-70' : ''}`}>
        
        {/* Reply Preview */}
        {replyingTo && (
          <div className="flex items-center justify-between px-4 py-2 bg-white/5 border-b border-white/5 rounded-t-2xl shrink-0">
            <div className="flex items-center gap-2 min-w-0">
              <Reply size={14} className="text-gray-400 shrink-0" />
              <span className="text-[12px] text-gray-400 font-medium shrink-0">{t('chat.messageInput.replyPrefix', { name: replyingTo.senderName })}</span>
              <span className="text-[12px] text-gray-500 truncate">{replyingTo.content}</span>
            </div>
            <button 
              onClick={onCancelReply}
              className="text-gray-500 hover:text-gray-300 transition-colors shrink-0 ml-2"
            >
              <X size={14} />
            </button>
          </div>
        )}

        {/* Text Area */}
        <div 
          className={cn("flex-1 overflow-y-auto custom-scrollbar px-4 py-3 relative flex flex-col transition-colors", isDragOver ? "bg-[#3a3a3a]" : "")}
          onKeyDownCapture={handleKeyDown}
          onPaste={handlePaste}
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
        >
          {isDragOver && (
            <div className="absolute inset-0 z-50 flex flex-col items-center justify-center bg-[#1e1e1e]/80 backdrop-blur-sm shadow-inner rounded-lg m-2 border-2 border-dashed border-indigo-500/50">
               <ArrowUp size={32} className="text-indigo-400 mb-2 animate-bounce" />
               <p className="text-gray-200 font-medium">{t('chat.messageInput.dropToSend')}</p>
            </div>
          )}
          <EditorContent editor={editor} className="h-full" />
        </div>
        
        {/* Bottom Actions */}
        <div className="flex items-center justify-between px-3 pb-3 pt-1 shrink-0 relative">
          <div className="flex items-center gap-1">
            {/* Hidden File Input */}
            <input 
              type="file" 
              ref={fileInputRef} 
              className="hidden" 
              multiple 
              onChange={handleFileChange} 
              disabled={disabled || isTyping}
            />
            
            <motion.button 
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.9 }}
              title={t('chat.messageInput.actions.sendFile')}
              onClick={() => fileInputRef.current?.click()}
              disabled={disabled || isTyping}
              className="w-8 h-8 rounded-lg flex items-center justify-center text-gray-400 hover:text-gray-200 hover:bg-white/5 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Paperclip size={18} />
            </motion.button>
            <motion.button 
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.9 }}
              title={t('chat.messageInput.actions.screenshot')}
              onClick={async () => {
                try {
                  if (!navigator.mediaDevices || !navigator.mediaDevices.getDisplayMedia) {
                    toast(t('chat.messageInput.toast.screenshotUnsupported'), 'error');
                    return;
                  }
                  const stream = await navigator.mediaDevices.getDisplayMedia({ video: true });
                  const video = document.createElement('video');
                  video.srcObject = stream;
                  await new Promise(resolve => video.onloadedmetadata = resolve);
                  video.play();
                  // wait a moment to ensure frame is available
                  await new Promise(resolve => setTimeout(resolve, 300));
                  
                  const canvas = document.createElement('canvas');
                  canvas.width = video.videoWidth;
                  canvas.height = video.videoHeight;
                  const ctx = canvas.getContext('2d');
                  if (ctx) {
                    ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
                    canvas.toBlob(async blob => {
                      if (blob) {
                        const file = new File([blob], `Screenshot_${new Date().getTime()}.png`, { type: 'image/png' });
                        if (onSend) {
                          sendFileMessage(file, onSend, 'image');
                        }
                      }
                      stream.getTracks().forEach(t => t.stop());
                    }, 'image/png');
                  } else {
                    stream.getTracks().forEach(t => t.stop());
                  }
                } catch (e: any) {
                  console.error(e);
                  if (e?.message?.includes('display-capture')) {
                    toast(t('chat.messageInput.toast.screenshotDenied'), 'error');
                  } else {
                    toast(t('chat.messageInput.toast.screenshotCancelled'), 'success');
                  }
                }
              }}
              disabled={disabled || isTyping}
              className="w-8 h-8 rounded-lg flex items-center justify-center text-gray-400 hover:text-gray-200 hover:bg-white/5 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Scissors size={18} />
            </motion.button>
            
            {/* Emoji Button & Picker */}
            <div className="relative">
              <motion.button 
                whileHover={{ scale: 1.1 }}
                whileTap={{ scale: 0.9 }}
                title={t('chat.messageInput.actions.emoji')}
                disabled={disabled || isTyping}
                className={`w-8 h-8 rounded-lg flex items-center justify-center transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${showEmojiPicker ? 'text-[#00b42a] bg-[#00b42a]/10' : 'text-gray-400 hover:text-gray-200 hover:bg-white/5'}`}
                onClick={() => setShowEmojiPicker(!showEmojiPicker)}
              >
                <Smile size={18} />
              </motion.button>
              
              <AnimatePresence>
                {showEmojiPicker && (
                  <motion.div 
                    initial={{ opacity: 0, y: 10, scale: 0.95 }}
                    animate={{ opacity: 1, y: 0, scale: 1 }}
                    exit={{ opacity: 0, y: 10, scale: 0.95 }}
                    transition={{ type: "spring", stiffness: 300, damping: 25 }}
                    className="absolute bottom-full left-0 mb-3 z-50 origin-bottom-left"
                  >
                    <EmojiPicker
                      show={showEmojiPicker}
                      onClose={() => setShowEmojiPicker(false)}
                      activeEmojiTab={activeEmojiTab}
                      setActiveEmojiTab={setActiveEmojiTab}
                      onEmojiClick={onEmojiClick}
                      onStickerClick={onStickerClick}
                    />
                  </motion.div>
                )}
              </AnimatePresence>
            </div>

            <motion.button 
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.9 }}
              title={t('chat.messageInput.actions.history')}
              onClick={onHistoryClick}
              disabled={disabled || isTyping}
              className="w-8 h-8 rounded-lg flex items-center justify-center text-gray-400 hover:text-gray-200 hover:bg-white/5 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Clock size={18} />
            </motion.button>
            <motion.button 
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.9 }}
              title={isRecording ? t('chat.messageInput.actions.stopRecording') : t('chat.messageInput.actions.recordVoice')}
              onClick={toggleVoiceRecording}
              disabled={disabled || isTyping}
              className={`w-8 h-8 rounded-lg flex items-center justify-center transition-colors disabled:opacity-50 disabled:cursor-not-allowed relative ${isRecording ? 'text-[#00b42a] bg-[#00b42a]/10 hover:bg-[#00b42a]/20' : 'text-gray-400 hover:text-gray-200 hover:bg-white/5'}`}
            >
              <Mic size={18} className={isRecording ? 'animate-pulse' : ''} />
              {isRecording && <div className="absolute -top-8 left-1/2 -translate-x-1/2 bg-[#00b42a] text-white text-[11px] px-2 py-0.5 rounded shadow-sm whitespace-nowrap">{voiceDuration}s</div>}
            </motion.button>
          </div>
          
          {isTyping ? (
            <motion.button 
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.9 }}
              title={t('chat.messageInput.actions.stopGenerating')}
              onClick={onStop}
              className="w-8 h-8 rounded-full bg-red-500/20 hover:bg-red-500/30 flex items-center justify-center text-red-500 transition-colors shadow-sm"
            >
              <StopCircle size={18} strokeWidth={2.5} />
            </motion.button>
          ) : (
            <motion.button 
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.9 }}
              title={t('chat.messageInput.actions.send')}
              onClick={handleSend}
              disabled={disabled || isEmpty}
              className="w-8 h-8 rounded-full bg-[#00b42a] hover:bg-[#009a24] disabled:bg-white/10 disabled:text-gray-500 flex items-center justify-center text-white transition-colors shadow-sm"
            >
              <ArrowUp size={18} strokeWidth={2.5} />
            </motion.button>
          )}
        </div>
      </div>
    </div>
  );
};
