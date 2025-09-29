import { AxiosInstance } from "axios";
import { Marcaje } from "../modelos/marcaje";
import { MarcajeOutDTO } from "../modelos/dto";
import dayjs from "dayjs";
import { formatDateTimeForServer } from "../modelos/formatos";

export interface MarcajeApi {
  marcajes_por_fecha(usuarioId: string, fecha: dayjs.Dayjs): Promise<Marcaje[]>;
  ultimos_marcajes(usuarioId: string): Promise<Marcaje[]>;
  registrar(reg: MarcajeOutDTO): Promise<void>;
}


// Implementación de MarcajeApi en modo producción
export class MarcajeAxiosApi implements MarcajeApi {
  private axios: AxiosInstance;

  constructor(axiosInstance: AxiosInstance) {
    this.axios = axiosInstance;
  }

  async marcajes_por_fecha(usuarioId: string, fecha: dayjs.Dayjs): Promise<Marcaje[]> {
    const response = await this.axios.get(
      `api/usuarios/${usuarioId}/marcajes_fecha/${formatDateTimeForServer(fecha)}`);
    const marcajesData = response.data;

    return Array.isArray(marcajesData)
      ? marcajesData.map(Marcaje.fromRequest)
      : [];
  }

  async ultimos_marcajes(usuarioId: string): Promise<Marcaje[]> {
    const response = await this.axios.get(
      `api/usuarios/${usuarioId}/ultimos_marcajes`);
    const marcajesData = response.data;

    return Array.isArray(marcajesData)
      ? marcajesData.map(Marcaje.fromRequest)
      : [];
  }

  async registrar(reg: MarcajeOutDTO): Promise<void> {
    return this.axios.post('api/marcajes', reg);
  }
}

// Implementación de MarcajeApi en modo test
export class MarcajeTestApi implements MarcajeApi {
  async marcajes_por_fecha(usuario: string, __: dayjs.Dayjs): Promise<Marcaje[]> {
    return this.ultimos_marcajes(usuario)
  }

  async ultimos_marcajes(_: string): Promise<Marcaje[]> {
    const marcajesFicticios = [
      // Caso 1: Registro normal completo
      {
        usuario_reg: {
          id: 1,
          nombre: 'Juan',
          primer_apellido: 'Pérez',
          segundo_apellido: 'Gómez'
        },
        horario: {
          dia: 'L',
          hora_inicio: '08:00',
          hora_fin: '16:00',
          horas_a_trabajar: 8
        },
        fecha: '2024-01-15',
        hora_inicio: '08:00',
        hora_fin: '16:00',
        hora_trabajadas: 8
      },
      // Caso 2: usuario_reg nulo (registro sin usuario asociado)
      {
        usuario_reg: null,
        horario: {
          dia: 'M',
          hora_inicio: '09:00',
          hora_fin: '17:00',
          horas_a_trabajar: 8
        },
        fecha: '2024-01-16',
        hora_inicio: '09:00',
        hora_fin: '17:00',
        hora_trabajadas: 8
      },
      // Caso 3: hora_fin nula (registro sin finalizar)
      {
        usuario_reg: {
          id: 2,
          nombre: 'María',
          primer_apellido: 'López',
          segundo_apellido: 'Martínez'
        },
        horario: {
          dia: 'X',
          hora_inicio: '07:30',
          hora_fin: '15:30',
          horas_a_trabajar: 8
        },
        fecha: '2024-01-17',
        hora_inicio: '07:30',
        hora_fin: null,
        hora_trabajadas: null
      },
      // Caso 4: hora_trabajadas nula (sin calcular)
      {
        usuario_reg: {
          id: 3,
          nombre: 'Carlos',
          primer_apellido: 'García',
          segundo_apellido: 'Hernández'
        },
        horario: {
          dia: 'J',
          hora_inicio: '08:30',
          hora_fin: '16:30',
          horas_a_trabajar: 8
        },
        fecha: '2024-01-18',
        hora_inicio: '08:30',
        hora_fin: null,
        hora_trabajadas: null
      },
      {
        usuario_reg: {
          id: 3,
          nombre: 'Carlos',
          primer_apellido: 'García',
          segundo_apellido: 'Hernández'
        },
        horario: {
          dia: 'J',
          hora_inicio: '08:30',
          hora_fin: '16:30',
          horas_a_trabajar: 8
        },
        fecha: '2024-01-18',
        hora_inicio: '08:30',
        hora_fin: null,
        hora_trabajadas: null
      },
      {
        usuario_reg: {
          id: 3,
          nombre: 'Carlos',
          primer_apellido: 'García',
          segundo_apellido: 'Hernández'
        },
        horario: {
          dia: 'J',
          hora_inicio: '08:30',
          hora_fin: '16:30',
          horas_a_trabajar: 8
        },
        fecha: '2024-01-18',
        hora_inicio: '08:30',
        hora_fin: null,
        hora_trabajadas: null
      },
    ];

    return marcajesFicticios.map(Marcaje.fromRequest);
  }

  async registrar(_: MarcajeOutDTO): Promise<void> {
    return;
  }
}