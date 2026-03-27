import { Component, ReactNode } from "react";

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

/**
 * 错误边界组件
 * 
 * 捕获子组件树中的 JavaScript 错误，防止整个应用崩溃白屏。
 * 显示友好的错误提示界面。
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error("应用错误:", error);
    console.error("错误堆栈:", errorInfo.componentStack);
  }

  handleReload = () => {
    window.location.reload();
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="flex h-screen w-screen items-center justify-center bg-background">
          <div className="max-w-md rounded-lg border bg-card p-6 text-center shadow-lg">
            <div className="mb-4 text-4xl">😵</div>
            <h2 className="mb-2 text-lg font-semibold text-foreground">
              应用程序出错了
            </h2>
            <p className="mb-4 text-sm text-muted-foreground">
              {this.state.error?.message || "发生了未知错误"}
            </p>
            <div className="flex justify-center gap-2">
              <button
                onClick={this.handleReload}
                className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
              >
                重新加载
              </button>
            </div>
            {import.meta.env.DEV && this.state.error?.stack && (
              <details className="mt-4 text-left">
                <summary className="cursor-pointer text-xs text-muted-foreground">
                  错误详情
                </summary>
                <pre className="mt-2 max-h-40 overflow-auto rounded bg-muted p-2 text-xs">
                  {this.state.error.stack}
                </pre>
              </details>
            )}
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
