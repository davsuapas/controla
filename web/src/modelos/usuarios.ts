import dayjs, { Dayjs } from 'dayjs';
import { matchPath } from 'react-router';
import { dateToStr } from './formatos';


// Si cambia el nombre cambiarlo en Login.tsx
export enum RolID {
  Empleado = 1,
  Gestor = 2,
  Admin = 3,
  Director = 4,
  Registrador = 5,
  Inspector = 6,
  Configurador = 7
}

// El orden es imporante para establecer la ruta del login
export const ROLES: Map<RolID, {
  nombre: string;
  ruta_login: string;
  rutas_acceso: string[];
  // otras propiedades que necesites
}> = new Map([
  [RolID.Admin, {
    nombre: 'Admin',
    ruta_login: '/usuarios',
    rutas_acceso: ['/miarea/*', '/usuarios/*']
  }],
  [RolID.Registrador, {
    nombre: 'Registrador',
    ruta_login: '/registro/manual',
    rutas_acceso: ['/miarea/*', '/registro/manual']
  }],
  [RolID.Gestor, {
    nombre: 'Gestor',
    ruta_login: '',
    rutas_acceso: ['/miarea/*']
  }],
  [RolID.Empleado, {
    nombre: 'Empleado',
    ruta_login: '',
    rutas_acceso: ['/miarea/*']
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
  [RolID.Configurador, {
    nombre: 'Configurador',
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

  nombreCompleto(): string {
    return `${this.nombre} 
    ${this.primer_apellido} ${this.segundo_apellido}`.trim();
  }
}

export class Usuario {
  public roles: Rol[];
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
  ) {
    this.roles = rolesIds.map(Rol.desdeId);
  }

  nombreCompleto(): string {
    return `${this.nombre} 
    ${this.primerApellido} ${this.segundoApellido}`.trim();
  }

  activoToStr(): string {
    return this.activo ? dateToStr(this.activo) : 'No activo';
  }

  inicioToStr(): string {
    return this.inicio ? dateToStr(this.inicio) : 'No logeado';
  }

  anyRoles(ids: RolID[]): boolean {
    return this.roles.some(rol => ids.includes(rol.id));
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
    );
  }
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

export class Horario {
  constructor(
    public dia: DiaSemana,
    public horaInicio: string,
    public horaFin: string,
    public horasATrabajar: number
  ) { }

  // Crea una instancia desde la solicitudo del servidor
  static fromRequest(obj: any): Horario {
    return new Horario(
      diaSemanafromLetra[obj.dia],
      obj.hora_inicio,
      obj.hora_fin,
      obj.horas_a_trabajar
    );
  }

  horasATrabajarToStr(): string {
    return this.horasATrabajar.toFixed(2);
  }

  horarioToStr(): string {
    return `${this.horaInicio} - ${this.horaFin}`;
  }
}