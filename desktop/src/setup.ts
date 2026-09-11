import "./setup.css";

declare global {
    interface Window {
        lemmaDesktop?: {
            getServerUrl: () => Promise<string | undefined>;
            setServerUrl: (url: string) => Promise<void>;
        };
    }
}

const connectView = document.getElementById("connect-view") as HTMLElement;
const doneView = document.getElementById("done-view") as HTMLElement;
const form = document.getElementById("form") as HTMLFormElement;
const input = document.getElementById("url") as HTMLInputElement;
const reason = document.getElementById("reason") as HTMLParagraphElement;
const connectButton = document.getElementById("connect") as HTMLButtonElement;
const connectedUrl = document.getElementById(
    "connected-url",
) as HTMLParagraphElement;
const changeButton = document.getElementById("change") as HTMLButtonElement;
const doneButton = document.getElementById("done") as HTMLButtonElement;

let serverUrl = "";

const showReason = (text: string) => {
    reason.textContent = text;
    reason.hidden = false;
};

const reasonText = new URLSearchParams(window.location.search).get("reason");
if (reasonText) {
    showReason(reasonText);
}

window.lemmaDesktop?.getServerUrl().then((url) => {
    if (url) input.value = url;
});

// Any HTTP response means the server is reachable; a network failure or the
// 5s timeout rejects.
const probe = async (url: string): Promise<void> => {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), 5000);
    try {
        await fetch(`${url}/`, { signal: controller.signal });
    } finally {
        clearTimeout(timer);
    }
};

form.addEventListener("submit", (event) => {
    event.preventDefault();
    const url = input.value.trim().replace(/\/+$/, "");
    if (!/^https?:\/\/.+/.test(url)) {
        showReason("Enter a full address starting with http:// or https://");
        return;
    }
    connectButton.disabled = true;
    connectButton.textContent = "Connecting…";
    probe(url)
        .then(() => {
            serverUrl = url;
            connectedUrl.textContent = url;
            connectView.hidden = true;
            doneView.hidden = false;
        })
        .catch(() => {
            showReason(`Cannot reach ${url}`);
        })
        .finally(() => {
            connectButton.disabled = false;
            connectButton.textContent = "Connect";
        });
});

changeButton.addEventListener("click", () => {
    reason.hidden = true;
    doneView.hidden = true;
    connectView.hidden = false;
});

doneButton.addEventListener("click", () => {
    window.lemmaDesktop?.setServerUrl(serverUrl);
});
