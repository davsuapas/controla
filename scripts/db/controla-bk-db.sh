#!/bin/bash

# Este script realiza un backup de MariaDB usando autenticación por socket 
# y realiza una rotación automática de los archivos.
# Los parámetros son: db_name, bk_dir, bk_keep

# --- CONFIGURACIÓN ESTÁTICA ---
# El usuario de MariaDB para autenticación por socket (típicamente 'root')
DB_USER="root" 
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
# ------------------------------

# Función para mostrar el mensaje de uso
uso() {
    echo ""
    echo "❌ ERROR: Número de parámetros incorrecto."
    echo ""
    echo "Uso: $0 <db_name> <bk_dir> <bk_keep>"
    echo ""
    echo "  <db_name>  : Nombre de la base de datos a respaldar."
    echo "  <bk_dir>   : Directorio donde se guardará el backup."
    echo "  <bk_keep>  : Número de copias de seguridad a mantener (rotación)."
    echo ""
    echo "Ejemplo: $0 mi_app_db /var/backups/mariadb 5"
    echo ""
}

# 1. Validar el número de argumentos
if [ "$#" -ne 3 ]; then
    uso
    exit 1
fi

# 2. Asignar los parámetros a variables
DB_NAME="$1"
BACKUP_DIR="$2"
KEEP_BACKUPS="$3"
FILE_NAME="${DB_NAME}_${TIMESTAMP}.sql.gz"

echo "=========================================================="
echo "         INICIO DE BACKUP DE MARIA DB (bak-controla)"
echo "=========================================================="
echo "  Base de Datos     : ${DB_NAME}"
echo "  Directorio        : ${BACKUP_DIR}"
echo "  Copias a mantener : ${KEEP_BACKUPS}"
echo "----------------------------------------------------------"

# 3. Validar y Crear Directorio de Backup
if [ ! -d "$BACKUP_DIR" ]; then
    echo "ℹ️ Creando directorio de backup: ${BACKUP_DIR}"
    # Se usa 'mkdir -p' para crear el directorio si no existe y si sus padres tampoco.
    mkdir -p "$BACKUP_DIR"
    if [ $? -ne 0 ]; then
        echo "❌ Error al crear el directorio ${BACKUP_DIR}. Verifique permisos."
        exit 1
    fi
fi

# 4. Realizar el Volcado Comprimido (Autenticación por Socket)
echo "⏳ Realizando volcado de ${DB_NAME}..."
# Ejecutamos mysqldump como root para usar autenticación por socket.
# Es crucial que el usuario que ejecuta el script tenga permisos de sudo para mysqldump.
sudo mysqldump ${DB_NAME} | gzip > "${BACKUP_DIR}/${FILE_NAME}"

# Verificar el código de salida de mysqldump
if [ $? -eq 0 ]; then
    echo "✅ Copia de seguridad creada exitosamente: ${FILE_NAME}"
else
    echo "❌ ERROR: Falló la creación de la copia de seguridad de ${DB_NAME}."
    echo "   Verifique la conexión por socket y los permisos de MariaDB para ${DB_USER}."
    exit 1
fi

# 5. Rotación (Limpieza) de Backups Antiguos
echo "🧹 Iniciando rotación: Manteniendo las últimas ${KEEP_BACKUPS} copias..."

# Comando 'find' para listar, 'sort' para ordenar por fecha, 'tail' para saltar las N más nuevas, y 'xargs rm' para eliminar.
# Usamos -prune para optimizar la búsqueda, aunque aquí solo se busca en un directorio.
find "${BACKUP_DIR}" -name "${DB_NAME}_*.sql.gz" -type f | sort -r | tail -n +$((KEEP_BACKUPS + 1)) | xargs -r rm -f

# El código de salida de find/sort/tail/xargs no siempre indica un error crítico aquí, 
# pero podemos revisar si el comando de rotación se ejecutó correctamente.

echo "✅ Proceso de rotación completado."
echo "=========================================================="