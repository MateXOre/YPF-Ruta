#!/bin/bash
# filepath: start.sh
# Script para iniciar el cluster YPF y las estaciones en consolas separadas

set -euo pipefail

echo "Iniciando cluster YPF y estaciones..."

# Función para limpiar procesos al salir
cleanup() {
    echo ""
    echo "Deteniendo todos los procesos..."
    pkill -P $$ || true
    exit 0
}

trap cleanup SIGINT SIGTERM

if command -v gnome-terminal &> /dev/null; then
    TERM_CMD="gnome-terminal"
    TERM_ARGS="-- bash -c"
elif command -v xterm &> /dev/null; then
    TERM_CMD="xterm"
    TERM_ARGS="-hold -e"
elif command -v konsole &> /dev/null; then
    TERM_CMD="konsole"
    TERM_ARGS="--hold -e"
else
    echo "No se encontró un emulador de terminal compatible"
    echo "Instala gnome-terminal, xterm o konsole"
    exit 1
fi

echo "Usando: $TERM_CMD"
echo ""

# Iniciar cluster YPF RUTA
echo "[1] - Iniciando Cluster YPF RUTA..."
cd ypf_ruta
$TERM_CMD $TERM_ARGS "./tmux.sh" &
YPF_PID=$!
cd ..

sleep 2

# Iniciar estaciones región 1
echo "[2] - Iniciando Estaciones Región 1..."
cd estacion
$TERM_CMD $TERM_ARGS "./tmux.sh" &
EST1_PID=$!
cd ..

sleep 1

# Iniciar estaciones región 2
echo "[3] - Iniciando Estaciones Región 2..."
cd estacion
$TERM_CMD $TERM_ARGS "./tmux_2.sh" &
EST2_PID=$!
cd ..

echo ""
echo "Todos los procesos iniciados"
echo ""
echo "Procesos en ejecución:"
echo "   - Cluster YPF RUTA (PID: $YPF_PID)"
echo "   - Estaciones Región 1 (PID: $EST1_PID)"
echo "   - Estaciones Región 2 (PID: $EST2_PID)"
echo ""
echo "Presiona Ctrl+C para detener todos los procesos"
echo ""

wait