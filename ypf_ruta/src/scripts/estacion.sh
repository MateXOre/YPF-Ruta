#!/usr/bin/env bash
# Script para simular una estación conectándose a YPF Ruta
# Uso: ./estacion.sh [puerto_ypf] [id_estacion]

set -euo pipefail

YPF_HOST="127.0.0.1"
YPF_PORT="${1:-18080}"  # Puerto por defecto: 8080 + 10000 = 18080
ID_ESTACION="${2:-999}" # ID de estación por defecto

echo "Conectando a YPF Ruta en ${YPF_HOST}:${YPF_PORT}..."

# Formato: HashMap<id_empresa, HashMap<id_estacion, Vec<Venta>>>
# Solicitud con empresa 1, estación ${ID_ESTACION}, y dos ventas
SOLICITUD_JSON="{
  \"1\": {
    \"${ID_ESTACION}\": [
      {
        \"id_venta\": 100,
        \"id_tarjeta\": 1,
        \"id_estacion\": ${ID_ESTACION},
        \"monto\": 5000.0,
        \"offline\": false,
        \"estado\": \"Pendiente\"
      },
      {
        \"id_venta\": 101,
        \"id_tarjeta\": 2,
        \"id_estacion\": ${ID_ESTACION},
        \"monto\": 3000.0,
        \"offline\": false,
        \"estado\": \"Pendiente\"
      }
    ]
  }
}"

echo ""
echo "📤 Enviando solicitud de validación:"
echo "$SOLICITUD_JSON" | jq '.' 2>/dev/null || echo "$SOLICITUD_JSON"
echo ""

if exec 3<>/dev/tcp/"${YPF_HOST}"/"${YPF_PORT}" 2>/dev/null; then
    echo "✅ Conexión establecida"
    
    # Crear JSON compacto (sin espacios ni saltos de línea)
    JSON_COMPACTO=$(echo "$SOLICITUD_JSON" | jq -c '.' 2>/dev/null || echo "$SOLICITUD_JSON" | tr -d '\n' | tr -s ' ')
    
    # Calcular longitud en bytes
    JSON_LEN=${#JSON_COMPACTO}
    
    echo "📏 Longitud del mensaje: $JSON_LEN bytes"
    
    # Enviar longitud como 4 bytes big-endian, luego el JSON
    python3 -c "import sys; sys.stdout.buffer.write(($JSON_LEN).to_bytes(4, 'big')); sys.stdout.buffer.write(b'$JSON_COMPACTO')" >&3
    
    echo "📨 Solicitud enviada, esperando respuesta..."
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
