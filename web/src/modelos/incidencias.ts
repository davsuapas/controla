import 'reflect-metadata';
import { Expose, plainToInstance } from 'class-transformer';
import dayjs, { Dayjs } from "dayjs";
import { DescriptorMarcaje } from "./marcaje";
import { formatDateForServer, formatDateTimeForServer, formatTimeForServer } from "./formatos";

export enum TipoIncidencia {
  NuevoMarcaje = 1,
  EliminacionMarcaje = 2,
  CorrecionSalida = 3,
}

export enum EstadoIncidencia {
  Solicitud = 1,
  Inconsistente = 2,
  ErrorInterno = 3,
  Rechazada = 4,
  Resuelta = 5,
}

// Entidad incidencia que es válida tanto de entrada como salida
// del servidor
export class Incidencia {
  id: number;
  tipo: TipoIncidencia;
  @Expose({ name: 'fecha_solicitud' })
  fechaSolicitud: Dayjs | string;
  fecha: Dayjs | string;
  @Expose({ name: 'hora_inicio' })
  horaInicio: string | null;
  @Expose({ name: 'hora_fin' })
  horaFin: string | null;
  marcaje: DescriptorMarcaje | null;
  estado: EstadoIncidencia;
  error: string | null;
  @Expose({ name: 'usuario_creador' })
  usuarioCreador: number;
  @Expose({ name: 'usuario_gestor' })
  usuarioGestor: number | null;
  @Expose({ name: 'motivo_solicitud' })
  motivoSolicitud: string | null;

  constructor(data: Partial<Incidencia>) {
    Object.assign(this, data);
  }

  // Crea una entidad de incidencias para una solicitud que
  // será enviada al servidor
  static crearSolicitud(
    tipo: TipoIncidencia,
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
      fecha: formatDateForServer(fechaMarcaje) as string,
      horaInicio: formatTimeForServer(horaInicio),
      horaFin: formatTimeForServer(horaFin),
      marcaje: marcaje,
      estado: EstadoIncidencia.Solicitud,
      error: null,
      usuario_creador: usuarioCreador,
      usuario_gestor: null,
      motivo_solicitud: motivo
    });
  }
}