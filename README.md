# YPF Ruta

Trabajo Práctico Grupal - Programación Concurrente - Cátedra Deymonnaz

Segundo Cuatrimestre 2025

**Integrantes**:

* Melina Retamozo - 110065
* Ariel Folgueira - 109473
* Matias Daniel Mendiola Escalante - 110379
* Gian Luca Spagnolo - 108072

## Tabla de Contenidos

- [YPF Ruta](#ypf-ruta)
  - [Tabla de Contenidos](#tabla-de-contenidos)
  - [Detalles de la Implementacion](#detalles-de-la-implementacion)
  - [Entidades Principales](#entidades-principales)
    - [Estación](#estación)
    - [Surtidor](#surtidor)
    - [Cliente](#cliente)
    - [YPF RUTA](#ypf-ruta-1)
    - [Empresa](#empresa)
  - [Structs del Payload de los mensajes](#structs-del-payload-de-los-mensajes)
    - [Mensajes de Estación](#mensajes-de-estación)
    - [Mensajes de Surtidor](#mensajes-de-surtidor)
    - [Mensajes de Cliente](#mensajes-de-cliente)
    - [Mensajes de YPF RUTA](#mensajes-de-ypf-ruta)
    - [Mensajes de Empresa](#mensajes-de-empresa)
  - [Protocolo de Transporte](#protocolo-de-transporte)
  - [Casos de Interés](#casos-de-interés)

---

## Detalles de la Implementacion

Las estaciones estan divididas regionalmente, en cada region hay una estacion que se usa como lider para comunicarse con **YPF RUTA**. Se utiliza el algoritmo `Ring Algorithm` para eleccion de lider.

```code
Cliente --(arribar_a_estacion)-> Estacion --(recibir_cliente)-> Surtidor --(pedir_datos_de_cobro)-> Cliente --(devolver_datos_de_cobro)-> Surtidor --(cobrar_a_cliente)-> Estacion-- 
--(solicitud_venta)-> EstacionLider --(validar_ventas)-> YPFRUTA --(transacciones_por_estacion)-> EstacionLider --(confirmar_transacciones)-> Estacion --(venta_validada)-> Surtidor --(resultado_carga)-> Cliente
```

* Poner un buffer en EstacionLider para que acumule los ventas a validar en YPFRUTA y mandarlos luego de 1 segundo de recibidos para mandar un mensaje con varias ventas a validar (reducimos mensajes)
* informar_ventas_offline se usa en modo anillo, cada X tiempo. Todos los demas mensajes son 1 a 1
* Los surtidores son tasks (actix) de cada **Estacion**.

---

## Entidades Principales

### Estación

**Finalidad**
Representa una estación de YPF que recibe a los clientes y los distribuye entre los surtidores disponibles. Además, se encarga de informar a **YPF RUTA** sobre los ventas realizados. En caso de que se caiga la conexión con el servidor central, puede almacenar temporalmente las ventas para reenviarlas una vez restablecida la comunicación.

**Estado interno**

```rust
Estacion {
    surtidores_estado_sender: List<channel>,
    ventas_sin_informar: HashMap<idTarjeta, Monto>,
    ypf_socket: socket,
    estaciones_regionales : List<i32>
    id_lider : i32
    clientes_en_cola: Queue<client_sender>
}
```

Estacion crea tarea -> tarea simula hacer algo -> tarea le responde a estacion -> estacion hace las cosas y eventualmente le dice a tarea que termine

**Mensajes que recibe**

<!-- RECIBO COMO LIDER  -->
* `transacciones_por_estacion(List<Estacion, List<Transaccion>>)`: (Unicamente recibido por Estación Lider) recibe los resultados de las validaciones de venta desde `YPF RUTA` y envía a cada estación el resultado correspondiente.

<!-- RECIBO COMO NO LIDER -->
* `cobrar_a_cliente`: enviarle al lider la solicitud de validación de venta o guardar la venta en modo offline si no hay conexión. Si sos lider lo acumulas con el resto de ventas pendientes.
* `confirmar_transacciones(List<Transaccion>)`: le informa a cada estación el resultado de sus transacciones

* `solicitud_venta`: Encola los pedidos y eventualmente informa a `YPFRUTA` que tiene que cobrarle a un cliente.
* `informar_ventas_offline`: Levanta todos los pedidos realizados en modo offline y los envia al anillo para que sean informados a `YPFRUTA`.

* `eleccion(id)`: Se detectó una caida del lider actual entonces agrega su id y reenvia el mensaje al siguiente nodo del anillo.
* `coordinador(id_lider)`: Cambia el lider actual al id recibido.

**Mensajes que envía**

<!-- ENVIO POR ANILLO -->
* `eleccion` -> `Estacion`
* `coordinador` -> `Estacion`
* `informar_ventas_offline` -> `Estacion`

<!-- ENVIO COMO LIDER -->
* `validar_ventas` -> `YPFRuta`
* `confirmar_transacciones` -> `Estacion`

<!-- ENVIO COMO NO LIDER -->
* `finalizar_venta` -> `Surtidor`
* `solicitud_venta` -> `Estacion`  

**Protocolo de transporte**

* Comunicación TCP entre la estación y YPF RUTA.
* Comunicación TCP entre estación y estación.
* Comunicación local con los surtidores mediante canales. *(TODO: ACLARAR COMO LO IMPLEMENTAMOS)*

**Protocolo de aplicación**

TODO: *(VER QUE PROTOCOLO USAMOS PARA LA APP EN GENERAL)*

<!-- 
**Casos de interés positivos**

**Casos de interés negativos (caídas)**

* Desconexión temporal del servidor central.
* Saturación de surtidores (clientes en cola). 
-->

---

### Surtidor

**Finalidad**
Simula una unidad de carga de combustible que atiende a un cliente por vez. Envía a la estación las solicitudes de venta cuando finaliza la carga.

**Estado interno**

```rust
Surtidor {
    estacion_sender: channel,
    estacion_receiver: channel
}
```

**Mensajes que recibe**

* `finalizar_venta`: finaliza la conexión con el cliente y queda disponible para el siguiente.
* `devolver_datos_de_cobro()`: recibe los datos del cliente para poder cobrarle.

**Mensajes que envía**

* `cobrar_a_cliente` -> `Estacion`
* `resultado_carga` -> `Cliente`
* `pedir_datos_de_cobro` -> `Cliente`

**Protocolo de transporte**
Canal interno hacia la estación correspondiente.

<!-- 
**Casos de interés**

* Cliente cancela o abandona durante la carga.
* Comunicación interrumpida con la estación. 
-->

---

### Cliente

**Finalidad**
Representa a un conductor que llega a la estación para realizar una carga de combustible. Cada cliente tiene asociada una tarjeta identificadora para el venta.

**Estado interno**

```rust
Cliente {
    id_tarjeta: u32,
}
```

**Mensajes que recibe**

* `pedir_datos_de_cobro`: devuelve el monto que quiere cargar de nafta y el id de su tarjeta.
* `resultado_carga`: es libre de irse.

**Mensajes que envía**

* `devolver_datos_de_cobro` -> Surtidor

**Protocolo de transporte**
TCP hacia la estación.

<!-- 
**Casos de interés**

* Reintento de venta por falta de conexión o validación fallida. 
-->

---

### YPF RUTA

**Finalidad**
Actúa como servidor central del sistema. Administra la comunicación entre estaciones y empresas, y mantiene el registro global de ventas y límites de tarjetas.

**Estado interno**

```rust
YPFRuta {
    limites_generales: HashMap<idEmpresa, Monto>,
    limites_por_tarjetas: HashMap<idTarjeta, Monto>,
    repositorio_ventas: RepositorioVentas,
}
```

**Mensajes que recibe**

* `gastos_empresa`: recibe la solicitud de gastos de una empresa y responde con la lista de gastos asociados a sus vehículos.
* `configurar_limite`: recibe la solicitud de configuración de límite para una tarjeta específica y actualiza el estado interno. Envía confirmación a la empresa.
* `configurar_limite_general`: recibe la solicitud de configuración de límite general para una empresa y actualiza el estado interno. Envía confirmación a la empresa.
* `validar_ventas`: por cada venta recibido valida si puede ser aprobado según los límites establecidos y actualiza el repositorio de ventas. Además, envía el resultado de las validaciones a la estación correspondiente sólo para el caso de ventas online.

**Mensajes que envía**

* `gastos_empresa_respuesta` -> Empresa
* `confirmacion_limite` -> Empresa
* `confirmacion_limite_general`-> Empresa
* `transacciones_por_estacion` -> Estación

**Protocolo de transporte**
TCP contra estaciones y empresas.

<!-- 
**Casos de interés**

* Sincronización de ventas entre estaciones desconectadas.
* Manejo de límites de tarjetas y empresas. 
-->

---

### Empresa

**Finalidad**
Representa una empresa asociada a tarjetas YPF Ruta. Se encarga de validar ventas y administrar límites de gasto de sus vehículos.

**Estado interno**

```rust
Empresa {
    idEmpresa: i32,
    ypf_socket: Socket

}
```

**Mensajes que recibe**

* `gastos_empresa_respuesta`: recibe lista de gastos por vehículo y los transforma para mostrar al administrador.
* `confirmacion_limite`: muestra el resultado de la operación y para que vehículo.
* `confirmacion_limite_general`: muestra el resultado de la operación.

**Mensajes que envía**

* `configurar_limite` -> `YPFRuta`
* `gastos_empresa` -> `YPFRuta`
* `configurar_limite_general` -> `YPFRuta`

**Protocolo de transporte**
TCP entre YPF Ruta y cada empresa.

<!-- 
**Casos de interés**

* Excedente del límite individual o general.
* Configuración dinámica de límites. 
-->

---

## Structs del Payload de los mensajes

* `venta`

```rust
struct Venta {
    id_tarjeta: i32,
    monto: f32,
    id_estacion: i32,
    timestamp: i64,
    offline: bool,
    estado: VentaEstado,
}
```

* `venta_estado`

```rust
enum VentaEstado {
    Pendiente,
    Aprobada,
    Rechazada,
}
```

### Mensajes de Estación

* `eleccion`

Cuando una estación intenta notificar un pago puede descubrir que el lider no responde, entonces inicia una eleccion. Cada estación dentro de una región recibe este mensaje a través del anillo, agrega su id a la lista y lo reenvia al siguiente nodo.

```rust
struct Eleccion {
    aspirantes_ids: List<i32>,
}
```

* `coordinacion`

Una vez que el mensaje de eleccion vuelve al nodo que lo inició, este determina el nuevo lider (el id mas alto) y envía el mensaje de coordinacion a través del anillo para notificar a todos las estaciones de la región.

```rust
struct Cordinador {
    id_lider: i32,
}
```

* `informar_ventas_offline`

Cada cierto periodo de tiempo, el líder envía este mensaje a través del anillo para que cada estación le envíe las ventas offline que haya acumulado.

```rust
struct InformarVentasOffline {
    ventas_offline: List<Venta>
}
```

* `validar_ventas`

La estación líder de cada región envía periódicamente este mensaje a `YPFRUTA` con la lista de ventas a validar.

```rust
struct ValidarVentas {
    ventas: List<Venta>
}
```

* `confirmar_transacciones`

La estación líder envía este mensaje a cada estación con el resultado de las validaciones de ventas.

```rust
struct ConfirmarTransacciones {
    transacciones: List<Venta>,
}
```

* `venta_validada`

Cuando la estación recibe la respuesta del líder le envía este mensaje al surtidor para que le informe al cliente el resultado de la carga.

```rust
struct VentaValidada {
    venta: Venta,
}
```

* `solicitud_venta`

La estación envía este mensaje al líder para notificarle de una nueva venta a validar.

```rust
struct SolicitudVenta {
    venta: Venta,
}
```

### Mensajes de Surtidor

* `cobrar_a_cliente`

El surtidor envía este mensaje a la estación para notificarle a la Estación que debe cobrarle a un cliente.

```rust
struct CobrarACliente {
    venta: Venta,
}
```

* `resultado_carga`

El surtidor envía este mensaje al cliente para informarle el resultado de la carga.

```rust
struct ResultadoCarga {
    exito: bool
}
```

* `pedir_datos_de_cobro`

El surtidor envía este mensaje al cliente para solicitarle los datos necesarios para realizar el cobro.

```rust
struct PedirDatosDeCobro {
    hola: bool
}
```

### Mensajes de Cliente

* `devolver_datos_de_cobro`

Cliente devuelve los datos necesarios para que empiece el cobro

```rust
struct devolver_datos_de_cobro {
    monto: f32,
    id_tarjeta: i32
}
```

### Mensajes de YPF RUTA

* `gastos_empresa_respuesta`

YPFruta envía este mensaje a una empresa con la lista de gastos asociados a sus vehículos.

```rust
struct GastosEmpresaRespuesta {
    gastos_por_vehiculo: HashMap<i32, List<Venta>>
}
```

* `confirmacion_limite`

YPFruta envía este mensaje a una empresa con el resultado de la actualización del límite para un vehículo en particular.

```rust
struct ConfirmacionLimite {
    id_vehiculo: i32
    exito: bool
}
```

* `confirmacion_limite_general`

YPFruta envía este mensaje a una empresa con el resultado de la actualización del límite general.

```rust
struct ConfirmacionLimiteGeneral {
    exito: bool
}
```

* `transacciones_por_estacion`

YPFruta envía este mensaje a una estación lider como respuesta a confirmar transacciones, conteniendo el estado actualizado de las ventas.

```rust
struct TransaccionesPorEstacion {
    transacciones: List<Venta>
}
```

### Mensajes de Empresa

* `configurar_limite`

La empresa envia este mensaje a YPFruta para actualizar el límite de un vehiculo en particular

```rust
struct ConfigurarLimite {
    id_empresa: i32,
    id_vehiculo: i32,
    nuevo_limite: i32,
}
```

* `configurar_limite_general`

La empresa envia este mensaje a YPFruta para actualizar su limite general mensual

```rust
struct ConfigurarLimiteGeneral {
    id_empresa: i32,
    nuevo_limite: i32,
}
```

* `gastos_empresa`

La empresa envia este mensaje a YPFruta para obtener todos sus gastos

```rust
struct GastosEmpresa {
    id_empresa: i32,
}
```

---

## Protocolo de Transporte

TODO

---

## Casos de Interés

TODO
