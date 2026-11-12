import { cpSync, createReadStream, existsSync, statSync } from 'node:fs'
import { extname, join, normalize } from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { loadScopedJournalCss } from './src/subapps/journal/prefixJournalCss.js'

const journalStylesId = 'virtual:journal-scoped.css'

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
  plugins: [react(), scopedJournalStyles(), aiAssistantDevAssets(), galleryPublicAssets()],
  server: {
    fs: {
      allow: [repoRoot],
    },
    proxy: {
      "/api": {
        target: "http://127.0.0.1:8787",
        changeOrigin: true,
      },
      "/cos": {
        target: "https://tzyy-1330068502.cos.ap-beijing.myqcloud.com",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/cos/, ""),
      },
    },
  },
})
