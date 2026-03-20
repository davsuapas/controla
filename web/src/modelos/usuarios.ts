import { Expose, plainToInstance } from 'class-transformer';
import dayjs, { Dayjs } from 'dayjs';
import { matchPath } from 'react-router';
import { dateToStr, formatDateForServer } from './formatos';


export enum RolID {
  Empleado = 1,
  Gestor = 2,
  Admin = 3,
  Director = 4,
  Registrador = 5,
  Inspector = 6,
  Supervisor = 7
}

// El orden es imporante para establecer la ruta del login
export const ROLES: Map<RolID, {
  nombre: string;
  ruta_login: string;
  rutas_acceso: string[];
}> = new Map([
  [RolID.Empleado, {
    nombre: 'Empleado',
    ruta_login: '/marcaje/auto',
    rutas_acceso: [
      '/miarea/*',
      '/marcaje/auto',
      '/marcaje/consulta',
      '/incidencias/solicitud',
      '/incidencias/gestion',
      '/informes/cumplimiento'
    ]
  }],
  [RolID.Registrador, {
    nombre: 'Registrador',
    ruta_login: '/marcaje/manual',
    rutas_acceso: [
      '/miarea/*',
      '/marcaje/manual',
      '/marcaje/auto',
      '/marcaje/consulta',
      '/incidencias/solicitud',
      '/incidencias/gestion'
    ]
  }],
  [RolID.Gestor, {
    nombre: 'Gestor',
    ruta_login: '/incidencias/revision',
    rutas_acceso: [
      '/miarea/*',
      '/incidencias/revision'
    ]
  }],
  [RolID.Supervisor, {
    nombre: 'Supervisor',
    ruta_login: '/incidencias/revision',
    rutas_acceso: [
      '/miarea/*',
      '/marcaje/consulta',
      '/incidencias/solicitud',
      '/incidencias/revision',
      '/incidencias/gestion',
      '/informes/cumplimiento'
    ]
  }],
  [RolID.Admin, {
    nombre: 'Admin',
    ruta_login: '/usuarios',
    rutas_acceso: [
      '/miarea/*',
      '/usuarios/*',
      '/horarios/*',
      '/calendarios/*'
    ]
  }],
  [RolID.Director, {
    nombre: 'Director',
    ruta_login: '/',
    rutas_acceso: ['/miarea/*']
  }],
  [RolID.Inspector, {
    nombre: 'Inspector',
    ruta_login: '',
    rutas_acceso: ['/miarea/*']
  }],
]);


export function nombresTodosRoles(): string[] {
  return Array.from(ROLES.values()).map(rol => rol.nombre);
}

export function nombresRoles(roles: Rol[]): string[] {
  return roles.map(rol => rol.nombre);
}

export function idRolPorNombre(nombre: string): number {
  const entry = Array.from(ROLES.entries()).find(
    ([_, value]) => value.nombre === nombre
  );

  return entry ? entry[0] : 0;
}

export class Rol {
  constructor(
    public id: number,
    public nombre: string
  ) { }

  static desdeId(id: RolID): Rol {
    return new Rol(id, ROLES.get(id)!.nombre);
  }

  static desdeNombre(nombre: string): Rol {
    return new Rol(idRolPorNombre(nombre), nombre);
  }
}

export class DescriptorUsuario {
  constructor(
    public id: number,
    public nombre: string,
    public primer_apellido: string,
    public segundo_apellido: string,
  ) { }

  static fromRequest(obj: any): DescriptorUsuario {
    return new DescriptorUsuario(
      obj.id,
      obj.nombre,
      obj.primer_apellido,
      obj.segundo_apellido,
    );
  }

  nombreCorto(): string {
    return `${this.nombre} ${this.primer_apellido}`.trim();
  }

  nombreCompleto(): string {
    return `${this.nombre} 
    ${this.primer_apellido} ${this.segundo_apellido}`.trim();
  }
}

export class Usuario {
  public roles: Rol[];
  public calendarios: UsuarioCalendario[];
  public password?: string;
  public passConfirm?: string;

  constructor(
    public id: number,
    public autor: number | null,
    public dni: string,
    public email: string,
    public nombre: string,
    public primerApellido: string,
    public segundoApellido: string,
    public activo: Dayjs | null,
    public inicio: Dayjs | null,
    rolesIds: number[],
    calendarios: any[],
  ) {
    this.roles = rolesIds.map(Rol.desdeId);
    this.calendarios = (calendarios || []).map(UsuarioCalendario.fromRequest);
  }

  nombreCompleto(): string {
    return `${this.nombre} 
    ${this.primerApellido} ${this.segundoApellido}`.trim();
  }

  activoToStr(): string {
    return this.activo ? dateToStr(this.activo)! : 'No activo';
  }

  inicioToStr(): string {
    return this.inicio ? dateToStr(this.inicio)! : 'No logeado';
  }

  anyRoles(ids: RolID[]): boolean {
    return this.roles.some(rol => ids.includes(rol.id));
  }

  tieneRol(id: RolID): boolean {
    return this.roles.some(rol => rol.id === id);
  }

  toDescriptor(): DescriptorUsuario {
    return new DescriptorUsuario(
      this.id,
      this.nombre,
      this.primerApellido,
      this.segundoApellido
    );
  }

  acceso_a_ruta(ruta: string): boolean {
    return this.roles.some(rol => {
      const rolInfo = ROLES.get(rol.id as RolID);
      if (!rolInfo) return false;

      return rolInfo.rutas_acceso.some(rutaAcceso =>
        matchPath(rutaAcceso, ruta) !== null
      );
    });
  }

  static fromRequest(obj: any): Usuario {
    return new Usuario(
      obj.id,
      null, // Desde el servidor el autor no es enviado
      obj.dni,
      obj.email,
      obj.nombre,
      obj.primer_apellido,
      obj.segundo_apellido,
      obj.activo ? dayjs(obj.activo) : null,
      obj.inicio ? dayjs(obj.inicio) : null,
      obj.roles,
      obj.calendarios ?? [],
    );
  }
}

export function nombresTodosLosCalendarios(
  calendarios: UsuarioCalendario[]): string[] {
  return calendarios.map(cal => cal.nombre);
}

export function nombresCalendariosAsignados(
  calendarios: UsuarioCalendario[]): string[] {
  return calendarios.filter(cal => cal.asignado).map(cal => cal.nombre);
}

export class UsuarioCalendario {
  constructor(
    public calendario: number,
    public nombre: string,
    public asignado: boolean,
  ) { }

  static fromRequest(obj: any): UsuarioCalendario {
    return new UsuarioCalendario(
      obj.calendario,
      obj.nombre,
      obj.asignado,
    );
  }
}

// Obtiene el usuario registrador para filtrar
export function filtroUsuarioRegistra(
  usuario: number, usuarioLog: Usuario): number | undefined {
  // Por defecto, solo se obtienen las solicitudes
  // del usuario
  let usuarioReg: number | undefined;

  // Si el usuario que se logeo es el mismo que el
  // se selecciona se obtienen todos sus marcajes,
  // ya que se esta actuando como rol empleado
  if (usuario != usuarioLog.id) {
    if (usuarioLog.tieneRol(RolID.Supervisor)) {
      // Si usuarioReg es igual a cero se buscarán
      // todos los marcajes del usuario selecionado que
      // hayan sido registradas por cualquier rol registrador
      usuarioReg = 0;
    } else if (usuarioLog.tieneRol(RolID.Registrador)) {
      // Solo se pueden obtener solicitudes de incidencias
      // de los marcajes realizados por el registrador
      usuarioReg = usuarioLog.id;
    }
  }

  return usuarioReg;
}

export enum DiaSemana {
  Lunes = 'L',
  Martes = 'M',
  Miércoles = 'X',
  Jueves = 'J',
  Viernes = 'V',
  Sábado = 'S',
  Domingo = 'D'
}

export const diaSemanafromLetra: { [key: string]: DiaSemana } = {
  'L': DiaSemana.Lunes,
  'M': DiaSemana.Martes,
  'X': DiaSemana.Miércoles,
  'J': DiaSemana.Jueves,
  'V': DiaSemana.Viernes,
  'S': DiaSemana.Sábado,
  'D': DiaSemana.Domingo
};

export const diaSemanaToPalabra: { [key in DiaSemana]: string } = {
  [DiaSemana.Lunes]: 'Lunes',
  [DiaSemana.Martes]: 'Martes',
  [DiaSemana.Miércoles]: 'Miércoles',
  [DiaSemana.Jueves]: 'Jueves',
  [DiaSemana.Viernes]: 'Viernes',
  [DiaSemana.Sábado]: 'Sábado',
  [DiaSemana.Domingo]: 'Domingo'
};

export class Horario {
  id: number;
  dia: DiaSemana;
  horas: number;

  constructor(data: Partial<Horario>) {
    Object.assign(this, data);
  }

  // Crea una instancia desde la solicitud del servidor
  static fromRequest(obj: any): Horario {
    return plainToInstance(Horario, {
      id: obj.id,
      dia: diaSemanafromLetra[obj.dia],
      horas: obj.horas,
    });
  }

  horarioToStr(): string {
    return `${diaSemanaToPalabra[this.dia]}: ${this.horas} hora/s`;
  }
}
export class ConfigHorario {
  id: number;
  usuario: number;
  horario: Horario;
  cortesia: number;
  @Expose({ name: 'fecha_creacion' })
  fechaCreacion: Dayjs
  @Expose({ name: 'caducidad_fecha_ini' })
  caducidadFechaIni: Dayjs | null
  @Expose({ name: 'caducidad_fecha_fin' })
  caducidadFechaFin: Dayjs | null

  constructor(data: Partial<ConfigHorario>) {
    Object.assign(this, data);
  }

  static fromRequest(obj: any): ConfigHorario {
    return new ConfigHorario({
      id: obj.id,
      usuario: obj.usuario,
      horario: Horario.fromRequest({
        id: obj.id,
        dia: obj.dia,
        horas: obj.horas,
      }),
      cortesia: obj.cortesia,
      fechaCreacion: dayjs(obj.fecha_creacion),
      caducidadFechaIni: obj.caducidad_fecha_ini ?
        dayjs(obj.caducidad_fecha_ini) : null,
      caducidadFechaFin: obj.caducidad_fecha_fin ?
        dayjs(obj.caducidad_fecha_fin) : null
    });
  }

  static fromRequestArray(objs: any[]): ConfigHorario[] {
    return objs.map(obj => ConfigHorario.fromRequest(obj));
  }

  toServer(): {} {
    return {
      id: this.id,
      usuario: this.usuario,
      dia: this.horario.dia,
      horas: this.horario.horas,
      cortesia: this.cortesia,
      fecha_creacion: formatDateForServer(this.fechaCreacion),
      caducidad_fecha_ini: formatDateForServer(this.caducidadFechaIni),
      caducidad_fecha_fin: formatDateForServer(this.caducidadFechaFin),
    };
  }
}
