<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import type { SessionState } from './types'

const props = defineProps<{ state: SessionState }>()

const CELL_WIDTH = 192
const CELL_HEIGHT = 208
const SCALE = 0.5
const FRAME_MS = 140
// spritesheet 的列號與幀數（8 欄 × 11 列）
const ROWS: Record<SessionState, { row: number; frames: number }> = {
  idle: { row: 0, frames: 7 },
  thinking: { row: 8, frames: 6 },
  working: { row: 7, frames: 6 },
  asking: { row: 6, frames: 6 },
  error: { row: 5, frames: 8 },
  complete: { row: 4, frames: 5 },
}

const frame = ref(0)
const timer = setInterval(() => {
  frame.value = (frame.value + 1) % ROWS[props.state].frames
}, FRAME_MS)
watch(() => props.state, () => (frame.value = 0))
onUnmounted(() => clearInterval(timer))

const style = computed(() => ({
  width: `${CELL_WIDTH * SCALE}px`,
  height: `${CELL_HEIGHT * SCALE}px`,
  backgroundImage: 'url(/spritesheet.webp)',
  backgroundSize: `${8 * CELL_WIDTH * SCALE}px ${11 * CELL_HEIGHT * SCALE}px`,
  backgroundPosition: `-${frame.value * CELL_WIDTH * SCALE}px -${ROWS[props.state].row * CELL_HEIGHT * SCALE}px`,
  imageRendering: 'pixelated' as const,
}))
</script>

<template>
  <div :style="style" />
</template>
