#!/bin/bash

# Termina la ejecución si algún comando falla
set -e

PACK_DIR="./target/pack"
ESQUEMAS_DIR="./config/apps"

# Nombre del script para mensajes de error.
SCRIPT_NAME=$(basename "$0")

# Función para mostrar el uso del script y salir con error.
mostrar_uso_y_salir() {
echo "Uso: $SCRIPT_NAME [-h] [-crear] [-actualizar=seccion] [-script-db=nombre] [custom-config=carpeta] [-app=nombre]"
    echo ""
    echo "Paquetiza los ficheros y procesos a instalar de las aplicaciones (tenant) configuradas en ./config"
    echo ""
    echo "La configuración para paquetizar se encuentra en './config', aunque se puede elegir otra carpeta de configuración con el parámetro -custom-config:"
    echo "  - Carpeta db: Carpetas con los scripts sql. Cada script se ejecuta en el orden marcado por el número inicial."
    echo "  - Carpeta apps: El esquema de configuración por aplicación (tenant). Debe crear esta carpeta para crear su propio esquema de configuración por aplicación. Esta carpeta es ignorada por git."
    echo "  - Fichero config-api.json: Fichero plantilla de la configuración para el servicio api."
    echo "  - Fichero systemd.service: Fichero plantilla para construir el servicio api systemd."
    echo "  - Fichero logrotate: Fichero con la configuración para rotar log con logrotate. Si no existe no se aplica."
    echo ""
    echo "Si desea cambiar cualquier plantilla puede crear su propia carpeta de configuración con el parámetro -custom-config. Tenga en cuenta, que si elige esta opción, todos los ficheros de configuración debe estar ubicados en esta carpeta, inclusos los esquemas de las aplicaciones."
    echo ""
    echo "La carpeta ./config/apps debe contener la siguiente estructura:"
    echo "- Una carpeta por cada aplicación (tenant):"
    echo "  - Carpeta llamada 'secretos': Todos los ficheros de secretos:"
    echo "    - admin-passw: Fichero con la clave del administrador inicial de controla."  
    echo "    - secreto: Fichero con la clave para encriptar todo lo necesario dentro de la base de datos."  
    echo "  - Fichero config.var: Una línea por cada variable que se puede usar en las plantillas:"
    echo "    - @DB_SOCKET: Ruta al socket de la base de datos."
    echo "    - @DB_NOMBRE: Nombre de la base de datos."
    echo "    - @DB_MAX_CONN: Número máximo de conexiones abiertas de la base de datos."
    echo "    - @LOG_LEVEL: Nivel de log. (trace, debug, info, warn, error)"
    echo "    - @SRV_PUERTO: Puerto del servidor api."
    echo "    - @SRV_PROD: true, si es entorno de producción"
    echo "    - @BOOT_ADMIN_CREAR: true, si se crea el usuario admin al iniciar el servidor api con la clave definida en el secreto 'admin-passw'."
    echo "    - @BOOT_ADMIN_DNI: DNI del usuario admin."
    echo ""
    echo "Argumentos:"
    echo "  -h: Visualiza la ayuda."
    echo "  -crear: Paquetiza todo para crear una nueva app (tenant)."
    echo "  -actualizar=seccion: Paquetizar solo la sección especificada o la combinación de varias seperadas por coma sin espacios."
    echo "     Secciones:"
    echo "        bin: Construye y paquetiza los binarios."
    echo "        config: Paquetiza la configuración y secretos."
    echo "        servicio: Paquetiza la configuración del servicio systemd para el api."
    echo "        db: Paquetiza los scripts sql de la base de datos."
    echo "  -script-db=nombre: Carpeta de scripts sql (ubicados en ./config/db). Si se usa la opción -crear se utiliza directamente el script sql 'db/inicio'."
    echo "  -custom-config: Carpeta con los fichero personalizables de configuración."
    echo "  -app=nombre: Paquetizar solo la aplicación (tenant) especificada."
        
    exit 1
}

# Lee el fichero con las variables de configuración
leer_variables_configuracion() {
  local CONFIG_FILE="$1"
  
  if [ -f "$CONFIG_FILE" ]; then
    echo "     -> Leyendo archivo: $CONFIG_FILE"
    
    # Lee las líneas del archivo en un array
    readarray -t CONFIG_VARS < "$CONFIG_FILE"
    
    # Comprobar que tenemos 6 variables.
    if [ ${#CONFIG_VARS[@]} -lt 5 ]; then
          echo "❌ Error: El archivo $CONFIG_FILE debe contener exactamente 5 líneas (valores). Se encontraron ${#CONFIG_VARS[@]}." >&2
          exit
    fi

    DB_SOCKET="${CONFIG_VARS[0]}"
    DB_NOMBRE="${CONFIG_VARS[1]}"
    DB_MAX_CONN="${CONFIG_VARS[2]}"
    LOG_LEVEL="${CONFIG_VARS[3]}"
    SRV_PUERTO="${CONFIG_VARS[4]}"
    SRV_PROD="${CONFIG_VARS[5]}"
    BOOT_ADMIN_CREAR="${CONFIG_VARS[6]}"
    BOOT_ADMIN_DNI="${CONFIG_VARS[7]}"
  else
    echo "❌ Error: No se encontró el archivo de configuración en $CONFIG_FILE." >&2
    exit 1
  fi
}


# Función para crear las aplicaciones.
build() {
  find "$ESQUEMAS_DIR" -maxdepth 1 -type d -print | while read -r dir; do
    local APP_NAME=$(basename "$dir")

    if [ "$APP_NAME" != "$(basename "$ESQUEMAS_DIR")" ]; then
      # Si hay filtro de app, saltar si no coincide
      if [ -n "$APP_FILTRO" ] && [ "$APP_NAME" != "$APP_FILTRO" ]; then
        continue
      fi

      echo "  ➡️ Construyendo y desplegando la aplicación $APP_NAME..."

      # Se construye por cada app porque se incrusta la app
      # dentro de los binarios. Esto no es necesario en el servicio api
      # porque la app va en el fichero de configuración.
      ./scripts/web/build.sh "$APP_NAME"
      if [ $? -ne 0 ]; then
        echo "❌ Error: build.sh falló" >&2
        exit 1
      fi

      local RUST_SOURCE="./target/x86_64-unknown-linux-gnu/release/controla-api"
      local WEB_SOURCE="./web/dist"

      local PACK_OPT_APP="$PACK_DIR/opt/$APP_NAME"

      local BIN_DEST="$PACK_OPT_APP/api"
      local WEB_DEST="$PACK_OPT_APP/web"

      echo "Iniciando el proceso de preparación del despliegue..."

      # Crear directorios de destino si no existen
      echo "Verificando y creando directorios de destino..."

      mkdir -p "$BIN_DEST"
      mkdir -p "$WEB_DEST"
      echo "   - Directorios de despliegue listos."

      # Copiar el ejecutable de Rust
      echo "Copiando binario de Rust: $RUST_SOURCE a $BIN_DEST/"
      if [ -f "$RUST_SOURCE" ]; then
          cp "$RUST_SOURCE" "$BIN_DEST/"
          echo "   - ✅ Binario copiado exitosamente."
      else
          echo "❌ Error: El binario de Rust no se encontró en $RUST_SOURCE. ¿Ejecutaste 'api/build.sh'?"
          exit 1
      fi

      # Copiar los archivos de la aplicación web
      echo "Copiando artefactos Web: $WEB_SOURCE a $WEB_DEST/"
      if [ -d "$WEB_SOURCE" ]; then
          cp -r "$WEB_SOURCE"/* "$WEB_DEST/"
          echo "   - ✅ Archivos web copiados exitosamente."
      else
          echo "⚠️ Advertencia: El directorio web de distribución no se encontró en $WEB_SOURCE. ¿Ejecutaste 'web/build.sh'?"
      fi

      echo "✅ Despliegue listo en el directorio: **$DEST**"
    fi
  done  
}

config() {
  find "$ESQUEMAS_DIR" -maxdepth 1 -type d -print | while read -r dir; do
    local APP_NAME=$(basename "$dir")

    if [ "$APP_NAME" != "$(basename "$ESQUEMAS_DIR")" ]; then
      # Si hay filtro de app, saltar si no coincide
      if [ -n "$APP_FILTRO" ] && [ "$APP_NAME" != "$APP_FILTRO" ]; then
        continue
      fi
      
      echo "  ➡️ Generando el fichero de configuración para la aplicación: $APP_NAME..."

      local PACK_ETC_APP="$PACK_DIR/etc/$APP_NAME"
      local PACK_ETC_APP_CONFIG="$PACK_ETC_APP/config.json"
      mkdir -p $PACK_ETC_APP

      leer_variables_configuracion "$dir/config.var"    
      
      ./scripts/dist/config.sh "$CONFIG_FOLDER/config-api.json" "$PACK_ETC_APP_CONFIG" $APP_NAME $DB_SOCKET $APP_NAME $DB_NOMBRE $DB_MAX_CONN $LOG_LEVEL $SRV_PUERTO $SRV_PROD $BOOT_ADMIN_CREAR $BOOT_ADMIN_DNI

      # Copia los secretos
      cp -r "$dir/secretos" "$PACK_ETC_APP/secretos"
      if [ $? -ne 0 ]; then
        echo "❌ Error: La copia de secretos de la aplicación falló: $PACK_ETC_APP" >&2
        exit 1
      fi

      echo "✅ Se genero con existo el fichero de configuración en: $PACK_ETC_APP_CONFIG."
    fi
  done
}

servicio() {
  find "$ESQUEMAS_DIR" -maxdepth 1 -type d -print | while read -r dir; do
    local APP_NAME=$(basename "$dir")

    if [ "$APP_NAME" != "$(basename "$ESQUEMAS_DIR")" ]; then
      # Si hay filtro de app, saltar si no coincide
      if [ -n "$APP_FILTRO" ] && [ "$APP_NAME" != "$APP_FILTRO" ]; then
        continue
      fi
      
      local PLANTILLA_SYSTEMD="$CONFIG_FOLDER/systemd.service"
      
      echo "  ➡️ Generando el fichero servicio systemd para la aplicación: $APP_NAME..."

      if [ ! -f "$PLANTILLA_SYSTEMD" ]; then
          echo "❌ Error: La plantilla $PLANTILLA_SYSTEMD no existe."
          exit 1
      fi

      local PACK_SYS_APP="$PACK_DIR/systemd/$APP_NAME"

      mkdir -p $PACK_SYS_APP

      leer_variables_configuracion "$dir/config.var"    

      local SERVICIO="$PACK_SYS_APP/controla-$APP_NAME.service"

      sed \
          -e "s|@APP|$APP_NAME|g" \
          -e "s|@USUARIO|$APP_NAME|g" \
          "$PLANTILLA_SYSTEMD" > "$SERVICIO"

      echo "✅ Fichero servicio systemd creado en: $SERVICIO"
    fi
  done  
}

log() {
  find "$ESQUEMAS_DIR" -maxdepth 1 -type d -print | while read -r dir; do
    local APP_NAME=$(basename "$dir")

    if [ "$APP_NAME" != "$(basename "$ESQUEMAS_DIR")" ]; then
      # Si hay filtro de app, saltar si no coincide
      if [ -n "$APP_FILTRO" ] && [ "$APP_NAME" != "$APP_FILTRO" ]; then
        continue
      fi
      
      local PLANTILLA_LOGROTATE="$CONFIG_FOLDER/logrotate"
      
      echo "  ➡️ Generando el fichero logrotate para la aplicación: $APP_NAME..."

      if [ ! -f "$PLANTILLA_LOGROTATE" ]; then
          echo "⚠️ No existe configuración para log."
          exit 0
      fi

      local PACK_LOG_APP="$PACK_DIR/log/$APP_NAME"

      mkdir -p $PACK_LOG_APP

      leer_variables_configuracion "$dir/config.var"    

      local LOG="$PACK_LOG_APP/controla-$APP_NAME.log"

      sed \
          -e "s|@APP|$APP_NAME|g" \
          -e "s|@USUARIO|$APP_NAME|g" \
          "$PLANTILLA_LOGROTATE" > "$LOG"

      echo "✅ Fichero logrotate creado en: $LOG"
    fi
  done  
}

# Función que persisite un script sql para ejecutar
db() {
  if [ ! -n "$SCRIPTS_DB" ]; then
    return 0
  fi

  find "$ESQUEMAS_DIR" -maxdepth 1 -type d -print | while read -r dir; do
    local APP_NAME=$(basename "$dir")
    if [ "$APP_NAME" != "$(basename "$ESQUEMAS_DIR")" ]; then
      # Si hay filtro de app, saltar si no coincide
      if [ -n "$APP_FILTRO" ] && [ "$APP_NAME" != "$APP_FILTRO" ]; then
        continue
      fi
      
      local SCRIPTS_DB_DIR="$CONFIG_FOLDER/db/$SCRIPTS_DB"
      
      echo "  ➡️ Persistiendo los scripts SQL para la aplicación: $APP_NAME..."
      if [ ! -d "$SCRIPTS_DB_DIR" ]; then
          echo "❌ Error: La carpeta $SCRIPTS_DB_DIR no existe."
          exit 1
      fi
      
      local PACK_DB_APP="$PACK_DIR/db/$APP_NAME"
      mkdir -p "$PACK_DB_APP"

      leer_variables_configuracion "$dir/config.var"
      
      # Bucle para procesar cada fichero SQL
      find "$SCRIPTS_DB_DIR" -maxdepth 1 -type f -name "*.sql" | while read -r script_file; do
        local NOMBRE_FICHERO=$(basename "$script_file")
        echo "      Procesando script: $NOMBRE_FICHERO..."
        
        sed \
          -e "s|@USUARIO|$APP_NAME|g" \
          -e "s|@DB_NOMBRE|$DB_NOMBRE|g" \
          "$script_file" > "$PACK_DB_APP/$NOMBRE_FICHERO"

        # Guardar el nombre de la base de datos en metadata
        echo "$DB_NOMBRE" > "$PACK_DB_APP/metadata"
      done
      
      echo "✅ Scripts SQL procesados en: $PACK_DB_APP"
    fi
  done  
}


# Variables para almacenar los argumentos.
CREAR=false
ACTUALIZAR_SECCION=""
SCRIPTS_DB=""
APP_FILTRO=""
CONFIG_FOLDER="./config"

# Validación y lectura de Parámetros.

# Comprobar que se ha proporcionado al menos un argumento además del script.
if [ "$#" -lt 1 ]; then
    mostrar_uso_y_salir
fi

# Iterar sobre todos los argumentos.
for arg in "$@"; do
    case "$arg" in
        # Argumento de ayuda
        "-h"|"--help")
            mostrar_uso_y_salir
            ;;
        # Argumento opcional "-crear"
        "-crear")
            CREAR=true
            ;;
        # Argumento opcional "-script-db=nombre"
        -script-db=*)
            SCRIPTS_DB="${arg#*=}"
            if [ -z "$SCRIPTS_DB" ]; then
                echo "Error: El parámetro -script-db requiere un valor" >&2
                mostrar_uso_y_salir
            fi
            ;;
        # Argumento opcional "-app=nombre"
        -app=*)
            APP_FILTRO="${arg#*=}"
            if [ -z "$APP_FILTRO" ]; then
                echo "Error: El parámetro -app requiere un valor" >&2
                mostrar_uso_y_salir
            fi
            ;;
        -actualizar=*)
            ACTUALIZAR_SECCION="${arg#*=}"
            if [ -z "$ACTUALIZAR_SECCION" ]; then
                echo "Error: El parámetro -actualizar requiere un valor" >&2
                mostrar_uso_y_salir
            fi
            ;;            
        -custom-config=*)
            CONFIG_FOLDER="${arg#*=}"
            if [ -z "$CONFIG_FOLDER" ]; then
                echo "Error: El parámetro -custom-config requiere una carpeta válida" >&2
                mostrar_uso_y_salir
            fi
            ;;
        # Cualquier otro argumento no reconocido
        *)
            echo "Error: Argumento no reconocido: $arg" >&2
            mostrar_uso_y_salir
            ;;
    esac
done

# Verificar directorio del paquete destino
if [ -d "$PACK_DIR" ]; then
  rm -r "$PACK_DIR"
fi

mkdir -p "$PACK_DIR"

# Actualizar ESQUEMAS_DIR con la carpeta de configuración personalizada
ESQUEMAS_DIR="$CONFIG_FOLDER/apps"

# Leer la carpeta de Esquemas.
echo "---"
echo "Directorio de aplicaciones: $ESQUEMAS_DIR"
echo "---"
echo "🚀 Iniciando el proceso de empaquetado..."

if $CREAR; then
  echo "✨ Iniciando el proceso para la creación..."
  SCRIPTS_DB=inicio
  build
  config
  servicio
  log
  db
else
  # Dividimos por comas
  IFS=',' read -ra SECCIONES <<< "$ACTUALIZAR_SECCION"

  for seccion in "${SECCIONES[@]}"; do
    # 2. LIMPIEZA: Eliminamos espacios en blanco accidentales
    seccion_limpia="${seccion// /}"
    
    case "$seccion_limpia" in
      "bin") build ;;
      "config")   config ;;
      "servicio") servicio ;;
      "db") db ;;
      "")         ;; 
      *)          echo "⚠️ Sección desconocida: '$seccion_limpia'" ;;
    esac
  done
fi

(
    cd $PACK_DIR
    rm -f ../controla-pack.tar.gz
    # Empaquetar con tar y comprimir con gzip (máxima compresión -9)
    tar -c . | gzip -9 > ../controla-pack.tar.gz
)

if [ $? -eq 0 ]; then
    echo "✅ Paquete 'controla-pack.tar.gz' generado con éxito en el directorio ./target."
else
    echo "❌ Error al generar el paquete tar.gz." >&2
fi

echo "🎉 Script completado."
