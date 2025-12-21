#!/bin/bash

# Termina la ejecución si algún comando falla
set -e

# Este script sustituye los marcadores de posición en el archivo JSON
# ubicado en config
# y copia a un destino definido como parámetro.

# Verificar el número de argumentos
if [ "$#" -ne 7 ]; then
    echo "❌ ERROR: Uso incorrecto."
    echo "Uso: $0 <destino> <DB_USUARIO> <DB_NOMBRE> <DB_MAX_CONN> <LOG_LEVEL> <SERVIDOR_PUERTO> <SERVIDOR_PROD>"
    exit 1
fi

echo "Generando el fichero de configuración..."

# Asignar los argumentos a variables
ORIGEN="./config/config-api.json"
DESTINO="$1"
USUARIO="$2"
DB_NOMBRE="$3"
DB_MAX_CONN="$4"
LOG_LEVEL="$5"
SRV_PUERTO="$6"
SRV_PROD="$7"

# Verificar si el archivo de origen existe
if [ ! -f "$ORIGEN" ]; then
    echo "Error: El archivo de configuración de origen no existe en $ORIGEN"
    exit 1
fi

# Realizar las sustituciones usando sed
# Nota: La sintaxis 's|old|new|g' usa '|' como delimitador para evitar problemas con '/' en las rutas.
# También se usa '&' como referencia al patrón en 'sed'.

sed \
    -e "s|@USUARIO|$USUARIO|g" \
    -e "s|@DB_NOMBRE|$DB_NOMBRE|g" \
    -e "s|@DB_MAX_CONN|$DB_MAX_CONN|g" \
    -e "s|@LOG_LEVEL|$LOG_LEVEL|g" \
    -e "s|@SRV_PUERTO|$SRV_PUERTO|g" \
    -e "s|@SRV_PROD|$SRV_PROD|g" \
    "$ORIGEN" > "$DESTINO"

echo "✅ Configuración generada en: $DESTINO"