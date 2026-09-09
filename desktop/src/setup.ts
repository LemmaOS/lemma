import "./setup.css";

declare global {
    interface Window {
        lemmaDesktop?: {
            getServerUrl: () => Promise<string | undefined>;
            setServerUrl: (url: string) => Promise<void>;
        };
    }
}

const form = document.getElementById("form") as HTMLFormElement;
const input = document.getElementById("url") as HTMLInputElement;
const reason = document.getElementById("reason") as HTMLParagraphElement;

const reasonText = new URLSearchParams(window.location.search).get("reason");
if (reasonText) {
    reason.textContent = reasonText;
    reason.hidden = false;
}

window.lemmaDesktop?.getServerUrl().then((url) => {
    if (url) input.value = url;
});

form.addEventListener("submit", (event) => {
    event.preventDefault();
    const url = input.value.trim().replace(/\/+$/, "");
    if (!/^https?:\/\/.+/.test(url)) {
        reason.textContent =
            "Enter a full address starting with http:// or https://";
        reason.hidden = false;
        return;
    }
    window.lemmaDesktop?.setServerUrl(url);
});
