#!/bin/bash

# Termina si algún comando falla
set -e

PROJECT_DIR="./web"
DIST_DIR="dist"

echo "Iniciando el proceso de build optimizado de Vite..."

cd $PROJECT_DIR

# 1. Instalar dependencias si no están (opcional, pero buena práctica)
if [ ! -d "node_modules" ]; then
    echo "Instalando dependencias de Node..."
    npm install
fi

# Ejecutar el comando de build de Vite
echo "Ejecutando 'npm run build'..."
npm run build

# Mover o confirmar la ubicación de la distribución
if [ -d "$DIST_DIR" ]; then
    echo "✅ Distribución creada exitosamente en el directorio: **$DIST_DIR**"
else
    echo "❌ Error: No se encontró el directorio de distribución '$DIST_DIR'."
    exit 1
fi
