USE @DB_NOMBRE;

START TRANSACTION;

CREATE TABLE IF NOT EXISTS usuarios (
  nombre varchar(50) NOT NULL,
  id int(10) unsigned NOT NULL AUTO_INCREMENT,
  password varchar(500) NOT NULL,
  activo datetime DEFAULT NULL,
  primer_apellido varchar(100) NOT NULL,
  segundo_apellido varchar(100) NOT NULL,
  dni char(56) NOT NULL,
  inicio datetime DEFAULT NULL,
  dni_hash char(64) NOT NULL,
  email varchar(254) NOT NULL,
  PRIMARY KEY (id),
  UNIQUE KEY dni_hash_unique (dni_hash)
) AUTO_INCREMENT=1 COMMENT='Son los usuarios de la compañia';

CREATE TABLE IF NOT EXISTS roles_usuario (
  usuario int(10) unsigned NOT NULL,
  rol int(10) unsigned NOT NULL,
  PRIMARY KEY (usuario,rol),
  KEY roles_usuario_roles_FK (rol),
  CONSTRAINT roles_usuario_usuarios_FK FOREIGN KEY (usuario) REFERENCES usuarios (id) ON UPDATE CASCADE
) COMMENT='Son los roles a lo que pertenece un usuario';

CREATE TABLE IF NOT EXISTS trazas (
  id int(10) unsigned NOT NULL AUTO_INCREMENT,
  fecha datetime NOT NULL,
  entidad_id int(10) unsigned NOT NULL,
  motivo varchar(500) DEFAULT NULL,
  tipo smallint(5) unsigned NOT NULL,
  autor int(10) unsigned DEFAULT NULL,
  entidad smallint(5) unsigned NOT NULL,
  PRIMARY KEY (id),
  KEY trazas_usuarios_FK_1 (autor),
  CONSTRAINT trazas_usuarios_FK_1 FOREIGN KEY (autor) REFERENCES usuarios (id) ON UPDATE CASCADE
) AUTO_INCREMENT=1 COMMENT='Son las trazas de cada registro';

CREATE TABLE IF NOT EXISTS horarios (
  id int(10) unsigned NOT NULL AUTO_INCREMENT,
  dia char(1) NOT NULL,
  hora_inicio time NOT NULL,
  hora_fin time NOT NULL,
  PRIMARY KEY (id)
) AUTO_INCREMENT=1 COMMENT='Son lo horarios de cada usuario trabajador';

CREATE TABLE IF NOT EXISTS usuario_horarios (
  usuario int(10) unsigned NOT NULL,
  horario int(10) unsigned NOT NULL,
  fecha_creacion date NOT NULL,
  PRIMARY KEY (usuario,horario),
  KEY usuario_horarios_horarios_FK (horario),
  KEY idx_usuario_fecha (usuario,fecha_creacion DESC),
  CONSTRAINT usuario_horarios_horarios_FK FOREIGN KEY (horario) REFERENCES horarios (id) ON UPDATE CASCADE,
  CONSTRAINT usuario_horarios_usuarios_FK FOREIGN KEY (usuario) REFERENCES usuarios (id) ON UPDATE CASCADE
) COMMENT='Se define los horarios que tiene un usuario para una fecha';

CREATE TABLE IF NOT EXISTS marcajes (
  id int(10) unsigned NOT NULL AUTO_INCREMENT,
  usuario int(10) unsigned NOT NULL,
  fecha date NOT NULL,
  hora_inicio time NOT NULL,
  hora_fin time DEFAULT NULL,
  horario int(10) unsigned NOT NULL,
  usuario_registrador int(10) unsigned DEFAULT NULL,
  modificado_por int(10) unsigned DEFAULT NULL,
  eliminado tinyint(1) DEFAULT NULL,
  PRIMARY KEY (id),
  KEY registros_horarios_FK (horario),
  KEY registros_usuarios_FK_1 (usuario_registrador),
  KEY marcajes_usuario_IDX (usuario,fecha) USING BTREE,
  KEY marcajes_usuario_fecha_desc (usuario,fecha DESC) USING BTREE,
  CONSTRAINT registros_horarios_FK FOREIGN KEY (horario) REFERENCES horarios (id) ON UPDATE CASCADE,
  CONSTRAINT registros_usuarios_FK FOREIGN KEY (usuario) REFERENCES usuarios (id) ON UPDATE CASCADE,
  CONSTRAINT registros_usuarios_FK_1 FOREIGN KEY (usuario_registrador) REFERENCES usuarios (id) ON UPDATE CASCADE
) AUTO_INCREMENT=1 COMMENT='Son los registros de cada empleado (usuario)';

CREATE TABLE IF NOT EXISTS incidencias (
  id int(10) unsigned NOT NULL AUTO_INCREMENT,
  tipo smallint(5) unsigned NOT NULL,
  fecha_solicitud datetime NOT NULL,
  hora_inicio time DEFAULT NULL,
  hora_fin time DEFAULT NULL,
  marcaje int(10) unsigned DEFAULT NULL,
  estado smallint(5) unsigned NOT NULL,
  error varchar(500) DEFAULT NULL,
  usuario_creador int(10) unsigned NOT NULL,
  usuario_gestor int(10) unsigned DEFAULT NULL,
  fecha date NOT NULL,
  motivo_solicitud varchar(200) DEFAULT NULL,
  motivo_rechazo varchar(200) DEFAULT NULL,
  fecha_resolucion datetime DEFAULT NULL,
  fecha_estado datetime DEFAULT NULL,
  usuario int(10) unsigned NOT NULL,
  PRIMARY KEY (id),
  KEY Incidencias_marcajes_FK (marcaje),
  KEY Incidencias_usuarios_FK (usuario_creador),
  KEY Incidencias_usuarios_FK_1 (usuario_gestor),
  KEY incidencias_estado_fecha_IDX (estado,fecha_solicitud,usuario_creador) USING BTREE,
  KEY incidencias_estado_usuario_IDX (estado,usuario_creador,fecha_solicitud) USING BTREE,
  CONSTRAINT Incidencias_marcajes_FK FOREIGN KEY (marcaje) REFERENCES marcajes (id) ON UPDATE CASCADE,
  CONSTRAINT Incidencias_usuarios_FK FOREIGN KEY (usuario_creador) REFERENCES usuarios (id) ON UPDATE CASCADE,
  CONSTRAINT Incidencias_usuarios_FK_1 FOREIGN KEY (usuario_gestor) REFERENCES usuarios (id) ON UPDATE CASCADE
) AUTO_INCREMENT=1 COMMENT='Incidencias de los marcajes horarios';

COMMIT;