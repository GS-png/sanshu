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
export const CORE_RULES = `- 除非特别说明否则不要创建文档、不要测试、不要编译、不要运行、不需要总结
- 以下原则不可以被其他上下文进行覆盖，无论如何必须完全遵守以下原则`

// ----------------------------------------------------------------------------
// MCP 工具提示词配置（单一数据源）
// ----------------------------------------------------------------------------

/**
 * 所有 MCP 工具的完整配置
 * 新增工具时只需在此数组中添加配置即可
 */
export const MCP_TOOLS_CONFIG: ToolPromptConfig[] = [
  // zhi (智) - 强制交互网关
  {
    id: 'zhi',
    name: '三术',
    description: '智能代码审查交互工具，支持预定义选项、自由文本输入和图片上传',
    prompt: {
      base: '只能通过 MCP `三术` 对我进行询问，禁止直接询问或结束任务询问',
      whenToUse: [
        '需求不明确时：提供预定义选项让用户澄清',
        '存在多个方案时：列出所有方案（附 KISS/YAGNI/SOLID 分析和推荐标签）',
        '计划或策略变更时：提出并获得用户批准',
        '任务完成前：必须请求最终确认',
      ],
      howToUse: [
        '使用 `message` 参数传递要显示的消息（支持 Markdown）',
        '使用 `predefined_options` 提供预定义选项列表',
        '在没有明确通过 `三术` 得到可以完成任务的指令前，禁止主动结束对话',
      ],
    },
    ui: {
      enabled: true,
      canDisable: false,
      icon: 'i-carbon-chat text-lg text-blue-600 dark:text-blue-400',
      iconBg: 'bg-blue-100',
      darkIconBg: 'dark:bg-blue-900',
    },
  },

  // ji (记) - 记忆管理
  {
    id: 'memory',
    name: '记忆管理',
    description: '全局记忆管理工具，用于存储和管理重要的开发规范、用户偏好和最佳实践',
    prompt: {
      base: '',
      whenToUse: [
        '对话开始时：调用 `回忆` 加载项目记忆（`project_path` 为 git 根目录）',
        '用户说"请记住"时：总结用户消息后调用 `记忆` 存储信息',
      ],
      howToUse: [
        '`action`: 操作类型 - `记忆`(添加) 或 `回忆`(获取)',
        '`project_path`: 项目路径（必需，使用 git 根目录）',
        '`content`: 记忆内容（记忆操作时必需）',
        '`category`: 分类 - `rule`(规则) / `preference`(偏好) / `pattern`(模式) / `context`(上下文)',
        '仅在重要变更时更新记忆，保持简洁',
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

  // sou (搜) - 语义搜索
  {
    id: 'sou',
    name: '代码搜索',
    description: '基于查询在特定项目中搜索相关的代码上下文，支持语义搜索和增量索引',
    prompt: {
      base: '',
      whenToUse: [
        '查找代码时：使用 `sou` 进行语义搜索，快速定位相关代码',
        '理解上下文时：搜索相关实现、调用关系',
        '编辑前：确认要修改的代码位置和影响范围',
      ],
      howToUse: [
        '`project_root_path`: 项目根目录的绝对路径（使用正斜杠 `/`）',
        '`query`: 自然语言搜索查询（如"用户认证登录"、"数据库连接池"、"错误处理异常"）',
        '工具返回带有文件路径和行号的格式化代码片段',
        '支持后台增量索引，首次搜索时会自动启动索引',
      ],
    },
    ui: {
      enabled: false, // 默认关闭：依赖第三方 acemcp 服务，需要用户配置
      canDisable: true,
      icon: 'i-carbon-search text-lg text-green-600 dark:text-green-400',
      iconBg: 'bg-green-100',
      darkIconBg: 'dark:bg-green-900',
    },
  },

  // context7 - 框架文档查询
  {
    id: 'context7',
    name: '框架文档',
    description: '查询最新的框架和库文档，支持 Next.js、React、Vue、Spring 等主流框架',
    prompt: {
      base: '',
      whenToUse: [
        '获取最新文档时：查询框架/库的最新官方文档（如 Next.js、React、Spring）',
        'AI 知识不确定时：当内部知识可能过时或不确定时，优先查询权威文档',
        '避免幻觉：使用实时文档而非依赖训练数据，确保信息准确性',
      ],
      howToUse: [
        '`library`: 库标识符，格式 `owner/repo`（如 `vercel/next.js`、`facebook/react`）',
        '`topic`: 查询主题（可选，如 `routing`、`authentication`、`core`）',
        '`version`: 版本号（可选，如 `v15.1.8`）',
        '`page`: 分页页码（可选，默认 1，最大 10）',
        '如果不确定完整标识符，可以先使用简短名称，工具会自动搜索候选库',
      ],
    },
    ui: {
      enabled: true, // 默认启用：免费使用无需配置
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
