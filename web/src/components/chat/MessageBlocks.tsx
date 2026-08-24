import { Button } from "@/components/ui/button";
import type { InlineSegment, MessageBlock } from "@/mocks";
import { Check, Copy } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";

/** Render inline segments (text / code / strong) inside a block. */
function Inline({ segments }: { segments: InlineSegment[] }) {
    return (
        <>
            {segments.map((seg, i) => {
                switch (seg.type) {
                    case "code":
                        return (
                            <code
                                key={i}
                                className="bg-code text-code-foreground border border-code-border rounded px-1 py-0.5 text-[0.8125rem] font-mono"
                            >
                                {seg.text}
                            </code>
                        );
                    case "strong":
                        return (
                            <strong key={i} className="font-semibold">
                                {seg.text}
                            </strong>
                        );
                    default:
                        return <span key={i}>{seg.text}</span>;
                }
            })}
        </>
    );
}

function CodeBlock({ language, code }: { language: string; code: string }) {
    const { t } = useTranslation();
    const [copied, setCopied] = useState(false);

    const handleCopy = async () => {
        try {
            await navigator.clipboard.writeText(code);
        } catch {
            /* clipboard unavailable in demo */
        }
        setCopied(true);
        window.setTimeout(() => setCopied(false), 1500);
    };

    return (
        <div className="rounded-lg border border-code-border overflow-hidden">
            <div className="flex items-center justify-between px-3 py-1.5 bg-code">
                <span className="text-xs text-muted-foreground font-mono">
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
            <pre className="p-4 font-mono text-[0.8125rem] leading-relaxed overflow-x-auto bg-code/60 text-code-foreground">
                <code>{code}</code>
            </pre>
        </div>
    );
}

/** Render structured message blocks with the Markdown style preset (design.md §3.2). */
export function MessageBlocks({ blocks }: { blocks: MessageBlock[] }) {
    return (
        <div className="space-y-3 text-sm leading-relaxed">
            {blocks.map((block, i) => {
                switch (block.type) {
                    case "paragraph":
                        return (
                            <p key={i}>
                                <Inline segments={block.segments} />
                            </p>
                        );
                    case "heading":
                        return block.level === 2 ? (
                            <h2
                                key={i}
                                className="text-base font-semibold mt-6 mb-2"
                            >
                                {block.text}
                            </h2>
                        ) : (
                            <h3
                                key={i}
                                className="text-sm font-semibold mt-4 mb-1.5"
                            >
                                {block.text}
                            </h3>
                        );
                    case "list": {
                        const items = block.items.map((item, j) => (
                            <li key={j}>
                                <Inline segments={item} />
                            </li>
                        ));
                        return block.ordered ? (
                            <ol
                                key={i}
                                className="list-decimal pl-5 space-y-1 marker:text-muted-foreground"
                            >
                                {items}
                            </ol>
                        ) : (
                            <ul
                                key={i}
                                className="list-disc pl-5 space-y-1 marker:text-muted-foreground"
                            >
                                {items}
                            </ul>
                        );
                    }
                    case "quote":
                        return (
                            <blockquote
                                key={i}
                                className="border-l-2 border-border pl-3 text-muted-foreground"
                            >
                                <Inline segments={block.segments} />
                            </blockquote>
                        );
                    case "code":
                        return (
                            <CodeBlock
                                key={i}
                                language={block.language}
                                code={block.code}
                            />
                        );
                    default:
                        return null;
                }
            })}
        </div>
    );
}

/** Flatten blocks to plain text (used by the message-level copy action). */
// W4 换 react-markdown 渲染后本文件整体删除，届时此函数一并消失
// eslint-disable-next-line react-refresh/only-export-components
export function blocksToPlainText(blocks: MessageBlock[]): string {
    const inline = (segs: InlineSegment[]) => segs.map((s) => s.text).join("");
    return blocks
        .map((block) => {
            switch (block.type) {
                case "paragraph":
                case "quote":
                    return inline(block.segments);
                case "heading":
                    return block.text;
                case "list":
                    return block.items
                        .map((item) => `- ${inline(item)}`)
                        .join("\n");
                case "code":
                    return block.code;
                default:
                    return "";
            }
        })
        .join("\n\n");
}
