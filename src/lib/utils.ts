import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"
import { text } from "@/lib/text"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/**
 * 格式化时间戳为日期字符串
 *
 * @param timestamp 时间戳（毫秒）
 */
export function formatDate(timestamp?: number): string {
  if (!timestamp) return text("time.never")
  const date = new Date(timestamp)
  return date.toLocaleDateString("zh-CN", {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  })
}

/**
 * 格式化时间戳为相对时间
 *
 * @param timestamp 时间戳（毫秒）
 */
export function formatRelativeTime(timestamp?: number): string {
  if (!timestamp) return text("time.never")

  const now = Date.now()
  const diff = now - timestamp
  const seconds = Math.floor(diff / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)

  if (days > 30) {
    return formatDate(timestamp)
  } else if (days > 0) {
    return text("time.daysAgo", { count: days })
  } else if (hours > 0) {
    return text("time.hoursAgo", { count: hours })
  } else if (minutes > 0) {
    return text("time.minutesAgo", { count: minutes })
  } else {
    return text("time.justNow")
  }
}

/**
 * 格式化文件大小
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'

  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))

  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}
