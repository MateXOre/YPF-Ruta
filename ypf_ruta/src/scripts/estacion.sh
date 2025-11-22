#!/usr/bin/env bash
# Script para simular una estación conectándose a YPF Ruta
# Uso: ./estacion.sh [puerto_ypf]

set -euo pipefail

YPF_HOST="127.0.0.1"
YPF_PORT="${1:-18080}"  # Puerto por defecto: 8080 + 10000 = 18080

echo "Conectando a YPF Ruta en ${YPF_HOST}:${YPF_PORT}..."

VENTAS_JSON='[{"id": 100,"tarjeta_id": 1,"estacion_id": 999,"monto": 5000,"fecha": "2024-11-21T10:30:00Z"},{"id": 101,"tarjeta_id": 2,"estacion_id": 999,"monto": 3000,"fecha": "2024-11-21T11:00:00Z"}]'

echo ""
echo "📤 Enviando ventas:"
echo "$VENTAS_JSON" | jq '.' 2>/dev/null || echo "$VENTAS_JSON"
echo ""

if exec 3<>/dev/tcp/"${YPF_HOST}"/"${YPF_PORT}" 2>/dev/null; then
    echo "✅ Conexión establecida"
    
    printf "%s\n" "$VENTAS_JSON" >&3
    echo "📨 Ventas enviadas, esperando respuesta..."
    echo ""
    
    if timeout 10 cat <&3; then
        echo ""
        echo "✅ Respuesta recibida del servidor"
    else
        echo ""
        echo "⏱️  Timeout esperando respuesta (10s)"
    fi
    
    exec 3>&-
    echo ""
    echo "🔌 Conexión cerrada"
else
    echo "❌ Error: No se pudo conectar a ${YPF_HOST}:${YPF_PORT}"
    echo "   Asegúrate de que YPF Ruta esté ejecutándose con el puerto correcto"
    exit 1
fi
