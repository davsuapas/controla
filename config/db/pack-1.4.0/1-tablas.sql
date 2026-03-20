USE @DB_NOMBRE;

-- ISSUE #13

-- Se refactoriza la gestión de horarios para consolidar la información en una única tabla `horarios`.

-- Crear una tabla temporal que contendrá la nueva estructura de horarios.
CREATE TABLE `horarios_new` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `usuario` int(10) unsigned NOT NULL,
  `fecha_creacion` date NOT NULL,
  `caducidad_fecha_ini` date NOT NULL DEFAULT '1900-01-01',
  `caducidad_fecha_fin` date DEFAULT NULL,
  `dia` char(1) NOT NULL,
  `horas` TINYINT UNSIGNED NOT NULL,
  `cortesia` TINYINT UNSIGNED DEFAULT 0,
  PRIMARY KEY (`id`)
) COMMENT='Define las horas a trabajar por un usuario para un día de la semana a partir de una fecha.';

-- Poblar la nueva tabla agregando los datos de las tablas `usuario_horarios` y `horarios`.
-- Se agrupa por usuario, fecha de creación, fechas de caducidad y día de la semana, sumando las horas de los tramos.
INSERT INTO horarios_new (usuario, fecha_creacion, caducidad_fecha_ini, caducidad_fecha_fin, dia, horas)
SELECT
    uh.usuario,
    uh.fecha_creacion,
    uh.caducidad_fecha_ini,
    uh.caducidad_fecha_fin,
    h.dia,
    SUM(TIME_TO_SEC(TIMEDIFF(h.hora_fin, h.hora_inicio))) DIV 3600
FROM
    usuario_horarios uh
JOIN
    horarios h ON uh.horario = h.id
GROUP BY
    uh.usuario,
    uh.fecha_creacion,
    uh.caducidad_fecha_ini,
    uh.caducidad_fecha_fin,
    h.dia;

-- Actualizar la tabla `marcajes` para referenciar a la nueva tabla `horarios`.
-- 1. Eliminar la restricción de clave foránea antigua.
ALTER TABLE marcajes DROP FOREIGN KEY marcajes_usuario_horarios_FK;

-- 2. Añadir la nueva columna `horario`.
ALTER TABLE marcajes ADD COLUMN horario int(10) unsigned;

-- 3. Actualizar la columna `horario` con el id correspondiente de `horarios_new`.
UPDATE marcajes m
JOIN usuario_horarios uh ON m.usuario_horario = uh.id
JOIN horarios h ON uh.horario = h.id
JOIN horarios_new hn ON 
    hn.usuario = uh.usuario 
    AND hn.fecha_creacion = uh.fecha_creacion 
    AND hn.dia = h.dia
    AND hn.caducidad_fecha_ini = uh.caducidad_fecha_ini
    AND (hn.caducidad_fecha_fin = uh.caducidad_fecha_fin OR (hn.caducidad_fecha_fin IS NULL AND uh.caducidad_fecha_fin IS NULL))
SET m.horario = hn.id;

-- 4. Eliminar la columna antigua y su índice.
ALTER TABLE marcajes DROP KEY registros_horarios_FK;
ALTER TABLE marcajes DROP COLUMN usuario_horario;

-- 5. Configurar la nueva columna.
ALTER TABLE marcajes MODIFY COLUMN horario int(10) unsigned NOT NULL;
ALTER TABLE marcajes ADD KEY marcajes_horarios_FK (horario);

-- Eliminar las tablas antiguas.
DROP TABLE usuario_horarios;
DROP TABLE horarios;

-- Renombrar la tabla temporal al nombre final `horarios`.
RENAME TABLE horarios_new TO horarios;

-- Añadir los índices y claves foráneas necesarios a la nueva tabla `horarios`.
ALTER TABLE horarios
  ADD UNIQUE KEY `idx_usuario_fecha_dia_caducidad` (`usuario`, `fecha_creacion` DESC, `dia`, `caducidad_fecha_ini`),
  ADD CONSTRAINT `horarios_usuarios_FK` FOREIGN KEY (`usuario`) REFERENCES `usuarios` (`id`) ON UPDATE CASCADE;

-- Añadir la clave foránea a la tabla `marcajes` (después de renombrar horarios_new a horarios).
ALTER TABLE marcajes ADD CONSTRAINT marcajes_horarios_FK FOREIGN KEY (horario) REFERENCES horarios (id) ON UPDATE CASCADE;

-- ACTUALIZACIÓN VERSIÓN
UPDATE schema_info SET version_actual = '1.4.0' WHERE id = 1;
