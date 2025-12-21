#!/bin/bash

# Termina la ejecución si algún comando falla
set -e

# La constante para el puerto de destino
DESTINATION_PORT="3306"

# Variable global para la nueva IP (se inicializa en la función principal)
NEW_IP=""

# --- Funciones de UFW ---

# Función para verificar si UFW está instalado y activo
verificar_ufw() {
    if ! command -v ufw &> /dev/null; then
        echo "❌ Error: El comando 'ufw' no se encontró." >&2
        echo "Por favor, asegúrate de que UFW esté instalado en tu sistema." >2
        exit 1
    fi

    STATUS=$(sudo ufw status | head -n 1 | awk '{print $2}')
    if [ "$STATUS" != "active" ]; then
        echo "⚠️ Advertencia: UFW está actualmente '$STATUS'."
        echo "Las reglas se modificarán, pero no serán efectivas hasta que UFW se active."
    fi
}

# Función para modificar la regla UFW
modificar_regla_ufw() {
    echo "--- 📋 Modificación de Regla UFW ---"
    echo "Buscando reglas que afecten al puerto $DESTINATION_PORT..."
    # Muestra las reglas numeradas y captura la salida
    local UFW_STATUS=$(sudo ufw status numbered)
    echo "$UFW_STATUS"
    echo "-----------------------------------"

    # 1. Solicitar el número de regla
    read -p "➡️ UFW: Ingresa el NÚMERO de la regla a modificar (o deja vacío si no aplica): " RULE_NUMBER
    
    if [ -z "$RULE_NUMBER" ]; then
        echo "⏩ UFW: Saltando modificación de regla de UFW."
        return 0
    fi
    
    # Validación simple
    if ! [[ "$RULE_NUMBER" =~ ^[0-9]+$ ]]; then
        echo "❌ Error: Entrada inválida. Debes ingresar un número para UFW." >&2
        exit 1
    fi

    # 2. Confirmar la nueva IP (ya la tenemos en NEW_IP)
    local OLD_RULE_LINE=$(echo "$UFW_STATUS" | grep -E "\[\s*$RULE_NUMBER\]")

    if [ -z "$OLD_RULE_LINE" ]; then
        echo "❌ Error: No se encontró la regla UFW con el número [$RULE_NUMBER]." >&2
        exit 1
    fi

    # 3. Eliminar la regla antigua
    echo ""
    echo "⚙️ Aplicando cambios en UFW..."
    echo "1. Eliminando la regla antigua [$RULE_NUMBER]..."
    echo y | sudo ufw delete "$RULE_NUMBER"
    echo "   ✅ Regla [$RULE_NUMBER] eliminada."

    # 4. Añadir la nueva regla
    echo "2. Añadiendo la nueva regla: ALLOW FROM $NEW_IP TO ANY PORT $DESTINATION_PORT..."
    sudo ufw allow from "$NEW_IP" to any port "$DESTINATION_PORT"
    if [ $? -ne 0 ]; then
        echo "❌ Error al añadir la nueva regla UFW. Verifique el formato de la IP." >&2
        exit 1
    fi
    echo "   ✅ Nueva regla UFW añadida con éxito."
    
    echo ""
    echo "Lista de reglas UFW actualizada:"
    sudo ufw status numbered
    echo "--- ✅ UFW FINALIZADO ---"
}

# --- Funciones de MariaDB ---

# Función para verificar si el cliente mariadb está instalado
verificar_mariadb_client() {
    # Intenta usar 'mariadb' primero, luego 'mysql' como fallback
    if command -v mariadb &> /dev/null; then
        return 0
    elif command -v mysql &> /dev/null; then
        # Renombramos el comando para usarlo dentro de esta función
        alias mariadb='mysql'
        return 0
    else
        echo "❌ Error: No se encontró el cliente 'mariadb' ni 'mysql'." >&2
        echo "Asegúrate de tener el cliente de MariaDB/MySQL instalado." >2
        exit 1
    fi
}

# Función para modificar el host del usuario en MariaDB
modificar_host_mariadb() {
    echo ""
    echo "--- 💾 Modificación de Usuario MariaDB ---"
    
    # 1. Nombre de usuario por defecto
    DB_USER="elipcero"
    read -p "➡️ MariaDB: Ingresa el NOMBRE del usuario [default: $DB_USER]: " USER_INPUT
    if [ -n "$USER_INPUT" ]; then
        DB_USER="$USER_INPUT"
    fi

    # 2. Solicitar HOST ANTIGUO
    read -p "➡️ MariaDB: Ingresa el HOST ANTIGUO (ej. la IP vieja, dominio, o %) de '$DB_USER': " OLD_HOST

    if [ -z "$OLD_HOST" ]; then
        echo "❌ Error: El HOST ANTIGUO no puede estar vacío." >&2
        exit 1
    fi

    # 3. Comando SQL
    # La variable NEW_IP debe estar disponible globalmente
    local SQL_COMMAND="RENAME USER '$DB_USER'@'$OLD_HOST' TO '$DB_USER'@'$NEW_IP';"

    # Intenta ejecutar el comando de renombrar
    echo "Ejecutando SQL: $SQL_COMMAND"
    if sudo mariadb -e "$SQL_COMMAND"; then
        echo ""
        echo "✅ MariaDB: Host del usuario '$DB_USER' renombrado con éxito."
        echo "   Antiguo Host: $OLD_HOST"
        echo "   Nuevo Host: $NEW_IP"
        
        # Recargar privilegios
        sudo mariadb -e "FLUSH PRIVILEGES;"
        echo "   ✅ Privilegios recargados."
        
        # 4. Mostrar el resultado del SELECT
        echo ""
        echo "--- Resultado de la tabla mysql.user ---"
        sudo mariadb -e "SELECT User, Host FROM mysql.user;"
        echo "----------------------------------------"
    else
        echo "❌ Error al renombrar el usuario en MariaDB." >&2
        echo "Asegúrate de que: 1) El usuario '$DB_USER'@'$OLD_HOST' exista y 2) La instalación de MariaDB/MySQL permita acceso root vía sudo." >&2
        exit 1
    fi
    echo "--- ✅ MariaDB FINALIZADO ---"
}

# --- Ejecución Principal del Script ---

if [ "$EUID" -ne 0 ]; then
    echo "🔐 Por favor, ejecuta este script usando 'sudo' o como root." >&2
    exit 1
fi

# 1. Preguntar por la nueva IP (la variable clave que une ambos procesos)
read -p "➡️ GLOBAL: Ingresa la NUEVA dirección IP/Red que sustituirá a la antigua: " NEW_IP
if [ -z "$NEW_IP" ]; then
    echo "❌ Error: La Nueva IP no puede estar vacía." >&2
    exit 1
fi
echo "------------------------------------------------"

# Ejecución de funciones
verificar_ufw
modificar_regla_ufw

verificar_mariadb_client
modificar_host_mariadb

echo ""
echo "================================================="
echo "🎉 ¡Proceso Completado!"
echo "UFW y MariaDB se han actualizado para usar la IP: $NEW_IP"
echo "================================================="