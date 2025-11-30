import { AxiosInstance } from "axios";
import { Marcaje } from "../modelos/marcaje";
import { DominiosWithCacheUsuarioDTO, MarcajeOutDTO } from "../modelos/dto";
import dayjs from "dayjs";
import { formatDateForServer, formatDateTimeForServer } from "../modelos/formatos";

export interface MarcajeApi {
  marcajes(
    usuarioId: number,
    fecha_inicio: dayjs.Dayjs | null,
    fecha_fin: dayjs.Dayjs | null,
    usuario_reg: number | null): Promise<Marcaje[]>;
  marcajesSinInc(
    usuarioId: string,
    fecha: dayjs.Dayjs,
    usuarioReg: string | undefined): Promise<Marcaje[]>;
  marcajesPorFecha(usuarioId: string, fecha: dayjs.Dayjs): Promise<Marcaje[]>;
  ultimosMarcajes(usuarioId: string): Promise<Marcaje[]>;
  registrar(reg: MarcajeOutDTO): Promise<void>;
  // Devuelve si existe algún marcaje para un usuario y fecha
  // tiene su hora final sin registrar (nula)
  marcajeSinFinalizar(usuarioId: number, fecha: dayjs.Dayjs): Promise<boolean>;
  // Registra la salida del marcaje que se encuentra sin registrar (nulo)
  // para un usuario y una fecha
  registrarSalida(usuarioId: number, horaFin: dayjs.Dayjs): Promise<void>;
}


// Implementación de MarcajeApi en modo producción
export class MarcajeAxiosApi implements MarcajeApi {
  private axios: AxiosInstance;

  constructor(axiosInstance: AxiosInstance) {
    this.axios = axiosInstance;
  }

  async marcajes(
    usuarioId: number,
    fecha_inicio: dayjs.Dayjs | null,
    fecha_fin: dayjs.Dayjs | null,
    usuario_reg: number | null): Promise<Marcaje[]> {

    const response = await this.axios.post('api/marcajes/entre/fechas',
      {
        'usuario': usuarioId,
        'fecha_inicio': formatDateForServer(fecha_inicio),
        'fecha_fin': formatDateForServer(fecha_fin),
        'usuario_reg': usuario_reg
      }
    );

    return Marcaje.fromRequest(
      DominiosWithCacheUsuarioDTO.fromResponse(response.data));
  }

  async marcajesSinInc(
    usuarioId: string,
    fecha: dayjs.Dayjs,
    usuarioReg: string | undefined): Promise<Marcaje[]> {
    let uri = `api/usuarios/${usuarioId}/marcajes/sin/inc/${formatDateTimeForServer(fecha)}`
    if (usuarioReg) {
      uri = `api/usuarios/${usuarioId}/marcajes/sin/inc/${formatDateTimeForServer(fecha)}/registrador/${usuarioReg}`
    }

    const response = await this.axios.get(uri);

    return Marcaje.fromRequest(
      DominiosWithCacheUsuarioDTO.fromResponse(response.data));
  }

  async marcajesPorFecha(
    usuarioId: string, fecha: dayjs.Dayjs): Promise<Marcaje[]> {
    const response = await this.axios.get(
      `api/usuarios/${usuarioId}/marcajes/por/fecha/${formatDateTimeForServer(fecha)}`);

    return Marcaje.fromRequest(
      DominiosWithCacheUsuarioDTO.fromResponse(response.data));
  }

  async ultimosMarcajes(usuarioId: string): Promise<Marcaje[]> {
    const response = await this.axios.get(
      `api/usuarios/${usuarioId}/ultimos_marcajes`);

    return Marcaje.fromRequest(
      DominiosWithCacheUsuarioDTO.fromResponse(response.data));
  }

  async registrar(reg: MarcajeOutDTO): Promise<void> {
    return this.axios.post('api/marcajes', reg);
  }

  async marcajeSinFinalizar(usuarioId: number, fecha: dayjs.Dayjs): Promise<boolean> {
    const response = await this.axios.get(
      `api/usuarios/${usuarioId}/marcajes/fecha/${formatDateTimeForServer(fecha)}/sin/finalizar`);

    return response.status == 200;
  }

  async registrarSalida(usuarioId: number, horaFin: dayjs.Dayjs): Promise<void> {
    return this.axios.put(
      `api/usuarios/${usuarioId}/finalizar/marcaje/${formatDateTimeForServer(horaFin)}`);
  }
}

// Implementación de MarcajeApi en modo test
export class MarcajeTestApi implements MarcajeApi {
  async marcajes(
    _: number,
    __: dayjs.Dayjs | null,
    ___: dayjs.Dayjs | null,
    ____: number | null): Promise<Marcaje[]> {
    return this.ultimosMarcajes('')
  }

  async marcajesSinInc(
    usuario: string,
    _: dayjs.Dayjs,
    __: string | undefined): Promise<Marcaje[]> {
    return this.ultimosMarcajes(usuario)
  }

  async marcajesPorFecha(
    usuario: string, __: dayjs.Dayjs): Promise<Marcaje[]> {
    return this.ultimosMarcajes(usuario)
  }

  async ultimosMarcajes(_: string): Promise<Marcaje[]> {
    const dto = DominiosWithCacheUsuarioDTO.fromResponse({
      items: [
        // Caso 1: Registro normal completo
        {
          id: 1,
          usuario: 1,
          usuario_reg: 1,
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
          id: 2,
          usuario: 1,
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
          id: 3,
          usuario: 1,
          usuario_reg: 2,
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
          id: 4,
          usuario: 1,
          usuario_reg: 3,
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
          id: 5,
          usuario: 1,
          usuario_reg: 3,
          horario: {
            dia: 'V',
            hora_inicio: '07:00',
            hora_fin: '15:00',
            horas_a_trabajar: 8
          },
          fecha: '2024-01-19',
          hora_inicio: '07:00',
          hora_fin: '15:00',
          hora_trabajadas: 8
        },
        {
          id: 6,
          usuario: 1,
          usuario_reg: 2,
          horario: {
            dia: 'S',
            hora_inicio: '08:00',
            hora_fin: '12:00',
            horas_a_trabajar: 4
          },
          fecha: '2024-01-20',
          hora_inicio: '08:00',
          hora_fin: null,
          hora_trabajadas: null
        },
        {
          id: 7,
          usuario: 1,
          usuario_reg: 2,
          horario: {
            dia: 'S',
            hora_inicio: '08:00',
            hora_fin: '12:00',
            horas_a_trabajar: 4
          },
          fecha: '2024-01-20',
          hora_inicio: '08:00',
          hora_fin: null,
          hora_trabajadas: null
        },
        {
          id: 8,
          usuario: 1,
          usuario_reg: 2,
          horario: {
            dia: 'S',
            hora_inicio: '08:00',
            hora_fin: '12:00',
            horas_a_trabajar: 4
          },
          fecha: '2024-01-20',
          hora_inicio: '08:00',
          hora_fin: null,
          hora_trabajadas: null
        },
        {
          id: 9,
          usuario: 1,
          usuario_reg: 2,
          horario: {
            dia: 'S',
            hora_inicio: '08:00',
            hora_fin: '12:00',
            horas_a_trabajar: 4
          },
          fecha: '2024-01-20',
          hora_inicio: '08:00',
          hora_fin: null,
          hora_trabajadas: null
        },
        {
          id: 10,
          usuario: 1,
          usuario_reg: 2,
          horario: {
            dia: 'S',
            hora_inicio: '08:00',
            hora_fin: '12:00',
            horas_a_trabajar: 4
          },
          fecha: '2024-01-20',
          hora_inicio: '08:00',
          hora_fin: null,
          hora_trabajadas: null
        },
        {
          id: 11,
          usuario: 1,
          usuario_reg: 2,
          horario: {
            dia: 'S',
            hora_inicio: '08:00',
            hora_fin: '12:00',
            horas_a_trabajar: 4
          },
          fecha: '2024-01-20',
          hora_inicio: '08:00',
          hora_fin: null,
          hora_trabajadas: null
        },
        {
          id: 12,
          usuario: 1,
          usuario_reg: 2,
          horario: {
            dia: 'S',
            hora_inicio: '08:00',
            hora_fin: '12:00',
            horas_a_trabajar: 4
          },
          fecha: '2024-01-20',
          hora_inicio: '08:00',
          hora_fin: null,
          hora_trabajadas: null
        }
      ],
      cache: {
        "1": {
          id: 1,
          nombre: 'Juan',
          primer_apellido: 'Pérez',
          segundo_apellido: 'Gómez'
        },
        "2": {
          id: 2,
          nombre: 'María',
          primer_apellido: 'López',
          segundo_apellido: 'Martínez'
        },
        "3": {
          id: 3,
          nombre: 'Carlos',
          primer_apellido: 'García',
          segundo_apellido: 'Hernández'
        },
        "4": {
          id: 4,
          nombre: 'Ana',
          primer_apellido: 'Rodríguez',
          segundo_apellido: 'Fernández'
        },
        "5": {
          id: 5,
          nombre: 'Pedro',
          primer_apellido: 'Sánchez',
          segundo_apellido: 'Díaz'
        }
      }
    });

    return Marcaje.fromRequest(dto);
  }

  async registrar(_: MarcajeOutDTO): Promise<void> {
    return;
  }

  async marcajeSinFinalizar(_: number, __: dayjs.Dayjs): Promise<boolean> {
    return false;
  }

  async registrarSalida(_: number, __: dayjs.Dayjs): Promise<void> {
    return;
  }
}