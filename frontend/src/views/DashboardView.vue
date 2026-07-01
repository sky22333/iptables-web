<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { Loader2, LogOut, Plus, RefreshCw, Search } from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import Label from '@/components/ui/Label.vue'
import Select from '@/components/ui/Select.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import StatusBadge from '@/components/ui/StatusBadge.vue'
import Textarea from '@/components/ui/Textarea.vue'
import { useToast } from '@/composables/useToast'
import {
  api,
  type DashboardStats,
  type RuleWithTraffic,
  type UpdateRulePayload,
} from '@/lib/api'
import { formatBytes, quotaPeriodLabel } from '@/lib/utils'
import { useAuthStore } from '@/stores/auth'

type PortMode = 'auto' | 'specific' | 'manual'
type QuotaPeriod = 'total' | 'daily' | 'monthly'

const router = useRouter()
const auth = useAuthStore()
const toast = useToast()

const rules = ref<RuleWithTraffic[]>([])
const stats = ref<DashboardStats | null>(null)
const loading = ref(false)
const submitting = ref(false)
const search = ref('')

const mode = ref<PortMode>('specific')
const startPort = ref('')
const manualPorts = ref('')
const targets = ref('')
const quotaGb = ref('')
const quotaPeriod = ref<QuotaPeriod>('total')

const selected = ref<Set<number>>(new Set())
const editing = ref<RuleWithTraffic | null>(null)
const editTargetHost = ref('')
const editTargetPort = ref('')
const editQuotaGb = ref('')
const editQuotaPeriod = ref<QuotaPeriod>('total')
const editUnsetQuota = ref(false)
const editSaving = ref(false)

const modeOptions = [
  { value: 'auto', label: '自动' },
  { value: 'specific', label: '起始端口' },
  { value: 'manual', label: '手动' },
]

const quotaPeriodOptions = [
  { value: 'total', label: '累计' },
  { value: 'daily', label: '每日' },
  { value: 'monthly', label: '每月' },
]

const filteredRules = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return rules.value
  const terms = q.split(',').map((t) => t.trim()).filter(Boolean)
  return rules.value.filter((item) =>
    terms.some(
      (t) =>
        String(item.rule.local_port).includes(t) ||
        item.rule.target_host.toLowerCase().includes(t) ||
        String(item.rule.target_port).includes(t),
    ),
  )
})

const allSelected = computed({
  get: () =>
    filteredRules.value.length > 0 &&
    filteredRules.value.every((r) => selected.value.has(r.rule.local_port)),
  set: (v: boolean) => {
    filteredRules.value.forEach((r) => {
      if (v) selected.value.add(r.rule.local_port)
      else selected.value.delete(r.rule.local_port)
    })
  },
})

function trafficTotal(item: RuleWithTraffic) {
  const t = item.traffic.totals
  return t.tcp_rx + t.tcp_tx + t.udp_rx + t.udp_tx
}

function trafficBreakdown(item: RuleWithTraffic) {
  const t = item.traffic.totals
  const tcp = t.tcp_rx + t.tcp_tx
  const udp = t.udp_rx + t.udp_tx
  return { tcp, udp }
}

function quotaLabel(item: RuleWithTraffic) {
  const q = item.rule.quota_bytes
  if (!q) return '不限'
  const period = quotaPeriodLabel(item.rule.quota_period)
  return `${formatBytes(q)} · ${period}`
}

function quotaPercent(item: RuleWithTraffic) {
  return Math.round((item.traffic.quota_used_ratio ?? 0) * 100)
}

function isQuotaBlocked(item: RuleWithTraffic) {
  return (
    !item.rule.enabled &&
    item.rule.quota_bytes != null &&
    (item.traffic.quota_used_ratio ?? 0) >= 1
  )
}

function statusVariant(item: RuleWithTraffic): 'success' | 'warning' | 'muted' {
  if (item.rule.enabled) return 'success'
  if (isQuotaBlocked(item)) return 'warning'
  return 'muted'
}

function statusLabel(item: RuleWithTraffic) {
  if (item.rule.enabled) return '运行中'
  if (isQuotaBlocked(item)) return '配额停服'
  return '已停用'
}

function toggleSelect(port: number, checked: boolean) {
  if (checked) selected.value.add(port)
  else selected.value.delete(port)
}

async function refresh() {
  loading.value = true
  try {
    const [list, s] = await Promise.all([api.listRules(), api.stats()])
    rules.value = list
    stats.value = s
    selected.value.clear()
  } catch (e) {
    toast.error(e instanceof Error ? e.message : '加载失败')
  } finally {
    loading.value = false
  }
}

async function submitRules() {
  if (!targets.value.trim()) {
    toast.error('请输入目标列表')
    return
  }
  const lines = targets.value.split('\n').filter((l) => l.trim())
  const body: Parameters<typeof api.addRules>[0] = { mode: mode.value, targets: targets.value }

  if (mode.value === 'specific') {
    if (!startPort.value) return toast.error('请输入起始端口')
    body.start_port = Number(startPort.value)
  }
  if (mode.value === 'manual') {
    const ports = manualPorts.value.split(',').map((p) => p.trim()).filter(Boolean)
    if (ports.length !== lines.length) {
      return toast.error(`端口数 (${ports.length}) 与目标数 (${lines.length}) 不一致`)
    }
    body.ports = ports.map(Number)
  }
  if (quotaGb.value.trim()) {
    body.quota_gb = Number(quotaGb.value)
    body.quota_period = quotaPeriod.value
  }

  submitting.value = true
  try {
    const res = await api.addRules(body)
    if (!res.success) return toast.error(res.message ?? '添加失败')
    toast.success(res.message ?? '添加成功')
    targets.value = ''
    startPort.value = ''
    manualPorts.value = ''
    quotaGb.value = ''
    quotaPeriod.value = 'total'
    await refresh()
  } catch (e) {
    toast.error(e instanceof Error ? e.message : '添加失败')
  } finally {
    submitting.value = false
  }
}

function openEdit(item: RuleWithTraffic) {
  editing.value = item
  editTargetHost.value = item.rule.target_host
  editTargetPort.value = String(item.rule.target_port)
  editUnsetQuota.value = false
  editQuotaGb.value = item.rule.quota_bytes
    ? String((item.rule.quota_bytes / 1024 ** 3).toFixed(2))
    : ''
  const p = item.rule.quota_period
  editQuotaPeriod.value = p === 'daily' || p === 'monthly' ? p : 'total'
}

function closeEdit() {
  editing.value = null
}

async function saveEdit() {
  if (!editing.value) return
  const port = editing.value.rule.local_port
  const body: UpdateRulePayload = {
    target_host: editTargetHost.value.trim(),
    target_port: Number(editTargetPort.value),
  }
  if (editUnsetQuota.value) {
    body.unset_quota = true
  } else if (editQuotaGb.value.trim()) {
    body.quota_gb = Number(editQuotaGb.value)
    body.quota_period = editQuotaPeriod.value
  }

  editSaving.value = true
  try {
    const res = await api.updateRule(port, body)
    if (!res.success) return toast.error(res.message ?? '保存失败')
    toast.success('规则已更新')
    closeEdit()
    await refresh()
  } catch (e) {
    toast.error(e instanceof Error ? e.message : '保存失败')
  } finally {
    editSaving.value = false
  }
}

async function toggleRule(item: RuleWithTraffic) {
  try {
    const res = await api.toggleRule(item.rule.local_port)
    if (!res.success) return toast.error(res.message ?? '操作失败')
    toast.success(item.rule.enabled ? '已停用' : '已启用')
    await refresh()
  } catch (e) {
    toast.error(e instanceof Error ? e.message : '操作失败')
  }
}

async function resetTraffic(item: RuleWithTraffic) {
  if (!confirm(`重置端口 ${item.rule.local_port} 的流量统计？`)) return
  try {
    const res = await api.resetTraffic(item.rule.local_port)
    if (!res.success) return toast.error(res.message ?? '重置失败')
    toast.success('流量已重置')
    await refresh()
  } catch (e) {
    toast.error(e instanceof Error ? e.message : '重置失败')
  }
}

async function removeRule(port: number) {
  if (!confirm(`确定删除端口 ${port}？`)) return
  try {
    const res = await api.deleteRule(port)
    if (!res.success) return toast.error(res.message ?? '删除失败')
    toast.success('已删除')
    await refresh()
  } catch (e) {
    toast.error(e instanceof Error ? e.message : '删除失败')
  }
}

async function removeSelected() {
  const ports = [...selected.value]
  if (!ports.length) return toast.error('请先选择规则')
  if (!confirm(`删除选中的 ${ports.length} 条规则？`)) return
  try {
    const res = await api.deleteBatch(ports)
    if (res.data?.failed.length) toast.error(`部分失败: ${res.data.failed.join(', ')}`)
    else toast.success('批量删除成功')
    await refresh()
  } catch (e) {
    toast.error(e instanceof Error ? e.message : '删除失败')
  }
}

function logout() {
  auth.logout()
  router.push({ name: 'login' })
}

let timer: ReturnType<typeof setInterval> | undefined
onMounted(() => {
  refresh()
  timer = setInterval(refresh, 30_000)
})
onUnmounted(() => clearInterval(timer))
</script>

<template>
  <div class="min-h-dvh bg-background">
    <header class="border-b border-border bg-card">
      <div class="mx-auto flex max-w-6xl items-center justify-between gap-3 px-4 h-12">
        <div class="font-medium text-sm tracking-tight">Realm 转发面板</div>
        <div class="flex items-center gap-2">
          <Button variant="outline" size="sm" :disabled="loading" @click="refresh">
            <RefreshCw class="h-3.5 w-3.5" :class="{ 'animate-spin': loading }" />
            刷新
          </Button>
          <Button variant="ghost" size="sm" @click="logout">
            <LogOut class="h-3.5 w-3.5" />
            退出
          </Button>
        </div>
      </div>
    </header>

    <main class="mx-auto max-w-6xl p-4 space-y-4">
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <div class="panel px-4 py-3">
          <div class="text-xs text-muted-foreground">规则总数</div>
          <div class="text-xl font-semibold mt-0.5 tabular-nums">{{ stats?.rule_count ?? '—' }}</div>
        </div>
        <div class="panel px-4 py-3">
          <div class="text-xs text-muted-foreground">运行中</div>
          <div class="text-xl font-semibold mt-0.5 tabular-nums text-primary">{{ stats?.active_count ?? '—' }}</div>
        </div>
        <div class="panel px-4 py-3">
          <div class="text-xs text-muted-foreground">总流量</div>
          <div class="text-xl font-semibold mt-0.5 tabular-nums">{{ stats ? formatBytes(stats.total_traffic_bytes) : '—' }}</div>
        </div>
        <div class="panel px-4 py-3">
          <div class="text-xs text-muted-foreground">配额停服</div>
          <div class="text-xl font-semibold mt-0.5 tabular-nums">{{ stats?.quota_blocked_count ?? '—' }}</div>
        </div>
      </div>

      <section class="panel">
        <div class="panel-header">添加转发</div>
        <div class="panel-body space-y-3">
          <SegmentedControl v-model="mode" :options="modeOptions" />
          <div class="grid gap-3 sm:grid-cols-2">
            <div v-if="mode === 'specific'" class="space-y-1.5">
              <Label>起始端口</Label>
              <Input v-model="startPort" type="number" placeholder="8000" />
            </div>
            <div v-if="mode === 'manual'" class="space-y-1.5 sm:col-span-2">
              <Label>本地端口（逗号分隔）</Label>
              <Input v-model="manualPorts" placeholder="8001,8002" />
            </div>
            <div class="space-y-1.5">
              <Label>流量配额 (GB，可选)</Label>
              <Input v-model="quotaGb" type="number" step="0.1" placeholder="不限" />
            </div>
            <div v-if="quotaGb.trim()" class="space-y-1.5">
              <Label>配额周期</Label>
              <Select v-model="quotaPeriod">
                <option v-for="opt in quotaPeriodOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </Select>
            </div>
          </div>
          <div class="space-y-1.5">
            <Label>目标列表（每行 主机:端口）</Label>
            <Textarea v-model="targets" placeholder="192.168.1.10:8080&#10;10.0.0.2:443" />
          </div>
          <Button :disabled="submitting" @click="submitRules">
            <Loader2 v-if="submitting" class="h-3.5 w-3.5 animate-spin" />
            <Plus v-else class="h-3.5 w-3.5" />
            添加
          </Button>
        </div>
      </section>

      <section class="panel">
        <div class="panel-header flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <span>规则列表 · {{ rules.length }} 条</span>
          <div class="flex gap-2">
            <div class="relative flex-1 sm:w-52">
              <Search class="absolute left-2 top-2 h-3.5 w-3.5 text-muted-foreground pointer-events-none" />
              <Input v-model="search" class="pl-7 h-8" placeholder="搜索端口或目标…" />
            </div>
            <Button variant="destructive" size="sm" :disabled="!selected.size" @click="removeSelected">
              删除 ({{ selected.size }})
            </Button>
          </div>
        </div>

        <div class="hidden md:block overflow-x-auto">
          <table class="w-full text-sm">
            <thead class="border-b border-border bg-muted/40 text-left text-xs text-muted-foreground">
              <tr>
                <th class="w-10 p-3"><input v-model="allSelected" type="checkbox" class="rounded" /></th>
                <th class="p-3 font-medium">端口</th>
                <th class="p-3 font-medium">目标</th>
                <th class="p-3 font-medium">状态</th>
                <th class="p-3 font-medium">流量</th>
                <th class="p-3 font-medium">配额</th>
                <th class="p-3 font-medium text-right">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-if="loading && !rules.length">
                <td colspan="7" class="p-8 text-center text-muted-foreground">
                  <Loader2 class="inline h-4 w-4 animate-spin" />
                </td>
              </tr>
              <tr v-else-if="!filteredRules.length">
                <td colspan="7" class="p-8 text-center text-muted-foreground">暂无规则</td>
              </tr>
              <tr
                v-for="item in filteredRules"
                v-else
                :key="item.rule.id"
                class="border-b border-border last:border-0 hover:bg-muted/20"
              >
                <td class="p-3">
                  <input
                    type="checkbox"
                    class="rounded"
                    :checked="selected.has(item.rule.local_port)"
                    @change="toggleSelect(item.rule.local_port, ($event.target as HTMLInputElement).checked)"
                  />
                </td>
                <td class="p-3 font-mono font-medium tabular-nums">{{ item.rule.local_port }}</td>
                <td class="p-3 font-mono text-muted-foreground text-xs">
                  {{ item.rule.target_host }}:{{ item.rule.target_port }}
                </td>
                <td class="p-3">
                  <StatusBadge :variant="statusVariant(item)">{{ statusLabel(item) }}</StatusBadge>
                </td>
                <td class="p-3">
                  <div class="text-muted-foreground tabular-nums">{{ formatBytes(trafficTotal(item)) }}</div>
                  <div class="text-[11px] text-muted-foreground/80 mt-0.5">
                    TCP {{ formatBytes(trafficBreakdown(item).tcp) }} · UDP {{ formatBytes(trafficBreakdown(item).udp) }}
                  </div>
                </td>
                <td class="p-3">
                  <div v-if="item.rule.quota_bytes" class="space-y-1">
                    <div class="text-xs text-muted-foreground">{{ quotaLabel(item) }}</div>
                    <div class="h-1 w-20 rounded-full bg-muted overflow-hidden">
                      <div
                        class="h-full rounded-full transition-all"
                        :class="quotaPercent(item) >= 100 ? 'bg-destructive' : 'bg-primary'"
                        :style="{ width: `${Math.min(quotaPercent(item), 100)}%` }"
                      />
                    </div>
                    <div class="text-[11px] text-muted-foreground tabular-nums">{{ quotaPercent(item) }}%</div>
                  </div>
                  <span v-else class="text-xs text-muted-foreground">不限</span>
                </td>
                <td class="p-3">
                  <div class="flex justify-end gap-0.5 flex-wrap">
                    <Button variant="ghost" size="sm" @click="openEdit(item)">编辑</Button>
                    <Button variant="ghost" size="sm" @click="toggleRule(item)">
                      {{ item.rule.enabled ? '停用' : '启用' }}
                    </Button>
                    <Button variant="ghost" size="sm" @click="resetTraffic(item)">清零</Button>
                    <Button variant="ghost" size="sm" class="text-destructive hover:text-destructive" @click="removeRule(item.rule.local_port)">
                      删除
                    </Button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="md:hidden divide-y divide-border">
          <div v-if="!filteredRules.length" class="p-6 text-center text-muted-foreground text-sm">暂无规则</div>
          <div v-for="item in filteredRules" :key="item.rule.id" class="p-4 space-y-2.5">
            <div class="flex items-center gap-2">
              <input
                type="checkbox"
                class="rounded"
                :checked="selected.has(item.rule.local_port)"
                @change="toggleSelect(item.rule.local_port, ($event.target as HTMLInputElement).checked)"
              />
              <span class="font-mono font-medium tabular-nums">:{{ item.rule.local_port }}</span>
              <StatusBadge :variant="statusVariant(item)" class="ml-auto">{{ statusLabel(item) }}</StatusBadge>
            </div>
            <div class="text-xs font-mono text-muted-foreground pl-6">
              → {{ item.rule.target_host }}:{{ item.rule.target_port }}
            </div>
            <div class="text-xs text-muted-foreground pl-6 tabular-nums">
              {{ formatBytes(trafficTotal(item)) }}
              · TCP {{ formatBytes(trafficBreakdown(item).tcp) }}
              · UDP {{ formatBytes(trafficBreakdown(item).udp) }}
            </div>
            <div v-if="item.rule.quota_bytes" class="pl-6 space-y-1">
              <div class="text-xs text-muted-foreground">{{ quotaLabel(item) }} · {{ quotaPercent(item) }}%</div>
              <div class="h-1 rounded-full bg-muted overflow-hidden">
                <div
                  class="h-full rounded-full"
                  :class="quotaPercent(item) >= 100 ? 'bg-destructive' : 'bg-primary'"
                  :style="{ width: `${Math.min(quotaPercent(item), 100)}%` }"
                />
              </div>
            </div>
            <div class="flex flex-wrap gap-1 pl-6 pt-0.5">
              <Button variant="outline" size="sm" @click="openEdit(item)">编辑</Button>
              <Button variant="outline" size="sm" @click="toggleRule(item)">
                {{ item.rule.enabled ? '停用' : '启用' }}
              </Button>
              <Button variant="outline" size="sm" @click="resetTraffic(item)">清零</Button>
              <Button variant="outline" size="sm" class="text-destructive" @click="removeRule(item.rule.local_port)">删除</Button>
            </div>
          </div>
        </div>
      </section>
    </main>

    <div
      v-if="editing"
      class="fixed inset-0 z-40 flex items-end sm:items-center justify-center bg-black/25 p-4"
      @click.self="closeEdit"
    >
      <div class="panel w-full max-w-md">
        <div class="panel-header flex items-center justify-between">
          <span>编辑 · 端口 {{ editing.rule.local_port }}</span>
          <button type="button" class="text-muted-foreground text-xs hover:text-foreground" @click="closeEdit">关闭</button>
        </div>
        <div class="panel-body space-y-3">
          <div class="space-y-1.5">
            <Label>目标 IP / 域名</Label>
            <Input v-model="editTargetHost" />
          </div>
          <div class="space-y-1.5">
            <Label>目标端口</Label>
            <Input v-model="editTargetPort" type="number" />
          </div>
          <div class="space-y-1.5">
            <Label>流量配额 (GB)</Label>
            <Input v-model="editQuotaGb" type="number" step="0.1" :disabled="editUnsetQuota" placeholder="不限" />
            <label class="flex items-center gap-2 text-xs text-muted-foreground cursor-pointer">
              <input v-model="editUnsetQuota" type="checkbox" class="rounded" />
              不限制流量
            </label>
          </div>
          <div v-if="!editUnsetQuota && editQuotaGb.trim()" class="space-y-1.5">
            <Label>配额周期</Label>
            <Select v-model="editQuotaPeriod">
              <option v-for="opt in quotaPeriodOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </option>
            </Select>
          </div>
          <div class="flex gap-2 pt-1">
            <Button class="flex-1" :disabled="editSaving" @click="saveEdit">
              <Loader2 v-if="editSaving" class="h-3.5 w-3.5 animate-spin" />
              保存
            </Button>
            <Button variant="outline" class="flex-1" @click="closeEdit">取消</Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
