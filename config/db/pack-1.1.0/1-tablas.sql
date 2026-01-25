USE @DB_NOMBRE;

-- ISSUE #1

-- horario

CREATE UNIQUE INDEX horarios_dia_IDX USING BTREE ON horarios (dia,hora_inicio,hora_fin);

-- usuario_horarios

ALTER TABLE usuario_horarios
  DROP PRIMARY KEY,
  ADD COLUMN `id` INT(10) UNSIGNED NOT NULL AUTO_INCREMENT FIRST,
  ADD PRIMARY KEY (`id`),
  ADD COLUMN `caducidad_fecha_ini` DATE NOT NULL DEFAULT '1900-01-01',
  ADD COLUMN `caducidad_fecha_fin` DATE NULL DEFAULT NULL;

-- Se crea porque al eliminar idx_usuario_fecha daría error
-- porque el FK de usuario no tiene índice. Al finalizar lo borramos
CREATE INDEX usuario_horarios_usuarios_FK USING BTREE ON usuario_horarios (usuario);
UPDATE schema_info SET version_actual = '1.1.0' WHERE id = 1;
ALTER TABLE usuario_horarios DROP INDEX idx_usuario_fecha;
CREATE UNIQUE INDEX idx_usuario_fecha USING BTREE ON usuario_horarios (`usuario`, `fecha_creacion` DESC, `horario`, `caducidad_fecha_ini`);

DROP INDEX usuario_horarios_usuarios_FK ON usuario_horarios;

-- marcajes

ALTER TABLE marcajes CHANGE horario usuario_horario int(10) unsigned NOT NULL;
ALTER TABLE marcajes DROP FOREIGN KEY registros_horarios_FK;

-- Cambiamos en el marcaje el id de horario por el id de usuario_horario
UPDATE marcajes m
SET m.usuario_horario = (
    SELECT uh.id 
    FROM usuario_horarios uh 
    WHERE uh.usuario = m.usuario 
      AND uh.horario = m.usuario_horario
      AND uh.fecha_creacion < m.fecha
    ORDER BY uh.fecha_creacion DESC
    LIMIT 1
);

ALTER TABLE marcajes 
  ADD CONSTRAINT marcajes_usuario_horarios_FK 
    FOREIGN KEY (usuario_horario) REFERENCES usuario_horarios(id) 
    ON DELETE RESTRICT ON UPDATE CASCADE;

-- ACTUALIZACIÓN VERSIÓN
UPDATE schema_info SET version_actual = '1.1.0' WHERE id = 1;
