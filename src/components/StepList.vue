<script setup lang="ts">
interface StepInfo {
  id: string
  name: string
  method: string
  path: string
}

defineProps<{
  steps: StepInfo[]
  isExecuting: boolean
}>()

const emit = defineEmits<{
  execute: [stepId: string]
  focusStep: [stepId: string]
}>()

function getMethodClass(method: string) {
  const m = method.toUpperCase()
  return {
    get: m === 'GET',
    post: m === 'POST',
    put: m === 'PUT',
    delete: m === 'DELETE'
  }
}

function executeStep(stepId: string) {
  emit('execute', stepId)
}

function handleStepClick(stepId: string) {
  emit('focusStep', stepId)
}
</script>

<template>
  <div class="step-list">
    <div v-if="steps.length === 0" class="empty">
      暂无测试步骤
    </div>
    <div 
      v-for="step in steps"
      :key="step.id"
      class="step-item"
      @click="handleStepClick(step.id)"
    >
      <div class="step-info">
        <div class="step-header">
          <span class="method" :class="getMethodClass(step.method)">
            {{ step.method }}
          </span>
          <span class="step-name">{{ step.name || step.id }}</span>
        </div>
        <div class="step-path">{{ step.path }}</div>
      </div>
      <button 
        class="run-btn"
        @click="executeStep(step.id)"
        :disabled="isExecuting"
        :title="'执行 ' + step.id"
      >
        ▶
      </button>
    </div>
  </div>
</template>

<style scoped>
.step-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.empty {
  padding: 12px;
  color: var(--text-secondary);
  text-align: center;
  font-size: 12px;
}

.step-item {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border);
  gap: 8px;
  cursor: pointer;
}

.step-item:hover {
  background: var(--bg-tertiary);
}

.step-info {
  flex: 1;
  overflow: hidden;
}

.step-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 2px;
}

.method {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 3px;
  background: var(--bg-tertiary);
  flex-shrink: 0;
}

.method.get { color: #61affe; }
.method.post { color: #49cc90; }
.method.put { color: #fca130; }
.method.delete { color: #f93e3e; }

.step-name {
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.step-path {
  font-size: 11px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.run-btn {
  width: 28px;
  height: 28px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--success);
  border-radius: 4px;
  flex-shrink: 0;
  font-size: 12px;
}

.run-btn:hover:not(:disabled) {
  filter: brightness(1.1);
}

.run-btn:disabled {
  opacity: 0.5;
}
</style>
