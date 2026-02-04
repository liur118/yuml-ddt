<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick } from 'vue'
import * as monaco from 'monaco-editor'

interface StepInfo {
  id: string
  name: string
  method: string
  path: string
  lineNumber?: number
}

const props = defineProps<{
  content: string
  steps: StepInfo[]
  parseError?: string | null
}>()

const emit = defineEmits<{
  change: [content: string]
  execute: [stepId: string]
}>()

const editorContainer = ref<HTMLDivElement>()
let editor: monaco.editor.IStandaloneCodeEditor | null = null
let decorations: string[] = []
let glyphMarginClickHandler: monaco.IDisposable | null = null

// 解析 YAML 找到每个 step 的行号
function findStepLines(content: string): Map<string, number> {
  const stepLines = new Map<string, number>()
  const lines = content.split('\n')
  let inSteps = false
  
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    const trimmed = line.trim()
    
    // 检测 steps: 开始
    if (trimmed === 'steps:') {
      inSteps = true
      continue
    }
    
    if (inSteps && trimmed.startsWith('- ')) {
      // 尝试匹配 step 名称
      const nameMatch = trimmed.match(/^-\s*(\w+):/)
      if (nameMatch) {
        const stepId = nameMatch[1]
        stepLines.set(stepId, i + 1)
      }
    }
  }
  
  return stepLines
}

// 更新运行按钮装饰
function updateRunButtons() {
  if (!editor) return
  
  const content = editor.getValue()
  const stepLines = findStepLines(content)
  
  const newDecorations: monaco.editor.IModelDeltaDecoration[] = []
  
  for (const step of props.steps) {
    let lineNumber = stepLines.get(step.id)
    
    if (!lineNumber) {
      const lines = content.split('\n')
      for (let i = 0; i < lines.length; i++) {
        if (lines[i].includes(step.id + ':')) {
          lineNumber = i + 1
          break
        }
      }
    }
    
    if (lineNumber) {
      newDecorations.push({
        range: new monaco.Range(lineNumber, 1, lineNumber, 1),
        options: {
          isWholeLine: false,
          glyphMarginClassName: 'run-glyph',
          glyphMarginHoverMessage: { value: `▶ 运行 ${step.name || step.id}` },
          stickiness: monaco.editor.TrackedRangeStickiness.NeverGrowsWhenTypingAtEdges,
        }
      })
    }
  }
  
  decorations = editor.deltaDecorations(decorations, newDecorations)
}

onMounted(() => {
  if (editorContainer.value) {
    monaco.languages.register({ id: 'yaml' })
    
    editor = monaco.editor.create(editorContainer.value, {
      value: props.content,
      language: 'yaml',
      theme: 'vs-dark',
      fontSize: 13,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      lineNumbers: 'on',
      minimap: { enabled: true },
      scrollBeyondLastLine: false,
      automaticLayout: true,
      tabSize: 2,
      insertSpaces: true,
      wordWrap: 'on',
      folding: true,
      renderWhitespace: 'selection',
      glyphMargin: true,
      lineDecorationsWidth: 10,
    })

    editor.onDidChangeModelContent(() => {
      if (editor) {
        emit('change', editor.getValue())
        nextTick(() => updateRunButtons())
      }
    })

    glyphMarginClickHandler = editor.onMouseDown((e) => {
      if (e.target.type === monaco.editor.MouseTargetType.GUTTER_GLYPH_MARGIN) {
        const lineNumber = e.target.position?.lineNumber
        if (lineNumber) {
          const content = editor?.getValue() || ''
          const lines = content.split('\n')
          const line = lines[lineNumber - 1]
          
          for (const step of props.steps) {
            if (line.includes(step.id + ':')) {
              emit('execute', step.id)
              break
            }
          }
        }
      }
    })

    nextTick(() => updateRunButtons())
  }
})

onUnmounted(() => {
  if (glyphMarginClickHandler) {
    glyphMarginClickHandler.dispose()
  }
  if (editor) {
    editor.dispose()
  }
})

watch(() => props.content, (newContent) => {
  if (editor && editor.getValue() !== newContent) {
    editor.setValue(newContent)
    nextTick(() => updateRunButtons())
  }
})

watch(() => props.steps, () => {
  nextTick(() => updateRunButtons())
}, { deep: true })

// 暴露方法：跳转到指定 step 所在行
function focusStep(stepId: string) {
  if (!editor) return
  
  const content = editor.getValue()
  const stepLines = findStepLines(content)
  let lineNumber = stepLines.get(stepId)
  
  if (!lineNumber) {
    // 备用查找方式
    const lines = content.split('\n')
    for (let i = 0; i < lines.length; i++) {
      if (lines[i].includes(stepId + ':')) {
        lineNumber = i + 1
        break
      }
    }
  }
  
  if (lineNumber) {
    editor.revealLineInCenter(lineNumber)
    editor.setPosition({ lineNumber, column: 1 })
    editor.focus()
  }
}

defineExpose({
  focusStep
})
</script>

<template>
  <div class="yaml-editor">
    <div v-if="parseError" class="error-bar">
      <span class="error-icon">⚠️</span>
      <span class="error-text">YAML解析错误: {{ parseError }}</span>
    </div>
    <div ref="editorContainer" class="editor-container" :class="{ 'with-error': parseError }"></div>
  </div>
</template>

<style scoped>
.yaml-editor {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.error-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: #ff4444;
  color: white;
  font-size: 13px;
  font-family: Menlo, Monaco, "Courier New", monospace;
  border-bottom: 1px solid #cc0000;
  z-index: 1000;
}

.error-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.error-text {
  flex: 1;
  overflow-x: auto;
  white-space: pre;
}

.editor-container {
  width: 100%;
  flex: 1;
}

.editor-container.with-error {
  height: calc(100% - 36px);
}
</style>

<style>
/* 全局样式 - 运行按钮 */
.run-glyph {
  background: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3E%3Cpath fill='%234ec9b0' d='M4 2 L4 14 L14 8 Z'/%3E%3C/svg%3E") center center no-repeat;
  background-size: 14px;
  cursor: pointer;
  opacity: 0.7;
  transition: opacity 0.15s;
}

.run-glyph:hover {
  opacity: 1;
}
</style>
