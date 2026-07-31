USE @DB_NOMBRE;

CREATE TABLE IF NOT EXISTS config (
  id int(11) NOT NULL CHECK (id = 1),
  lat double DEFAULT NULL,
  lng double DEFAULT NULL,
  accuracy double DEFAULT NULL,
  margen_recinto int(10) DEFAULT NULL,
  PRIMARY KEY (id)
) COMMENT='Configuración general de la aplicación';

INSERT INTO config (id) VALUES(1) ON DUPLICATE KEY UPDATE id=id;

UPDATE schema_info SET version_actual = '1.5.0' WHERE id = 1;
