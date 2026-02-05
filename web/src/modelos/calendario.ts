import { Expose } from 'class-transformer';
import dayjs, { Dayjs } from 'dayjs';
import { formatDateForServer } from './formatos';

export class Calendario {
  constructor(
    public id: number,
    public nombre: string,
    public descripcion: string | null,
  ) { }

  static fromRequest(obj: any): Calendario {
    return new Calendario(
      obj.id,
      obj.nombre,
      obj.descripcion,
    );
  }

  static fromRequestArray(objs: any[]): Calendario[] {
    return objs.map(obj => Calendario.fromRequest(obj));
  }
}

export enum TipoCalendarioFecha {
  Baja = 0,
  Vacaciones = 1,
  DiasPropios = 2,
  Permiso = 3,
  Festivo = 4,
  Cierre = 5,
  Otros = 6,
}

export const NombresTipoCalendarioFecha: Record<TipoCalendarioFecha, string> = {
  [TipoCalendarioFecha.Baja]: 'Baja',
  [TipoCalendarioFecha.Vacaciones]: 'Vacaciones',
  [TipoCalendarioFecha.DiasPropios]: 'Días propios',
  [TipoCalendarioFecha.Permiso]: 'Días de permiso',
  [TipoCalendarioFecha.Festivo]: 'Festivo',
  [TipoCalendarioFecha.Cierre]: 'Cierre temporal',
  [TipoCalendarioFecha.Otros]: 'Otros',
};

export class CalendarioFecha {
  id: number;
  @Expose({ name: 'calendario' })
  calendario: number;
  @Expose({ name: 'fecha_inicio' })
  fechaInicio: Dayjs;
  @Expose({ name: 'fecha_fin' })
  fechaFin: Dayjs;
  tipo: TipoCalendarioFecha;

  constructor(data: Partial<CalendarioFecha>) {
    Object.assign(this, data);
  }

  static fromRequest(obj: any): CalendarioFecha {
    return new CalendarioFecha({
      id: obj.id,
      calendario: obj.calendario,
      fechaInicio: dayjs(obj.fecha_inicio),
      fechaFin: dayjs(obj.fecha_fin),
      tipo: obj.tipo,
    });
  }

  toServer(): {} {
    return {
      id: this.id,
      calendario: this.calendario,
      fecha_inicio: formatDateForServer(this.fechaInicio),
      fecha_fin: formatDateForServer(this.fechaFin),
      tipo: this.tipo,
    };
  }

  static fromRequestArray(objs: any[]): CalendarioFecha[] {
    return objs.map(obj => CalendarioFecha.fromRequest(obj));
  }
}