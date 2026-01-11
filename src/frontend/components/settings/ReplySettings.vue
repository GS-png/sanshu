<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { onMounted, ref } from 'vue'

interface ReplyConfig {
  enable_continue_reply: boolean
  auto_continue_threshold: number
  continue_prompt: string
}

const localConfig = ref<ReplyConfig>({
  enable_continue_reply: true,
  auto_continue_threshold: 1000,
  continue_prompt: '请按照最佳实践继续',
})

const interactionWaitSeconds = ref(1800)

// 加载配置
async function loadConfig() {
  try {
    const config = await invoke('get_reply_config')
    localConfig.value = config as ReplyConfig

    const waitMs = await invoke('get_interaction_wait_ms')
    const ms = Number(waitMs)
    interactionWaitSeconds.value = Number.isFinite(ms) ? Math.max(0, Math.round(ms / 1000)) : 1800
  }
  catch (error) {
    console.error('加载继续回复配置失败:', error)
  }
}

// 更新配置
async function updateConfig() {
  try {
    await invoke('set_reply_config', { replyConfig: localConfig.value })
  }
  catch (error) {
    console.error('保存继续回复配置失败:', error)
  }
}

async function updateInteractionWaitSeconds() {
  try {
    const seconds = Math.max(0, Math.floor(interactionWaitSeconds.value || 0))
    await invoke('set_interaction_wait_ms', { waitMs: seconds * 1000 })
  }
  catch (error) {
    console.error('保存交互等待阈值失败:', error)
  }
}

onMounted(() => {
  loadConfig()
})
</script>

<template>
  <!-- 设置内容 -->
  <n-space vertical size="large">
    <!-- 启用继续回复 -->
    <div class="flex items-center justify-between">
      <div class="flex items-center">
        <div class="w-1.5 h-1.5 bg-info rounded-full mr-3 flex-shrink-0" />
        <div>
          <div class="text-sm font-medium leading-relaxed">
            启用继续回复
          </div>
          <div class="text-xs opacity-60">
            启用后将显示继续按钮
          </div>
        </div>
      </div>
      <n-switch
        v-model:value="localConfig.enable_continue_reply"
        size="small"
        @update:value="updateConfig"
      />
    </div>

    <!-- 继续提示词 -->
    <div v-if="localConfig.enable_continue_reply">
      <div class="flex items-center mb-3">
        <div class="w-1.5 h-1.5 bg-info rounded-full mr-3 flex-shrink-0" />
        <div>
          <div class="text-sm font-medium leading-relaxed">
            继续提示词
          </div>
          <div class="text-xs opacity-60">
            点击继续按钮时发送的提示词
          </div>
        </div>
      </div>
      <n-input
        v-model:value="localConfig.continue_prompt"
        size="small"
        placeholder="请按照最佳实践继续"
        @input="updateConfig"
      />
    </div>

    <div>
      <div class="flex items-center mb-3">
        <div class="w-1.5 h-1.5 bg-info rounded-full mr-3 flex-shrink-0" />
        <div>
          <div class="text-sm font-medium leading-relaxed">
            交互等待阈值（秒）
          </div>
          <div class="text-xs opacity-60">
            用于避免一次工具调用卡太久被 IDE 自动结束；建议 8-20，设为 0 表示不限制
          </div>
        </div>
      </div>
      <n-input-number
        v-model:value="interactionWaitSeconds"
        size="small"
        :min="0"
        :step="1"
        placeholder="例如 15"
        @update:value="updateInteractionWaitSeconds"
      />
    </div>
  </n-space>
</template>
