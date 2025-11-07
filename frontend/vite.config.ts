import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    // 设置代理规则，将 API 和 WebSocket 请求转发到后端
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true, // 确保主机头被正确设置
      },
      '/ws': { // 如果使用 WebSocket
        target: 'ws://localhost:3000',
        ws: true, // 启用 WebSocket 代理
        changeOrigin: true,
      },
    },
    port: 5173, // 确保前端在独立端口运行
  },
})