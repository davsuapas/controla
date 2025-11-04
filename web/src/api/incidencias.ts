import { AxiosInstance } from "axios";
import { EstadoIncidencia, Incidencia, TipoIncidencia } from "../modelos/incidencias";
import { instanceToPlain } from "class-transformer";
import dayjs from "dayjs";
import { DescriptorMarcaje } from "../modelos/marcaje";
import { DescriptorUsuario } from "../modelos/usuarios";
import { DominiosWithCacheUsuarioDTO, IncidenciaProcesoDTO } from "../modelos/dto";
import { formatDateForServer } from "../modelos/formatos";

export interface IncidenciaApi {
  crearIncidencia(inc: Incidencia): Promise<void>;
  // Procesa las incidencias y devuelve las incidencias procesadas
  // según un filtro y las incidencias con errores faltales
  procesar(
    usuario_gestor: number,
    incidencias: IncidenciaProcesoDTO[],
    fechaInicio: dayjs.Dayjs | null,
    fechaFin: dayjs.Dayjs | null,
    estados: EstadoIncidencia[],
    usuarioId: number | null
  ): Promise<{ inc_erroneas: number[], incs: Incidencia[] }>;
  incidencias(
    fechaInicio: dayjs.Dayjs | null,
    fechaFin: dayjs.Dayjs | null,
    estados: EstadoIncidencia[],
    usuarioId: number | null
  ): Promise<Incidencia[]>;
}


// Implementación de IncidenciaApi en modo producción
export class IncidenciaAxiosApi implements IncidenciaApi {
  private axios: AxiosInstance;

  constructor(axiosInstance: AxiosInstance) {
    this.axios = axiosInstance;
  }

  async crearIncidencia(inc: Incidencia): Promise<void> {
    return this.axios.post('api/incidencias', instanceToPlain(inc));
  }

  async procesar(
    usuario_gestor: number,
    incidencias: IncidenciaProcesoDTO[],
    fechaInicio: dayjs.Dayjs | null,
    fechaFin: dayjs.Dayjs | null,
    estados: EstadoIncidencia[],
    usuarioId: number | null
  ): Promise<{ inc_erroneas: number[], incs: Incidencia[] }> {
    const response = await this.axios.post(
      'api/incidencias/procesar',
      {
        usuario_gestor: usuario_gestor,
        param_filtro_inc: paramFiltro(
          fechaInicio, fechaFin, estados, usuarioId),
        incidencias: incidencias
      }
    );

    return {
      inc_erroneas: response.data.incidencias_erroneas,
      incs: Incidencia.fromRequest(
        DominiosWithCacheUsuarioDTO.fromResponse(response.data.incidencias))
    };
  }

  async incidencias(
    fechaInicio: dayjs.Dayjs | null,
    fechaFin: dayjs.Dayjs | null,
    estados: EstadoIncidencia[],
    usuarioId: number | null
  ): Promise<Incidencia[]> {
    const response = await this.axios.post(
      'api/incidencias/por/fechas', paramFiltro(
        fechaInicio, fechaFin, estados, usuarioId));

    return Incidencia.fromRequest(
      DominiosWithCacheUsuarioDTO.fromResponse(response.data));
  }
}

// Implementación de MarcajeApi en modo test
export class IncidenciaTestApi implements IncidenciaApi {
  async crearIncidencia(_: Incidencia): Promise<void> {
    return;
  }

  async procesar(
    _: number,
    __: IncidenciaProcesoDTO[],
    ___: dayjs.Dayjs | null,
    ____: dayjs.Dayjs | null,
    _____: EstadoIncidencia[],
    ______: number | null
  ): Promise<{ inc_erroneas: number[], incs: Incidencia[] }> {
    return {
      inc_erroneas: [2],
      incs: await this.incidencias(null, null, [], null)
    };
  }

  async incidencias(
    _: dayjs.Dayjs | null,
    __: dayjs.Dayjs | null,
    ___: EstadoIncidencia[],
    ____: number | null
  ): Promise<Incidencia[]> {
    const incidencias: Incidencia[] = [
      {
        id: 1,
        tipo: TipoIncidencia.NuevoMarcaje,
        fechaSolicitud: "2024-01-15",
        fechaResolucion: "2024-01-15",
        fecha: "2024-01-15",
        horaInicio: "08:00",
        horaFin: "17:00",
        marcaje: new DescriptorMarcaje(101, "08:00", "17:00"),
        estado: EstadoIncidencia.Solicitud,
        fechaEstado: null,
        error: null,
        usuario: new DescriptorUsuario(1, "Juan", "Pérez", "Gómez"),
        usuarioCreador: new DescriptorUsuario(1, "Juan", "Pérez", "Gómez"),
        usuarioGestor: new DescriptorUsuario(2, "María", "López", "Rodríguez"),
        motivoSolicitud: "El verdadero éxito no se mide por los triunfos momentáneos, sino por la capacidad de levantarse después de cada caída. La vida nos enseña que los obstáculos son oportunidades disfrazadas para crecer, fortalecernos y descubrir nuestra verdadera resiliencia. Lo importante es mantener la esperanza y seguir adelante con determinación.",
        motivoRechazo: 'El verdadero éxito no se mide por los triunfos momentáneos, sino por la capacidad de levantarse después de cada caída. La vida nos enseña que los obstáculos son oportunidades disfrazadas para crecer, fortalecernos y descubrir nuestra verdadera resiliencia. Lo importante es mantener la esperanza y seguir adelante con determinación.'
      },
      {
        id: 2,
        tipo: TipoIncidencia.EliminacionMarcaje,
        fechaSolicitud: "2024-01-16",
        fechaResolucion: "2024-01-16",
        fecha: "2024-01-16",
        horaInicio: null,
        horaFin: null,
        marcaje: null,
        fechaEstado: "2024-01-15",
        estado: EstadoIncidencia.Conflicto,
        error: "Conflicto con marcaje existente",
        usuario: new DescriptorUsuario(1, "Juan", "Pérez", "Gómez"),
        usuarioCreador: new DescriptorUsuario(1, "Juan", "Pérez", "Gómez"),
        usuarioGestor: null,
        motivoSolicitud: "Marcaje duplicado por error del sistema",
        motivoRechazo: null
      },
      {
        id: 3,
        tipo: TipoIncidencia.CorrecionSalida,
        fechaSolicitud: "2024-01-17",
        fechaResolucion: null,
        fecha: "2024-01-17",
        horaInicio: "09:15",
        horaFin: "18:30",
        marcaje: new DescriptorMarcaje(205, "09:15", null),
        estado: EstadoIncidencia.Resuelta,
        fechaEstado: null,
        error: null,
        usuario: new DescriptorUsuario(1, "Juan", "Pérez", "Gómez"),
        usuarioCreador: new DescriptorUsuario(4, "Carlos", "Martínez", 'Perez'),
        usuarioGestor: new DescriptorUsuario(5, "Ana", "Sánchez", "Fernández"),
        motivoSolicitud: "Corrección de hora de salida",
        motivoRechazo: null
      },
      {
        id: 4,
        tipo: TipoIncidencia.NuevoMarcaje,
        fechaSolicitud: "2024-01-18",
        fechaResolucion: null,
        fecha: "2024-01-18",
        horaInicio: null,
        horaFin: "07:45",
        marcaje: null,
        estado: EstadoIncidencia.Rechazada,
        fechaEstado: "2024-01-15",
        error: "Horario no permitido",
        usuario: new DescriptorUsuario(1, "Juan", "Pérez", "Gómez"),
        usuarioCreador: new DescriptorUsuario(1, "Juan", "Pérez", "Gómez"),
        usuarioGestor: new DescriptorUsuario(5, "Ana", "Sánchez", "Fernández"),
        motivoSolicitud: null,
        motivoRechazo: ''
      },
      {
        id: 5,
        tipo: TipoIncidencia.EliminacionMarcaje,
        fechaSolicitud: "2024-01-19",
        fechaResolucion: null,
        fecha: "2024-01-19",
        horaInicio: null,
        horaFin: "16:00",
        marcaje: new DescriptorMarcaje(310, "08:00", "17:00"),
        estado: EstadoIncidencia.ErrorResolver,
        fechaEstado: "2024-01-15",
        error: "Error en base de datos",
        usuario: new DescriptorUsuario(1, "Juan", "Pérez", "Gómez"),
        usuarioCreador: new DescriptorUsuario(7, "Laura", "García", "Hernández"),
        usuarioGestor: null,
        motivoSolicitud: "Eliminar marcaje incorrecto",
        motivoRechazo: null
      },
      {
        id: 6,
        tipo: TipoIncidencia.EliminacionMarcaje,
        fechaSolicitud: "2024-01-19",
        fechaResolucion: null,
        fecha: "2024-01-19",
        horaInicio: "07:45",
        horaFin: "16:00",
        marcaje: new DescriptorMarcaje(310, "08:00", "17:00"),
        estado: EstadoIncidencia.ErrorResolver,
        fechaEstado: "2024-01-15",
        error: "Error en base de datos",
        usuario: new DescriptorUsuario(1, "Juan", "Pérez", "Gómez"),
        usuarioCreador: new DescriptorUsuario(7, "Laura", "García", "Hernández"),
        usuarioGestor: new DescriptorUsuario(7, "Laura", "García", "Hernández"),
        motivoSolicitud: "El verdadero éxito no se mide por los triunfos momentáneos, sino por la capacidad de levantarse después de cada caída. La vida nos enseña que los obstáculos son oportunidades disfrazadas para crecer, fortalecernos y descubrir nuestra verdadera resiliencia. Lo importante es mantener la esperanza y seguir adelante con determinación.",
        motivoRechazo: 'El verdadero éxito no se mide por los triunfos momentáneos, sino por la capacidad de levantarse después de cada caída. La vida nos enseña que los obstáculos son oportunidades disfrazadas para crecer, fortalecernos y descubrir nuestra verdadera resiliencia. Lo importante es mantener la esperanza y seguir adelante con determinación.'
      },
    ];

    return incidencias;
  }
}

function paramFiltro(
  fechaInicio: dayjs.Dayjs | null,
  fechaFin: dayjs.Dayjs | null,
  estados: EstadoIncidencia[],
  usuarioId: number | null): {
    fecha_inicio: string | null;
    fecha_fin: string | null;
    estados: EstadoIncidencia[]; usuario: number | null;
  } {
  return {
    fecha_inicio: fechaInicio ? formatDateForServer(fechaInicio) : null,
    fecha_fin: fechaFin ? formatDateForServer(fechaFin) : null,
    estados: estados,
    usuario: usuarioId
  };
}
