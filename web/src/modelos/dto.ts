import dayjs from "dayjs";
import { formatDateForServer, formatDateTimeForServer, formatTimeForServer } from "./formatos";
import { DescriptorUsuario, Usuario } from "./usuarios";
import { EstadoIncidencia } from "./incidencias";

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

  // No se olvide de proporcionar el autor si procede
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

// DTO para enviar marcajes al servidor
export class MarcajeOutDTO {
  constructor(
    public usuario: number,
    public usuario_reg: number | null,
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
  ): MarcajeOutDTO {
    const usuarioReg = usuarioId == usuarioLogeado.id ? null : usuarioLogeado.id

    return new MarcajeOutDTO(
      usuarioId,
      usuarioReg,
      formatDateForServer(fecha)!,
      formatTimeForServer(horaInicio)!,
      formatTimeForServer(horaFin)
    );
  }
}

// Gestiona una cache de usuarios
// Los datos son obtenidos del servidor
export class DominiosWithCacheUsuarioDTO<T> {
  constructor(
    public readonly items: T[],
    public readonly cache: { [key: string]: any }
  ) { }

  // Método estático para crear desde respuesta Axios
  static fromResponse<T>(responseData: any): DominiosWithCacheUsuarioDTO<T> {
    return new DominiosWithCacheUsuarioDTO<T>(
      responseData.items || [],
      responseData.cache || {}
    );
  }

  // Helper interno para obtener usuarios del cache
  private getUsuarioFromCache(id: number | string): DescriptorUsuario | null {
    const key = typeof id === 'number' ? id.toString() : id;
    const userObj = this.cache[key];

    if (!userObj) return null;

    return new DescriptorUsuario(
      userObj.id,
      userObj.nombre,
      userObj.primer_apellido,
      userObj.segundo_apellido
    );
  }

  // Helper interno para obtener usuario o lanzar error
  private TryGetUsuario(id: number | string): DescriptorUsuario {
    const usuario = this.getUsuarioFromCache(id);
    if (!usuario) {
      throw new Error(`Usuario con ID ${id} no encontrado en cache`);
    }
    return usuario;
  }

  // Método público para que fromRequest() pueda obtener usuarios
  usuario(id: number): DescriptorUsuario {
    return this.TryGetUsuario(id);
  }

  // Método público para obtener usuario opcional
  usuarioOptional(id: number | null | undefined): DescriptorUsuario | null {
    return id ? this.TryGetUsuario(id) : null;
  }
}

// La entidad que se utiliza para procesar incidencias
export class IncidenciaProcesoDTO {
  constructor(
    public id: number,
    public estado: EstadoIncidencia,
    public motivo_rechazo: string | null) { }
}
