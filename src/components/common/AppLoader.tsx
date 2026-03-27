import { useState, useEffect, ReactNode } from "react";

interface AppLoaderProps {
  children: ReactNode;
  /** 最小加载时间（毫秒），防止加载界面闪烁 */
  minLoadTime?: number;
}

/**
 * 应用加载器组件
 * 
 * 在应用初始化期间显示加载界面，防止白屏。
 * 等待 Tauri 环境就绪后再渲染主应用。
 */
export function AppLoader({ children, minLoadTime = 300 }: AppLoaderProps) {
  const [isReady, setIsReady] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const startTime = Date.now();

    async function initialize() {
      try {
        // 等待 Tauri API 就绪
        if (window.__TAURI_INTERNALS__) {
          // Tauri 环境已就绪
        }

        // 确保最小加载时间，防止闪烁
        const elapsed = Date.now() - startTime;
        if (elapsed < minLoadTime) {
          await new Promise((resolve) => setTimeout(resolve, minLoadTime - elapsed));
        }

        setIsReady(true);
      } catch (err) {
        console.error("应用初始化失败:", err);
        setError(err instanceof Error ? err.message : "初始化失败");
      }
    }

    initialize();
  }, [minLoadTime]);

  if (error) {
    return (
      <div className="loader-container">
        <div className="loader-content">
          <div className="loader-icon">❌</div>
          <div className="loader-text">初始化失败</div>
          <div className="loader-subtext">{error}</div>
          <button
            onClick={() => window.location.reload()}
            className="loader-button"
          >
            重试
          </button>
        </div>
      </div>
    );
  }

  if (!isReady) {
    return (
      <div className="loader-container">
        <div className="loader-content">
          <div className="loader-spinner" />
          <div className="loader-text">GameManager</div>
          <div className="loader-subtext">正在加载...</div>
        </div>
      </div>
    );
  }

  return <>{children}</>;
}

export default AppLoader;
