#!/bin/bash

# --- Configuración de Nomenclatura y Flags ---
# Código de retorno para errores
ERR_EXIT=1
# Prefijo para la copia de seguridad
BACKUP_PREFIX="controla"
# Usamos 'set -e' para salir inmediatamente si un comando falla
# excepto donde se maneja el error explícitamente (como en las comprobaciones 'if [ -d ... ]')
set -e

# --- Variables Globales ---
# Directorio temporal para la extracción del ZIP
EXTRACT_TEMP=""
# Directorio temporal para la copia de seguridad de la instalación
BAK_TEMP=""
# El nombre del archivo del paquete debe pasarse como argumento
PACKAGE_FILE="$1"
# Argumento de ayuda
HELP_FLAG=0
SISTEMA="controla"

# --- Funciones Auxiliares ---

# Función para mostrar la ayuda de uso
# Función para mostrar la ayuda de uso
mostrar_ayuda() {
    echo "=========================================================================="
    echo "Uso: $0 [-h] [-cliente-db=herramienta] <fichero_tar_gz>"
    echo "=========================================================================="
    echo "Descomprime e instala aplicaciones (tenants) desde un paquete .tar.gz."
    echo ""
    echo "Argumentos:"
    echo "  <fichero_tar_gz>  Ruta al archivo comprimido con la estructura de instalación."
    echo "  -cliente-db       Aplicación cliente de la base de datos. (Por defecto: mariadb)."
    echo "  -h                Muestra esta ayuda detallada y sale."
    echo ""
    echo "RESUMEN DE INSTALACIÓN Y SEGURIDAD:"
    echo "--------------------------------------------------------------------------"
    echo "El script procesa cada aplicación (tenant) detectada y aplica las siguientes reglas:"
    echo ""
    echo "1. 📂 DIRECTORIOS Y DESTINOS:"
    echo "   • Binarios/Web:  /opt/<app>/           (Contenido previo es eliminado)"
    echo "   • Config:        /etc/<app>/           (config.json y secretos)"
    echo "   • Logs:          /var/log/<app>/       (Estructura y rotación)"
    echo "   • Systemd:       /etc/systemd/system/  (Servicios con prefijo '$SISTEMA-')"
    echo ""
    echo "2. 🔒 PERMISOS Y PROPIEDAD:"
    echo "   • Usuario:       Se crea un usuario de sistema único por aplicación."
    echo "   • /opt/<app>:    Propietario <app>, permisos 755 (644 para archivos web)."
    echo "   • /opt/<app>/api:Permisos 500 (Lectura/Ejecución restringida al dueño)."
    echo "   • /etc/<app>:    Permisos 500 (Solo lectura para el servicio)."
    echo "   • Secretos:      Permisos 400 (Solo lectura para el dueño, sin ejecución)."
    echo "   • Logs:          Permisos 600 (Solo lectura/escritura para el dueño)."
    echo ""
    echo "3. 🛡️ SEGURIDAD Y BACKUP:"
    echo "   • Antes de instalar, se crea un backup .tar.gz en el directorio actual"
    echo "     con el estado previo de /opt, /etc, servicios y logrotate."
    echo "   • Antes de ejecutar los scripts de db, se crea un backup de la base de datos."
    echo "   • Se ejecutan scripts SQL de forma ordenada (01_..., 02_...) en MariaDB."
    echo "--------------------------------------------------------------------------"
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
    if [ -n "$EXTRACT_TEMP" ] && [ -d "$EXTRACT_TEMP" ]; then
        echo "➡️ Limpiando directorio temporal de extracción: $EXTRACT_TEMP"
        rm -rf "$EXTRACT_TEMP" || echo "❌ No se pudo eliminar $EXTRACT_TEMP"
    fi
    if [ -n "$BAK_TEMP" ] && [ -d "$BAK_TEMP" ]; then
        echo "➡️ Limpiando directorio temporal de copia de seguridad: $BAK_TEMP"
        rm -rf "$BAK_TEMP" || echo "❌ No se pudo eliminar $BAK_TEMP"
    fi
}

# Comprobar si se está ejecutando como root (sudo)
if [ "$(id -u)" != "0" ]; then
   echo "❌ ERROR: Este script debe ejecutarse con permisos de super usuario (root), por ejemplo, usando 'sudo'." 
   exit 1
fi

# --- Procesamiento de Argumentos ---
DB_CLIENT="mariadb"
PACKAGE_FILE=""

for arg in "$@"; do
    case "$arg" in
        -cliente-db=*)
            DB_CLIENT="${arg#*=}"
            if [ -z "$DB_CLIENT" ]; then
              DB_CLIENT="mariadb"
            fi
            ;;            
        -h|--help)
            HELP_FLAG=1
            ;;
        -*)
            manejar_error "Opción inválida: $arg"
            ;;
        *)
            if [ -z "$PACKAGE_FILE" ]; then
                PACKAGE_FILE="$arg"
            else
                manejar_error "Argumento inesperado o múltiple fichero definido: $arg"
            fi
            ;;
    esac
done

if [ "$HELP_FLAG" -eq 1 ]; then
    mostrar_ayuda
    exit 0
fi

if [ -z "$PACKAGE_FILE" ]; then
    mostrar_ayuda
    manejar_error "Debe especificar un fichero .tar.gz para instalar."
fi

if [ ! -f "$PACKAGE_FILE" ]; then
    manejar_error "El fichero '$PACKAGE_FILE' no existe."
fi

# --- Comienzo del Script ---

echo "🚀 Iniciando proceso de instalación desde: $PACKAGE_FILE"

# Crear directorio temporal para la extracción
EXTRACT_TEMP=$(mktemp -d -t extract_XXXXXX) || manejar_error "No se pudo crear el directorio temporal para la extracción."
echo "➡️ Descomprimiendo '$PACKAGE_FILE' en temporal: $EXTRACT_TEMP"

# Descomprimir el .tar.gz
tar -xzf "$PACKAGE_FILE" -C "$EXTRACT_TEMP" || manejar_error "Fallo al descomprimir el fichero .tar.gz."
# Definir la ubicación de la extracción
EXTRACT_DIR="$EXTRACT_TEMP"

# Crear directorio temporal para las copias de seguridad de las carpetas de instalación
BAK_TEMP=$(mktemp -d -t install_bak_XXXXXX) || manejar_error "No se pudo crear el directorio temporal para la copia de seguridad."

# Definir la carpeta raíz para la iteración de aplicaciones (en orden de prioridad)
APP_ROOT=""
if [ -d "$EXTRACT_DIR/opt" ]; then
    APP_ROOT="$EXTRACT_DIR/opt"
    echo "Usando 'pack/opt' para la lista de aplicaciones."
elif [ -d "$EXTRACT_DIR/etc" ]; then
    APP_ROOT="$EXTRACT_DIR/etc"
    echo "Usando 'pack/etc' para la lista de aplicaciones."
elif [ -d "$EXTRACT_DIR/db" ]; then
    APP_ROOT="$EXTRACT_DIR/db"
    echo "Usando 'pack/db' para la lista de aplicaciones."
elif [ -d "$EXTRACT_DIR/log" ]; then
    APP_ROOT="$EXTRACT_DIR/log"
    echo "Usando 'pack/log' para la lista de aplicaciones."
elif [ -d "$EXTRACT_DIR/systemd" ]; then
    APP_ROOT="$EXTRACT_DIR/systemd"
    echo "Usando 'pack/systemd' para la lista de aplicaciones."
else
    manejar_error "No existe 'pack/opt', 'pack/etc' o 'pack/db'. No hay nada para instalar."
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

        # Creación del tar.gz de copia de seguridad
        BACKUP_FILE="$APP-$DATE_TIME.bak.tar.gz"
        echo "➡️ Creando fichero de copia de seguridad: **$BACKUP_FILE**"
        
        # Comprimir el directorio temporal de backup de esta aplicación
        tar -c -C "$BAK_TEMP" "$(basename "$APP_BAK_DIR")" | gzip -9 > "$BAK_TEMP/$BACKUP_FILE" || manejar_error "Fallo al crear el tar.gz de copia de seguridad $BACKUP_FILE."

        # Mover el tar.gz creado (que está en $BAK_TEMP) en el raíz de ejecución
        mv "$BAK_TEMP/$BACKUP_FILE" . || manejar_error "Fallo al mover el tar.gz de copia de seguridad al directorio actual."

        # Borrar la carpeta temporal de la copia de seguridad de esta aplicación
        rm -rf "$APP_BAK_DIR" || manejar_error "Fallo al borrar el directorio temporal de backup: $APP_BAK_DIR."
        echo "✅ Copia de seguridad creada como **$BACKUP_FILE** y temporal borrado."

        # Gestión del servicio systemd (Parar y Comprobación de la carpeta del paquete)
        EXTRACT_SYSTEMD_DIR="$EXTRACT_DIR/systemd/$APP"

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

        # Si existe pack/systemd, instalar
        if [ -d "$EXTRACT_SYSTEMD_DIR" ]; then
            echo "➡️ Instalando servicio systemd para **$APP** desde: $EXTRACT_SYSTEMD_DIR"
            # Copiar ficheros de servicio (asumimos que deben ir a /etc/systemd/system para overrides)
            cp -f "$EXTRACT_SYSTEMD_DIR"/* /etc/systemd/system/ || manejar_error "Fallo al copiar ficheros systemd para **$APP**."

            # Recargar daemon
            echo "➡️ Recargando systemd daemon y habilitando servicio."
            systemctl daemon-reload || manejar_error "Fallo en 'systemctl daemon-reload'."
            # Habilitar servicio (NOTA: el enunciado dice swpc.service, pero debe ser el de la aplicación)
            systemctl enable "$SERVICE_NAME" || manejar_error "Fallo al habilitar el servicio **$SERVICE_NAME**."
            echo "✅ Servicios systemd instalados y habilitados."
        fi

        # Si existe pack/opt, instalar
        EXTRACT_OPT_DIR="$EXTRACT_DIR/opt/$APP"
        if [ -d "$EXTRACT_OPT_DIR" ]; then
            echo "➡️ Instalando binarios de aplicación para **$APP** desde: $EXTRACT_OPT_DIR"
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
            
            # Copiar pack/opt/* a /opt/{app}
            echo "- Copiando archivos de aplicación a: $TARGET_OPT_DIR"
            # Usar cp -a para preservar enlaces simbólicos y permisos (mejor que /*, pero requiere recrear la estructura)
            cp -a "$EXTRACT_OPT_DIR/." "$TARGET_OPT_DIR" || manejar_error "Fallo al copiar pack/opt/$APP/ a $TARGET_OPT_DIR."
            echo "✅ Archivos copiados."

            TARGET_OPT_WEB="$TARGET_OPT_DIR/web"
            TARGET_OPT_API="$TARGET_OPT_DIR/api"
            
            # Cambiar propietario y permisos
            echo "- Estableciendo propietario a **$APP** y permiso recursivamente en: $TARGET_OPT_DIR"
            chown -R "$APP":"$APP" "$TARGET_OPT_DIR" || manejar_error "Fallo al cambiar el propietario de $TARGET_OPT_DIR."
            sudo chmod 755 "$TARGET_OPT_DIR" || manejar_error "Fallo al cambiar permisos de $TARGET_OPT_DIR."

            chown -R www-data:www-data "$TARGET_OPT_WEB" || manejar_error "Fallo al cambiar el propietario de $TARGET_OPT_WEB."
            sudo find "$TARGET_OPT_WEB" -type d -exec chmod 755 {} \;
            sudo find "$TARGET_OPT_WEB" -type f -exec chmod 644 {} \;

            sudo chmod -R 500 "$TARGET_OPT_API" || manejar_error "Fallo al cambiar permisos de $TARGET_OPT_API"
            echo "✅ Permisos y propietario ajustados."
        else
            echo "⚠️ Carpeta 'pack/opt/$APP' no encontrada. No se procede a su instalación."
        fi

        # Si existe pack/etc, instalar
        EXTRACT_ETC_DIR="$EXTRACT_DIR/etc/$APP"
        if [ -d "$EXTRACT_ETC_DIR" ]; then
            echo "➡️ Instalando configuración para **$APP** desde: $EXTRACT_ETC_DIR"
            TARGET_ETC_DIR="/etc/$APP"

            # Verificar y crear /etc/{app}
            if [ ! -d "$TARGET_ETC_DIR" ]; then
                mkdir -p "$TARGET_ETC_DIR" || manejar_error "Fallo al crear la carpeta: $TARGET_ETC_DIR."
                echo "✅ Carpeta **$TARGET_ETC_DIR** creada."
            fi
            
            # Copiar config.json
            EXTRACT_CONFIG="$EXTRACT_ETC_DIR/config.json"
            if [ -f "$EXTRACT_CONFIG" ]; then
                echo "- Copiando $EXTRACT_CONFIG a $TARGET_ETC_DIR/config.json"
                cp -f "$EXTRACT_CONFIG" "$TARGET_ETC_DIR/config.json" || manejar_error "Fallo al copiar config.json."
                echo "✅ config.json copiado."
            else
                echo "⚠️ Fichero 'pack/etc/$APP/config.json' no encontrado. No se procede a su instalación."
            fi

            # Copiar secretos
            EXTRACT_SECRETS_DIR="$EXTRACT_ETC_DIR/secretos"
            TARGET_SECRETS_DIR="$TARGET_ETC_DIR/secretos"
            if [ -d "$EXTRACT_SECRETS_DIR" ]; then
                echo "➡️ Copiando secretos de configuración."
                # Verificar y crear /etc/{app}/secretos
                if [ ! -d "$TARGET_SECRETS_DIR" ]; then
                    mkdir -p "$TARGET_SECRETS_DIR" || manejar_error "Fallo al crear la carpeta: $TARGET_SECRETS_DIR."
                    echo "✅ Carpeta **$TARGET_SECRETS_DIR** creada."
                fi
                # Copiar recursivamente
                cp -a "$EXTRACT_SECRETS_DIR/." "$TARGET_SECRETS_DIR" || manejar_error "Fallo al copiar pack/etc/$APP/secretos a $TARGET_SECRETS_DIR."
                echo "✅ Secretos copiados."
            fi

            # Cambiar propietario y permisos
            echo "➡️ Estableciendo propietario a **$APP** y permisos .  recursivamente en: $TARGET_ETC_DIR"
            chown -R "$APP":"$APP" "$TARGET_ETC_DIR" || manejar_error "Fallo al cambiar el propietario de $TARGET_ETC_DIR."
            chmod -R 500 "$TARGET_ETC_DIR" || manejar_error "Fallo al cambiar permisos de $TARGET_ETC_DIR."
            find "$TARGET_SECRETS_DIR" -type f -exec chmod 400 {} + || manejar_error "Fallo al cambiar permisos de $TARGET_SECRETS_DIR."
            echo "✅ Permisos y propietario ajustados."
        else
            echo "⚠️ Carpeta 'pack/etc/$APP' no encontrada. No se procede a su instalación."
        fi

        # Si existe pack/log, instalar
        EXTRACT_LOG_DIR="$EXTRACT_DIR/log/$APP"
        if [ -d "$EXTRACT_LOG_DIR" ]; then
            echo "➡️ Instalando logrotate de aplicación para **$APP** desde: $EXTRACT_LOG_DIR"
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
            cp -f "$EXTRACT_LOG_DIR/$SISTEMA-$APP.log" "$LOGRORATE_APP" || manejar_error "Fallo al copiar logrotate en $LOGRORATE_APP."            
            echo "✅ Configuración logrorate en '$LOGRORATE_APP' copiada."
        else
            echo "⚠️ Carpeta 'pack/log/$APP' no encontrada. No se procede a su instalación."
        fi

        # Si existe pack/db, instalar
        EXTRACT_DB_DIR="$EXTRACT_DIR/db/$APP"
        if [ -d "$EXTRACT_DB_DIR" ]; then
            echo "➡️ Ejecutando scripts SQL para **$APP** desde: $EXTRACT_DB_DIR"
            
            # Leer el nombre de la base de datos desde metadata
            METADATA_FILE="$EXTRACT_DB_DIR/metadata"
            if [ -f "$METADATA_FILE" ]; then
                DB_NOMBRE=$(head -n 1 "$METADATA_FILE")
                echo "   - Base de datos detectada: $DB_NOMBRE"
                
                # Crear copia de seguridad de la base de datos
                DB_BACKUP_FILE="$APP-$DATE_TIME-db.bak.sql.gz"
                echo "➡️ Creando copia de seguridad de la base de datos: **$DB_BACKUP_FILE**"
                
              # Verificar si la base de datos existe antes de hacer el dump
              if "$DB_CLIENT" -e "SELECT SCHEMA_NAME FROM INFORMATION_SCHEMA.SCHEMATA WHERE SCHEMA_NAME='$DB_NOMBRE'" 2>/dev/null | grep -q "$DB_NOMBRE"; then
                  # Realizar el dump de la base de datos y comprimirlo
                  if "$DB_CLIENT"-dump "$DB_NOMBRE" | gzip -9 > "$DB_BACKUP_FILE" 2>/dev/null; then
                      echo "✅ Copia de seguridad de la base de datos creada: **$DB_BACKUP_FILE**"
                  else
                      manejar_error "ERROR: No se pudo crear la copia de seguridad de la base de datos."
                  fi
              else
                  echo "ℹ️  Base de datos '$DB_NOMBRE' no existe. Saltando backup de BD."
              fi
            else
                manejar_error "No se encontró el fichero metadata. No se puede determinar el nombre de la base de datos."
            fi

            # Buscar ficheros .sql y ordenarlos por nombre (que incluye el número inicial)
            SQL_FILES=$(find "$EXTRACT_DB_DIR" -maxdepth 1 -type f -name "*.sql" | sort)
            
            if [ -n "$SQL_FILES" ]; then
                for SQL_FILE in $SQL_FILES; do
                    echo "   - Ejecutando script SQL: $(basename "$SQL_FILE")"
                    # Se requiere que el usuario tenga los permisos necesarios
                    "$DB_CLIENT" < "$SQL_FILE" || manejar_error "Fallo al ejecutar el script SQL: $(basename "$SQL_FILE")."
                done
                echo "✅ Todos los scripts SQL ejecutados."
            else
                echo "⚠️ No se encontraron ficheros .sql en 'pack/db/$APP'."
            fi
        else
            echo "⚠️ Carpeta 'pack/db/$APP' no encontrada. No se procede a su instalación."
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