export type SessionState = 'idle' | 'thinking' | 'working' | 'asking' | 'error' | 'complete'

export interface SessionView {
  sessionId: string
  name: string
  cwd: string
  state: SessionState
  detail: string
  background: boolean
  at: number
}

export const LABELS: Record<SessionState, string> = {
  idle: '待機中',
  thinking: '思考中',
  working: '工作中',
  asking: '等你回覆',
  error: '出錯了',
  complete: '完成了',
}
