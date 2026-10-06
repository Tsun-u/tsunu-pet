<script setup lang="ts">
import { LABELS, type SessionView } from './types'
defineProps<{ sessions: SessionView[] }>()
</script>

<template>
  <ul class="panel">
    <li v-if="sessions.length === 0" class="empty">目前沒有 session 回報</li>
    <li v-for="s in sessions" :key="s.sessionId" :class="['row', s.state]">
      <span class="name">{{ s.name }}<small v-if="s.background">（背景）</small></span>
      <span class="state">{{ LABELS[s.state] }}<template v-if="s.detail">：{{ s.detail }}</template></span>
    </li>
  </ul>
</template>

<style scoped>
.panel { list-style: none; margin: 0; padding: 8px; max-height: 300px; overflow-y: auto;
  background: rgba(20, 24, 32, 0.92); color: #eee; border-radius: 10px; font-size: 13px; }
.row { display: flex; justify-content: space-between; gap: 8px; padding: 4px 2px; }
.row.asking .state { color: #ff9ff3; font-weight: bold; }
.row.error .state { color: #ff6b6b; }
.empty { color: #aaa; }
</style>
