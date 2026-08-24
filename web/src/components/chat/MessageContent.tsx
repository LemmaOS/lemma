import { Check, Copy } from "lucide-react";
import {
    type ComponentProps,
    isValidElement,
    type ReactNode,
    useRef,
    useState,
} from "react";
import { useTranslation } from "react-i18next";
import ReactMarkdown, { type Components } from "react-markdown";
import rehypeHighlight from "rehype-highlight";

import { Button } from "@/components/ui/button";

// rehype-highlight 的 detect 靠猜， fenced block 没写语言时容易猜错，关掉
const rehypePlugins = [[rehypeHighlight, { detect: false }]] as ComponentProps<
    typeof ReactMarkdown
>["rehypePlugins"];

/** 代码块外壳：语言标签 + 复制按钮，配色全部来自 code 系列 token */
function CodeBlock({ children }: { children?: ReactNode }) {
    const { t } = useTranslation();
    const [copied, setCopied] = useState(false);
    const codeRef = useRef<HTMLPreElement>(null);

    let language = "";
    if (isValidElement(children)) {
        const className = (children.props as { className?: string }).className;
        language = /language-(\w+)/.exec(className ?? "")?.[1] ?? "";
    }

    const handleCopy = async () => {
        try {
            await navigator.clipboard.writeText(
                codeRef.current?.textContent ?? "",
            );
        } catch {
            // 剪贴板不可用（非安全上下文）时静默
        }
        setCopied(true);
        window.setTimeout(() => setCopied(false), 1500);
    };

    return (
        <div className="overflow-hidden rounded-lg border border-code-border">
            <div className="flex items-center justify-between bg-code px-3 py-1.5">
                <span className="font-mono text-xs text-muted-foreground">
                    {language}
                </span>
                <Button
                    variant="ghost"
                    size="icon"
                    className="size-6"
                    onClick={handleCopy}
                    aria-label={copied ? t("chat.copied") : t("chat.copy")}
                >
                    {copied ? (
                        <Check className="size-3.5" />
                    ) : (
                        <Copy className="size-3.5" />
                    )}
                </Button>
            </div>
            <pre
                ref={codeRef}
                className="overflow-x-auto bg-code/60 p-4 font-mono text-[0.8125rem] leading-relaxed text-code-foreground"
            >
                {children}
            </pre>
        </div>
    );
}

// Markdown 排版预设（对齐设计稿 §3.2）：行间节奏、标题、列表、引用、行内代码
const components: Components = {
    pre: ({ children }) => <CodeBlock>{children}</CodeBlock>,
    p: ({ node: _node, ...props }) => <p {...props} />,
    h1: ({ node: _node, ...props }) => (
        <h2 className="mt-6 mb-2 text-base font-semibold" {...props} />
    ),
    h2: ({ node: _node, ...props }) => (
        <h2 className="mt-6 mb-2 text-base font-semibold" {...props} />
    ),
    h3: ({ node: _node, ...props }) => (
        <h3 className="mt-4 mb-1.5 text-sm font-semibold" {...props} />
    ),
    ul: ({ node: _node, ...props }) => (
        <ul
            className="list-disc space-y-1 pl-5 marker:text-muted-foreground"
            {...props}
        />
    ),
    ol: ({ node: _node, ...props }) => (
        <ol
            className="list-decimal space-y-1 pl-5 marker:text-muted-foreground"
            {...props}
        />
    ),
    blockquote: ({ node: _node, ...props }) => (
        <blockquote
            className="border-l-2 border-border pl-3 text-muted-foreground"
            {...props}
        />
    ),
    a: ({ node: _node, ...props }) => (
        <a
            className="text-primary underline underline-offset-2"
            target="_blank"
            rel="noreferrer"
            {...props}
        />
    ),
};

interface MessageContentProps {
    content: string;
}

/** assistant 消息的 Markdown 渲染；行内代码样式用 :not(pre)>code 选择器避免误伤代码块 */
export function MessageContent({ content }: MessageContentProps) {
    return (
        <div className="space-y-3 text-sm leading-relaxed [&_:not(pre)>code]:rounded [&_:not(pre)>code]:border [&_:not(pre)>code]:border-code-border [&_:not(pre)>code]:bg-code [&_:not(pre)>code]:px-1 [&_:not(pre)>code]:py-0.5 [&_:not(pre)>code]:font-mono [&_:not(pre)>code]:text-[0.8125rem] [&_:not(pre)>code]:text-code-foreground">
            <ReactMarkdown
                rehypePlugins={rehypePlugins}
                components={components}
            >
                {content}
            </ReactMarkdown>
        </div>
    );
}
