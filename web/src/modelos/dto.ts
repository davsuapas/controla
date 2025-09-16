import { formatDateForServer, Usuario } from "./usuarios";

// La entidad UsuarioDTO se usa para enviar a el servidor
export class UsuarioDTO {
  public inicio = null;

  constructor(
    public id: number,
    public autor: number,
    public dni: string,
    public email: string,
    public nombre: string,
    public primer_apellido: string,
    public segundo_apellido: string,
    public password: string,
    public activo: string | null,
    public roles: number[],
  ) {
  }

  static fromUsuario(usr: Usuario): UsuarioDTO {
    return new UsuarioDTO(
      usr.id,
      12, // TODO: Cambiar por el usuario logueado
      usr.dni,
      usr.email,
      usr.nombre,
      usr.primer_apellido,
      usr.segundo_apellido,
      usr.password!,
      formatDateForServer(usr.activo),
      usr.roles.map((r) => r.id),
    );
  }
}
