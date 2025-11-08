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
  - [Detalles de la Implementación](#detalles-de-la-implementación)
    - [Procesos y Threads](#procesos-y-threads)
    - [División Regional](#división-regional)
    - [Elección de Líder](#elección-de-líder)
    - [Agrupación de Ventas](#agrupación-de-ventas)
    - [Validación de Ventas Offline](#validación-de-ventas-offline)
    - [Validación de Ventas Online](#validación-de-ventas-online)
    - [Surtidores como Tasks (actix) de cada Estación](#surtidores-como-tasks-actix-de-cada-estación)
    - [YPF RUTA como Coordinador](#ypf-ruta-como-coordinador)
    - [Ubicaciones de las estaciones](#ubicaciones-de-las-estaciones)
    - [Otras Suposiciones](#otras-suposiciones)
  - [Entidades Principales](#entidades-principales)
    - [Estación](#estación)
    - [Surtidor](#surtidor)
    - [Cliente](#cliente)
    - [YPF RUTA](#ypf-ruta-1)
    - [Empresa](#empresa)
  - [Structs del Payload de los Mensajes](#structs-del-payload-de-los-mensajes)
    - [Mensajes de Estación](#mensajes-de-estación)
    - [Mensajes de Surtidor](#mensajes-de-surtidor)
    - [Mensajes de Cliente](#mensajes-de-cliente)
    - [Mensajes de YPF RUTA](#mensajes-de-ypf-ruta)
    - [Mensajes de Empresa](#mensajes-de-empresa)
  - [Casos de Interés](#casos-de-interés)
    - [Casos de Interés Positivos](#casos-de-interés-positivos)
    - [Casos de Interés Negativos](#casos-de-interés-negativos)
      - [Desconexion de Estacion Lider](#desconexion-de-estacion-lider)
      - [Casos bordes](#casos-bordes)

---

## Detalles de la Implementación

### Procesos y Threads

Para la implementación del sistema se identifican como usuarios principales a las estaciones de servicio y a los administradores de empresas. Por lo tanto, se implementará un programa independiente para cada uno de estos roles. Además, se implementará un programa servidor que representará a **YPF RUTA** y un pequeño programa cliente que simulará a los clientes que llegan a las estaciones de servicio.

<figure>
  <img src="./res/procesos.png" alt="Procesos independientes agrupados según funcionalidad">
  <figcaption>Procesos independientes agrupados según funcionalidades</figcaption>
</figure>

**Procesos secuenciales**\
Los programas de administrador de **Empresa** y **Cliente** serán secuenciales, ya que no requieren concurrencia interna para su funcionamiento. Tampoco es problema que estos procesos bloqueen la ejecución mientras esperan respuestas de YPF RUTA o de la estación de servicio, respectivamente.

**Procesos concurrentes**\
Los programas de **Estación** y **YPF RUTA** serán concurrentes, ya que ambos deben manejar la recepción y gestión de múltiples conexiones en simultáneo. En el caso de las estaciones, deben atender a múltiples clientes así como comunicarse con otras estaciones y con YPF RUTA. Por otro lado, YPF RUTA debe manejar solicitudes concurrentes de múltiples estaciones y empresas.

* **Estación**: Escuchará conexiones de clientes y estaciones en distintos hilos. Para los clientes creará tareas que representen los surtidores, con un límite configurable según la estación. Contará también con un hilo dedicado a la recepción de solicitudes de pago de parte de los surtidores y el envío de las mismas al líder regional. Por otra parte, por cada mensaje entrante de otra estacion, se creará un hilo que ejecute las tareas necesarias en base al mensaje recibido. En caso de ser líder de la región, contará con un hilo adicional que periódicamente enviará las ventas acumuladas a YPF RUTA.

* **YPF RUTA**: Escuchará conexiones de estaciones y empresas en distintos hilos. Para cada conexión entrante, se creará un hilo que maneje las solicitudes recibidas y envíe las respuestas correspondientes.

<figure>
  <img src="./res/hilos.png" alt="Diagrama de threads para YPF RUTA y Estacion">
  <figcaption>Diagrama de planificación de Threads para YPF RUTA y Estación</figcaption>
</figure>

### División Regional

Las estaciones se encuentran divididas por región, cada una con su respectivo líder. Este se encarga de centralizar la comunicación con  **YPF RUTA** y se elige mediante el algoritmo `Ring Algorithm`.

El propósito de esto es el de minimizar los mensajes enviados a YPF RUTA por medio de la agrupación de mensajes de venta de todas las estaciones de una misma región.

### Elección de Líder

Para la elección de líder se utilizará el algoritmo de anillo (Ring Algorithm). Cada estación conoce las demás estaciones de su región y sus respectivos IDs. Cuando una estación detecta que el líder no responde, inicia el proceso de elección enviando un mensaje a través del anillo. Cada estación que recibe el mensaje agrega su ID y lo reenvía al siguiente nodo del anillo. Cuando el mensaje vuelve al nodo que lo inició, este determina el nuevo líder (el ID más alto) y le envía el mensaje de coordinación al ganador, quien lo reenvía a través del anillo para notificar a todas las estaciones de la región.

<figure>
  <img src="./res/eleccion_1.png" alt="Una estación intenta notificar una venta pero descubre que el lider no responde.">
  <figcaption>Una estación intenta notificar una venta pero descubre que el lider no responde.</figcaption>
</figure>

<figure>
  <img src="./res/eleccion_2.png" alt="La estación inicia una elección enviando un mensaje a través del anillo. Cada estación agrega su id y reenvía el mensaje.">
  <figcaption>La estación inicia una elección enviando un mensaje a través del anillo. Cada estación agrega su id y reenvía el mensaje.</figcaption>
</figure>

<figure>
  <img src="./res/eleccion_3.png" alt="El mensaje vuelve al nodo que lo inició, este determina el nuevo lider y envía el mensaje de coordinación al ganador.">
  <figcaption>El mensaje vuelve al nodo que lo inició, este determina el nuevo lider y envía el mensaje de coordinación al ganador.</figcaption>
</figure>

<figure>
  <img src="./res/eleccion_4.png" alt="El nuevo lider reenvía el mensaje de coordinación a través del anillo para notificar a todas las estaciones de la región.">
  <figcaption>El nuevo lider reenvía el mensaje de coordinación a través del anillo para notificar a todas las estaciones de la región.</figcaption>
</figure>

### Agrupación de Ventas

Una vez elegido el líder, cada una de las estaciones de la región le enviarán las ventas por confirmar. Estas se acumulan durante un período de tiempo razonable (de tres a cinco segundos) para su posterior envío a YPF RUTA en un único mensaje.

### Validación de Ventas Offline

En el caso de que una estación se encuentre totalmente incomunicada, se aprobarán de forma temporal todas las ventas realizadas sin validar con YPF RUTA (las cuales serán marcadas como que fueron realizadas sin conexión), priorizando que la estación continue funcionando. Una vez recupere la conexión, se notificarán todas las ventas realizadas a YPF RUTA.

La notificación de ventas offline se realizará por medio de un "anillo" iniciado periódicamente por el líder (30 segundos o más) donde se pasará un mensaje entre estaciónes levantando todas las ventas realizadas de forma offline que se encuentren pendientes de informar. Una vez que el líder recibe nuevamente el mensaje, las agrega a la lista de ventas a validar, y se notificarán cuando se envíe dicho mensaje.

De esta forma YPF asume el riesgo de validar una venta por fuera del límite de una empresa perdiendo el monto de dicha transacción.

### Validación de Ventas Online

Cuando una estación recibe una venta para validar, se la envía al líder y la guarda hasta recibir el rechazo o confirmación de la misma. Esto con el propósito de evitar pérdidas de información en el caso de que la estación lider sufra una desconexión.

### Surtidores como Tasks (actix) de cada Estación

Los surtidores se implementarán como tasks de cada estación cuya función es simular el tiempo requerido para la carga de combustible y manejar la comunicación con el cliente. Cuando la estación recibe un cliente, lanza una task surtidor que se encargará de pedirle los datos de cobro y enviar por medio de un canal interno el mensaje a la estación para que esta gestione la venta quedando a la espera de la respuesta para enviar el resultado al cliente y finalizar la task junto a la conexión del cliente.

### YPF RUTA como Coordinador

Se decidió que **YPF RUTA** actúe como un coordinador que no pasa tokens para autorizar el acceso de una **Estación** a la sección crítica. En su lugar, las estaciones encolan pedidos para acceder a la seccion critica, e **YPF RUTA** se encarga de procesar los pedidos de forma segura. Se tomo este camino ya que seria redundante pasar un token a la estacion para que este mismo le responda a **YPF RUTA** con los datos necesarios a guardar, aumentando asi la cantidad de mensajes en la red.

### Ubicaciones de las estaciones

Las estaciones son cercanas segun su numero de id, ya que se asume que las mismas se encuentran en ubicaciones fijas.

### Otras Suposiciones

* La desconexión de una estación implica únicamente la pérdida de comunicación con la región y no la propia caída de su sistema.

* En el caso de la aprobación de una venta por fuera del límite de la empresa o de (por falta de conexión), YPF asumirá la pérdida.

* YPF RUTA no puede perder la conexión.

---

## Entidades Principales

### Estación

**Finalidad** \
Representa una estación de YPF que recibe a los clientes y los distribuye entre los surtidores disponibles. Además, se encarga de informar a **YPF RUTA** sobre las ventas realizadas. En caso de que se caiga la conexión con el servidor central, puede almacenar temporalmente las ventas para reenviarlas una vez restablecida la comunicación.

**Estado Interno**

```rust
Estacion {
    id_estacion: i32,
    surtidores_estado_sender: List<channel>,
    ventas_sin_informar: List<Venta>,
    ypf_socket: socket,
    estaciones_regionales : List<i32>,
    id_lider : i32,
    clientes_en_cola: Queue<client_sender>
}
```

**Mensajes que Recibe**

<!-- RECIBO COMO LIDER  -->
* `transacciones_por_estacion`: (Unicamente recibido por Estación Lider) recibe los resultados de las validaciones de venta desde `YPF RUTA` y envía a cada estación el resultado correspondiente.

<!-- RECIBO COMO NO LIDER -->
* `cobrar_a_cliente`: enviarle al lider la solicitud de validación de venta o guardar la venta en modo offline si no hay conexión. Si sos lider lo acumulas con el resto de ventas pendientes.
* `confirmar_transacciones`: le informa a cada estación el resultado de sus transacciones

* `informar_venta`: Encola los pedidos y eventualmente informa a `YPF RUTA` que tiene que cobrarle a un cliente.
* `informar_ventas_offline`: Levanta todos los pedidos realizados en modo offline y los envia al anillo para que sean informados a `YPF RUTA`.

* `eleccion`: Se detectó una caida del lider actual entonces agrega su id y reenvia el mensaje al siguiente nodo del anillo.
* `coordinador`: Cambia el lider actual al id recibido.

**Mensajes que Envía**

<!-- ENVIO POR ANILLO -->
* `eleccion` -> `Estacion`
* `coordinador` -> `Estacion`
* `informar_ventas_offline` -> `Estacion`

<!-- ENVIO COMO LIDER -->
* `validar_ventas` -> `YPF Ruta`
* `confirmar_transacciones` -> `Estacion`

<!-- ENVIO COMO NO LIDER -->
* `finalizar_venta` -> `Surtidor`
* `informar_venta` -> `Estacion`

**Protocolo de Transporte**

* Comunicación TCP entre la estación y YPF RUTA.
* Comunicación TCP entre estación y estación.
* Comunicación local con los surtidores mediante canales.

---

### Surtidor

**Finalidad** \
Simula una unidad de carga de combustible que atiende a un cliente por vez. Envía a la estación las solicitudes de venta cuando finaliza la carga.

**Estado Interno**

```rust
Surtidor {
    estacion_sender: channel,
    estacion_receiver: channel
}
```

**Mensajes que Recibe**

* `finalizar_venta`: finaliza la conexión con el cliente y queda disponible para el siguiente.
* `devolver_datos_de_cobro`: recibe los datos del cliente para poder cobrarle.

**Mensajes que envía**

* `cobrar_a_cliente` -> `Estacion`
* `resultado_carga` -> `Cliente`
* `pedir_datos_de_cobro` -> `Cliente`

**Protocolo de Transporte** \
Canal interno hacia la estación correspondiente.

---

### Cliente

**Finalidad** \
Representa a un conductor que llega a la estación para realizar una carga de combustible. Cada cliente tiene asociada una tarjeta identificadora para el venta.

**Estado Interno**

```rust
Cliente {
    id_tarjeta: u32,
}
```

**Mensajes que Recibe**

* `pedir_datos_de_cobro`: devuelve el monto que quiere cargar de nafta y el id de su tarjeta.
* `resultado_carga`: es libre de irse.

**Mensajes que Envía**

* `devolver_datos_de_cobro` -> Surtidor

**Protocolo de Transporte** \
TCP hacia la estación.

---

### YPF RUTA

**Finalidad** \
Actúa como servidor central del sistema. Administra la comunicación entre estaciones y empresas, y mantiene el registro global de ventas y límites de tarjetas.

**Estado Interno**

```rust
YPFRuta {
    limites_generales: HashMap<idEmpresa, Monto>,
    limites_por_tarjetas: HashMap<idTarjeta, Monto>,
    repositorio_ventas: RepositorioVentas,
}
```

**Mensajes que Recibe**

* `gastos_empresa`: recibe la solicitud de gastos de una empresa y responde con la lista de gastos asociados a sus vehículos.
* `configurar_limite`: recibe la solicitud de configuración de límite para una tarjeta específica y actualiza el estado interno. Envía confirmación a la empresa.
* `configurar_limite_general`: recibe la solicitud de configuración de límite general para una empresa y actualiza el estado interno. Envía confirmación a la empresa.
* `validar_ventas`: por cada venta recibida valida si puede ser aprobado según los límites establecidos y actualiza el repositorio de ventas. Además, envía el resultado de las validaciones a la estación correspondiente sólo para el caso de ventas online.

**Mensajes que Envía**

* `gastos_empresa_respuesta` -> Empresa
* `confirmacion_limite` -> Empresa
* `confirmacion_limite_general`-> Empresa
* `transacciones_por_estacion` -> Estación

**Protocolo de Transporte** \
TCP contra estaciones y empresas.

---

### Empresa

**Finalidad** \
Representa una empresa asociada a tarjetas YPF Ruta. Se encarga de validar ventas y administrar límites de gasto de sus vehículos.

**Estado Interno**

```rust
Empresa {
    idEmpresa: i32,
    ypf_socket: Socket

}
```

**Mensajes que Recibe**

* `gastos_empresa_respuesta`: recibe lista de gastos por vehículo y los transforma para mostrar al administrador.
* `confirmacion_limite`: muestra el resultado de la operación y para que vehículo.
* `confirmacion_limite_general`: muestra el resultado de la operación.

**Mensajes que Envía**

* `configurar_limite` -> `YPF Ruta`
* `gastos_empresa` -> `YPF Ruta`
* `configurar_limite_general` -> `YPF Ruta`

**Protocolo de Transporte** \
TCP entre YPF Ruta y cada empresa.

---

## Structs del Payload de los Mensajes

* `venta`

```rust
struct Venta {
    id_venta: i32,
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

Una vez que el mensaje de eleccion vuelve al nodo que lo inició, este determina el nuevo lider (el id mas alto) y le envía el mensaje de coordinacion al ganador de la elección y este lo reenvía a través del anillo para notificar a todos las estaciones de la región.

```rust
struct Coordinador {
    id_lider: i32,
}
```

* `informar_ventas_offline`

Cada cierto periodo de tiempo, el líder envía este mensaje a través del anillo para que cada estación le envíe las ventas offline que haya acumulado.

```rust
struct InformarVentasOffline {
    id_lider: i32
    ventas_offline: List<Venta>
}
```

* `validar_ventas`

La estación líder de cada región envía periódicamente este mensaje a `YPF RUTA` con la lista de ventas a validar.

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

* `finalizar_venta`

Cuando la estación recibe la respuesta del líder le envía este mensaje al surtidor para que le informe al cliente el resultado de la carga.

```rust
struct FinalizarVenta {
    venta: Venta,
}
```

* `informar_venta`

La estación envía este mensaje al líder para notificarle de una nueva venta a validar.

```rust
struct InformarVenta {
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

YPF Ruta envía este mensaje a una empresa con la lista de gastos asociados a sus vehículos.

```rust
struct GastosEmpresaRespuesta {
    gastos_por_vehiculo: HashMap<i32, List<Venta>>
}
```

* `confirmacion_limite`

YPF Ruta envía este mensaje a una empresa con el resultado de la actualización del límite para un vehículo en particular.

```rust
struct ConfirmacionLimite {
    id_vehiculo: i32
    exito: bool
}
```

* `confirmacion_limite_general`

YPF Ruta envía este mensaje a una empresa con el resultado de la actualización del límite general.

```rust
struct ConfirmacionLimiteGeneral {
    exito: bool
}
```

* `transacciones_por_estacion`

YPF Ruta envía este mensaje a una estación lider como respuesta a confirmar transacciones, conteniendo el estado actualizado de las ventas.

```rust
struct TransaccionesPorEstacion {
    transacciones: List<Venta>
}
```

### Mensajes de Empresa

* `configurar_limite`

La empresa envia este mensaje a YPF Ruta para actualizar el límite de un vehiculo en particular

```rust
struct ConfigurarLimite {
    id_empresa: i32,
    id_vehiculo: i32,
    nuevo_limite: i32,
}
```

* `configurar_limite_general`

La empresa envia este mensaje a YPF Ruta para actualizar su limite general mensual

```rust
struct ConfigurarLimiteGeneral {
    id_empresa: i32,
    nuevo_limite: i32,
}
```

* `gastos_empresa`

La empresa envia este mensaje a YPF Ruta para obtener todos sus gastos

```rust
struct GastosEmpresa {
    id_empresa: i32,
}
```

---

## Casos de Interés

A continuación, se detallarán casos de interés correspondientes en base al análisis de nuestra implementación propuesta. Estos son acompañados de diagramas de secuencia correspondientes que facilitan la visualización de la ejecución del sistema general.

### Casos de Interés Positivos

![Diagrama en caso funcional de estacion siendo lider](./res/diagrama_siendo_lider.png)
Diagrama en caso funcional de una estacion siendo lider

![Diagrama en caso funcional de estacion sin ser lider](./res/diagrama_sin_ser_lider.png)
Diagrama en caso funcional de una estacion sin ser lider

### Casos de Interés Negativos

#### Desconexion de Estacion Lider

![Diagrama en caso de desconexion de estacion lider](./res/diagrama_de_desconexion.png)

En caso de que una estacion pierda conexion, la misma intentará comunicarse con la estación lider pero notará que no lo puede hacer dado que perdió la conexion, entonces guardará las ventas realizadas como offline. Cuando eventualmente recupere la conexion y reciba el mensaje `informar_ventas_offline`, actualizara su lider a partir del id recibido en el mismo y agregara las ventas pendientes de informar.

#### Casos bordes

* **Estacion líder pierde conexión luego de recibir la respuesta de YPF RUTA**. \
    Cuando un líder pierde la conexión, eventualmente se elegirá un nuevo líder y puede suceder que alguna estación estuviese esperando la confirmación de una venta por parte de aquel líder caído. Al no recibirla, intentará reenviar la venta al nuevo líder. Si el líder anterior se desconectó luego de enviar las ventas a *YPF RUTA* puede ocurrir que al servidor le llegue una venta duplicada, pero esto no afectará el comportamiento del sistema ya que *YPF RUTA* se encarga de validar las ventas y, en caso de encontrar algún id de venta duplicado, simplemente enviará el estado (confimado/rechazado) que ya validó previamente.

* **Estacion lider pierde conexion antes de mandar las ventas a YPF RUTA (validar_ventas) teniendo ventas a informar**. \
    La estación líder almacena las ventas a informar recibida por parte de otras estaciones junto a las propias y las envía periódicamente a YPF RUTA. Si la estación líder pierde la conexión antes de enviar las ventas a YPF RUTA, entonces deberá descartar todas las ventas online almacenadas (excepto las propias) dado que cada estación se encargará de reenviar las ventas pendientes al nuevo líder una vez que sea elegido.

* **Estacion pierde conexion con clientes encolados**. \
    Si una estación pierde la conexion y aún tiene clientes encolados, estos serán atendidos normalmente. La estación continuará funcionando y procesando las ventas de forma offline hasta que recupere la conexion.

* **Ex lider puede intentar mandar mensaje de informar ventas offline**. \
    Si una estación lider pierde la conexion, este mismo se quita el estado de lider de modo que para cuando vuelva a reconectarse esta estación no intentará crear una ronda de informar ventas offline como si fuese un lider, solo simplemente asumirá que hay un nuevo lider.

* **Estacion pierde conexion luego de informar venta al lider**. \
    Si una estación pierde la conexion luego de mandar el mensaje `informar_venta` a la Estacion Lider, eventualmente esta última intentará confirmarle las ventas, pero no lo logrará. Mientras esto ocurra, las ventas activas (aún en proceso de aceptarse) de esta estación pasarán a offline y los clientes podrán retirarse. Eventualmente la Estacion lider tendrá que confirmarle a cada estación otras ventas realizadas, y en ese momento revisará si tiene que confirmarle alguna venta a una estacion desconectada.
