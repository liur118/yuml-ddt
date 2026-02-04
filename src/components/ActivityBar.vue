<script setup lang="ts">
import filesIcon from '@/assets/icons/files.svg'
import testsIcon from '@/assets/icons/tests.svg'
import settingsIcon from '@/assets/icons/settings.svg'
import helpIcon from '@/assets/icons/help.svg'

const props = defineProps<{
  activeView: string
}>()

const emit = defineEmits<{
  'update:activeView': [view: string]
}>()

const views = [
  { id: 'files', icon: filesIcon, title: '文件资源管理器' },
  { id: 'tests', icon: testsIcon, title: '测试用例' },
  { id: 'settings', icon: settingsIcon, title: '设置' },
]

function selectView(viewId: string) {
  // 如果点击当前已选中的，则隐藏侧边栏
  if (props.activeView === viewId) {
    emit('update:activeView', '')
  } else {
    emit('update:activeView', viewId)
  }
}
</script>

<template>
  <div class="activity-bar">
    <div class="activity-icons">
      <button
        v-for="view in views"
        :key="view.id"
        class="activity-icon"
        :class="{ active: activeView === view.id }"
        :title="view.title"
        @click="selectView(view.id)"
      >
        <img :src="view.icon" :alt="view.title" class="icon" />
      </button>
    </div>
    <div class="activity-bottom">
      <button class="activity-icon" title="帮助">
        <img :src="helpIcon" alt="帮助" class="icon" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.activity-bar {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  width: 48px;
  background: var(--bg-tertiary);
  border-right: 1px solid var(--border);
}

.activity-icons {
  display: flex;
  flex-direction: column;
}

.activity-bottom {
  display: flex;
  flex-direction: column;
  padding-bottom: 8px;
}

.activity-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  font-size: 20px;
  background: transparent;
  border: none;
  cursor: pointer;
  opacity: 0.6;
  position: relative;
  transition: opacity 0.15s;
}

.activity-icon .icon {
  width: 22px;
  height: 22px;
  filter: var(--icon-filter, invert(0.7));
}

.activity-icon:hover {
  opacity: 1;
}

.activity-icon.active {
  opacity: 1;
}

.activity-icon.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 2px;
  background: var(--accent);
}
</style>
