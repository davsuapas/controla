-- ISSUE #9 y #11 

-- Calendarios

CREATE TABLE IF NOT EXISTS calendarios (
  id int(10) unsigned NOT NULL AUTO_INCREMENT,
  nombre varchar(100) NOT NULL,
  descripcion varchar(500) DEFAULT NULL,
  PRIMARY KEY (id)
) AUTO_INCREMENT=1 COMMENT='Calendarios laborales';

CREATE TABLE IF NOT EXISTS calendario_fechas (
  id int(10) unsigned NOT NULL AUTO_INCREMENT,
  calendario int(10) unsigned NOT NULL,
  fecha_inicio date NOT NULL,
  fecha_fin date NOT NULL,
  tipo smallint(5) unsigned NOT NULL,
  PRIMARY KEY (id),
  KEY calendario_fechas_calendario_FK (calendario, fecha_inicio DESC),
  CONSTRAINT calendario_fechas_calendario_FK FOREIGN KEY (calendario) REFERENCES calendarios (id) ON UPDATE CASCADE
) AUTO_INCREMENT=1 COMMENT='Fechas señaladas en los calendarios';

CREATE TABLE IF NOT EXISTS calendarios_usuario (
  usuario int(10) unsigned NOT NULL,
  calendario int(10) unsigned NOT NULL,
  PRIMARY KEY (usuario, calendario),
  KEY usuario_calendario_calendario_FK (calendario),
  CONSTRAINT usuario_calendario_usuario_FK FOREIGN KEY (usuario) REFERENCES usuarios (id) ON UPDATE CASCADE,
  CONSTRAINT usuario_calendario_calendario_FK FOREIGN KEY (calendario) REFERENCES calendarios (id) ON UPDATE CASCADE
) COMMENT='Calendarios asignados a un usuario';

-- ACTUALIZACIÓN VERSIÓN
UPDATE schema_info SET version_actual = '1.2.0' WHERE id = 1;
