<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow, LogicalPosition, LogicalSize } from '@tauri-apps/api/window'
import Sprite from './Sprite.vue'
import SessionPanel from './SessionPanel.vue'
import LaunchForm from './LaunchForm.vue'
import { LABELS, type SessionView } from './types'

const COLLAPSED = new LogicalSize(240, 260)
const EXPANDED = new LogicalSize(360, 520)
const DRAG_THRESHOLD_PX = 4

const sessions = ref<SessionView[]>([])
const expanded = ref(false)
const focus = computed(() => sessions.value[0])
const bubble = computed(() =>
  focus.value ? `${focus.value.name}：${LABELS[focus.value.state]}` : '沒有 session 在跑',
)

onMounted(async () => {
  sessions.value = await invoke<SessionView[]>('current_sessions')
  await listen<SessionView[]>('sessions-changed', (event) => (sessions.value = event.payload))
})

// 按下後移動超過門檻才算拖曳；沒移動就當點擊，切換展開面板。
function onPointerDown(down: PointerEvent) {
  const onMove = (move: PointerEvent) => {
    if (Math.hypot(move.clientX - down.clientX, move.clientY - down.clientY) > DRAG_THRESHOLD_PX) {
      cleanup()
      getCurrentWindow().startDragging()
    }
  }
  const onUp = () => {
    cleanup()
    toggle()
  }
  const cleanup = () => {
    window.removeEventListener('pointermove', onMove)
    window.removeEventListener('pointerup', onUp)
  }
  window.addEventListener('pointermove', onMove)
  window.addEventListener('pointerup', onUp)
}

// 視窗從左上角長大，角色卻站在底部中央；改尺寸時一併移動視窗，讓角色留在原地。
async function toggle() {
  const appWindow = getCurrentWindow()
  const [from, to] = expanded.value ? [EXPANDED, COLLAPSED] : [COLLAPSED, EXPANDED]
  const scale = await appWindow.scaleFactor()
  const position = (await appWindow.outerPosition()).toLogical(scale)
  expanded.value = !expanded.value
  await appWindow.setPosition(
    new LogicalPosition(position.x - (to.width - from.width) / 2, position.y - (to.height - from.height)),
  )
  await appWindow.setSize(to)
}
</script>

<template>
  <div class="root">
    <template v-if="expanded">
      <SessionPanel :sessions="sessions" />
      <LaunchForm />
    </template>
    <div class="bubble" :class="focus?.state">{{ bubble }}</div>
    <div class="pet" @pointerdown="onPointerDown">
      <Sprite :state="focus?.state ?? 'idle'" />
    </div>
  </div>
</template>

<style>
html, body, #app { background: transparent; margin: 0; overflow: hidden; font-family: 'Noto Sans TC', sans-serif; }
.root { display: flex; flex-direction: column; align-items: center; justify-content: flex-end; height: 100vh; gap: 6px; padding: 6px; box-sizing: border-box; }
.bubble { background: #fff; color: #222; border-radius: 12px; padding: 4px 10px; font-size: 13px; max-width: 220px; text-align: center; box-shadow: 0 1px 4px rgba(0,0,0,.3); }
.bubble.asking { background: #ffe0f7; font-weight: bold; }
.bubble.error { background: #ffe0e0; }
.pet { cursor: grab; user-select: none; }
</style>
