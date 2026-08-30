import { ConnectError } from "@connectrpc/connect";
import type { TFunction } from "i18next";

import { ErrorInfoSchema, ErrorReason } from "@/gen/lemma/v1/errors_pb";

// reason → i18n 键的穷举映射：proto 加码不加文案，这里漏一项 tsc 就报错
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
    [ErrorReason.STORAGE_ACCESS_KEY_REQUIRED]: "errors.storageAccessKeyRequired",
    [ErrorReason.STORAGE_SECRET_KEY_REQUIRED]: "errors.storageSecretKeyRequired",
    [ErrorReason.STORAGE_NOT_CONFIGURED]: "errors.storageNotConfigured",
    [ErrorReason.MIGRATION_NOT_PENDING]: "errors.migrationNotPending",
    [ErrorReason.STORAGE_HAS_ARCHIVES]: "errors.storageHasArchives",
    [ErrorReason.BUCKET_NOT_FOUND]: "errors.bucketNotFound",
};

/**
 * ConnectError → 展示文案：带业务码走 i18n（attrs 作插值）；
 * 运维错误（internal）、未知码与非 connect 错误落英文/原始文案，永不本地化
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
