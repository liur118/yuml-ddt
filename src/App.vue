<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import ActivityBar from './components/ActivityBar.vue'
import FileTree from './components/FileTree.vue'
import StepList from './components/StepList.vue'
import YamlEditor from './components/YamlEditor.vue'
import ResultPanel from './components/ResultPanel.vue'

interface StepInfo {
  id: string
  name: string
  method: string
  path: string
}

interface ExecutionResult {
  success: boolean
  step_name: string
  request_url: string
  request_method: string
  request_headers: Record<string, string>
  request_body: any
  response_status: number
  response_headers: Record<string, string>
  response_body: any
  validations: Array<{
    field: string
    operator: string
    expected: string
    actual: string
    passed: boolean
  }>
  duration_ms: number
  error: string | null
}

interface RecentWorkspace {
  path: string
  name: string
  last_opened: string
}

// 编辑器引用
const yamlEditorRef = ref<InstanceType<typeof YamlEditor> | null>(null)

// 状态
const activeView = ref<string>('files')
const workspacePath = ref<string>('')
const yamlFiles = ref<string[]>([])
const currentFile = ref<string>('')
const fileContent = ref<string>('')
const steps = ref<StepInfo[]>([])
const parseError = ref<string | null>(null)
const executionResult = ref<ExecutionResult | null>(null)
const isExecuting = ref(false)
const hasChanges = ref(false)
const recentWorkspaces = ref<RecentWorkspace[]>([])

// 初始化 - 加载最近工作区
onMounted(async () => {
  await loadRecentWorkspaces()
})

// 加载最近工作区列表
async function loadRecentWorkspaces() {
  try {
    recentWorkspaces.value = await invoke('get_recent_workspaces')
  } catch (e) {
    console.error('加载最近工作区失败:', e)
  }
}

// 打开目录
async function openWorkspace(path?: string) {
  let selectedPath = path
  
  if (!selectedPath) {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择测试用例目录'
    })
    if (selected) {
      selectedPath = selected as string
    }
  }
  
  if (selectedPath) {
    workspacePath.value = selectedPath
    await loadYamlFiles()
    // 保存到最近工作区
    try {
      const result = await invoke<{ recent_workspaces: RecentWorkspace[] }>('add_recent_workspace', { path: selectedPath })
      recentWorkspaces.value = result.recent_workspaces
    } catch (e) {
      console.error('保存最近工作区失败:', e)
    }
  }
}

// 移除最近工作区
async function removeRecentWorkspace(path: string, event: Event) {
  event.stopPropagation()
  try {
    const result = await invoke<{ recent_workspaces: RecentWorkspace[] }>('remove_recent_workspace', { path })
    recentWorkspaces.value = result.recent_workspaces
  } catch (e) {
    console.error('移除最近工作区失败:', e)
  }
}

// 清空最近工作区
async function clearRecentWorkspaces() {
  try {
    await invoke('clear_recent_workspaces')
    recentWorkspaces.value = []
  } catch (e) {
    console.error('清空最近工作区失败:', e)
  }
}

// 加载 YAML 文件列表
async function loadYamlFiles() {
  if (!workspacePath.value) return
  
  try {
    yamlFiles.value = await invoke('list_yaml_files', { 
      directory: workspacePath.value 
    })
  } catch (e) {
    console.error('加载文件列表失败:', e)
  }
}

// 打开文件
async function openFile(filePath: string) {
  if (hasChanges.value) {
    // TODO: 提示保存
  }
  
  try {
    currentFile.value = filePath
    fileContent.value = await invoke('read_yaml_file', { filePath })
    await parseSteps()
    hasChanges.value = false
  } catch (e) {
    console.error('读取文件失败:', e)
  }
}

// 解析 steps
async function parseSteps() {
  if (!fileContent.value) {
    steps.value = []
    parseError.value = null
    return
  }
  
  try {
    steps.value = await invoke('parse_yaml_steps', { 
      content: fileContent.value 
    })
    parseError.value = null
  } catch (e) {
    console.error('解析 steps 失败:', e)
    steps.value = []
    parseError.value = String(e)
  }
}

// 保存文件
async function saveFile() {
  if (!currentFile.value) return
  
  try {
    await invoke('save_yaml_file', {
      filePath: currentFile.value,
      content: fileContent.value
    })
    hasChanges.value = false
    await parseSteps()
  } catch (e) {
    console.error('保存文件失败:', e)
  }
}

// 执行 step
async function executeStep(stepId: string) {
  if (!currentFile.value || isExecuting.value) return
  
  // 如果有修改，先保存
  if (hasChanges.value) {
    await saveFile()
  }
  
  isExecuting.value = true
  executionResult.value = null
  
  try {
    executionResult.value = await invoke('execute_step', {
      filePath: currentFile.value,
      stepName: stepId,
      variables: null
    })
  } catch (e) {
    console.error('执行失败:', e)
    executionResult.value = {
      success: false,
      step_name: stepId,
      request_url: '',
      request_method: '',
      request_headers: {},
      request_body: null,
      response_status: 0,
      response_headers: {},
      response_body: null,
      validations: [],
      duration_ms: 0,
      error: String(e)
    }
  } finally {
    isExecuting.value = false
  }
}

// 内容变化
function onContentChange(content: string) {
  fileContent.value = content
  hasChanges.value = true
  parseSteps()
}

// 文件名
const currentFileName = computed(() => {
  if (!currentFile.value) return ''
  return currentFile.value.split('/').pop() || ''
})

// 关闭文件
function closeFile() {
  currentFile.value = ''
  fileContent.value = ''
  steps.value = []
  parseError.value = null
  hasChanges.value = false
  executionResult.value = null
}

// 聚焦到 step 对应行
function focusStep(stepId: string) {
  yamlEditorRef.value?.focusStep(stepId)
}

// 结果面板宽度
const resultPaneWidth = ref(400)
const isResizing = ref(false)
const minResultWidth = 200
const maxResultWidth = 800

function startResize(_e: MouseEvent) {
  isResizing.value = true
  document.addEventListener('mousemove', handleResize)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
}

function handleResize(e: MouseEvent) {
  if (!isResizing.value) return
  const container = document.querySelector('.editor-content')
  if (!container) return
  const containerRect = container.getBoundingClientRect()
  const newWidth = containerRect.right - e.clientX
  resultPaneWidth.value = Math.max(minResultWidth, Math.min(maxResultWidth, newWidth))
}

function stopResize() {
  isResizing.value = false
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}

// 快捷键
function handleKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === 's') {
    e.preventDefault()
    saveFile()
  }
}
</script>

<template>
  <div class="app" @keydown="handleKeydown">
    <!-- 主容器 -->
    <div class="main-container">
      <!-- 活动栏 - 最左侧 -->
      <ActivityBar v-model:activeView="activeView" />

      <!-- 侧边栏 -->
      <aside v-if="activeView" class="sidebar">
        <!-- 文件视图 -->
        <template v-if="activeView === 'files'">
          <div class="sidebar-header">
            <span>资源管理器</span>
            <button class="icon-btn" @click="openWorkspace()" title="打开目录">📂</button>
          </div>
          <div v-if="workspacePath" class="workspace-info">
            <span class="workspace-name">{{ workspacePath.split('/').pop() }}</span>
          </div>
          <FileTree 
            :files="yamlFiles"
            :currentFile="currentFile"
            :workspacePath="workspacePath"
            @select="openFile"
          />
        </template>

        <!-- 测试视图 -->
        <template v-if="activeView === 'tests'">
          <div class="sidebar-header">
            <span>测试用例</span>
          </div>
          <StepList 
            :steps="steps"
            :isExecuting="isExecuting"
            @execute="executeStep"
            @focus-step="focusStep"
          />
        </template>

        <!-- 设置视图 -->
        <template v-if="activeView === 'settings'">
          <div class="sidebar-header">
            <span>设置</span>
          </div>
          <div class="settings-content">
            <div class="setting-item">
              <label>工作目录</label>
              <div class="setting-value">{{ workspacePath || '未选择' }}</div>
            </div>
          </div>
        </template>
      </aside>

      <!-- 编辑区域 -->
      <main class="editor-area">
        <!-- 标签栏 -->
        <div class="tab-bar" v-if="currentFile">
          <div class="tab active">
            <span class="tab-name">{{ currentFileName }}</span>
            <span v-if="hasChanges" class="unsaved">●</span>
            <button class="tab-close" @click="closeFile">×</button>
          </div>
          <div class="tab-actions">
            <button 
              class="icon-btn"
              @click="saveFile"
              :disabled="!hasChanges"
              title="保存 (⌘S)"
            >
              💾
            </button>
          </div>
        </div>

        <!-- 欢迎页面 -->
        <div v-if="!currentFile" class="welcome">
          <div class="welcome-content">
            <h1>YUML-DDT</h1>
            <p>YAML 数据驱动测试工具</p>
            <div class="welcome-actions">
              <button @click="openWorkspace()">📂 打开目录</button>
            </div>
            
            <!-- 最近工作区 -->
            <div v-if="recentWorkspaces.length > 0" class="recent-section">
              <div class="recent-header">
                <span>最近打开</span>
                <button class="clear-btn" @click="clearRecentWorkspaces" title="清空列表">清空</button>
              </div>
              <div class="recent-list">
                <div 
                  v-for="workspace in recentWorkspaces"
                  :key="workspace.path"
                  class="recent-item"
                  @click="openWorkspace(workspace.path)"
                >
                  <span class="recent-icon">📁</span>
                  <div class="recent-info">
                    <span class="recent-name">{{ workspace.name }}</span>
                    <span class="recent-path">{{ workspace.path }}</span>
                  </div>
                  <button 
                    class="recent-remove" 
                    @click="removeRecentWorkspace(workspace.path, $event)"
                    title="移除"
                  >×</button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 编辑器 + 结果面板 -->
        <div v-else class="editor-content">
          <div class="editor-pane">
            <YamlEditor 
              ref="yamlEditorRef"
              :content="fileContent"
              :steps="steps"
              :parse-error="parseError"
              @change="onContentChange"
              @execute="executeStep"
            />
          </div>
          <div 
            class="resize-handle" 
            @mousedown="startResize"
            :class="{ active: isResizing }"
          ></div>
          <div class="result-pane" :style="{ width: resultPaneWidth + 'px' }">
            <div class="pane-header">
              <span>执行结果</span>
              <span v-if="isExecuting" class="loading">⏳ 执行中...</span>
            </div>
            <ResultPanel :result="executionResult" />
          </div>
        </div>
      </main>
    </div>

    <!-- 状态栏 -->
    <footer class="status-bar">
      <div class="status-left">
        <span v-if="currentFile">{{ currentFile }}</span>
      </div>
      <div class="status-right">
        <span>YAML</span>
        <span>UTF-8</span>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background: var(--bg-primary);
}

.main-container {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* 侧边栏 */
.sidebar {
  width: 260px;
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  overflow: hidden;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border);
}

.workspace-info {
  padding: 8px 12px;
  border-bottom: 1px solid var(--border);
}

.workspace-name {
  font-size: 13px;
  font-weight: 500;
}

.icon-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 4px;
  font-size: 14px;
  opacity: 0.7;
  transition: opacity 0.15s;
}

.icon-btn:hover {
  opacity: 1;
}

.icon-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

/* 设置内容 */
.settings-content {
  padding: 12px;
}

.setting-item {
  margin-bottom: 16px;
}

.setting-item label {
  display: block;
  font-size: 11px;
  color: var(--text-secondary);
  margin-bottom: 4px;
  text-transform: uppercase;
}

.setting-value {
  font-size: 13px;
  color: var(--text-primary);
  word-break: break-all;
}

/* 编辑区域 */
.editor-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 标签栏 */
.tab-bar {
  display: flex;
  align-items: center;
  height: 35px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border);
}

.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 100%;
  padding: 0 12px;
  font-size: 13px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  cursor: pointer;
}

.tab.active {
  background: var(--bg-primary);
  border-bottom: 1px solid var(--bg-primary);
  margin-bottom: -1px;
}

.tab-name {
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.unsaved {
  color: var(--warning);
}

.tab-close {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 14px;
  padding: 2px 4px;
  opacity: 0;
  transition: opacity 0.15s;
}

.tab:hover .tab-close {
  opacity: 1;
}

.tab-close:hover {
  color: var(--text-primary);
}

.tab-actions {
  margin-left: auto;
  padding-right: 8px;
}

/* 欢迎页面 */
.welcome {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.welcome-content {
  text-align: center;
  max-width: 500px;
}

.welcome-content h1 {
  font-size: 32px;
  font-weight: 300;
  margin-bottom: 8px;
  color: var(--text-primary);
}

.welcome-content p {
  color: var(--text-secondary);
  margin-bottom: 24px;
}

.welcome-actions button {
  font-size: 14px;
  padding: 10px 24px;
}

/* 最近工作区 */
.recent-section {
  margin-top: 32px;
  text-align: left;
}

.recent-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  padding: 0 4px;
}

.recent-header span {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
}

.clear-btn {
  font-size: 11px;
  padding: 2px 8px;
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 3px;
}

.clear-btn:hover {
  color: var(--text-primary);
  border-color: var(--text-secondary);
}

.recent-list {
  background: var(--bg-secondary);
  border-radius: 6px;
  border: 1px solid var(--border);
  overflow: hidden;
}

.recent-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  cursor: pointer;
  transition: background 0.1s;
  border-bottom: 1px solid var(--border);
}

.recent-item:last-child {
  border-bottom: none;
}

.recent-item:hover {
  background: var(--bg-tertiary);
}

.recent-icon {
  font-size: 20px;
  flex-shrink: 0;
}

.recent-info {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.recent-name {
  font-weight: 500;
  font-size: 13px;
  color: var(--text-primary);
}

.recent-path {
  font-size: 11px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.recent-remove {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 16px;
  padding: 4px 8px;
  opacity: 0;
  transition: opacity 0.15s;
}

.recent-item:hover .recent-remove {
  opacity: 1;
}

.recent-remove:hover {
  color: var(--error);
}

/* 编辑器内容 */
.editor-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.editor-pane {
  flex: 1;
  overflow: hidden;
  min-width: 300px;
}

.resize-handle {
  width: 4px;
  cursor: col-resize;
  background: var(--border);
  transition: background 0.15s;
  flex-shrink: 0;
}

.resize-handle:hover,
.resize-handle.active {
  background: var(--accent);
}

.result-pane {
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  flex-shrink: 0;
  min-width: 200px;
  max-width: 800px;
}

.pane-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--text-secondary);
  border-bottom: 1px solid var(--border);
}

.loading {
  font-weight: normal;
  text-transform: none;
  color: var(--accent);
}

/* 状态栏 */
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 22px;
  padding: 0 10px;
  background: var(--accent);
  font-size: 12px;
  color: white;
}

.status-left,
.status-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-left {
  flex: 1;
  overflow: hidden;
}

.status-left span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
