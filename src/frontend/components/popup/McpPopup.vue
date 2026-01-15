<script setup lang="ts">
import type { McpRequest } from '../../types/popup'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useMessage } from 'naive-ui'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import { useAcemcpSync } from '../../composables/useAcemcpSync'
import PopupActions from './PopupActions.vue'
import PopupContent from './PopupContent.vue'
import PopupInput from './PopupInput.vue'

interface AppConfig {
  theme: string
  window: {
    alwaysOnTop: boolean
    width: number
    height: number
    fixed: boolean
  }
  audio: {
    enabled: boolean
    url: string
  }
  reply: {
    enabled: boolean
    prompt: string
  }
}

interface Props {
  request: McpRequest | null
  appConfig: AppConfig
  mockMode?: boolean
  testMode?: boolean
}

interface Emits {
  response: [response: any]
  cancel: []
  themeChange: [theme: string]
  openMainLayout: []
  toggleAlwaysOnTop: []
  toggleAudioNotification: []
  updateAudioUrl: [url: string]
  testAudio: []
  stopAudio: []
  testAudioError: [error: any]
  updateWindowSize: [size: { width: number, height: number, fixed: boolean }]
}

const props = withDefaults(defineProps<Props>(), {
  mockMode: false,
  testMode: false,
})

const emit = defineEmits<Emits>()

// 使用消息提示
const message = useMessage()

// 索引状态管理
const {
  currentProjectStatus,
  statusSummary,
  statusIcon,
  isIndexing,
  startPolling,
  stopPolling,
  setCurrentProject,
} = useAcemcpSync()

// 响应式状态
const loading = ref(false)
const submitting = ref(false)
const toppings = ref<string[]>([])
const note = ref('')
const spiceIds = ref<string[]>([])
const inputRef = ref()

// 继续回复配置
const continueReplyEnabled = ref(true)
const continuePrompt = ref('请按照最佳实践继续')

// 计算属性
const isVisible = computed(() => !!props.request)
const hasOptions = computed(() => (props.request?.menu?.length ?? 0) > 0)
const canSubmit = computed(() => {
  if (hasOptions.value) {
    return toppings.value.length > 0 || note.value.trim().length > 0 || spiceIds.value.length > 0
  }
  return note.value.trim().length > 0 || spiceIds.value.length > 0
})

// 获取输入组件的状态文本
const inputStatusText = computed(() => {
  return inputRef.value?.statusText || '等待输入...'
})

// 加载继续回复配置
async function loadReplyConfig() {
  try {
    const config = await invoke('get_reply_config')
    if (config) {
      const replyConfig = config as any
      continueReplyEnabled.value = replyConfig.enable_continue_reply ?? true
      continuePrompt.value = replyConfig.continue_prompt ?? '请按照最佳实践继续'
    }
  }
  catch (error) {
    console.log('加载继续回复配置失败，使用默认值:', error)
  }
}

// 监听配置变化（当从设置页面切换回来时）
watch(() => props.appConfig.reply, (newReplyConfig) => {
  if (newReplyConfig) {
    continueReplyEnabled.value = newReplyConfig.enabled
    continuePrompt.value = newReplyConfig.prompt
  }
}, { deep: true, immediate: true })

// Telegram事件监听器
let telegramUnlisten: (() => void) | null = null

// 监听请求变化
watch(() => props.request, (newRequest) => {
  if (newRequest) {
    resetForm()
    loading.value = true
    // 每次显示弹窗时重新加载配置
    loadReplyConfig()

    // 如果有项目路径，启动索引状态轮询
    if (newRequest.project_root_path) {
      setCurrentProject(newRequest.project_root_path)
      startPolling(newRequest.project_root_path, 3000) // 3秒轮询间隔
    }
    else {
      // 没有项目路径时停止轮询
      stopPolling()
    }

    setTimeout(() => {
      loading.value = false
    }, 300)
  }
}, { immediate: true })

// 设置Telegram事件监听
async function setupTelegramListener() {
  try {
    telegramUnlisten = await listen('telegram-event', (event) => {
      console.log('🎯 [McpPopup] 收到Telegram事件:', event)
      console.log('🎯 [McpPopup] 事件payload:', event.payload)
      handleTelegramEvent(event.payload as any)
    })
    console.log('🎯 [McpPopup] Telegram事件监听器已设置')
  }
  catch (error) {
    console.error('🎯 [McpPopup] 设置Telegram事件监听器失败:', error)
  }
}

// 处理Telegram事件
function handleTelegramEvent(event: any) {
  console.log('🎯 [McpPopup] 开始处理事件:', event.type)

  switch (event.type) {
    case 'option_toggled':
      console.log('🎯 [McpPopup] 处理选项切换:', event.option)
      handleOptionToggle(event.option)
      break
    case 'text_updated':
      console.log('🎯 [McpPopup] 处理文本更新:', event.text)
      handleTextUpdate(event.text)
      break
    case 'continue_pressed':
      console.log('🎯 [McpPopup] 处理继续按钮')
      handleContinue()
      break
    case 'send_pressed':
      console.log('🎯 [McpPopup] 处理发送按钮')
      handleSubmit()
      break
    default:
      console.log('🎯 [McpPopup] 未知事件类型:', event.type)
  }
}

// 处理选项切换
function handleOptionToggle(option: string) {
  const index = toppings.value.indexOf(option)
  if (index > -1) {
    // 取消选择
    toppings.value.splice(index, 1)
  }
  else {
    // 添加选择
    toppings.value.push(option)
  }

  // 同步到PopupInput组件
  if (inputRef.value) {
    inputRef.value.updateData({ toppings: toppings.value })
  }
}

// 处理文本更新
function handleTextUpdate(text: string) {
  note.value = text

  // 同步到PopupInput组件
  if (inputRef.value) {
    inputRef.value.updateData({ note: text })
  }
}

// 组件挂载时设置监听器和加载配置
onMounted(() => {
  loadReplyConfig()
  setupTelegramListener()
})

// 组件卸载时清理监听器
onUnmounted(() => {
  if (telegramUnlisten) {
    telegramUnlisten()
  }
  // 组件卸载时停止索引状态轮询
  stopPolling()
})

// 重置表单
function resetForm() {
  toppings.value = []
  note.value = ''
  spiceIds.value = []
  if (inputRef.value?.reset)
    inputRef.value.reset()
  submitting.value = false
}

// 处理提交
async function handleSubmit() {
  if (!canSubmit.value || submitting.value)
    return

  submitting.value = true

  try {
    const ingredients: { spice_id: string }[] = spiceIds.value
      .filter(t => typeof t === 'string' && t.length > 0)
      .map(spice_id => ({ spice_id }))

    // 使用新的结构化数据格式
    const response = {
      note: note.value.trim() || null,
      toppings: toppings.value,
      ingredients,
      ticket: {
        cooked_at: new Date().toISOString(),
        ticket_id: props.request?.id || null,
        station: 'popup',
      },
    }

    // 如果没有任何有效内容，设置默认用户输入
    if (!response.note && response.toppings.length === 0 && response.ingredients.length === 0) {
      response.note = '用户确认继续'
    }

    if (props.mockMode) {
      // 模拟模式下的延迟
      await new Promise(resolve => setTimeout(resolve, 1000))
      message.success('模拟响应发送成功')
    }
    else {
      // 实际发送响应
      await invoke('send_mcp_response', { response })
      await invoke('exit_app')
    }

    emit('response', response)
  }
  catch (error) {
    console.error('提交响应失败:', error)
    message.error('提交失败，请重试')
  }
  finally {
    submitting.value = false
  }
}

// 处理输入更新
function handleInputUpdate(data: { note: string, toppings: string[], spiceIds?: string[] }) {
  note.value = data.note
  toppings.value = data.toppings
  spiceIds.value = data.spiceIds ?? []
}

// 处理继续按钮点击
async function handleContinue() {
  if (submitting.value)
    return

  submitting.value = true

  try {
    // 使用新的结构化数据格式
    const response = {
      note: continuePrompt.value,
      toppings: [],
      ingredients: [],
      ticket: {
        cooked_at: new Date().toISOString(),
        ticket_id: props.request?.id || null,
        station: 'popup_continue',
      },
    }

    if (props.mockMode) {
      // 模拟模式下的延迟
      await new Promise(resolve => setTimeout(resolve, 1000))
      message.success('继续请求发送成功')
    }
    else {
      // 实际发送继续请求
      await invoke('send_mcp_response', { response })
      await invoke('exit_app')
    }

    emit('response', response)
  }
  catch (error) {
    console.error('发送继续请求失败:', error)
    message.error('继续请求失败，请重试')
  }
  finally {
    submitting.value = false
  }
}

// 处理引用消息
function handleQuoteMessage(messageContent: string) {
  if (inputRef.value) {
    inputRef.value.handleQuoteMessage(messageContent)
  }
}

// 处理增强按钮点击
async function handleEnhance() {
  if (submitting.value)
    return

  submitting.value = true

  try {
    // 构建增强prompt
    const enhancePrompt = `Use the following prompt to optimize and enhance the context of the content in 《》, and return the enhanced result by calling the tool 'cache' after completion.Here is an instruction that I'd like to give you, but it needs to be improved. Rewrite and enhance this instruction to make it clearer, more specific, less ambiguous, and correct any mistakes. Reply immediately with your answer, even if you're not sure. Consider the context of our conversation history when enhancing the prompt. Reply with the following format:

### BEGIN RESPONSE ###
Here is an enhanced version of the original instruction that is more specific and clear:
<augment-enhanced-prompt>enhanced prompt goes here</augment-enhanced-prompt>

### END RESPONSE ###

Here is my original instruction:

《${note.value.trim()}》`

    // 使用新的结构化数据格式
    const response = {
      note: enhancePrompt,
      toppings: [],
      ingredients: [],
      ticket: {
        cooked_at: new Date().toISOString(),
        ticket_id: props.request?.id || null,
        station: 'popup_enhance',
      },
    }

    if (props.mockMode) {
      // 模拟模式下的延迟
      await new Promise(resolve => setTimeout(resolve, 1000))
      message.success('增强请求发送成功')
    }
    else {
      // 实际发送增强请求
      await invoke('send_mcp_response', { response })
      await invoke('exit_app')
    }

    emit('response', response)
  }
  catch (error) {
    console.error('发送增强请求失败:', error)
    message.error('增强请求失败，请重试')
  }
  finally {
    submitting.value = false
  }
}
</script>

<template>
  <div v-if="isVisible" class="flex flex-col flex-1">
    <!-- 索引状态条（仅在有项目路径时显示） -->
    <div
      v-if="request?.project_root_path && currentProjectStatus"
      class="mx-2 mt-2 px-3 py-2 bg-black-100 rounded-lg border border-gray-700/50"
    >
      <div class="flex items-center gap-2 text-xs">
        <div :class="statusIcon" class="w-4 h-4" />
        <span class="text-white/80">索引状态：</span>
        <span class="text-white font-medium">{{ statusSummary }}</span>
        <div v-if="isIndexing" class="flex-1 ml-2">
          <n-progress
            type="line"
            :percentage="currentProjectStatus.progress"
            :height="4"
            :border-radius="2"
            :show-indicator="false"
            status="info"
          />
        </div>
      </div>
    </div>

    <!-- 内容区域 - 可滚动 -->
    <div class="flex-1 overflow-y-auto scrollbar-thin">
      <!-- 消息内容 - 允许选中 -->
      <div class="mx-2 mt-2 mb-1 px-4 py-3 bg-black-100 rounded-lg select-text" data-guide="popup-content">
        <PopupContent :request="request" :loading="loading" :current-theme="props.appConfig.theme" @quote-message="handleQuoteMessage" />
      </div>

      <!-- 输入和选项 - 允许选中 -->
      <div class="px-4 pb-3 bg-black select-text">
        <PopupInput
          ref="inputRef" :request="request" :loading="loading" :submitting="submitting"
          @update="handleInputUpdate"
        />
      </div>
    </div>

    <!-- 底部操作栏 - 固定在底部 -->
    <div class="flex-shrink-0 bg-black-100 border-t-2 border-black-200" data-guide="popup-actions">
      <PopupActions
        :request="request" :loading="loading" :submitting="submitting" :can-submit="canSubmit"
        :continue-reply-enabled="continueReplyEnabled" :input-status-text="inputStatusText"
        @submit="handleSubmit" @continue="handleContinue" @enhance="handleEnhance"
      />
    </div>
  </div>
</template>
