USE @DB_NOMBRE;

START TRANSACTION;

ALTER TABLE controla.usuario_horarios MODIFY COLUMN fecha_creacion DATETIME NOT NULL;

UPDATE schema_info SET version_actual = '1.1.0' WHERE id = 1;

COMMIT;