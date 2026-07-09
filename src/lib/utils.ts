import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"
import { getLocale, translate } from "@/i18n"
import type { Locale } from "@/i18n/types"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/**
 * 格式化时间戳为日期字符串
 *
 * @param timestamp 时间戳（毫秒）
 * @param locale 可选语言，默认使用当前 i18n 语言
 */
export function formatDate(timestamp?: number, locale?: Locale): string {
  if (!timestamp) return translate("time.never", undefined, locale ?? getLocale())
  const date = new Date(timestamp)
  return date.toLocaleDateString(locale ?? getLocale(), {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  })
}

/**
 * 格式化时间戳为相对时间
 *
 * @param timestamp 时间戳（毫秒）
 * @param locale 可选语言，默认使用当前 i18n 语言
 */
export function formatRelativeTime(timestamp?: number, locale?: Locale): string {
  const lng = locale ?? getLocale()
  if (!timestamp) return translate("time.never", undefined, lng)

  const now = Date.now()
  const diff = now - timestamp
  const seconds = Math.floor(diff / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)

  if (days > 30) {
    return formatDate(timestamp, lng)
  } else if (days > 0) {
    return translate("time.daysAgo", { count: days }, lng)
  } else if (hours > 0) {
    return translate("time.hoursAgo", { count: hours }, lng)
  } else if (minutes > 0) {
    return translate("time.minutesAgo", { count: minutes }, lng)
  } else {
    return translate("time.justNow", undefined, lng)
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
