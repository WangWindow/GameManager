import { useState, useMemo, useRef } from "react";
import type { TaskStatus } from "@/types";

export function useTaskStatus() {
  const [currentTask, setCurrentTask] = useState<TaskStatus | null>(null);
  const taskClearTimerRef = useRef<number | null>(null);

  function updateTask(label: string, progress: number) {
    if (taskClearTimerRef.current) {
      window.clearTimeout(taskClearTimerRef.current);
      taskClearTimerRef.current = null;
    }
    const safeProgress = Math.max(0, Math.min(100, Number(progress) || 0));
    setCurrentTask({ label, progress: safeProgress });
    if (safeProgress >= 100) {
      taskClearTimerRef.current = window.setTimeout(() => {
        setCurrentTask(null);
      }, 1200);
    }
  }

  const statusBarVisible = useMemo(() => currentTask !== null, [currentTask]);

  return { currentTask, statusBarVisible, updateTask };
}
