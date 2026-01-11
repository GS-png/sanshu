<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import hljs from 'highlight.js'
import MarkdownIt from 'markdown-it'
import { useMessage } from 'naive-ui'
import { computed, onMounted, ref, watch } from 'vue'

interface HistoryEntrySummary {
  id: string
  timestamp: string
  request_id?: string | null
  source?: string | null
  preview: string
}

function normalizeSearchText(input: string) {
  return (input || '')
    .trim()
    .replaceAll('：', ':')
    .replaceAll('／', '/')
    .replaceAll('　', ' ')
    .toLowerCase()
}

function formatTimeForSearch(ts: string) {
  try {
    const d = new Date(ts)
    if (Number.isNaN(d.getTime()))
      return ts

    const y = d.getFullYear()
    const m = d.getMonth() + 1
    const day = d.getDate()
    const hh = String(d.getHours()).padStart(2, '0')
    const mm = String(d.getMinutes()).padStart(2, '0')
    const ss = String(d.getSeconds()).padStart(2, '0')
    return `${y}/${m}/${day} ${hh}:${mm}:${ss} ${hh}:${mm} ${y}/${m}`
  }
  catch {
    return ts
  }
}

function getRangeIso() {
  if (!range.value)
    return { start: null as string | null, end: null as string | null }

  const [startMs, endMs] = range.value
  if (!startMs || !endMs)
    return { start: null as string | null, end: null as string | null }

  return {
    start: new Date(startMs).toISOString(),
    end: new Date(endMs).toISOString(),
  }
}

async function exportRangeZip() {
  const { start, end } = getRangeIso()
  if (!start || !end) {
    message.warning('请先选择开始时间和结束时间')
    return
  }
  try {
    const path = await invoke('export_mcp_history_by_time_range_zip', { start, end }) as string
    message.success(`已导出: ${path}`)
  }
  catch (err) {
    console.error('按时间段导出失败:', err)
    message.error(`按时间段导出失败: ${err}`)
  }
}

async function deleteRange() {
  const { start, end } = getRangeIso()
  if (!start || !end) {
    message.warning('请先选择开始时间和结束时间')
    return
  }
  try {
    const deleted = await invoke('delete_mcp_history_by_time_range', { start, end }) as number
    message.success(`已删除 ${deleted} 条`)
    selectedId.value = ''
    selectedDetail.value = null
    await loadEntries()
  }
  catch (err) {
    console.error('按时间段删除失败:', err)
    message.error(`按时间段删除失败: ${err}`)
  }
}

interface HistoryImage {
  filename: string
  media_type: string
  data_uri: string
}

interface PopupRequest {
  id: string
  message: string
  predefined_options?: string[] | null
  is_markdown: boolean
  project_root_path?: string | null
}

interface HistoryEntryDetail {
  summary: HistoryEntrySummary
  request?: PopupRequest | null
  response: any
  markdown: string
  images: HistoryImage[]
}

const message = useMessage()

const loading = ref(false)
const entries = ref<HistoryEntrySummary[]>([])
const search = ref('')

function getLocalDayKey(d: Date) {
  if (Number.isNaN(d.getTime()))
    return 'unknown'

  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  return `${y}-${m}-${day}`
}

function getDayLabel(dayKey: string) {
  const todayKey = getLocalDayKey(new Date())
  if (dayKey === todayKey)
    return `今天 ${dayKey}`

  const y = new Date()
  y.setDate(y.getDate() - 1)
  const yesterdayKey = getLocalDayKey(y)
  if (dayKey === yesterdayKey)
    return `昨天 ${dayKey}`

  return dayKey
}

const range = ref<[number, number] | null>(null)

const selectedId = ref<string>('')
const detailLoading = ref(false)
const selectedDetail = ref<HistoryEntryDetail | null>(null)

const md = new MarkdownIt({
  html: false,
  xhtmlOut: false,
  breaks: true,
  langPrefix: 'language-',
  linkify: true,
  typographer: true,
  highlight(str: string, lang: string) {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(str, { language: lang }).value
      }
      catch {
      }
    }
    return ''
  },
})

const groupedEntries = computed(() => {
  const map = new Map<string, HistoryEntrySummary[]>()
  for (const e of filteredEntries.value) {
    const key = getLocalDayKey(new Date(e.timestamp))
    const list = map.get(key) || []
    list.push(e)
    map.set(key, list)
  }

  const keys = Array.from(map.keys()).sort((a, b) => b.localeCompare(a))
  return keys.map(key => ({
    key,
    label: getDayLabel(key),
    entries: map.get(key) || [],
  }))
})

const expandedGroups = ref<string[]>([])

function ensureDefaultExpandedGroups() {
  if (search.value.trim()) {
    expandedGroups.value = groupedEntries.value.map(g => g.key)
    return
  }

  const todayKey = getLocalDayKey(new Date())
  const hasToday = groupedEntries.value.some(g => g.key === todayKey)
  if (hasToday) {
    expandedGroups.value = [todayKey]
    return
  }

  const first = groupedEntries.value[0]?.key
  expandedGroups.value = first ? [first] : []
}

function expandAllGroups() {
  expandedGroups.value = groupedEntries.value.map(g => g.key)
}

function collapseAllGroups() {
  expandedGroups.value = []
}

md.renderer.rules.link_open = function (tokens, idx, options, env, renderer) {
  const token = tokens[idx]
  const href = token.attrGet('href')
  if (href && (href.startsWith('http://') || href.startsWith('https://'))) {
    token.attrSet('href', '#')
    token.attrSet('onclick', 'return false;')
    token.attrSet('style', 'cursor: default; text-decoration: none;')
    token.attrSet('title', `外部链接已禁用: ${href}`)
  }
  return renderer.renderToken(tokens, idx, options)
}

md.renderer.rules.autolink_open = function (tokens, idx, options, env, renderer) {
  const token = tokens[idx]
  const href = token.attrGet('href')
  if (href && (href.startsWith('http://') || href.startsWith('https://'))) {
    token.attrSet('href', '#')
    token.attrSet('onclick', 'return false;')
    token.attrSet('style', 'cursor: default; text-decoration: none;')
    token.attrSet('title', `外部链接已禁用: ${href}`)
  }
  return renderer.renderToken(tokens, idx, options)
}

const filteredEntries = computed(() => {
  const q = normalizeSearchText(search.value)
  if (!q)
    return entries.value

  return entries.value.filter((e) => {
    const timeText = formatTimeForSearch(e.timestamp || '')
    const haystack = normalizeSearchText(
      [
        e.id,
        e.preview || '',
        e.request_id || '',
        e.source || '',
        e.timestamp || '',
        timeText,
      ].join(' '),
    )
    return (
      haystack.includes(q)
    )
  })
})

const renderedMarkdown = computed(() => {
  if (!selectedDetail.value)
    return ''

  let content = selectedDetail.value.markdown || ''
  for (const img of selectedDetail.value.images || []) {
    content = content.replaceAll(`images/${img.filename}`, img.data_uri)
  }

  try {
    return md.render(content)
  }
  catch (error) {
    console.error('Markdown 渲染失败:', error)
    return content
  }
})

function formatTime(ts: string) {
  try {
    return new Date(ts).toLocaleString()
  }
  catch {
    return ts
  }
}

async function loadEntries() {
  loading.value = true
  try {
    const list = await invoke('list_mcp_history_entries', { limit: 500 }) as HistoryEntrySummary[]
    entries.value = list || []
    ensureDefaultExpandedGroups()
  }
  catch (err) {
    console.error('加载历史记录失败:', err)
    const text = String(err)
    if (text.includes('not found') && text.includes('list_mcp_history_entries')) {
      message.error('加载失败：后端命令不存在。你大概率运行的是旧版 sanshu-ui（前端更新了但后端没重编译/没重启）。请重新编译并重启三术。')
    } else {
      message.error(`加载历史记录失败: ${err}`)
    }
  }
  finally {
    loading.value = false
  }
}

async function openEntry(id: string) {
  const entry = entries.value.find(e => e.id === id)
  if (entry) {
    const key = getLocalDayKey(new Date(entry.timestamp))
    if (!expandedGroups.value.includes(key))
      expandedGroups.value = [...expandedGroups.value, key]
  }

  selectedId.value = id
  detailLoading.value = true
  try {
    const d = await invoke('get_mcp_history_entry', { id }) as HistoryEntryDetail
    selectedDetail.value = d
  }
  catch (err) {
    console.error('加载历史详情失败:', err)
    message.error(`加载历史详情失败: ${err}`)
  }
  finally {
    detailLoading.value = false
  }
}

watch(() => search.value, (v) => {
  if (v.trim())
    expandAllGroups()
  else
    ensureDefaultExpandedGroups()
})

async function deleteSelected() {
  if (!selectedId.value)
    return

  try {
    await invoke('delete_mcp_history_entry', { id: selectedId.value })
    message.success('已删除')
    selectedId.value = ''
    selectedDetail.value = null
    await loadEntries()
  }
  catch (err) {
    console.error('删除失败:', err)
    message.error(`删除失败: ${err}`)
  }
}

async function exportSelectedZip() {
  if (!selectedId.value)
    return

  try {
    const path = await invoke('export_mcp_history_entry_zip', { id: selectedId.value }) as string
    message.success(`已导出: ${path}`)
  }
  catch (err) {
    console.error('导出失败:', err)
    message.error(`导出失败: ${err}`)
  }
}

onMounted(async () => {
  await loadEntries()
})
</script>

<template>
  <div class="max-w-6xl mx-auto tab-content p-4">
    <n-space vertical size="large">
      <div class="flex flex-wrap items-center gap-3">
        <n-input
          v-model:value="search"
          size="small"
          placeholder="搜索：内容/时间/request_id/source"
          clearable
          class="w-64"
        />

        <n-date-picker
          v-model:value="range"
          size="small"
          type="datetimerange"
          clearable
          class="w-72"
          placeholder="选择时间范围"
        />

        <n-button size="small" secondary :loading="loading" @click="loadEntries">
          刷新
        </n-button>

        <n-button size="small" secondary :disabled="!range" @click="exportRangeZip">
          按时间段导出 ZIP
        </n-button>
        <n-button size="small" type="error" secondary :disabled="!range" @click="deleteRange">
          按时间段删除
        </n-button>

        <div class="flex-1" />
        <n-button size="small" secondary :disabled="!selectedId" @click="exportSelectedZip">
          导出 ZIP
        </n-button>
        <n-button size="small" type="error" secondary :disabled="!selectedId" @click="deleteSelected">
          删除
        </n-button>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <n-card size="small" class="md:col-span-1" :bordered="false">
          <template #header>
            历史列表
          </template>
          <template #header-extra>
            <n-space size="small">
              <n-button size="tiny" secondary :disabled="groupedEntries.length === 0" @click="expandAllGroups">
                展开全部
              </n-button>
              <n-button size="tiny" secondary :disabled="groupedEntries.length === 0" @click="collapseAllGroups">
                折叠全部
              </n-button>
            </n-space>
          </template>
          <div class="h-[70vh] overflow-auto pr-1">
            <n-spin :show="loading">
              <div v-if="filteredEntries.length === 0" class="text-sm opacity-60">
                暂无历史记录
              </div>
              <div v-else>
                <n-collapse v-model:expanded-names="expandedGroups" :accordion="false">
                  <n-collapse-item
                    v-for="g in groupedEntries"
                    :key="g.key"
                    :name="g.key"
                    :title="`${g.label} (${g.entries.length})`"
                  >
                    <div class="space-y-2 pt-2">
                      <div
                        v-for="e in g.entries"
                        :key="e.id"
                        class="p-3 rounded-lg border border-surface-200 dark:border-surface-700 cursor-pointer"
                        :class="selectedId === e.id ? 'bg-surface-100 dark:bg-surface-800' : ''"
                        @click="openEntry(e.id)"
                      >
                        <div class="text-xs opacity-60 mb-1">
                          {{ formatTime(e.timestamp) }}
                        </div>
                        <div class="text-sm font-medium break-words preview-line-clamp">
                          {{ e.preview || '(无标题)' }}
                        </div>
                        <div class="text-xs opacity-60 mt-1 break-all">
                          {{ e.source || '' }} {{ e.request_id || '' }}
                        </div>
                      </div>
                    </div>
                  </n-collapse-item>
                </n-collapse>
              </div>
            </n-spin>
          </div>
        </n-card>

        <n-card size="small" class="md:col-span-2" :bordered="false">
          <template #header>
            预览
          </template>

          <div class="h-[70vh] overflow-auto pr-1">
            <n-spin :show="detailLoading">
              <div v-if="!selectedDetail" class="text-sm opacity-60">
                点击左侧一条记录查看详情
              </div>
              <div v-else>
                <div class="text-xs opacity-60 mb-2">
                  {{ selectedDetail.summary.id }}
                </div>
                <div class="markdown-content" v-html="renderedMarkdown" />
              </div>
            </n-spin>
          </div>
        </n-card>
      </div>
    </n-space>
  </div>
</template>

<style scoped>
.markdown-content {
  user-select: text;
  -webkit-user-select: text;
}

.preview-line-clamp {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
