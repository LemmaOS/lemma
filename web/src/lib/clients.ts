import { createClient } from "@connectrpc/connect";

import { AuthService } from "@/gen/lemma/v1/auth_pb";
import { ChatService } from "@/gen/lemma/v1/chat_pb";
import { ConversationService } from "@/gen/lemma/v1/conversation_pb";
import { ProviderService } from "@/gen/lemma/v1/provider_pb";
import { StorageService } from "@/gen/lemma/v1/storage_pb";
import { SyncService } from "@/gen/lemma/v1/sync_pb";
import { transport } from "./transport";

export const authClient = createClient(AuthService, transport);
export const chatClient = createClient(ChatService, transport);
export const conversationClient = createClient(ConversationService, transport);
export const providerClient = createClient(ProviderService, transport);
export const syncClient = createClient(SyncService, transport);
export const storageClient = createClient(StorageService, transport);
