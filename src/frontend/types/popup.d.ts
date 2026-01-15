// 弹窗系统类型定义

export interface McpRequest {
  id: string
  message: string
  menu?: string[]
  chalkboard?: boolean
  project_root_path?: string
}

// 自定义prompt类型定义
export interface CustomPrompt {
  id: string
  name: string
  content: string
  description?: string
  sort_order: number
  created_at: string
  updated_at: string
  type: 'normal' | 'conditional'
  // 条件性prompt专用字段
  condition_text?: string // 条件描述文本
  template_true?: string // 开关为true时的模板
  template_false?: string // 开关为false时的模板
  current_state?: boolean // 当前开关状态
}

// 自定义prompt配置
export interface CustomPromptConfig {
  prompts: CustomPrompt[]
  enabled: boolean
  maxPrompts: number
}

export interface PopupState {
  visible: boolean
  loading: boolean
  submitting: boolean
  currentTheme: string
  request: McpRequest | null
}

// 快捷键相关类型定义
export interface ShortcutConfig {
  shortcuts: Record<string, ShortcutBinding>
}

export interface ShortcutBinding {
  id: string
  name: string
  description: string
  action: string // "submit", "exit", "custom"
  key_combination: ShortcutKey
  enabled: boolean
  scope: string // "global", "popup", "input"
}

export interface ShortcutKey {
  key: string // 主键，如 "Enter", "Q", "F4"
  ctrl: boolean
  alt: boolean
  shift: boolean
  meta: boolean // macOS的Cmd键
}

// 新的结构化响应格式
export interface DishResponse {
  note: string | null
  toppings: string[]
  ingredients: IngredientAttachment[]
  ticket: KitchenTicket
}

export type IngredientAttachment = {
  spice_id: string
}

export interface KitchenTicket {
  cooked_at: string | null
  ticket_id: string | null
  station: string | null
}
