import { AxiosInstance } from 'axios';
import { Calendario, CalendarioFecha, TipoCalendarioFecha } from '../modelos/calendario';
import dayjs, { Dayjs } from 'dayjs';
import { formatDateForServer } from '../modelos/formatos';

export interface CalendariosApi {
  calendarios(): Promise<Calendario[]>;
  descriptorCalendarios(): Promise<Calendario[]>;
  calendario(id: string): Promise<Calendario>;
  crearCalendario(calendario: Calendario): Promise<void>;
  actualizarCalendario(calendario: Calendario): Promise<void>;
  eliminarCalendario(id: number): Promise<void>;
  fechas(calendarioId: number, fechaInicio?: Dayjs | null, fechaFin?: Dayjs | null): Promise<CalendarioFecha[]>;
  fecha(id: number): Promise<CalendarioFecha>;
  crearFecha(fecha: CalendarioFecha): Promise<void>;
  actualizarFecha(fecha: CalendarioFecha): Promise<void>;
  eliminarFecha(id: number): Promise<void>;
}

export class CalendariosAxiosApi implements CalendariosApi {
  private axios: AxiosInstance;

  constructor(axiosInstance: AxiosInstance) {
    this.axios = axiosInstance;
  }

  async calendarios(): Promise<Calendario[]> {
    const response = await this.axios.get('calendarios');
    const data = response.data;
    return Array.isArray(data) ? Calendario.fromRequestArray(data) : [];
  }

  async descriptorCalendarios(): Promise<Calendario[]> {
    const response = await this.axios.get('calendarios/descriptores');
    const data = response.data;
    return Array.isArray(data) ? Calendario.fromRequestArray(data) : [];
  }

  async calendario(id: string): Promise<Calendario> {
    const response = await this.axios.get(`calendarios/${id}`);
    return Calendario.fromRequest(response.data);
  }

  async crearCalendario(calendario: Calendario): Promise<void> {
    return this.axios.post('calendarios', calendario);
  }

  async actualizarCalendario(calendario: Calendario): Promise<void> {
    return this.axios.put('calendarios', calendario);
  }

  async eliminarCalendario(id: number): Promise<void> {
    return this.axios.delete(`calendarios/${id}`);
  }

  async fechas(calendarioId: number, fechaInicio?: Dayjs | null, fechaFin?: Dayjs | null): Promise<CalendarioFecha[]> {
    const params: any = {};

    if (fechaInicio) {
      params.fecha_inicio = formatDateForServer(fechaInicio);
    }
    if (fechaFin) {
      params.fecha_fin = formatDateForServer(fechaFin);
    }
    const response = await this.axios.get(`calendarios/${calendarioId}/fechas`, { params });
    return CalendarioFecha.fromRequestArray(response.data);
  }

  async fecha(id: number): Promise<CalendarioFecha> {
    const response = await this.axios.get(`calendarios/fechas/${id}`);
    return CalendarioFecha.fromRequest(response.data);
  }

  async crearFecha(fecha: CalendarioFecha): Promise<void> {
    return this.axios.post('calendarios/fechas', fecha.toServer());
  }

  async actualizarFecha(fecha: CalendarioFecha): Promise<void> {
    return this.axios.put('calendarios/fechas', fecha.toServer());
  }

  async eliminarFecha(id: number): Promise<void> {
    return this.axios.delete(`calendarios/fechas/${id}`);
  }
}

export class CalendariosTestApi implements CalendariosApi {
  private calendariosData: Calendario[] = [
    { id: 1, nombre: 'Calendario Laboral 2024', descripcion: 'Festivos nacionales y autonómicos' },
    { id: 2, nombre: 'Calendario Vacaciones Verano', descripcion: 'Periodo de vacaciones estivales' },
  ];
  private nextId = 3;
  private fechasData: CalendarioFecha[] = [
    new CalendarioFecha({ id: 1, calendario: 1, fechaInicio: dayjs('2024-01-01'), fechaFin: dayjs('2024-01-01'), tipo: TipoCalendarioFecha.Otros }),
    new CalendarioFecha({ id: 2, calendario: 1, fechaInicio: dayjs('2024-12-25'), fechaFin: dayjs('2024-12-25'), tipo: TipoCalendarioFecha.Otros }),
  ];
  private nextFechaId = 3;

  async calendarios(): Promise<Calendario[]> {
    await new Promise(resolve => setTimeout(resolve, 100));
    return Calendario.fromRequestArray(this.calendariosData);
  }

  async descriptorCalendarios(): Promise<Calendario[]> {
    await new Promise(resolve => setTimeout(resolve, 100));
    return Calendario.fromRequestArray(this.calendariosData.map(c => ({ ...c, descripcion: null })));
  }

  async calendario(id: string): Promise<Calendario> {
    await new Promise(resolve => setTimeout(resolve, 100));
    const calendario = this.calendariosData.find(c => c.id === parseInt(id, 10));
    if (!calendario) throw new Error('Calendario no encontrado');
    return Calendario.fromRequest(calendario);
  }

  async crearCalendario(calendario: Calendario): Promise<void> {
    await new Promise(resolve => setTimeout(resolve, 100));
    const newCalendario = { ...calendario, id: this.nextId++ };
    this.calendariosData.push(newCalendario);
  }

  async actualizarCalendario(calendario: Calendario): Promise<void> {
    await new Promise(resolve => setTimeout(resolve, 100));
    const index = this.calendariosData.findIndex(c => c.id === calendario.id);
    if (index !== -1) {
      this.calendariosData[index] = new Calendario(
        calendario.id,
        calendario.nombre,
        calendario.descripcion
      );
    }
  }

  async eliminarCalendario(id: number): Promise<void> {
    await new Promise(resolve => setTimeout(resolve, 100));
    this.calendariosData = this.calendariosData.filter(c => c.id !== id);
  }

  async fechas(calendarioId: number, fechaInicio?: Dayjs | null, fechaFin?: Dayjs | null): Promise<CalendarioFecha[]> {
    await new Promise(resolve => setTimeout(resolve, 100));
    let data = this.fechasData.filter(f => f.calendario === calendarioId);

    if (fechaInicio) {
      data = data.filter(f => f.fechaFin.isAfter(fechaInicio) || f.fechaFin.isSame(fechaInicio, 'day'));
    }

    if (fechaFin) {
      data = data.filter(f => f.fechaInicio.isBefore(fechaFin) || f.fechaInicio.isSame(fechaFin, 'day'));
    }

    return CalendarioFecha.fromRequestArray(data);
  }

  async fecha(id: number): Promise<CalendarioFecha> {
    await new Promise(resolve => setTimeout(resolve, 100));
    const fecha = this.fechasData.find(f => f.id === id);
    if (!fecha) throw new Error('Fecha no encontrada');
    return CalendarioFecha.fromRequest(fecha);
  }

  async crearFecha(fecha: CalendarioFecha): Promise<void> {
    await new Promise(resolve => setTimeout(resolve, 100));
    const newFecha = { ...fecha, id: this.nextFechaId++ };
    this.fechasData.push(new CalendarioFecha(newFecha));
  }

  async actualizarFecha(fecha: CalendarioFecha): Promise<void> {
    await new Promise(resolve => setTimeout(resolve, 100));
    const index = this.fechasData.findIndex(f => f.id === fecha.id);
    if (index !== -1) {
      this.fechasData[index] = new CalendarioFecha({ ...this.fechasData[index], ...fecha });
    }
  }

  async eliminarFecha(id: number): Promise<void> {
    await new Promise(resolve => setTimeout(resolve, 100));
    this.fechasData = this.fechasData.filter(f => f.id !== id);
  }
}