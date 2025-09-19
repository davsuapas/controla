import dayjs, { Dayjs } from 'dayjs';
import { matchPath } from 'react-router';


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

// El orden es imporante para establece la ruta del login
export const ROLES: Record<RolID, {
  nombre: string;
  ruta_login: string;
  rutas_acceso: string[];
  // otras propiedades que necesites
}> = {
  [RolID.Director]: {
    nombre: 'Empleado',
    ruta_login: '/usuarios',
    rutas_acceso: ['/usuarios/*']
  },
  [RolID.Admin]: {
    nombre: 'Empleado',
    ruta_login: '/usuarios',
    rutas_acceso: ['/usuarios/*']
  },
  [RolID.Configurador]: {
    nombre: 'Empleado',
    ruta_login: '',
    rutas_acceso: []
  },
  [RolID.Gestor]: {
    nombre: 'Empleado',
    ruta_login: '',
    rutas_acceso: []
  },
  [RolID.Registrador]: {
    nombre: 'Empleado',
    ruta_login: '',
    rutas_acceso: []
  },
  [RolID.Empleado]: {
    nombre: 'Empleado',
    ruta_login: '',
    rutas_acceso: []
  },
  [RolID.Inspector]: {
    nombre: 'Empleado',
    ruta_login: '',
    rutas_acceso: []
  },
};

export function nombresTodosRoles(): string[] {
  return Object.values(ROLES).map(rol => rol.nombre);
}

export function nombresRoles(roles: Rol[]): string[] {
  return roles.map(rol => rol.nombre);
}

export function idRolPorNombre(nombre: string): number {
  const entry = Object.entries(ROLES).find(
    ([_, value]) => value.nombre === nombre);

  return entry ? parseInt(entry[0]) : 0;
}

export class Rol {
  constructor(
    public id: number,
    public nombre: string
  ) { }

  static desdeId(id: RolID): Rol {
    return new Rol(id, ROLES[id].nombre);
  }

  static desdeNombre(nombre: string): Rol {
    return new Rol(idRolPorNombre(nombre), nombre);
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
    public primer_apellido: string,
    public segundo_apellido: string,
    public activo: Dayjs | null,
    public inicio: Dayjs | null,
    rolesIds: number[],
  ) {
    this.roles = rolesIds.map(Rol.desdeId);
  }

  nombreCompleto(): string {
    return `${this.nombre} 
    ${this.primer_apellido} ${this.segundo_apellido || ''}`
      .trim();
  }

  activoToStr(): string {
    return this.activo ? this.activo.format('DD/MM/YYYY') : 'No activo';
  }

  inicioToStr(): string {
    return this.inicio ? this.inicio.format('DD/MM/YYYY') : 'No logeado';
  }

  anyRoles(ids: RolID[]): boolean {
    return this.roles.some(rol => ids.includes(rol.id));
  }

  acceso_a_ruta(ruta: string): boolean {
    return this.roles.some(rol => {
      const rolInfo = ROLES[rol.id as RolID];
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

export function formatDateForServer(
  date: dayjs.Dayjs | string | null | undefined): string | null {
  if (!date) return null;

  return dayjs(date).format('YYYY-MM-DDTHH:mm:ss');
}