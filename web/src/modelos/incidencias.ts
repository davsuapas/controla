import 'reflect-metadata';
import { Expose, plainToInstance } from 'class-transformer';
import dayjs, { Dayjs } from "dayjs";
import { DescriptorMarcaje } from "./marcaje";
import { dateToStr, formatDateForServer, formatDateTimeForServer, formatTimeForServer, formatTimeFromServer } from "./formatos";
import { DescriptorUsuario } from './usuarios';
import { DominiosWithCacheUsuarioDTO } from './dto';

export enum TipoIncidencia {
  NuevoMarcaje = 1,
  EliminacionMarcaje = 2,
  CorrecionSalida = 3,
}

export const NombresTipoIncidencia: Record<TipoIncidencia, string> = {
  [TipoIncidencia.NuevoMarcaje]: "Nuevo marcaje",
  [TipoIncidencia.EliminacionMarcaje]: "Eliminación marcaje",
  [TipoIncidencia.CorrecionSalida]: "Correción salida",
};

export enum EstadoIncidencia {
  Solicitud = 1,
  Conflicto = 2,
  ErrorResolver = 3,
  Rechazada = 4,
  Resuelta = 5,
  Resolver = 6,
  Rechazar = 7,
}

export const NombresEstadoIncidencia: Record<EstadoIncidencia, string> = {
  [EstadoIncidencia.Solicitud]: "Solicitud",
  [EstadoIncidencia.Conflicto]: "Conflictos",
  [EstadoIncidencia.ErrorResolver]: "Error resolviendo",
  [EstadoIncidencia.Rechazada]: "Rechazada",
  [EstadoIncidencia.Resuelta]: "Resuelta",
  [EstadoIncidencia.Resolver]: "Resolver",
  [EstadoIncidencia.Rechazar]: "Rechazar",
};

// Entidad incidencia que es válida tanto de entrada como salida
// del servidor
export class Incidencia {
  id: number;
  tipo: TipoIncidencia;
  @Expose({ name: 'fecha_solicitud' })
  fechaSolicitud: Dayjs | string;
  @Expose({ name: 'fecha_resolucion' })
  fechaResolucion: Dayjs | string | null;
  usuario: DescriptorUsuario | number;
  fecha: Dayjs | string;
  @Expose({ name: 'hora_inicio' })
  horaInicio: string | null;
  @Expose({ name: 'hora_fin' })
  horaFin: string | null;
  marcaje: DescriptorMarcaje | null;
  estado: EstadoIncidencia;
  @Expose({ name: 'fecha_estado' })
  fechaEstado: Dayjs | string | null;
  error: string | null;
  @Expose({ name: 'usuario_creador' })
  usuarioCreador: DescriptorUsuario | number;
  @Expose({ name: 'usuario_gestor' })
  usuarioGestor: DescriptorUsuario | number | null;
  @Expose({ name: 'motivo_solicitud' })
  motivoSolicitud: string | null;
  @Expose({ name: 'motivo_rechazo' })
  motivoRechazo: string | null;

  constructor(data: Partial<Incidencia>) {
    Object.assign(this, data);
  }

  // Crea una entidad de incidencias para una solicitud que
  // será enviada al servidor
  static crearSolicitud(
    tipo: TipoIncidencia,
    usuario: number,
    fechaMarcaje: Dayjs,
    horaInicio: Dayjs | null,
    horaFin: Dayjs | null,
    marcaje: DescriptorMarcaje | null,
    usuarioCreador: number,
    motivo: string | null
  ): Incidencia {
    return plainToInstance(Incidencia, {
      id: 0,
      tipo: tipo,
      fecha_solicitud: formatDateTimeForServer(dayjs()) as string,
      fecha_resolucion: null,
      usuario: usuario,
      fecha: formatDateForServer(fechaMarcaje) as string,
      hora_inicio: formatTimeForServer(horaInicio),
      hora_fin: formatTimeForServer(horaFin),
      marcaje: marcaje,
      estado: EstadoIncidencia.Solicitud,
      fecha_estado: null,
      error: null,
      usuario_creador: usuarioCreador,
      usuario_gestor: null,
      motivo_solicitud: motivo === '' ? null : motivo,
      motivo_rechazo: null
    });
  }

  static fromRequest(dto: DominiosWithCacheUsuarioDTO<any>): Incidencia[] {
    return dto.items.map(item => {
      return plainToInstance(Incidencia, {
        id: item.id,
        tipo: item.tipo,
        fecha_solicitud: dateToStr(dayjs(item.fecha_solicitud)),
        fecha_resolucion: dateToStr(dayjs(item.fecha_resolucion)),
        usuario: dto.usuario(item.usuario),
        fecha: dateToStr(dayjs(item.fecha)),
        hora_inicio: formatTimeFromServer(item.hora_inicio),
        hora_fin: formatTimeFromServer(item.hora_fin),
        marcaje: item.marcaje ?
          DescriptorMarcaje.fromRequest(item.marcaje) : null,
        estado: item.estado,
        error: item.error || null,
        fecha_estado: dateToStr(dayjs(item.fecha_estado)),
        usuario_creador: dto.usuario(item.usuario_creador),
        usuario_gestor: item.usuario_gestor ?
          dto.usuario(item.usuario_gestor) : null,
        motivo_solicitud: item.motivo_solicitud || null,
        motivo_rechazo: item.motivo_rechazo || null
      });
    });
  }
}
