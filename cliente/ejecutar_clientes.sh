#!/usr/bin/env bash
# ejecutar_clientes.sh
# Ejecuta N clientes simultáneos con datos aleatorios
# Uso: ./ejecutar_clientes.sh <num_clientes> [host]

set -euo pipefail

if [ "$#" -lt 1 ]; then
    echo "Uso: $0 <num_clientes> [host]" >&2
    echo "Ejemplo: $0 10 127.0.0.1" >&2
    exit 2
fi

NUM_CLIENTES="$1"
HOST="${2:-127.0.0.1}"

# IDs de tarjetas disponibles
TARJETAS=(4 6 7 3 5 1 2)

# Puertos disponibles
PUERTOS=(10000 10001 10002 10003 10004 10005 10006 10007 10008 10009)

echo "🚀 Ejecutando $NUM_CLIENTES clientes hacia $HOST"
echo "================================================"

for i in $(seq 1 "$NUM_CLIENTES"); do
    # Seleccionar tarjeta aleatoria
    TARJETA_ID=${TARJETAS[$RANDOM % ${#TARJETAS[@]}]}
    
    # Seleccionar puerto aleatorio
    PORT=${PUERTOS[$RANDOM % ${#PUERTOS[@]}]}
    
    # Generar monto aleatorio entre 100 y 200000
    MONTO=$((100 + RANDOM % 200000))
    
    echo "Cliente $i: Tarjeta=$TARJETA_ID, Monto=$MONTO, Puerto=$PORT"
    
    # Ejecutar cliente en background
    bash "$(dirname "$0")/conexion.sh" "$HOST" "$PORT" "$TARJETA_ID" "$MONTO" > "/tmp/cliente_${i}_response.txt" 2>&1 &
    
    # Pequeño delay para no saturar el sistema
    sleep 0.05
done

echo ""
echo "⏳ Esperando que todos los clientes terminen..."
wait

echo ""
echo "📊 Resultados:"
echo "================================================"

for i in $(seq 1 "$NUM_CLIENTES"); do
    if [ -f "/tmp/cliente_${i}_response.txt" ]; then
        echo "Cliente $i:"
        cat "/tmp/cliente_${i}_response.txt" | head -n 2
        echo ""
        rm -f "/tmp/cliente_${i}_response.txt"
    fi
done

echo "✨ Todos los clientes han finalizado"
