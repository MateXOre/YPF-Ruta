#!/usr/bin/env bash
# cliente/conexion.sh
# Uso: ./conexion.sh <ip> <puerto> <id> <monto>

set -euo pipefail

if [ "$#" -ne 4 ]; then
    echo "Uso: $0 <ip> <puerto> <id> <monto>" >&2
    exit 2
fi

host="$1"
port="$2"
id="$3"
monto="$4"

# intentar abrir conexión TCP en descriptor 3
if ! exec 3<>/dev/tcp/"$host"/"$port" 2>/dev/null; then
    echo "Error: no se pudo conectar a $host:$port" >&2
    exit 1
fi

printf "%s\n" "${id}=${monto}" >&3

# Espera PRIMER mensaje del servidor
if IFS= read -r response1 <&3; then
    printf "Mensaje 1: %s\n" "$response1"
else
    echo "Error: no se recibió primer mensaje" >&2
    exec 3>&- 2>/dev/null || true
    exit 1
fi

# Espera SEGUNDO mensaje del servidor
if IFS= read -r response2 <&3; then
    printf "Mensaje 2: %s\n" "$response2"
    echo "Conexión finalizada correctamente" >&2
else
    echo "Error: no se recibió segundo mensaje" >&2
fi

# cerrar la conexión y finalizar
exec 3>&- 2>/dev/null || true
exit 0