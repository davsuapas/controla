#!/bin/bash

# Termina la ejecución inmediatamente si cualquier comando falla
set -e

# Nombre de la carpeta del proyecto Rust (asume que el script se ejecuta desde la raíz)
PROJECT_DIR="."
# Nombre binario esperado
EXECUTABLE_NAME="controla-api"

echo "Iniciando construcción optimizada para Linux x86_64..."

# Verificar si el proyecto Rust existe
if [ ! -f "$PROJECT_DIR/Cargo.toml" ]; then
    echo "❌ Error: No se encontró el archivo Cargo.toml en $PROJECT_DIR."
    echo "Asegúrate de ejecutar este script desde la raíz del proyecto Rust."
    exit 1
fi

echo "--------------------------------------------------------"

# Limpiar compilaciones anteriores (opcional, pero buena práctica)
echo "Limpiando compilaciones anteriores..."
cargo clean

# Análisis de Código con Clippy
echo "Ejecutando 'cargo clippy' para análisis de calidad..."
# Usamos -- -D warnings para promover todas las advertencias de Clippy a errores
cargo clippy --release --target x86_64-unknown-linux-gnu -- -D warnings

# El comando anterior retornará un código de error si Clippy encuentra alguna advertencia (ahora un error)
echo "✅ Análisis de Clippy exitoso: ¡El código cumple los estándares!"

# Construir el ejecutable en modo release
# --target x86_64-unknown-linux-gnu es el valor por defecto para Linux x86_64,
# se incluye para claridad y posible uso futuro de cross-compiling.
# La bandera --release es lo que habilita el perfil [profile.release]
echo "Ejecutando 'cargo build --release'..."
cargo build --release --target x86_64-unknown-linux-gnu

# Verificar el resultado de la compilación
if [ $? -eq 0 ]; then
    echo "--------------------------------------------------------"
    echo "✅ ¡Construcción Exitosa!"
    
    # Ruta del ejecutable
    EXEC_PATH="$PROJECT_DIR/target/x86_64-unknown-linux-gnu/release/$EXECUTABLE_NAME"
    
    # Si no se encontró en la ruta específica, buscar en la ruta predeterminada (para binarios simples)
    if [ ! -f "$EXEC_PATH" ]; then
        EXEC_PATH="$PROJECT_DIR/target/release/$EXECUTABLE_NAME"
    fi

    if [ -f "$EXEC_PATH" ]; then
        echo "El ejecutable optimizado se encuentra en: **$EXEC_PATH**"
        echo "Tamaño del binario: $(du -h "$EXEC_PATH" | awk '{print $1}')"
    else
        echo "⚠️ Advertencia: No se pudo localizar el binario final en la ruta esperada."
        echo "Busca en el directorio 'target/release/'."
    fi
else
    echo "--------------------------------------------------------"
    echo "❌ Error de compilación: La construcción falló. Revisa los errores de Cargo."
    exit 1
fi