import { AxiosInstance } from 'axios';
import { CumplimientoHorario, InformeCumplimiento } from '../modelos/informes';
import dayjs from 'dayjs';

export interface InformeApi {
  cumplimientoHorario(empleadoId: number, mes: number, anio: number):
    Promise<InformeCumplimiento>;
}

export class InformeAxiosApi implements InformeApi {
  private axios: AxiosInstance;

  constructor(axiosInstance: AxiosInstance) {
    this.axios = axiosInstance;
  }

  async cumplimientoHorario(empleadoId: number, mes: number, anio: number): Promise<InformeCumplimiento> {
    const response = await this.axios.get(`informes/cumplimiento/horario`, {
      params: {
        empleadoId,
        mes,
        anio,
      },
    });
    return InformeCumplimiento.fromRequest(response.data);
  }
}

export class InformeTestApi implements InformeApi {
  async cumplimientoHorario(_: number, mes: number, anio: number): Promise<InformeCumplimiento> {
    await new Promise(resolve => setTimeout(resolve, 100));

    // Mock data for testing
    const lineas = [
      new CumplimientoHorario(
        dayjs(`${anio}-${mes}-01`),
        8,
        8,
        8,
        0,
        'OK',
      ),
      new CumplimientoHorario(
        dayjs(`${anio} - ${mes}-02`),
        7,
        7,
        8,
        -1,
        'Faltan horas',
      ),
      new CumplimientoHorario(
        dayjs(`${anio} - ${mes}-03`),
        7,
        7,
        8,
        -1,
        'Faltan horas',
      ),
      new CumplimientoHorario(
        dayjs(`${anio} - ${mes}-04`),
        7,
        7,
        8,
        -1,
        'Faltan horas',
      ),
      new CumplimientoHorario(
        dayjs(`${anio} - ${mes}-05`),
        7,
        7,
        8,
        -1,
        'Faltan horas',
      ),
    ];

    const totalSaldo = lineas.reduce((acc, curr) => acc + curr.saldo, 0);
    return new InformeCumplimiento(lineas, totalSaldo);
  }
}