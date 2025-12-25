#!/bin/bash

# Termina la ejecución si algún comando falla
set -e

# Este script sustituye los marcadores de posición en el archivo JSON
# ubicado en config
# y copia a un destino definido como parámetro.

# Verificar el número de argumentos
if [ "$#" -ne 12 ]; then
    echo "❌ ERROR: Uso incorrecto."
    echo "Uso: $0 <plantilla> <destino> <APP> <DB_SOCKET> <DB_USUARIO> <DB_NOMBRE> <DB_MAX_CONN> <LOG_LEVEL> <SERVIDOR_PUERTO> <SERVIDOR_PROD> <BOOT_ADMIN_CREAR> <BOOT_ADMIN_DNI>"

    exit 1
fi

echo "Generando el fichero de configuración..."

# Asignar los argumentos a variables
ORIGEN="$1"
DESTINO="$2"
APP="$3"
DB_SOCKET="$4"
DB_USUARIO="$5"
DB_NOMBRE="$6"
DB_MAX_CONN="$7"
LOG_LEVEL="$8"
SRV_PUERTO="$9"
SRV_PROD="${10}"
BOOT_ADMIN_CREAR="${11}"
BOOT_ADMIN_DNI="${12}"

# Verificar si el archivo de origen existe
if [ ! -f "$ORIGEN" ]; then
    echo "Error: El archivo de configuración de origen no existe en $ORIGEN"
    exit 1
fi

# Realizar las sustituciones usando sed
# Nota: La sintaxis 's|old|new|g' usa '|' como delimitador para evitar problemas con '/' en las rutas.
# También se usa '&' como referencia al patrón en 'sed'.

sed \
    -e "s|@APP|$APP|g" \
    -e "s|@DB_USUARIO|$DB_USUARIO|g" \
    -e "s|@DB_SOCKET|$DB_SOCKET|g" \
    -e "s|@DB_NOMBRE|$DB_NOMBRE|g" \
    -e "s|@DB_MAX_CONN|$DB_MAX_CONN|g" \
    -e "s|@LOG_LEVEL|$LOG_LEVEL|g" \
    -e "s|@SRV_PUERTO|$SRV_PUERTO|g" \
    -e "s|@SRV_PROD|$SRV_PROD|g" \
    -e "s|@BOOT_ADMIN_CREAR|$BOOT_ADMIN_CREAR|g" \
    -e "s|@BOOT_ADMIN_DNI|$BOOT_ADMIN_DNI|g" \
    "$ORIGEN" > "$DESTINO"

echo "✅ Configuración generada en: $DESTINO"