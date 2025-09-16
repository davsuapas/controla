import dayjs, { Dayjs } from 'dayjs';

export const ROLES: Record<number, string> = {
  1: 'Empleado',
  2: 'Gestor',
  3: 'Admin',
  4: 'Director',
  5: 'Registrador',
  6: 'Inspector',
  7: 'Configurador'
};

export function nombresTodosRoles(): string[] {
  return Object.values(ROLES);
}

export function nombresRoles(roles: Rol[]): string[] {
  return roles.map(rol => rol.nombre);
}

export function idRolPorNombre(nombre: string): number {
  const entry = Object.entries(ROLES).find(([_, value]) => value === nombre);
  return entry ? parseInt(entry[0]) : 0;
}

export class Rol {
  constructor(
    public id: number,
    public nombre: string
  ) { }

  static desdeId(id: number): Rol {
    return new Rol(id, ROLES[id]);
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