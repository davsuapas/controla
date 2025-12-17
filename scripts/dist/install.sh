#!/bin/bash

# --- Configuración de Nomenclatura y Flags ---
# Código de retorno para errores
ERR_EXIT=1
# Prefijo para la copia de seguridad
BACKUP_PREFIX="controla"
# Cliente de base de datos a usar
DB_CLIENT="mariadb"
# Usamos 'set -e' para salir inmediatamente si un comando falla
# excepto donde se maneja el error explícitamente (como en las comprobaciones 'if [ -d ... ]')
set -e

# --- Variables Globales ---
# Directorio temporal para la extracción del ZIP
ZIP_TEMP=""
# Directorio temporal para la copia de seguridad de la instalación
BAK_TEMP=""
# El nombre del archivo ZIP debe pasarse como argumento
ZIP_FILE="$1"
# Argumento de ayuda
HELP_FLAG=0
SISTEMA="controla"

# --- Funciones Auxiliares ---

# Función para mostrar la ayuda de uso
mostrar_ayuda() {
    echo "Uso: $0 [-h] <fichero_zip>"
    echo ""
    echo "Descomprime e instala las aplicaciones contenidas en el fichero ZIP."
    echo ""
    echo "Argumentos:"
    echo "  <fichero_zip>  El path al fichero .zip que contiene la estructura de instalación."
    echo "  -h             Muestra esta ayuda y sale."
}

# Función de salida con mensaje de error
manejar_error() {
    local error_msg="$1"
    echo "❌ ERROR: $error_msg" >&2
    # Intentar limpiar temporales antes de salir
    limpiar_temporales
    exit $ERR_EXIT
}

# Función para limpiar directorios temporales
limpiar_temporales() {
    if [ -n "$ZIP_TEMP" ] && [ -d "$ZIP_TEMP" ]; then
        echo "➡️ Limpiando directorio temporal de extracción: $ZIP_TEMP"
        rm -rf "$ZIP_TEMP" || echo "❌ No se pudo eliminar $ZIP_TEMP"
    fi
    if [ -n "$BAK_TEMP" ] && [ -d "$BAK_TEMP" ]; then
        echo "➡️ Limpiando directorio temporal de copia de seguridad: $BAK_TEMP"
        rm -rf "$BAK_TEMP" || echo "❌ No se pudo eliminar $BAK_TEMP"
    fi
}

# Comprobar si se está ejecutando como root (sudo)
if [ "$(id -u)" != "0" ]; then
   echo "❌ ERROR: Este script debe ejecutarse con permisos de superusuario (root), por ejemplo, usando 'sudo'." 
   exit 1
fi

# --- Procesamiento de Argumentos ---
while getopts "h" opt; do
    case ${opt} in
        h)
            HELP_FLAG=1
            ;;
        \?)
            # Argumento inválido
            manejar_error "Opción inválida: -$OPTARG"
            ;;
    esac
done
shift $((OPTIND -1))

if [ "$HELP_FLAG" -eq 1 ]; then
    mostrar_ayuda
    exit 0
fi

# El fichero ZIP es el argumento posicional restante
ZIP_FILE="$1"

if [ -z "$ZIP_FILE" ]; then
    mostrar_ayuda
    manejar_error "Debe especificar un fichero ZIP para instalar."
fi

if [ ! -f "$ZIP_FILE" ]; then
    manejar_error "El fichero ZIP '$ZIP_FILE' no existe."
fi

# --- Comienzo del Script ---

echo "🚀 Iniciando proceso de instalación desde: $ZIP_FILE"

# Crear directorio temporal para la extracción
ZIP_TEMP=$(mktemp -d -t zip_extract_XXXXXX) || manejar_error "No se pudo crear el directorio temporal para la extracción."
echo "➡️ Descomprimiendo '$ZIP_FILE' en temporal: $ZIP_TEMP"

# Descomprimir el ZIP
unzip -q "$ZIP_FILE" -d "$ZIP_TEMP" || manejar_error "Fallo al descomprimir el fichero ZIP."
# Definir la ubicación de la extracción para la nomenclatura "zip/{carpetas}"
ZIP_DIR="$ZIP_TEMP"

# Crear directorio temporal para las copias de seguridad de las carpetas de instalación
BAK_TEMP=$(mktemp -d -t install_bak_XXXXXX) || manejar_error "No se pudo crear el directorio temporal para la copia de seguridad."

# Definir la carpeta raíz para la iteración de aplicaciones (en orden de prioridad)
APP_ROOT=""
if [ -d "$ZIP_DIR/x" ]; then
    APP_ROOT="$ZIP_DIR/x"
    echo "Usando 'zip/x' para la lista de aplicaciones."
elif [ -d "$ZIP_DIR/etc" ]; then
    APP_ROOT="$ZIP_DIR/etc"
    echo "Usando 'zip/etc' para la lista de aplicaciones."
elif [ -d "$ZIP_DIR/db" ]; then
    APP_ROOT="$ZIP_DIR/db"
    echo "Usando 'zip/db' para la lista de aplicaciones."
elif [ -d "$ZIP_DIR/log" ]; then
    APP_ROOT="$ZIP_DIR/log"
    echo "Usando 'zip/log' para la lista de aplicaciones."
elif [ -d "$ZIP_DIR/systemd" ]; then
    APP_ROOT="$ZIP_DIR/systemd"
    echo "Usando 'zip/systemd' para la lista de aplicaciones."
else
    manejar_error "No existe 'zip/x', 'zip/etc' o 'zip/db'. No hay nada para instalar."
fi

# --- Loop de Instalación por Aplicación ---
for APP_PATH in "$APP_ROOT"/*; do
    if [ -d "$APP_PATH" ]; then
        APP_NAME=$(basename "$APP_PATH")
        echo -e "\n--- 📦 Procesando aplicación: **$APP_NAME** ---"

        # Obtener el nombre de la aplicación
        APP="$APP_NAME"

        # Crear usuario si no existe
        if ! id "$APP" &>/dev/null; then
            echo "➡️ Creando usuario de sistema: **$APP**"
            useradd -r -s /bin/false "$APP" || manejar_error "Fallo al crear el usuario **$APP**."
            echo "✅ Usuario **$APP** creado."
        else
            echo "✅ Usuario **$APP** ya existe."
        fi

        # Copia de seguridad temporal de las carpetas de instalación existentes
        DATE_TIME=$(date +%Y%m%d_%H%M%S)
        APP_BAK_DIR="$BAK_TEMP/$APP-$DATE_TIME"
        mkdir -p "$APP_BAK_DIR" || manejar_error "Fallo al crear el directorio temporal de copia de seguridad: $APP_BAK_DIR."
        echo "➡️ Copiando configuraciones existentes a temporal de backup: $APP_BAK_DIR"

        if [ -d "/opt/$APP" ]; then
            echo "   - Copiando /opt/$APP..."
            cp -a "/opt/$APP" "$APP_BAK_DIR/opt-$APP" || manejar_error "Fallo al copiar /opt/$APP para backup."
        fi
        if [ -d "/etc/$APP" ]; then
            echo "   - Copiando /etc/$APP..."
            cp -a "/etc/$APP" "$APP_BAK_DIR/etc-$APP" || manejar_error "Fallo al copiar /etc/$APP para backup."
        fi

        SERVICE_NAME="$SISTEMA-$APP.service"

        # Los servicios systemd están en /etc/systemd/system o /lib/systemd/system. Solo copiamos el específico.
        SERVICE_FILE="/etc/systemd/system/$SERVICE_NAME"
        if [ -f "$SERVICE_FILE" ]; then
            echo "   - Copiando $SERVICE_FILE..."
            cp -a "$SERVICE_FILE" "$APP_BAK_DIR/" || manejar_error "Fallo al copiar $SERVICE_FILE para backup."
        fi
        # Se copia el servicio logrotate específico
        LOGROTATE_FILE="/etc/logrotate.d/$SISTEMA-$APP"
        if [ -f "$LOGROTATE_FILE" ]; then
            echo "   - Copiando $LOGROTATE_FILE..."
            cp -a "$LOGROTATE_FILE" "$APP_BAK_DIR/" || manejar_error "Fallo al copiar $LOGROTATE_FILE para backup."
        fi

        # Creación del ZIP de copia de seguridad
        BACKUP_ZIP_FILE="$APP-$DATE_TIME.bak"
        echo "➡️ Creando fichero de copia de seguridad: **$BACKUP_ZIP_FILE**"
        
        # Comprimir el directorio temporal de backup de esta aplicación
        (
            cd "$APP_BAK_DIR/.." || manejar_error "Fallo al cambiar al directorio para crear el ZIP de backup."
            zip -qr "$BACKUP_ZIP_FILE" "$(basename "$APP_BAK_DIR")" || manejar_error "Fallo al crear el ZIP de copia de seguridad $BACKUP_ZIP_FILE."
        )

        # Mover el ZIP creado (que está en $BAK_TEMP) en el raíz de ejecución
        mv "$BAK_TEMP/$BACKUP_ZIP_FILE" . || manejar_error "Fallo al mover el ZIP de copia de seguridad al directorio actual."

        # Borrar la carpeta temporal de la copia de seguridad de esta aplicación
        rm -rf "$APP_BAK_DIR" || manejar_error "Fallo al borrar el directorio temporal de backup: $APP_BAK_DIR."
        echo "✅ Copia de seguridad creada como **$BACKUP_ZIP_FILE** y temporal borrado."

        # Gestión del servicio systemd (Parar y Comprobación de la carpeta ZIP)
        ZIP_SYSTEMD_DIR="$ZIP_DIR/systemd/$APP"

        # Si el servicio existe, pararlo
        if systemctl is-active --quiet "$SERVICE_NAME"; then
            echo "➡️ Parando servicio existente: **$SERVICE_NAME**"
            systemctl stop "$SERVICE_NAME" || manejar_error "Fallo al parar el servicio **$SERVICE_NAME**."
            echo "✅ Servicio **$SERVICE_NAME** parado."
        elif systemctl is-enabled --quiet "$SERVICE_NAME"; then
            echo "⚠️ Servicio **$SERVICE_NAME** no activo, pero está habilitado. Continuando."
        else
            echo "⚠️ Servicio **$SERVICE_NAME** no existe o no está habilitado."
        fi

        # Si existe zip/systemd, instalar
        if [ -d "$ZIP_SYSTEMD_DIR" ]; then
            echo "➡️ Instalando servicio systemd para **$APP** desde: $ZIP_SYSTEMD_DIR"
            # Copiar ficheros de servicio (asumimos que deben ir a /etc/systemd/system para overrides)
            cp -f "$ZIP_SYSTEMD_DIR"/* /etc/systemd/system/ || manejar_error "Fallo al copiar ficheros systemd para **$APP**."

            # Recargar daemon
            echo "➡️ Recargando systemd daemon y habilitando servicio."
            systemctl daemon-reload || manejar_error "Fallo en 'systemctl daemon-reload'."
            # Habilitar servicio (NOTA: el enunciado dice swpc.service, pero debe ser el de la aplicación)
            systemctl enable "$SERVICE_NAME" || manejar_error "Fallo al habilitar el servicio **$SERVICE_NAME**."
            echo "✅ Servicios systemd instalados y habilitados."
        fi

        # Si existe zip/opt, instalar
        ZIP_OPT_DIR="$ZIP_DIR/opt"
        if [ -d "$ZIP_OPT_DIR" ]; then
            echo "➡️ Instalando binarios de aplicación para **$APP** desde: $ZIP_OPT_DIR"
            TARGET_OPT_DIR="/opt/$APP"

            # Verificar y crear /opt/{app}
            if [ ! -d "$TARGET_OPT_DIR" ]; then
                mkdir -p "$TARGET_OPT_DIR" || manejar_error "Fallo al crear la carpeta: $TARGET_OPT_DIR."
                echo "✅ Carpeta **$TARGET_OPT_DIR** creada."
            fi

            # Eliminar contenido existente (antes de cambiar chown para evitar problemas de permisos)
            if [ "$(ls -A $TARGET_OPT_DIR 2>/dev/null)" ]; then
                echo "- Eliminando contenido existente en: $TARGET_OPT_DIR"
                rm -rf "$TARGET_OPT_DIR"/* || manejar_error "Fallo al eliminar el contenido de $TARGET_OPT_DIR."
            else
                echo "- Carpeta **$TARGET_OPT_DIR** vacía."
            fi
            
            # Copiar zip/opt/* a /opt/{app}
            echo "- Copiando archivos de aplicación a: $TARGET_OPT_DIR"
            # Usar cp -a para preservar enlaces simbólicos y permisos (mejor que /*, pero requiere recrear la estructura)
            cp -a "$ZIP_OPT_DIR/." "$TARGET_OPT_DIR" || manejar_error "Fallo al copiar zip/opt/$APP/ a $TARGET_OPT_DIR."
            echo "✅ Archivos copiados."
            
            # Cambiar propietario y permisos
            echo "- Estableciendo propietario a **$APP** y permisos 500 recursivamente en: $TARGET_OPT_DIR"
            chown -R "$APP":"$APP" "$TARGET_OPT_DIR" || manejar_error "Fallo al cambiar el propietario de $TARGET_OPT_DIR."
            chmod -R 500 "$TARGET_OPT_DIR" || manejar_error "Fallo al cambiar permisos de $TARGET_OPT_DIR."
            echo "✅ Permisos y propietario ajustados."
        else
            echo "⚠️ Carpeta 'zip/opt/$APP' no encontrada."
        fi

        # Si existe zip/etc, instalar
        ZIP_ETC_DIR="$ZIP_DIR/etc/$APP"
        if [ -d "$ZIP_ETC_DIR" ]; then
            echo "➡️ Instalando configuración para **$APP** desde: $ZIP_ETC_DIR"
            TARGET_ETC_DIR="/etc/$APP"

            # Verificar y crear /etc/{app}
            if [ ! -d "$TARGET_ETC_DIR" ]; then
                mkdir -p "$TARGET_ETC_DIR" || manejar_error "Fallo al crear la carpeta: $TARGET_ETC_DIR."
                echo "✅ Carpeta **$TARGET_ETC_DIR** creada."
            fi
            
            # Copiar config.json
            ZIP_CONFIG="$ZIP_ETC_DIR/config.json"
            if [ -f "$ZIP_CONFIG" ]; then
                echo "- Copiando $ZIP_CONFIG a $TARGET_ETC_DIR/config.json"
                cp -f "$ZIP_CONFIG" "$TARGET_ETC_DIR/config.json" || manejar_error "Fallo al copiar config.json."
                echo "✅ config.json copiado."
            else
                echo "⚠️ Fichero 'zip/etc/$APP/config.json' no encontrado."
            fi

            # Copiar secretos
            ZIP_SECRETS_DIR="$ZIP_ETC_DIR/secretos"
            TARGET_SECRETS_DIR="$TARGET_ETC_DIR/secretos"
            if [ -d "$ZIP_SECRETS_DIR" ]; then
                echo "➡️ Copiando secretos de configuración."
                # Verificar y crear /etc/{app}/secretos
                if [ ! -d "$TARGET_SECRETS_DIR" ]; then
                    mkdir -p "$TARGET_SECRETS_DIR" || manejar_error "Fallo al crear la carpeta: $TARGET_SECRETS_DIR."
                    echo "✅ Carpeta **$TARGET_SECRETS_DIR** creada."
                fi
                # Copiar recursivamente
                cp -a "$ZIP_SECRETS_DIR/." "$TARGET_SECRETS_DIR" || manejar_error "Fallo al copiar zip/etc/$APP/secretos a $TARGET_SECRETS_DIR."
                echo "✅ Secretos copiados."
            fi

            # Cambiar propietario y permisos
            echo "➡️ Estableciendo propietario a **$APP** y permisos 500 recursivamente en: $TARGET_ETC_DIR"
            chown -R "$APP":"$APP" "$TARGET_ETC_DIR" || manejar_error "Fallo al cambiar el propietario de $TARGET_ETC_DIR."
            # Permisos 500 (lectura solo para el propietario) recursivamente
            chmod -R 500 "$TARGET_ETC_DIR" || manejar_error "Fallo al cambiar permisos de $TARGET_ETC_DIR."
            echo "✅ Permisos y propietario ajustados."
        else
            echo "⚠️ Carpeta 'zip/etc/$APP' no encontrada."
        fi

        # Si existe zip/log, instalar
        ZIP_LOG_DIR="$ZIP_DIR/log/$APP"
        if [ -d "$ZIP_LOG_DIR" ]; then
            echo "➡️ Instalando logrotate de aplicación para **$APP** desde: $ZIP_LOG_DIR"
            TARGET_LOG_DIR="/var/log/$APP"

            # Verificar y crear var/log/{app}
            if [ ! -d "/var/log/$APP" ]; then
                echo "   - Creando /var/log/$APP..."
                mkdir -p "/var/log/$APP" || manejar_error "Fallo al crear el directorio con el log /var/log/$APP"

                # Cambiar propietario y permisos
                echo "- Estableciendo propietario a **$APP** y permisos 600 recursivamente en: $TARGET_LOG_DIR"
                chown -R "$APP":"$APP" "$TARGET_LOG_DIR" || manejar_error "Fallo al cambiar el propietario de $TARGET_LOG_DIR."
                chmod -R 600 "$TARGET_LOG_DIR" || manejar_error "Fallo al cambiar permisos de $TARGET_LOG_DIR."
                echo "✅ Permisos y propietario ajustados."                
            fi

            LOGRORATE_APP="/etc/logrotate.d/$SISTEMA-$APP"

            # Copiar recursivamente
            cp -f "$ZIP_LOG_DIR/$SISTEMA-$APP.log" "$LOGRORATE_APP" || manejar_error "Fallo al copiar logrotate en $LOGRORATE_APP."            
            echo "✅ Configuración logrorate en '$LOGRORATE_APP' copiada."
        else
            echo "⚠️ Carpeta 'zip/log/$APP' no encontrada."
        fi

        # Si existe zip/db, instalar
        ZIP_DB_DIR="$ZIP_DIR/db/$APP"
        if [ -d "$ZIP_DB_DIR" ]; then
            echo "➡️ Ejecutando scripts SQL para **$APP** desde: $ZIP_DB_DIR"
            
            # Buscar ficheros .sql y ordenarlos por nombre (que incluye el número inicial)
            SQL_FILES=$(find "$ZIP_DB_DIR" -maxdepth 1 -type f -name "*.sql" | sort)
            
            if [ -n "$SQL_FILES" ]; then
                for SQL_FILE in $SQL_FILES; do
                    echo "   - Ejecutando script SQL: $(basename "$SQL_FILE")"
                    # Se requiere que el usuario tenga los permisos necesarios
                    "$DB_CLIENT" < "$SQL_FILE" || manejar_error "Fallo al ejecutar el script SQL: $(basename "$SQL_FILE")."
                done
                echo "✅ Todos los scripts SQL ejecutados."
            else
                echo "⚠️ No se encontraron ficheros .sql en 'zip/db/$APP'."
            fi
        else
            echo "⚠️ Carpeta 'zip/db/$APP' no encontrada."
        fi

      # Iniciar el servicio y mostrar estado
      if systemctl is-enabled --quiet "$SERVICE_NAME"; then
          echo "➡️ Iniciando servicio: **$SERVICE_NAME**"
          systemctl start "$SERVICE_NAME" || manejar_error "Fallo al iniciar el servicio **$SERVICE_NAME**."
          echo "✅ Servicio iniciado. Estado actual:"
          systemctl status "$SERVICE_NAME" --no-pager
      else
          # Este 'else' ahora solo se activa si no está habilitado (lo que implica que no se instaló correctamente)
          echo "⚠️ No se puede iniciar el servicio **$SERVICE_NAME** (no está habilitado o no se encontró)."
      fi
    fi
done

# --- Fin del Script ---
limpiar_temporales
echo -e "\n🎉 Proceso de instalación finalizado correctamente."
exit 0