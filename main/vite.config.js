import { cpSync, createReadStream, existsSync, statSync } from 'node:fs'
import { extname, join, normalize } from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { loadScopedJournalCss } from './src/subapps/journal/prefixJournalCss.js'
import { loadWhereMenuCss } from './src/where/scopeWhereCss.js'

const journalStylesId = 'virtual:journal-scoped.css'
const whereMenuStylesId = 'virtual:where-menu.css'

const repoRoot = fileURLToPath(new URL('..', import.meta.url))
const aiAssistantDir = join(repoRoot, 'shared', 'ai-assistant')
const galleryPublicDir = join(repoRoot, 'previews', 'project-gallery', 'public')

function galleryPublicAssets() {
  const contentTypes = {
    '.webp': 'image/webp',
    '.json': 'application/json; charset=utf-8',
    '.txt': 'text/plain; charset=utf-8',
  }
  return {
    name: 'gallery-public-assets',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const urlPath = decodeURIComponent((req.url || '').split('?')[0])
        if (!urlPath.startsWith('/covers/') && !urlPath.startsWith('/thumbs/')) {
          next()
          return
        }
        const filePath = normalize(join(galleryPublicDir, urlPath))
        if (!filePath.startsWith(galleryPublicDir) || !existsSync(filePath) || !statSync(filePath).isFile()) {
          next()
          return
        }
        res.setHeader('Content-Type', contentTypes[extname(filePath)] || 'application/octet-stream')
        createReadStream(filePath).pipe(res)
      })
    },
    closeBundle() {
      const outDir = join(fileURLToPath(new URL('.', import.meta.url)), 'dist')
      cpSync(join(galleryPublicDir, 'covers'), join(outDir, 'covers'), { recursive: true })
      cpSync(join(galleryPublicDir, 'thumbs'), join(outDir, 'thumbs'), { recursive: true })
    },
  }
}

function clientPlaceFallback() {
  return {
    name: 'client-place-fallback',
    configureServer(server) {
      server.middlewares.use(async (req, res, next) => {
        const path = (req.url || '').split('?')[0]
        if (req.method !== 'GET' || path !== '/api/client/place') {
          next()
          return
        }
        const send = (body) => {
          res.statusCode = 200
          res.setHeader('Content-Type', 'application/json; charset=utf-8')
          res.end(JSON.stringify(body))
        }
        try {
          const upstream = await fetch('http://127.0.0.1:8080/api/client/place', {
            signal: AbortSignal.timeout(800),
          })
          if (upstream.ok) {
            const data = await upstream.json()
            if (data?.located) {
              send(data)
              return
            }
          }
        } catch {
          // 本地 API 没开，改用这台机器的出口 IP。
        }
        try {
          const geo = await fetch(
            'http://ip-api.com/json/?lang=zh-CN&fields=status,lat,lon,city,regionName',
            { signal: AbortSignal.timeout(8000) },
          )
          const data = await geo.json()
          if (data?.status === 'success' && Number.isFinite(data.lat) && Number.isFinite(data.lon)) {
            send({
              lat: data.lat,
              lon: data.lon,
              city: data.city || '',
              region: data.regionName || '',
              located: true,
            })
            return
          }
        } catch {
          // 交给后面的 /api 代理。
        }
        next()
      })
    },
  }
}

function whereMenuStyles() {
  const file = join(repoRoot, 'endfield_menu', 'src', 'styles.css')
  return {
    name: 'where-menu-styles',
    resolveId(id) {
      if (id === whereMenuStylesId) return `\0${id}`
    },
    async load(id) {
      if (id !== `\0${whereMenuStylesId}`) return null
      this.addWatchFile(file)
      return loadWhereMenuCss(file)
    },
  }
}

/**
 * endfield_menu/ 在 main/ 之外，它自己的 node_modules 不随仓库发布，
 * Vite 从它的文件位置往上找不到 react/three。这里只对 endfield_menu 的源码
 * 把这几个裸导入重定向到 main/node_modules，避免和 previews/project-gallery
 * 的 three/addons 解析互相干扰。
 */
function endfieldMenuDeps() {
  const menuRoot = fileURLToPath(new URL('../endfield_menu/', import.meta.url))
  const nm = fileURLToPath(new URL('./node_modules/', import.meta.url))
  const target = new Map([
    ['react', join(nm, 'react/index.js')],
    ['react/jsx-runtime', join(nm, 'react/jsx-runtime.js')],
    ['react/jsx-dev-runtime', join(nm, 'react/jsx-dev-runtime.js')],
    ['react-dom', join(nm, 'react-dom/index.js')],
    ['react-dom/client', join(nm, 'react-dom/client.js')],
    ['three', join(nm, 'three/build/three.module.js')],
  ])
  return {
    name: 'endfield-menu-deps',
    enforce: 'pre',
    resolveId(id, importer) {
      if (!importer) return null
      const resolved = target.get(id)
      if (!resolved) return null
      const importerPath = importer.split('?')[0]
      if (!importerPath.startsWith(menuRoot)) return null
      return resolved
    },
  }
}

function scopedJournalStyles() {
  return {
    name: 'scoped-journal-styles',
    resolveId(id) {
      if (id === journalStylesId) return `\0${id}`
    },
    async load(id) {
      if (id !== `\0${journalStylesId}`) return null
      return loadScopedJournalCss()
    },
  }
}

function aiAssistantDevAssets() {
  return {
    name: 'ai-assistant-dev-assets',
    configureServer(server) {
      server.middlewares.use('/ai-assistant', (req, res, next) => {
        const urlPath = decodeURIComponent((req.url || '').split('?')[0]).replace(/^\/+/, '')
        const filePath = normalize(join(aiAssistantDir, urlPath))
        if (!filePath.startsWith(aiAssistantDir) || !existsSync(filePath)) {
          next()
          return
        }
        const contentType = extname(filePath) === '.css' ? 'text/css; charset=utf-8' : 'application/javascript; charset=utf-8'
        res.setHeader('Content-Type', contentType)
        createReadStream(filePath).pipe(res)
      })
    },
  }
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), scopedJournalStyles(), whereMenuStyles(), endfieldMenuDeps(), clientPlaceFallback(), aiAssistantDevAssets(), galleryPublicAssets()],
  server: {
    fs: {
      allow: [repoRoot],
    },
    proxy: {
      "/api": {
        target: "http://127.0.0.1:8080",
        changeOrigin: true,
      },
    },
  },
})
