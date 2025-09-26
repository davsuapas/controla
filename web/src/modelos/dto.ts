import dayjs from "dayjs";
import { formatDateForServer, formatDateTimeForServer, formatTimeForServer } from "./formatos";
import { DescriptorUsuario, Usuario } from "./usuarios";

// La entidad UsuarioDTO se usa para enviar a el servidor
export class UsuarioOutDTO {
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

  // No se olvide de propor el autor si procede
  static fromUsuario(usr: Usuario): UsuarioOutDTO {
    return new UsuarioOutDTO(
      usr.id,
      usr.autor!,
      usr.dni,
      usr.email,
      usr.nombre,
      usr.primerApellido,
      usr.segundoApellido,
      usr.password!,
      formatDateTimeForServer(usr.activo),
      usr.roles.map((r) => r.id),
    );
  }
}

// DTO para enviar registros al servidor
export class RegistroOutDTO {
  constructor(
    public usuario: number,
    public usuario_reg: DescriptorUsuario | null,
    public fecha: string,
    public hora_inicio: string,
    public hora_fin: string | null,
  ) { }

  // Método estático para crear desde los datos del formulario
  static new(
    usuarioId: number,
    usuarioLogeado: DescriptorUsuario,
    fecha: dayjs.Dayjs,
    horaInicio: dayjs.Dayjs,
    horaFin: dayjs.Dayjs | undefined,
  ): RegistroOutDTO {
    const usuarioReg = usuarioId == usuarioLogeado.id ? null : usuarioLogeado

    return new RegistroOutDTO(
      usuarioId,
      usuarioReg,
      formatDateForServer(fecha)!,
      formatTimeForServer(horaInicio)!,
      formatTimeForServer(horaFin)
    );
  }
}
