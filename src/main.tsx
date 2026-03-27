import { createRoot } from "react-dom/client";
import App from "@/App";
import { ErrorBoundary } from "@/components/common/ErrorBoundary";
import { AppLoader } from "@/components/common/AppLoader";
import "@/assets/app.css";

const container = document.getElementById("app");

if (!container) {
  throw new Error("缺少 #app 根节点");
}

const root = createRoot(container);
root.render(
  <ErrorBoundary>
    <AppLoader>
      <App />
    </AppLoader>
  </ErrorBoundary>
);
