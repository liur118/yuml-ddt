<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  files: string[]
  currentFile: string
  workspacePath: string
}>()

const emit = defineEmits<{
  select: [filePath: string]
}>()

// 简化文件路径显示
const displayFiles = computed(() => {
  return props.files.map(f => ({
    path: f,
    name: f.replace(props.workspacePath + '/', '')
  }))
})

function selectFile(path: string) {
  emit('select', path)
}
</script>

<template>
  <div class="file-tree">
    <div v-if="files.length === 0" class="empty">
      暂无 YAML 文件
    </div>
    <div 
      v-for="file in displayFiles"
      :key="file.path"
      class="file-item"
      :class="{ active: file.path === currentFile }"
      @click="selectFile(file.path)"
      :title="file.path"
    >
      <span class="icon">📄</span>
      <span class="name">{{ file.name }}</span>
    </div>
  </div>
</template>

<style scoped>
.file-tree {
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

.file-item {
  display: flex;
  align-items: center;
  padding: 4px 12px;
  cursor: pointer;
  gap: 6px;
  transition: background 0.1s;
}

.file-item:hover {
  background: var(--bg-tertiary);
}

.file-item.active {
  background: var(--accent);
}

.icon {
  font-size: 14px;
  flex-shrink: 0;
}

.name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
}
</style>
