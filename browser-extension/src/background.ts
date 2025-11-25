import { z } from "zod/v4";

// WebSocket connection to the Stop It daemon
const DAEMON_WS_URL = "ws://127.0.0.1:8765";

// Track last seen state to avoid sending duplicate messages
let lastUrl = "";
let lastTitle = "";

// WebSocket connection
let ws: WebSocket | null = null;
let reconnectInterval: number | null = null;
const RECONNECT_DELAY = 5000; // 5 seconds

// Message types
type TabUpdateMessage = {
  type: "tab_update";
  url: string;
  title: string;
  domain: string | null;
  timestamp: number;
};

// Zod schema for daemon response validation
const NativeResponseSchema = z.object({
  success: z.boolean(),
  message: z.string().optional(),
});

//type NativeResponse = z.infer<typeof NativeResponseSchema>;

/**
 * Extract domain from URL
 * @param url - The full URL string
 * @returns The domain or null if parsing fails
 */
function extractDomain(url: string): string | null {
  try {
    const urlObj = new URL(url);
    // Return hostname (e.g., "github.com" from "https://github.com/user/repo")
    return urlObj.hostname.replace(/^www\./, "");
  } catch (error) {
    console.error("Failed to parse URL:", url, error);
    return null;
  }
}

/**
 * Check the currently active tab and send its info to the native app
 * @returns Promise<void>
 */
async function checkActiveTab(): Promise<void> {
  try {
    const [tab] = await chrome.tabs.query({
      active: true,
      currentWindow: true,
    });

    if (tab.url && tab.title) {
      // Skip chrome:// and other internal URLs
      if (
        tab.url.startsWith("chrome://") ||
        tab.url.startsWith("about:") ||
        tab.url.startsWith("chrome-extension://")
      ) {
        return;
      }

      // Only send if URL or title changed
      if (tab.url !== lastUrl || tab.title !== lastTitle) {
        lastUrl = tab.url;
        lastTitle = tab.title;

        const domain = extractDomain(tab.url);

        const message: TabUpdateMessage = {
          type: "tab_update",
          url: tab.url,
          title: tab.title,
          domain: domain,
          timestamp: Date.now(),
        };

        sendMessage(message);
      }
    }
  } catch (error) {
    console.error("Error checking active tab:", error);
  }
}

/**
 * Connect to the WebSocket daemon
 * @return void
 */
function connectWebSocket(): void {
  if (ws && ws.readyState === WebSocket.OPEN) {
    return; // Already connected
  }

  console.log("Connecting to Stop It daemon...");
  ws = new WebSocket(DAEMON_WS_URL);

  ws.onopen = () => {
    console.log("Connected to Stop It daemon");
    if (reconnectInterval) {
      clearInterval(reconnectInterval);
      reconnectInterval = null;
    }
    // Send current tab on connection (fire-and-forget)
    void checkActiveTab();
  };

  ws.onmessage = (event: MessageEvent<unknown>) => {
    try {
      const rawData = event.data;
      if (typeof rawData !== "string") {
        console.error("Expected string data from daemon, got:", typeof rawData);
        return;
      }
      const data = JSON.parse(rawData) as unknown;
      const response = NativeResponseSchema.parse(data);
      console.log("Daemon response:", response);
    } catch (error) {
      if (error instanceof z.ZodError) {
        console.error("Invalid daemon response format:", error.issues);
      } else {
        console.error("Failed to parse daemon response:", error);
      }
    }
  };

  ws.onerror = (error) => {
    console.error("WebSocket error:", error);
  };

  ws.onclose = () => {
    console.log("Disconnected from Stop It daemon");
    ws = null;
    // Auto-reconnect
    if (!reconnectInterval) {
      reconnectInterval = setInterval(() => {
        console.log("Attempting to reconnect...");
        connectWebSocket();
      }, RECONNECT_DELAY);
    }
  };
}

/**
 * Send message to the daemon via WebSocket
 * @param message - The message to send
 * @returns void
 */
function sendMessage(message: TabUpdateMessage): void {
  if (!ws || ws.readyState !== WebSocket.OPEN) {
    console.warn("WebSocket not connected. Message not sent.");
    return;
  }

  try {
    ws.send(JSON.stringify(message));
  } catch (error) {
    console.error("Failed to send message to daemon:", error);
  }
}

/**
 * Listen for tab changes
 */
chrome.tabs.onActivated.addListener(() => {
  void checkActiveTab();
});

/*
 * Listen for tab updates (URL/title changes)
 */
chrome.tabs.onUpdated.addListener((_tabId, changeInfo, tab) => {
  // Only check when the tab is active and URL/title changed
  if (tab.active && (changeInfo.url || changeInfo.title)) {
    void checkActiveTab();
  }
});

/*
 * Listen for window focus changes
 * When the user switches windows, check the active tab in the new window


 * windowId - The ID of the newly focused window, or chrome.windows.WINDOW_ID_NONE (-1) if no Chrome/Brave window has focus
 * indowId !== chrome.windows.WINDOW_ID_NONE means:
 * - ✅ If you focus a Chrome/Brave window, it will trigger the checkActiveTab function.
 * - ❌ If you focus outside of Chrome/Brave (like another application), it won't trigger the checkActiveTab function.
 */
chrome.windows.onFocusChanged.addListener((windowId) => {
  if (windowId !== chrome.windows.WINDOW_ID_NONE) {
    void checkActiveTab();
  }
});

// Connect to daemon on startup
connectWebSocket();

// Polling fallback (every 2 seconds) to catch cases where events might be missed
setInterval(checkActiveTab, 2000);

console.log("Stop It extension loaded and monitoring tabs");
