# CONTROLA

Controla gestiona el marcaje horario del personal de una organización.

El sistema cumple el Real Decreto-ley 8/2019, de 8 de marzo.

Puntos principales de la ley:
- Registro inmutable de la jornada laboral.
- Flexibilidad horaria.
- Resumen mensual.
- Resguardo de los datos durante al menos 4 años.
- Protección de datos.
- Acceso desde cualquier lugar por parte de los inspectores.
  
Casos de uso:
- Sistema de identificación de usuarios.
- Administración de usuarios y empleados.
- Gestión de horarios múltiples del empleado.
- Gestión de los calendarios de los empleados.
- Marcaje de entrada y salida por el empleado.
- Marcaje manual por el registrador en nombre del empleado.
- Consulta de los registros de marcajes.
- Informes de saldo horario: Diferencia entre horas marcadas y horas trabajadas.
- Informes del cumplimiento horario (resumen mensual): Horas trabajadas frente a horas previstas.
- Solicitud de incidencias: Solicitud para gestionar errores de marcaje.
- Gestión de las incidencias de marcajes por el gestor.
- Informes horarios e incidencias.
- Consultas por parte de los inspectores.
- Auditoría de acciones realizadas en el sistema.

## DESARROLLO

### Pre-requisitos

- Instalar MariaDB/MySQL. El servicio backend utiliza la autenticación vía socket para conectarse con la base de datos.
- Instalar Rust.
- Instalar npm.

### Test

- Creamos la base de datos, usuario y tablas en la base de datos:
  - Mediante la herramienta de cliente de la base de datos copiamos los .sql ubicados en *./config/db/inicio* en una carepta temporal (*no modifique los originales*) y los ejecutamos en el orden correcto:
    - @DB_NOMBRE: Nombre de la base de datos. Ejemplo: controla.
    - @DB_USUARIO: Usuario de la base de datos que usará el servicio. Este usuario debe ser un usuario válido del sistema (recuerde se utiliza autenticación vía socket).

- Creamos la configuración necesaria para ejecutar el serviciosbackend (API). Para que sea más cómodo en sucesivas ejecuciones, cree la carpeta *./config/test*. No sé preocupe porque esta carpeta es ignorada por git.
  - Crear un fichero llamado *admin-passw* dentro de esta carpeta test. En este fichero añadimos la clave que tendrá nuestro usuario administrador en controla.
  - Crear un fichero llamado *secreto* dentro de esta carpeta test. En este fichero añadimos una clave que sirve para encriptar todo lo necesario dentro de la base de datos.
  - Crear un script llamado *config-test.sh* dentro de esta carpeta test. Este script nos va a generar un fichero de configuración válido para la ejecución del servicio API. El contenido de este fichero script es el siguiente:

  ```
  #!/bin/bash

  # Este script genera un fichero configuración para ejecutar
  # controla-api en local

  ./scripts/dist/config.sh ./config/test/config.json '' <DB_RUTA_SOCKET> <DB_USUARIO> <DB_NOMBRE> <DB_MAX_CONN> debug <SERVIDOR_PUERTO> false <BOOT_ADMIN_CREAR> <BOOT_ADMIN_DNI> 
  ```

  Sustituimos <...> por el valor adecuado. La primera vez necesitamos que se cree el usuario administrador dentro de la base de datos. Para ello, en *<BOOT_ADMIN_CREAR>* escribimos true y en *<BOOT_ADMIN_DNI>* escribimos un DNI válido. Una vez que hemos ejecutado el servicio podremos volver a generar otro fichero de configuración con los valores false y '', tanto para *<BOOT_ADMIN_CREAR>* como para *<BOOT_ADMIN_DNI>*. Esto hará que el servicio no vuelva a intentar crear el usuario administrador de nuevo.

  - Generamos el fichero de configuración ejecutando:
    ```
    ./config/test/config-test.sh
    ```
- Para ejecutar la aplicación controla lanzamos tanto el servicio API como el interface web:
  - Ejecutamos el servicio API:
    ```
    cargo run -- -fichero_config=./config/test/config.json -carpeta_secretos=./config/test
    ```
  - Ejecutamos el interface de usuario web:
    ```
    cd web
    npm run dev
    ```

- A continuación ya podrá utilizar la aplicación a través de su navegador preferido.
