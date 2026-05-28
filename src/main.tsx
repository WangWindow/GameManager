import { createRoot } from "react-dom/client";
import App from "@/App";
import { ErrorBoundary } from "@/components/common/ErrorBoundary";
import { AppLoader } from "@/components/common/AppLoader";
import "@/assets/app.css";
import { getCurrentWindow } from "@tauri-apps/api/window";

const container = document.getElementById("app");

if (!container) {
  throw new Error("缺少 #app 根节点");
}

async function setupWindowListeners() {
  try {
    const appWindow = getCurrentWindow();
    const updateMaximizedState = async () => {
      const isMaximized = await appWindow.isMaximized();
      if (isMaximized) {
        document.body.classList.add("maximized");
      } else {
        document.body.classList.remove("maximized");
      }
    };
    await updateMaximizedState();
    await appWindow.onResized(updateMaximizedState);
  } catch (err) {
    console.warn("无法绑定窗口最大化事件", err);
  }
}
setupWindowListeners();

const root = createRoot(container);
root.render(
  <ErrorBoundary>
    <AppLoader>
      <App />
    </AppLoader>
  </ErrorBoundary>
);
