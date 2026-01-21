// Services (Tauri IPC wrappers, etc.)
export { copyToClipboard } from "./clipboard";
export {
  tauriService,
  isTauriAvailable,
  getConversations,
  getConversation,
  getProjects,
  searchConversations,
  TauriError,
  NotFoundError,
  NetworkError,
} from "./tauri";
