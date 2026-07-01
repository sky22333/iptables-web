import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

/** 合并 Tailwind 类名（shadcn 标准工具）。 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/** 格式化字节数为可读字符串。 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${(bytes / Math.pow(k, i)).toFixed(i > 0 ? 1 : 0)} ${sizes[i]}`
}

/** 配额周期中文标签。 */
export function quotaPeriodLabel(period: string): string {
  switch (period) {
    case 'daily':
      return '每日'
    case 'monthly':
      return '每月'
    case 'total':
      return '累计'
    default:
      return '—'
  }
}
