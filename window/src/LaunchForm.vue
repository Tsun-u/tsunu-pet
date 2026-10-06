<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const cwd = ref('')
onMounted(async () => (cwd.value = await invoke<string>('default_cwd')))
const resumeKind = ref<'new' | 'continue' | 'id'>('new')
const resumeId = ref('')
const permissionMode = ref('')
const effort = ref('')
const thinking = ref('')
const discord = ref(false)
const line = ref(false)
const cottage = ref(false)
const message = ref('')

async function launch() {
  const resume =
    resumeKind.value === 'id' && resumeId.value.trim()
      ? { kind: 'id', id: resumeId.value.trim() }
      : { kind: resumeKind.value === 'continue' ? 'continue' : 'new' }
  try {
    await invoke('launch_session', {
      options: {
        cwd: cwd.value,
        resume,
        permissionMode: permissionMode.value || null,
        effort: effort.value || null,
        thinking: thinking.value || null,
        discord: discord.value,
        line: line.value,
        cottage: cottage.value,
      },
    })
    message.value = '開好了'
  } catch (e) {
    message.value = String(e)
  }
}
</script>

<template>
  <form class="launch" @submit.prevent="launch">
    <label>目錄 <input v-model="cwd" /></label>
    <label>接續
      <select v-model="resumeKind">
        <option value="new">新 session</option>
        <option value="continue">接最近一次</option>
        <option value="id">指定 ID</option>
      </select>
    </label>
    <input v-if="resumeKind === 'id'" v-model="resumeId" placeholder="session ID" />
    <label>權限
      <select v-model="permissionMode">
        <option value="">預設</option>
        <option value="manual">manual</option>
        <option value="acceptEdits">acceptEdits</option>
        <option value="auto">auto</option>
        <option value="plan">plan</option>
        <option value="dontAsk">dontAsk</option>
        <option value="bypassPermissions">bypassPermissions</option>
      </select>
    </label>
    <label>effort
      <select v-model="effort">
        <option value="">預設</option>
        <option>low</option><option>medium</option><option>high</option><option>xhigh</option><option>max</option>
      </select>
    </label>
    <label>thinking
      <select v-model="thinking">
        <option value="">adaptive</option>
        <option>enabled</option><option>disabled</option>
      </select>
    </label>
    <div class="channels">
      <label><input type="checkbox" v-model="discord" />Discord</label>
      <label><input type="checkbox" v-model="line" />LINE</label>
      <label><input type="checkbox" v-model="cottage" />小屋</label>
    </div>
    <button type="submit">開 session</button>
    <small v-if="message">{{ message }}</small>
  </form>
</template>

<style scoped>
.launch { display: grid; gap: 4px; width: 100%; padding: 8px; box-sizing: border-box;
  background: rgba(20, 24, 32, 0.92); color: #eee; border-radius: 10px; font-size: 12px; }
.launch label { display: flex; justify-content: space-between; gap: 6px; }
.channels { display: flex; gap: 10px; }
</style>
