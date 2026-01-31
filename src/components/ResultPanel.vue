<script setup lang="ts">
import { ref, computed } from 'vue'

interface ValidationResult {
  field: string
  operator: string
  expected: string
  actual: string
  passed: boolean
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
  validations: ValidationResult[]
  duration_ms: number
  error: string | null
}

const props = defineProps<{
  result: ExecutionResult | null
}>()

// 折叠状态
const showRequestHeaders = ref(false)
const showResponseHeaders = ref(false)

// 复制状态 - 每个代码块独立状态
const copiedStates = ref<Record<string, boolean>>({})

const statusClass = computed(() => {
  if (!props.result) return ''
  return props.result.success ? 'success' : 'error'
})

function formatJson(obj: any): string {
  if (!obj) return ''
  return JSON.stringify(obj, null, 2)
}

function formatHeaders(headers: Record<string, string>): string {
  if (!headers || Object.keys(headers).length === 0) return ''
  return Object.entries(headers)
    .map(([k, v]) => `${k}: ${v}`)
    .join('\n')
}

function getHeaderCount(headers: Record<string, string>): number {
  return headers ? Object.keys(headers).length : 0
}

// 通用复制函数
async function copyContent(key: string, content: string) {
  try {
    await navigator.clipboard.writeText(content)
    copiedStates.value[key] = true
    setTimeout(() => {
      copiedStates.value[key] = false
    }, 2000)
  } catch (err) {
    console.error('复制失败:', err)
  }
}

function isCopied(key: string): boolean {
  return copiedStates.value[key] || false
}
</script>

<template>
  <div class="result-panel-content">
    <div v-if="!result" class="empty">
      点击步骤旁的 ▶ 按钮执行测试
    </div>
    
    <template v-else>
      <!-- 状态概览 -->
      <div class="status-section" :class="statusClass">
        <div class="status-header">
          <span class="status-icon">{{ result.success ? '✅' : '❌' }}</span>
          <span class="status-text">{{ result.success ? '成功' : '失败' }}</span>
          <span class="duration">{{ result.duration_ms }}ms</span>
        </div>
        <div class="step-name">{{ result.step_name }}</div>
      </div>

      <!-- 请求信息 -->
      <div class="section">
        <div class="section-header">📤 请求</div>
        <div class="section-content">
          <div class="info-row">
            <span class="method" :class="result.request_method.toLowerCase()">
              {{ result.request_method }}
            </span>
            <span class="url">{{ result.request_url }}</span>
          </div>
          
          <!-- 请求头 -->
          <div v-if="getHeaderCount(result.request_headers) > 0" class="headers-section">
            <div 
              class="headers-toggle" 
              @click="showRequestHeaders = !showRequestHeaders"
            >
              <span class="toggle-icon">{{ showRequestHeaders ? '▼' : '▶' }}</span>
              <span>Headers ({{ getHeaderCount(result.request_headers) }})</span>
            </div>
            <div v-if="showRequestHeaders" class="code-block-wrapper">
              <div class="code-block headers-block">
                <pre>{{ formatHeaders(result.request_headers) }}</pre>
              </div>
              <button 
                class="code-copy-btn" 
                @click="copyContent('req-headers', formatHeaders(result.request_headers))"
                :title="isCopied('req-headers') ? '已复制' : '复制'"
              >
                {{ isCopied('req-headers') ? '✓' : '📋' }}
              </button>
            </div>
          </div>
          
          <!-- 请求体 -->
          <div v-if="result.request_body" class="body-section">
            <div class="body-label">Body</div>
            <div class="code-block-wrapper">
              <div class="code-block">
                <pre>{{ formatJson(result.request_body) }}</pre>
              </div>
              <button 
                class="code-copy-btn" 
                @click="copyContent('req-body', formatJson(result.request_body))"
                :title="isCopied('req-body') ? '已复制' : '复制'"
              >
                {{ isCopied('req-body') ? '✓' : '📋' }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- 响应信息 -->
      <div class="section">
        <div class="section-header">
          📥 响应 
          <span class="status-code" :class="{ ok: result.response_status < 400 }">
            {{ result.response_status }}
          </span>
        </div>
        <div class="section-content">
          <!-- 响应头 -->
          <div v-if="getHeaderCount(result.response_headers) > 0" class="headers-section">
            <div 
              class="headers-toggle" 
              @click="showResponseHeaders = !showResponseHeaders"
            >
              <span class="toggle-icon">{{ showResponseHeaders ? '▼' : '▶' }}</span>
              <span>Headers ({{ getHeaderCount(result.response_headers) }})</span>
            </div>
            <div v-if="showResponseHeaders" class="code-block-wrapper">
              <div class="code-block headers-block">
                <pre>{{ formatHeaders(result.response_headers) }}</pre>
              </div>
              <button 
                class="code-copy-btn" 
                @click="copyContent('res-headers', formatHeaders(result.response_headers))"
                :title="isCopied('res-headers') ? '已复制' : '复制'"
              >
                {{ isCopied('res-headers') ? '✓' : '📋' }}
              </button>
            </div>
          </div>
          
          <!-- 响应体 -->
          <div v-if="result.response_body" class="body-section">
            <div class="body-label">Body</div>
            <div class="code-block-wrapper">
              <div class="code-block">
                <pre>{{ formatJson(result.response_body) }}</pre>
              </div>
              <button 
                class="code-copy-btn" 
                @click="copyContent('res-body', formatJson(result.response_body))"
                :title="isCopied('res-body') ? '已复制' : '复制'"
              >
                {{ isCopied('res-body') ? '✓' : '📋' }}
              </button>
            </div>
          </div>
          <div v-else-if="result.error" class="error-message">
            {{ result.error }}
          </div>
        </div>
      </div>

      <!-- 验证结果 -->
      <div v-if="result.validations.length > 0" class="section">
        <div class="section-header">🔍 验证</div>
        <div class="section-content">
          <div 
            v-for="(v, i) in result.validations" 
            :key="i"
            class="validation-item"
            :class="{ passed: v.passed, failed: !v.passed }"
          >
            <span class="v-icon">{{ v.passed ? '✓' : '✗' }}</span>
            <span class="v-field">{{ v.field }}</span>
            <span class="v-operator">{{ v.operator }}</span>
            <span class="v-expected">{{ v.expected }}</span>
            <span v-if="!v.passed" class="v-actual">(实际: {{ v.actual }})</span>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.result-panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.empty {
  padding: 24px 12px;
  color: var(--text-secondary);
  text-align: center;
  font-size: 12px;
}

.status-section {
  padding: 12px;
  border-radius: 4px;
  margin-bottom: 12px;
}

.status-section.success {
  background: rgba(78, 201, 176, 0.1);
  border: 1px solid var(--success);
}

.status-section.error {
  background: rgba(241, 76, 76, 0.1);
  border: 1px solid var(--error);
}

.status-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.status-icon {
  font-size: 16px;
}

.status-text {
  font-weight: 600;
}

.duration {
  margin-left: auto;
  color: var(--text-secondary);
  font-size: 12px;
}

.step-name {
  font-size: 12px;
  color: var(--text-secondary);
}

.section {
  margin-bottom: 12px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  overflow: hidden;
}

.section-header {
  padding: 8px 12px;
  font-size: 12px;
  font-weight: 600;
  background: var(--bg-primary);
  display: flex;
  align-items: center;
  gap: 8px;
}

.section-content {
  padding: 8px 12px;
}

.info-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.method {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 3px;
  background: var(--bg-primary);
}

.method.get { color: #61affe; }
.method.post { color: #49cc90; }
.method.put { color: #fca130; }
.method.delete { color: #f93e3e; }

.url {
  font-size: 11px;
  word-break: break-all;
}

.status-code {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 3px;
  background: var(--error);
}

.status-code.ok {
  background: var(--success);
}

.code-block-wrapper {
  position: relative;
  margin-top: 4px;
}

.code-block {
  background: var(--bg-primary);
  border-radius: 4px;
  padding: 8px;
  padding-right: 36px;
  overflow-x: auto;
}

.code-block pre {
  font-family: Menlo, Monaco, "Courier New", monospace;
  font-size: 11px;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
}

.code-copy-btn {
  position: absolute;
  top: 4px;
  right: 4px;
  padding: 4px 6px;
  font-size: 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-secondary);
  cursor: pointer;
  opacity: 0.7;
  transition: all 0.2s;
}

.code-copy-btn:hover {
  opacity: 1;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.headers-section {
  margin-bottom: 8px;
}

.headers-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 4px 0;
  user-select: none;
}

.headers-toggle:hover {
  color: var(--text-primary);
}

.toggle-icon {
  font-size: 8px;
  width: 10px;
}

.headers-block {
  margin-top: 4px;
}

.body-section {
  margin-top: 8px;
}

.body-label {
  font-size: 11px;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.error-message {
  color: var(--error);
  font-size: 12px;
}

.validation-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 0;
  font-size: 12px;
  flex-wrap: wrap;
}

.validation-item.passed .v-icon { color: var(--success); }
.validation-item.failed .v-icon { color: var(--error); }

.v-field {
  font-weight: 500;
}

.v-operator {
  color: var(--text-secondary);
}

.v-expected {
  color: var(--success);
}

.v-actual {
  color: var(--error);
  font-size: 11px;
}
</style>
