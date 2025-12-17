#!/bin/bash

# Termina la ejecución si algún comando falla
set -e

RUST_SOURCE="./target/x86_64-unknown-linux-gnu/release/controla-api"
WEB_SOURCE="./web/dist"

DEST="./target/dist"
BIN_DEST="$DEST/api"
WEB_DEST="$DEST/web"

echo "Iniciando el proceso de preparación del despliegue..."

# Crear directorios de destino si no existen
echo "Verificando y creando directorios de destino..."

rm -r $DEST 2>/dev/null
mkdir -p "$BIN_DEST"
mkdir -p "$WEB_DEST"
echo "   - Directorios de despliegue listos."

# 2. Copiar el ejecutable de Rust
echo "Copiando binario de Rust: $RUST_SOURCE a $BIN_DEST/"
if [ -f "$RUST_SOURCE" ]; then
    cp "$RUST_SOURCE" "$BIN_DEST/"
    echo "   - ✅ Binario copiado exitosamente."
else
    echo "❌ Error: El binario de Rust no se encontró en $RUST_SOURCE. ¿Ejecutaste 'api/build.sh'?"
    exit 1
fi

# 3. Copiar los archivos de la aplicación web
echo "Copiando artefactos Web: $WEB_SOURCE a $WEB_DEST/"
if [ -d "$WEB_SOURCE" ]; then
    cp -r "$WEB_SOURCE"/* "$WEB_DEST/"
    echo "   - ✅ Archivos web copiados exitosamente."
else
    echo "⚠️ Advertencia: El directorio web de distribución no se encontró en $WEB_SOURCE. ¿Ejecutaste 'web/build.sh'?"
fi

echo "✅ Despliegue listo en el directorio: **$DEST**"