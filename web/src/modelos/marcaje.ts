import dayjs, { Dayjs } from "dayjs"
import { DescriptorUsuario, Horario } from "./usuarios"

export class Marcaje {

  constructor(
    public usuario: number,
    public usuario_reg: DescriptorUsuario | null,
    public fecha: Dayjs,
    public horaInicio: string,
    public horaFin: string | null,
    public horario: Horario | null,
    public horasTrabajadas: number | null,
  ) { }

  // Crea una instancia desde la solicitudo del servidor
  static fromRequest(obj: any): Marcaje {
    const usuarioReg = obj.usuario_reg
      ? new DescriptorUsuario(
        obj.usuario_reg.id,
        obj.usuario_reg.nombre,
        obj.usuario_reg.primer_apellido,
        obj.usuario_reg.segundo_apellido
      )
      : null;

    // No es necesario asignar usuario cuando viene del servidor porque 
    // siempre esta filtrado por el usuario
    return new Marcaje(
      0,
      usuarioReg,
      dayjs(obj.fecha),
      obj.hora_inicio,
      obj.hora_fin ? obj.hora_fin : null,
      Horario.fromRequest(obj.horario),
      obj.hora_trabajadas,
    );
  }

  horaFinToStr(): string {
    return this.horaFin ? this.horaFin : 'Sin especificar';
  }

  horaTrabajadasToStr(): string {
    return this.horasTrabajadas ? this.horasTrabajadas.toFixed(2)
      : 'Sin especificar';
  }

}