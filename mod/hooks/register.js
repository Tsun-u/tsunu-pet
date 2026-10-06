// 追蹤這個 session 的狀態：在側邊欄顯示對應的阿宇立繪，並把狀態送給桌面總機小視窗。
// 所有 hook 都只觀察，一律原樣交回 next(e)，不改變 Claude Code 的行為。

const PANE = 'tsunu-avatar'
// 換成自己的角色時改這裡：側邊欄標題、圖片替代文字都用這個名字。
const CHARACTER = '阿宇'
const SWITCHBOARD_URL = 'http://127.0.0.1:47321/state'
const COMPLETE_HOLD_MS = 3000

const PORTRAITS = {
  idle: 'idle.png',
  thinking: 'thinking.png',
  working: 'working.png',
  asking: 'asking.png',
  error: 'error.png',
  complete: 'complete.png',
}

const LABELS = {
  idle: '待機中',
  thinking: '思考中',
  working: '工作中',
  asking: '等你回覆',
  error: '出錯了',
  complete: '完成了',
}

// 預先轉好的半格字元立繪（dev/build-rasters.py 產生），依側邊欄寬度挑最大放得下的。
const RASTER_WIDTHS = [56, 48, 42, 36, 30, 24]

let state = 'idle'
let detail = ''
let backToIdle = null
let showsPortraitImage = false
// 狀態送往哪些地方：桌面總機，加上啟動這個 session 的程式用 TSUNU_STATE_URL 指定的收件位址。
const reportUrls = [SWITCHBOARD_URL]
const rasters = new Map()

// Image 要靠 kitty 圖形協定的 Unicode 佔位字元定位，目前只有 kitty 和 Ghostty 支援；
// Windows 上要用自己編譯的 WezTerm（PR #7924），它的設定會帶 TSUNU_KITTY_PLACEHOLDERS=1。
async function terminalShowsImages($) {
  if ((await $.env.get('TSUNU_KITTY_PLACEHOLDERS')) === '1') return true
  if ((await $.env.get('OS')) === 'Windows_NT') return false
  // kitty 不設 TERM_PROGRAM，只能從 TERM（xterm-kitty）認出來；Ghostty 兩個都設。
  const terminalNames = [await $.env.get('TERM_PROGRAM'), await $.env.get('TERM')]
  return terminalNames.some((name) => /kitty|ghostty/i.test(name || ''))
}

async function loadRaster($, columns) {
  if (!rasters.has(columns)) {
    const text = await $.fs.read($.plugin.root + '/assets/raster-' + columns + '.json')
    rasters.set(columns, JSON.parse(text))
  }
  return rasters.get(columns)
}

function setState($, next, nextDetail = '') {
  if (backToIdle) {
    backToIdle.cancel()
    backToIdle = null
  }
  state = next
  detail = nextDetail
  $.ui.invalidate('ui.render')
  report($)
}

// 送出後不等回應：收件端沒開時 fetch 會失敗，不能拖慢 hook。
function report($) {
  const sent = state
  const sentDetail = detail
  Promise.all([$.session.id(), $.session.cwd()]).then(([sessionId, cwd]) => {
    const body = JSON.stringify({ sessionId, cwd, state: sent, detail: sentDetail, at: Date.now() })
    for (const url of reportUrls) {
      $.http.fetch(url, { method: 'POST', headers: { 'content-type': 'application/json' }, body })
        .catch((err) => $.ui.log('tsunu-avatar: 送不到 ' + url + ' ' + err, { to: 'debug' }))
    }
  })
}

export function register(on) {
  on('session.start', async ($, e, next) => {
    const extraUrl = await $.env.get('TSUNU_STATE_URL')
    if (extraUrl) reportUrls.push(extraUrl)
    // 宿主程式自己有立繪（例如 tsunu-alive-lite）時用 TSUNU_PANE=0 關掉側邊欄。
    if ((await $.env.get('TSUNU_PANE')) !== '0') {
      showsPortraitImage = await terminalShowsImages($).catch(() => false)
      await $.ui.open({ id: PANE, title: CHARACTER, columns: 42 })
    }
    setState($, 'idle')
    return next(e)
  })

  on('turn.start', async ($, e, next) => {
    if (!e.agentId) setState($, 'thinking')
    return next(e)
  })

  on('tool.check', async ($, e, next) => {
    const decision = await next(e)
    if (decision && decision.decision === 'ask') setState($, 'asking', e.tool)
    return decision
  })

  on('tool.call', async ($, e, next) => {
    setState($, e.tool === 'AskUserQuestion' ? 'asking' : 'working', e.tool)
    const result = await next(e)
    if (result && result.isError) setState($, 'error', e.tool)
    else setState($, 'thinking')
    return result
  })

  on('turn.complete', async ($, e, next) => {
    if (!e.agentId) {
      if (e.isAborted) {
        setState($, 'idle')
      } else {
        setState($, 'complete')
        backToIdle = $.clock.after(COMPLETE_HOLD_MS, () => setState($, 'idle'))
      }
    }
    return next(e)
  })

  on('session.end', async ($, e, next) => {
    setState($, 'ended')
    return next(e)
  })

  on('ui.render', { component: 'Pane' }, async ($, e, next) => {
    if (e.requestId !== PANE) return next(e)
    const { Box, Text, Image, Raster } = $.ui.resolve(e)
    const caption = LABELS[state] + (detail ? '：' + detail : '')
    if (e.surface !== 'terminal') {
      return Text({ children: [CHARACTER + ' ' + caption] })
    }
    const width = RASTER_WIDTHS.find((w) => w <= e.props.bodyColumns) || RASTER_WIDTHS[RASTER_WIDTHS.length - 1]
    const portrait = showsPortraitImage
      ? Image({
          source: { file: $.plugin.root + '/assets/' + PORTRAITS[state], format: 'png' },
          columns: width,
          rows: Math.round(width * 0.83),
          alt: CHARACTER,
        })
      : await loadRaster($, width).then((raster) =>
          Raster({ key: 'portrait', columns: width, rows: raster.rows, cells: raster.cells[state] }),
        )
    return Box({ flexDirection: 'column', children: [portrait, Text({ children: [caption] })] })
  })
}
