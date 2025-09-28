import { AxiosInstance } from "axios";
import { Registro } from "../modelos/registro";
import { RegistroOutDTO } from "../modelos/dto";
import dayjs from "dayjs";
import { formatDateTimeForServer } from "../modelos/formatos";

export interface RegistroApi {
  marcajes_por_fecha(usuarioId: string, fecha: dayjs.Dayjs): Promise<Registro[]>;
  ultimos_registros(usuarioId: string): Promise<Registro[]>;
  registrar(reg: RegistroOutDTO): Promise<void>;
}


// Implementación de REgistroApi en modo producción
export class RegistroAxiosApi implements RegistroApi {
  private axios: AxiosInstance;

  constructor(axiosInstance: AxiosInstance) {
    this.axios = axiosInstance;
  }

  async marcajes_por_fecha(usuarioId: string, fecha: dayjs.Dayjs): Promise<Registro[]> {
    const response = await this.axios.get(
      `api/usuarios/${usuarioId}/registros_fecha/${formatDateTimeForServer(fecha)}`);
    const registrosData = response.data;

    return Array.isArray(registrosData)
      ? registrosData.map(Registro.fromRequest)
      : [];
  }

  async ultimos_registros(usuarioId: string): Promise<Registro[]> {
    const response = await this.axios.get(
      `api/usuarios/${usuarioId}/ultimos_registros`);
    const registrosData = response.data;

    return Array.isArray(registrosData)
      ? registrosData.map(Registro.fromRequest)
      : [];
  }

  async registrar(reg: RegistroOutDTO): Promise<void> {
    return this.axios.post('api/registros', reg);
  }
}

// Implementación de RegistroApi en modo test
export class RegistroTestApi implements RegistroApi {
  async marcajes_por_fecha(usuario: string, __: dayjs.Dayjs): Promise<Registro[]> {
    return this.ultimos_registros(usuario)
  }

  async ultimos_registros(_: string): Promise<Registro[]> {
    const registrosFicticios = [
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

    return registrosFicticios.map(Registro.fromRequest);
  }

  async registrar(_: RegistroOutDTO): Promise<void> {
    return;
  }
}