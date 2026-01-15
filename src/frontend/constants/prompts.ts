// ============================================================================
// 提示词配置模块
// 遵循 KISS/YAGNI/SOLID 原则，支持 MCP 工具的可扩展提示词管理
// ============================================================================

// ----------------------------------------------------------------------------
// 接口定义
// ----------------------------------------------------------------------------

/**
 * 工具提示词内容结构
 * 分离"何时使用"和"如何使用"，提高可读性和可维护性
 */
export interface ToolPrompt {
  /** 基础规范（加入核心规则区，简短的一行描述） */
  base: string
  /** 何时使用（场景列表） */
  whenToUse: string[]
  /** 如何使用（操作指南） */
  howToUse: string[]
}

/**
 * 工具 UI 配置
 */
export interface ToolUIConfig {
  /** 默认启用状态 */
  enabled: boolean
  /** 是否可禁用 */
  canDisable: boolean
  /** 图标类名 */
  icon: string
  /** 图标背景色 */
  iconBg: string
  /** 暗色模式背景色 */
  darkIconBg: string
}

/**
 * 完整的工具提示词配置
 * 统一管理工具的标识、描述、提示词和 UI 配置
 */
export interface ToolPromptConfig {
  /** 工具标识符（与后端一致） */
  id: string
  /** 工具显示名称 */
  name: string
  /** 工具简短描述 */
  description: string
  /** 提示词内容 */
  prompt: ToolPrompt
  /** UI 配置 */
  ui: ToolUIConfig
}

/**
 * 旧版提示词结构（向后兼容）
 */
export interface PromptSection {
  base: string
  detail: string
}

/**
 * 旧版 MCP 工具配置接口（向后兼容）
 */
export interface MCPToolConfig {
  id: string
  name: string
  description: string
  enabled: boolean
  canDisable: boolean
  icon: string
  iconBg: string
  darkIconBg: string
}

// ----------------------------------------------------------------------------
// 核心规范
// ----------------------------------------------------------------------------

/**
 * 核心规范（基础交互规范）
 * 这些规则不可被其他上下文覆盖
 */
export const CORE_RULES = `
# Core Contract
1. **Code Quality**: Follow KISS / YAGNI / SOLID principles
2. **Output Control**: All responses must go through \`cache\` tool
3. **Data Store**: Load \`store\` data at start, save when requested
4. **Documentation**: Query \`docs\` for up-to-date library references
5. **Context Aware**: Understand project structure, dependencies and stack
6. **Silent Mode**: No doc creation, no tests, no builds, no runs

---
`

// ----------------------------------------------------------------------------
// MCP 工具提示词配置（单一数据源）
// ----------------------------------------------------------------------------

/**
 * 所有 MCP 工具的完整配置
 * 新增工具时只需在此数组中添加配置即可
 */
export const MCP_TOOLS_CONFIG: ToolPromptConfig[] = [
  // cache - build cache tool
  {
    id: 'cache',
    name: 'Cache',
    description: 'Interactive task: 1) call cache to start, get task_id 2) call cache_get to read',
    prompt: {
      base: '',
      whenToUse: [
        'When requirements unclear: provide options for clarification',
        'Multiple approaches: list all with analysis',
        'Plan changes: propose and get approval',
        'Before completion: request final confirmation',
      ],
      howToUse: [
        'Two-step process:',
        '1. Call cache to start interactive task, get task_id',
        '2. After confirmation, call cache_get(task_id)',
        'Do NOT auto-poll cache_get',
      ],
    },
    ui: {
      enabled: true,
      canDisable: false,
      icon: 'i-carbon-data-backup text-lg text-blue-600 dark:text-blue-400',
      iconBg: 'bg-blue-100',
      darkIconBg: 'dark:bg-blue-900',
    },
  },

  // store - key-value storage
  {
    id: 'store',
    name: 'Store',
    description: 'Key-value storage for project configuration and metadata',
    prompt: {
      base: '',
      whenToUse: [
        'At start: load project config with recall action',
        'When requested: save with store action',
      ],
      howToUse: [
        'Use git root for project_path',
        'Update only on important changes, keep concise',
      ],
    },
    ui: {
      enabled: true,
      canDisable: true,
      icon: 'i-carbon-data-base text-lg text-purple-600 dark:text-purple-400',
      iconBg: 'bg-purple-100',
      darkIconBg: 'dark:bg-purple-900',
    },
  },

  // index - code indexing
  {
    id: 'index',
    name: 'Index',
    description: 'Code indexing for fast file lookup and search',
    prompt: {
      base: '',
      whenToUse: [
        'Finding code: semantic search for quick location',
        'Understanding context: search related implementations',
      ],
      howToUse: [
        'Use absolute path and natural language query',
      ],
    },
    ui: {
      enabled: false,
      canDisable: true,
      icon: 'i-carbon-search text-lg text-green-600 dark:text-green-400',
      iconBg: 'bg-green-100',
      darkIconBg: 'dark:bg-green-900',
    },
  },

  // docs - documentation lookup
  {
    id: 'docs',
    name: 'Docs',
    description: 'Documentation lookup for libraries and frameworks',
    prompt: {
      base: '',
      whenToUse: [
        'Getting latest docs: query official documentation',
        'When uncertain: prefer authoritative docs over guessing',
      ],
      howToUse: [
        'Library format: owner/repo (e.g. vercel/next.js)',
        'Short names work too, tool will search',
      ],
    },
    ui: {
      enabled: true,
      canDisable: true,
      icon: 'i-carbon-document text-lg text-orange-600 dark:text-orange-400',
      iconBg: 'bg-orange-100',
      darkIconBg: 'dark:bg-orange-900',
    },
  },
]

// ----------------------------------------------------------------------------
// 提示词生成函数
// ----------------------------------------------------------------------------

/**
 * 根据工具配置生成完整提示词
 * @param tools 工具配置列表
 * @returns 格式化的完整提示词
 */
export function generateFullPromptFromConfig(tools: ToolPromptConfig[]): string {
  const enabledTools = tools.filter(t => t.ui.enabled)
  const parts: string[] = []

  // 1. 核心规范
  parts.push(CORE_RULES)

  // 2. 基础规范（紧凑连接到核心规范）
  const baseParts = enabledTools
    .map(t => t.prompt.base)
    .filter(Boolean)
    .map(b => `- ${b}`)

  if (baseParts.length > 0)
    parts[0] = `${parts[0]}\n${baseParts.join('\n')}`

  // 3. 工具使用细节（按工具分组，结构化输出）
  const toolDetails: string[] = []
  for (const tool of enabledTools) {
    const { whenToUse, howToUse } = tool.prompt
    // 跳过没有使用指南的工具
    if (whenToUse.length === 0 && howToUse.length === 0)
      continue

    const lines: string[] = []
    lines.push(`### ${tool.name} (${tool.id})`)

    if (whenToUse.length > 0) {
      lines.push('**何时使用：**')
      lines.push(...whenToUse.map(s => `- ${s}`))
    }

    if (howToUse.length > 0) {
      lines.push('**如何使用：**')
      lines.push(...howToUse.map(s => `- ${s}`))
    }

    toolDetails.push(lines.join('\n'))
  }

  if (toolDetails.length > 0) {
    parts.push('## 工具使用指南\n')
    parts.push(toolDetails.join('\n\n'))
  }

  return parts.join('\n\n')
}

/**
 * 生成完整提示词（兼容旧版 MCPToolConfig 接口）
 * @param mcpTools 旧版工具配置列表
 * @returns 格式化的完整提示词
 */
export function generateFullPrompt(mcpTools: MCPToolConfig[]): string {
  // 将旧版配置映射到新版配置
  const toolsWithPrompt: ToolPromptConfig[] = []

  for (const tool of mcpTools) {
    // eslint-disable-next-line ts/ban-ts-comment
    // @ts-expect-error
    const config = MCP_TOOLS_CONFIG.find(t => t.id === tool.id)
    if (config) {
      toolsWithPrompt.push({
        ...config,
        ui: {
          ...config.ui,
          enabled: tool.enabled, // 使用传入的启用状态
        },
      })
    }
    else {
      // 未找到配置的工具，返回空提示词
      toolsWithPrompt.push({
        id: tool.id,
        name: tool.name,
        description: tool.description,
        prompt: { base: '', whenToUse: [], howToUse: [] },
        ui: {
          enabled: tool.enabled,
          canDisable: tool.canDisable,
          icon: tool.icon,
          iconBg: tool.iconBg,
          darkIconBg: tool.darkIconBg,
        },
      })
    }
  }

  return generateFullPromptFromConfig(toolsWithPrompt)
}

// ----------------------------------------------------------------------------
// 向后兼容导出
// ----------------------------------------------------------------------------

/**
 * 从新配置派生的旧版 PROMPT_SECTIONS
 * 保持向后兼容性
 */
export const PROMPT_SECTIONS: Record<string, PromptSection> = MCP_TOOLS_CONFIG.reduce(
  (acc, tool) => {
    const { whenToUse, howToUse } = tool.prompt

    // 构建 detail 字符串
    const detailParts: string[] = [
      ...whenToUse.map(s => `- ${s}`),
      ...howToUse.map(s => `- ${s}`),
    ]

    acc[tool.id] = {
      base: tool.prompt.base ? `- ${tool.prompt.base}` : '',
      detail: detailParts.length > 0
        ? `${tool.name}工具使用细节：\n${detailParts.join('\n')}`
        : '',
    }
    return acc
  },
  {} as Record<string, PromptSection>,
)

/**
 * 从新配置派生的旧版 DEFAULT_MCP_TOOLS
 * 保持向后兼容性
 */
export const DEFAULT_MCP_TOOLS: MCPToolConfig[] = MCP_TOOLS_CONFIG.map(tool => ({
  id: tool.id,
  name: tool.name,
  description: tool.description,
  enabled: tool.ui.enabled,
  canDisable: tool.ui.canDisable,
  icon: tool.ui.icon,
  iconBg: tool.ui.iconBg,
  darkIconBg: tool.ui.darkIconBg,
}))

/**
 * 默认的完整提示词
 * 使用默认工具配置生成
 */
export const REFERENCE_PROMPT = generateFullPromptFromConfig(MCP_TOOLS_CONFIG)
