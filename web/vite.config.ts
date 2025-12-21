import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig(({ mode }) => {
  // loadEnv ya maneja la lectura de archivos .env
  const env = loadEnv(mode, '.', '')

  return {
    plugins: [react()],
    base: '/' + env.VITE_CONTROLA_WEB_APP || '',
  }
})