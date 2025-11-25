// Native messaging host name -
// NOTE: must match the name in the native manifest
const NATIVE_APP_NAME = "com.stopit.tracker";

// Track last seen state to avoid sending duplicate messages
let lastUrl = "";
let lastTitle = "";

// Message types
interface TabUpdateMessage {
  type: "tab_update";
  url: string;
  title: string;
  domain: string | null;
  timestamp: number;
}

interface NativeResponse {
  success: boolean;
  message?: string;
}

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

    if (tab && tab.url && tab.title) {
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
 * Send message to the native messaging host
 */
function sendMessage(message: TabUpdateMessage): void {
  try {
    chrome.runtime.sendNativeMessage(
      NATIVE_APP_NAME,
      message,
      (response: NativeResponse) => {
        if (chrome.runtime.lastError) {
          console.error(
            "Native messaging error:",
            chrome.runtime.lastError.message,
          );
          // This is expected if the native host isn't installed yet
          if (
            chrome.runtime.lastError.message?.includes(
              "Specified native messaging host not found",
            )
          ) {
            console.warn(
              "Native host not installed. Run the installation script from your Rust project.",
            );
          }
        } else if (response) {
          console.log("Native app response:", response);
        }
      },
    );
  } catch (error) {
    console.error("Failed to send to native app:", error);
  }
}

/**
 * Listen for tab changes
 */
chrome.tabs.onActivated.addListener(() => {
  checkActiveTab();
});

/*
 * Listen for tab updates (URL/title changes)
 */
chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  // Only check when the tab is active and URL/title changed
  if (tab.active && (changeInfo.url || changeInfo.title)) {
    checkActiveTab();
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
    checkActiveTab();
  }
});

// Initial check on startup
checkActiveTab();

// Polling fallback (every 2 seconds) to catch cases where events might be missed
setInterval(checkActiveTab, 2000);

console.log("Stop It extension loaded and monitoring tabs");
