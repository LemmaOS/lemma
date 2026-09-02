import { ConnectError } from "@connectrpc/connect";
import type { TFunction } from "i18next";

import { ErrorInfoSchema, ErrorReason } from "@/gen/lemma/v1/errors_pb";

// Maps every business error reason (the closed set in errors.proto) to its
// i18n key. The Record type makes the mapping exhaustive: adding a proto
// reason without a translation here fails compilation.
const reasonKeys: Record<ErrorReason, string> = {
    [ErrorReason.UNSPECIFIED]: "errors.unspecified",
    [ErrorReason.CREDENTIALS_INVALID]: "errors.credentialsInvalid",
    [ErrorReason.USERNAME_TAKEN]: "errors.usernameTaken",
    [ErrorReason.SIGNUP_FIELDS_REQUIRED]: "errors.signupFieldsRequired",
    [ErrorReason.LOGIN_TARGET_REQUIRED]: "errors.loginTargetRequired",
    [ErrorReason.TOKEN_INVALID]: "errors.tokenInvalid",
    [ErrorReason.USER_NOT_FOUND]: "errors.userNotFound",
    [ErrorReason.PROVIDER_FIELDS_REQUIRED]: "errors.providerFieldsRequired",
    [ErrorReason.PROVIDER_KIND_INVALID]: "errors.providerKindInvalid",
    [ErrorReason.PROVIDER_NOT_FOUND]: "errors.providerNotFound",
    [ErrorReason.PROVIDER_DISABLED]: "errors.providerDisabled",
    [ErrorReason.ID_INVALID]: "errors.idInvalid",
    [ErrorReason.TITLE_REQUIRED]: "errors.titleRequired",
    [ErrorReason.CONVERSATION_NOT_FOUND]: "errors.conversationNotFound",
    [ErrorReason.CONVERSATION_NOT_ACTIVE]: "errors.conversationNotActive",
    [ErrorReason.CONVERSATION_NOT_ARCHIVED]: "errors.conversationNotArchived",
    [ErrorReason.ARCHIVED_CONVERSATION_NOT_FOUND]:
        "errors.archivedConversationNotFound",
    [ErrorReason.MESSAGE_NOT_FOUND]: "errors.messageNotFound",
    [ErrorReason.NOT_ASSISTANT_MESSAGE]: "errors.notAssistantMessage",
    [ErrorReason.CONTENT_REQUIRED]: "errors.contentRequired",
    [ErrorReason.MODEL_REQUIRED]: "errors.modelRequired",
    [ErrorReason.STORAGE_ENDPOINT_REQUIRED]: "errors.storageEndpointRequired",
    [ErrorReason.STORAGE_BUCKET_REQUIRED]: "errors.storageBucketRequired",
    [ErrorReason.STORAGE_ACCESS_KEY_REQUIRED]:
        "errors.storageAccessKeyRequired",
    [ErrorReason.STORAGE_SECRET_KEY_REQUIRED]:
        "errors.storageSecretKeyRequired",
    [ErrorReason.STORAGE_NOT_CONFIGURED]: "errors.storageNotConfigured",
    [ErrorReason.MIGRATION_NOT_PENDING]: "errors.migrationNotPending",
    [ErrorReason.STORAGE_HAS_ARCHIVES]: "errors.storageHasArchives",
    [ErrorReason.BUCKET_NOT_FOUND]: "errors.bucketNotFound",
};

/**
 * Renders an error for display. Business errors are localized by reason
 * code; anything without an ErrorInfo detail (internal errors, network
 * failures) shows as its raw English message and is never localized.
 */
export function errorText(e: unknown, t: TFunction): string {
    if (e instanceof ConnectError) {
        const info = e.findDetails(ErrorInfoSchema)[0];
        const key = info ? reasonKeys[info.reason] : undefined;
        if (info && key) {
            return t(key, info.attrs);
        }
        return e.message;
    }
    return String(e);
}
