#!/bin/bash

# Script para restaurar una base de datos específica desde un backup completo de MariaDB
# Uso: ./restore_db.sh <nombre_bd> <fichero_backup.sql.gz> <ruta_destino>

# Verificar número de argumentos
if [ "$#" -ne 3 ]; then
    echo "Error: Número incorrecto de argumentos"
    echo "Uso: $0 <nombre_bd> <fichero_backup.sql.gz> <ruta_destino>"
    echo "Ejemplo: $0 controla /backups/backup_2024.sql.gz /tmp"
    exit 1
fi

# Asignar variables
DB_NAME="$1"
BACKUP_FILE="$2"
DEST_PATH="$3"
OUTPUT_FILE="${DEST_PATH}/${DB_NAME}_restore.sql"

# Colores para mensajes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Verificar que el fichero de backup existe
if [ ! -f "$BACKUP_FILE" ]; then
    echo -e "${RED}Error: El fichero de backup '$BACKUP_FILE' no existe${NC}"
    exit 1
fi

# Verificar que la ruta de destino existe
if [ ! -d "$DEST_PATH" ]; then
    echo -e "${YELLOW}La ruta de destino no existe. Creándola...${NC}"
    mkdir -p "$DEST_PATH"
    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: No se pudo crear la ruta de destino${NC}"
        exit 1
    fi
fi

# Verificar que el fichero es .gz
if [[ ! "$BACKUP_FILE" =~ \.gz$ ]]; then
    echo -e "${RED}Error: El fichero debe ser un archivo .gz${NC}"
    exit 1
fi

echo -e "${GREEN}=== Iniciando restauración de base de datos ===${NC}"
echo "Base de datos: $DB_NAME"
echo "Fichero backup: $BACKUP_FILE"
echo "Ruta destino: $DEST_PATH"
echo "Fichero salida: $OUTPUT_FILE"
echo ""

# Extraer la base de datos específica del backup
echo -e "${YELLOW}Extrayendo base de datos '$DB_NAME' del backup...${NC}"
gunzip < "$BACKUP_FILE" | awk -v db="$DB_NAME" '
    /^-- Current Database:/ {
        if ($0 ~ "`"db"`") {
            found=1
        } else if (found) {
            exit
        }
    }
    found {print}
' > "$OUTPUT_FILE"

# Verificar que se extrajo contenido
if [ ! -s "$OUTPUT_FILE" ]; then
    echo -e "${RED}Error: No se pudo extraer la base de datos o el fichero está vacío${NC}"
    echo -e "${YELLOW}Verificando si la base de datos existe en el backup...${NC}"
    gunzip < "$BACKUP_FILE" | grep "Current Database" | grep "$DB_NAME"
    if [ $? -ne 0 ]; then
        echo -e "${RED}La base de datos '$DB_NAME' no se encontró en el backup${NC}"
    fi
    rm -f "$OUTPUT_FILE"
    exit 1
fi

echo -e "${GREEN}✓ Extracción completada. Tamaño: $(du -h "$OUTPUT_FILE" | cut -f1)${NC}"
echo ""

# Añadir comandos para deshabilitar/habilitar foreign keys
echo -e "${YELLOW}Añadiendo configuración de FOREIGN_KEY_CHECKS...${NC}"
TEMP_FILE="${OUTPUT_FILE}.tmp"
(echo "SET FOREIGN_KEY_CHECKS=0;"; cat "$OUTPUT_FILE"; echo "SET FOREIGN_KEY_CHECKS=1;") > "$TEMP_FILE"
mv "$TEMP_FILE" "$OUTPUT_FILE"
echo -e "${GREEN}✓ Configuración añadida${NC}"
echo ""

# Preguntar si desea importar directamente
read -p "Edite antes el .sql y descomenta CHARACTER. ¿Desea importar la base de datos ahora? (s/n): " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Ss]$ ]]; then
    # Importar la base de datos (el dump ya incluye CREATE DATABASE y FOREIGN_KEY_CHECKS)
    echo -e "${YELLOW}Importando base de datos...${NC}"
    mysql -u root -p < "$OUTPUT_FILE"
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Base de datos restaurada exitosamente${NC}"
        
        # Preguntar si desea eliminar el fichero temporal
        read -p "¿Desea eliminar el fichero temporal '$OUTPUT_FILE'? (s/n): " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Ss]$ ]]; then
            rm -f "$OUTPUT_FILE"
            echo -e "${GREEN}✓ Fichero temporal eliminado${NC}"
        fi
    else
        echo -e "${RED}Error: Falló la importación de la base de datos${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}Fichero extraído guardado en: $OUTPUT_FILE${NC}"
    echo "Para importarlo manualmente ejecuta:"
    echo "  sudo mysql -u root -p < $OUTPUT_FILE"
fi

echo ""
echo -e "${GREEN}=== Proceso finalizado ===${NC}"