import dayjs, { Dayjs } from 'dayjs';

export class CumplimientoHorario {
  constructor(
    public fecha: Dayjs,
    public horasTrabajoEfectivo: number,
    public horasTrabajadas: number,
    public horasATrabajar: number,
    public saldo: number,
    public nota: string
  ) {
  }

  getDiaSemana(): string {
    return this.fecha.format('dddd');
  }

  static fromRequest(obj: any): CumplimientoHorario {
    return new CumplimientoHorario(
      dayjs(obj.fecha),
      obj.horas_trabajo_efectivo,
      obj.horas_trabajadas,
      obj.horas_a_trabajar,
      obj.saldo,
      obj.nota,
    );
  }

  static fromRequestArray(objs: any[]): CumplimientoHorario[] {
    return objs.map(obj => CumplimientoHorario.fromRequest(obj));
  }
}

export class InformeCumplimiento {
  constructor(
    public lineas: CumplimientoHorario[],
    public totalSaldo: number
  ) { }

  static fromRequest(obj: any): InformeCumplimiento {
    return new InformeCumplimiento(
      CumplimientoHorario.fromRequestArray(obj.lineas),
      obj.total_saldo
    );
  }
}